//! `fugit` provides a comprehensive library of [`Duration`] and [`Instant`] for the handling of
//! time in embedded systems. The library is specifically designed to maximize const-ification
//! which allows for most comparisons and changes of time-base to be made at compile time, rather
//! than run time.
//!
//! The library is aimed at ease-of-use and performance first.
//!
//! ```
//! use fugit::{Duration, ExtU32};
//!
//! // Efficient short-hands (`.millis()`, ...)
//! let d = Duration::<u32, 1, 1_000>::from_ticks(111);
//!
//! let sum1 = d + 300.millis();
//! //             ^^^ Compile time move of base, only a sum is needed and no change of base
//!
//!
//! // -----------------------
//!
//! // Best effort for fixed types
//! fn bar(d1: Duration<u32, 1, 1_000>, d2: Duration<u32, 1, 1_000_000>) {
//!     let sum = d1 + d2.convert();
//!     //        ^^^^^^^ Run time move of base, will use a `mul` and `div` instruction (Cortex-M3+) to
//!     //                perform the move of base.
//!     //                The `.convert()` explicitly signals the move of base.
//!
//!     let ops = d1 > d2;
//!     //        ^^^^^^^ Run time comparison of different base, will use 2 `mul` instructions
//!     //                (Cortex-M3+) to perform the comparison.
//! }
//!
//! fn baz(d1: Duration<u64, 1, 1_000>, d2: Duration<u64, 1, 1_000_000>) {
//!     let sum = d1 + d2.convert();
//!     //        ^^^^^^^ Run time move of base, will use a `mul` insruction and `div`
//!     //                soft-impl (Cortex-M3+) to perform the move of base.
//!     //                The `.convert()` explicitly signals the move of base.
//!
//!     let ops = d1 > d2;
//!     //        ^^^^^^^ Run time comparison of different base, will use 4 `mul` instructions
//!     //                (Cortex-M3+) to perform the comparison.
//! }
//! ```

#![cfg_attr(not(test), no_std)]
#![allow(unexpected_cfgs)] // `coverage_nightly` cfg is only recognised by nightly; stable toolchain would warn.
#![allow(dead_code)] // Under `certified_subset`, some items (e.g. RateExtU64) are gated out, leaving their helpers unused.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![deny(missing_docs)]

mod aliases;
mod duration;
mod helpers;
mod instant;
mod rate;

pub use aliases::*;
pub use duration::Duration;
pub use duration::ExtU32;
#[cfg(not(feature = "certified_subset"))]
pub use duration::ExtU32Ceil;
#[cfg(not(feature = "certified_subset"))]
pub use duration::ExtU64;
#[cfg(not(feature = "certified_subset"))]
pub use duration::ExtU64Ceil;
pub use instant::Instant;
pub use rate::{ExtU32 as RateExtU32, Rate};
#[cfg(not(feature = "certified_subset"))]
pub use rate::ExtU64 as RateExtU64;

#[cfg(test)]
mod test {
    use crate::Duration;
    use crate::Instant;
    use core::cmp::Ordering;
    use crate::Rate;
    use crate::{
        Hertz, HertzU32, HertzU64, Kilohertz, KilohertzU32, KilohertzU64, Megahertz, MegahertzU32,
        MegahertzU64, TimerRate, TimerRateU32, TimerRateU64,
    };

    ////////////////////////////////////////////////////////////////////////////////
    //
    // Duration tests
    //
    ////////////////////////////////////////////////////////////////////////////////

    #[cfg(not(feature = "certified_subset"))]
    #[test]
    fn large_duration_converstion() {
        use crate::ExtU64;

        let sum = Duration::<u64, 1, 80_000_000>::from_ticks(0) + 15.minutes();

        assert!(sum == Duration::<u64, 1, 80_000_000>::from_ticks(80_000_000 * 60 * 15));
    }

    fn take_ms(d: Duration<u32, 1, 1_000>) -> Duration<u32, 1, 1_000> {
        d
    }

    #[test]
    fn duration_functions() {
        assert!(
            take_ms(Duration::<u32, 1, 100>::from_ticks(1).convert())
                == Duration::<u32, 1, 1_000>::from_ticks(10)
        );
    }

    #[test]
    fn duration_compare_u32() {
        // Same fraction
        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(2) > Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(2) >= Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(1) >= Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(1) < Duration::<u32, 1, 1_000>::from_ticks(2)
        );
        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(1) <= Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(1) <= Duration::<u32, 1, 1_000>::from_ticks(2)
        );
        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(1) == Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(1) != Duration::<u32, 1, 1_000>::from_ticks(2)
        );

        // Different fraction
        assert!(
            Duration::<u32, 1, 10_000>::from_ticks(11) > Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 10_000>::from_ticks(11) >= Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 10_000>::from_ticks(10) >= Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 10_000>::from_ticks(11) < Duration::<u32, 1, 1_000>::from_ticks(2)
        );
        assert!(
            Duration::<u32, 1, 10_000>::from_ticks(1) <= Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 10_000>::from_ticks(10) <= Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 10_000>::from_ticks(10) == Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 10_000>::from_ticks(9) != Duration::<u32, 1, 1_000>::from_ticks(2)
        );

        // From instants
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(5)
                > Duration::<u32, 1, 1_000>::from_ticks(4)
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(5)
                >= Duration::<u32, 1, 1_000>::from_ticks(4)
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(5)
                >= Duration::<u32, 1, 1_000>::from_ticks(5)
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(5)
                < Duration::<u32, 1, 1_000>::from_ticks(6)
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(5)
                <= Duration::<u32, 1, 1_000>::from_ticks(6)
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(5)
                <= Duration::<u32, 1, 1_000>::from_ticks(5)
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(5)
                == Duration::<u32, 1, 1_000>::from_ticks(5)
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(5)
                != Duration::<u32, 1, 1_000>::from_ticks(4)
        );
    }

    #[test]
    fn duration_compare_u64() {
        // Same fraction
        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(2) > Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(2) >= Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(1) >= Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(1) < Duration::<u64, 1, 1_000>::from_ticks(2)
        );
        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(1) <= Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(1) <= Duration::<u64, 1, 1_000>::from_ticks(2)
        );
        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(1) == Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(1) != Duration::<u64, 1, 1_000>::from_ticks(2)
        );

        // Different fraction
        assert!(
            Duration::<u64, 1, 10_000>::from_ticks(11) > Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 10_000>::from_ticks(11) >= Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 10_000>::from_ticks(10) >= Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 10_000>::from_ticks(11) < Duration::<u64, 1, 1_000>::from_ticks(2)
        );
        assert!(
            Duration::<u64, 1, 10_000>::from_ticks(1) <= Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 10_000>::from_ticks(10) <= Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 10_000>::from_ticks(10) == Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 10_000>::from_ticks(9) != Duration::<u64, 1, 1_000>::from_ticks(2)
        );

        // From instants
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(5)
                > Duration::<u64, 1, 1_000>::from_ticks(4)
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(5)
                >= Duration::<u64, 1, 1_000>::from_ticks(4)
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(5)
                >= Duration::<u64, 1, 1_000>::from_ticks(5)
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(5)
                < Duration::<u64, 1, 1_000>::from_ticks(6)
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(5)
                <= Duration::<u64, 1, 1_000>::from_ticks(6)
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(5)
                <= Duration::<u64, 1, 1_000>::from_ticks(5)
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(5)
                == Duration::<u64, 1, 1_000>::from_ticks(5)
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(5)
                != Duration::<u64, 1, 1_000>::from_ticks(4)
        );
    }

    #[cfg(not(feature = "certified_subset"))]
    #[test]
    fn duration_compare_u64_u32() {
        // Same fraction
        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(2) > Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(2) >= Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(1) >= Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(1) < Duration::<u32, 1, 1_000>::from_ticks(2)
        );
        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(1) <= Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(1) <= Duration::<u32, 1, 1_000>::from_ticks(2)
        );
        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(1) == Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(1) != Duration::<u32, 1, 1_000>::from_ticks(2)
        );

        // Different fraction
        assert!(
            Duration::<u64, 1, 10_000>::from_ticks(11) > Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 10_000>::from_ticks(11) >= Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 10_000>::from_ticks(10) >= Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 10_000>::from_ticks(11) < Duration::<u32, 1, 1_000>::from_ticks(2)
        );
        assert!(
            Duration::<u64, 1, 10_000>::from_ticks(1) <= Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 10_000>::from_ticks(10) <= Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 10_000>::from_ticks(10) == Duration::<u32, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u64, 1, 10_000>::from_ticks(9) != Duration::<u32, 1, 1_000>::from_ticks(2)
        );

        // From instants
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(5)
                > Duration::<u32, 1, 1_000>::from_ticks(4)
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(5)
                >= Duration::<u32, 1, 1_000>::from_ticks(4)
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(5)
                >= Duration::<u32, 1, 1_000>::from_ticks(5)
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(5)
                < Duration::<u32, 1, 1_000>::from_ticks(6)
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(5)
                <= Duration::<u32, 1, 1_000>::from_ticks(6)
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(5)
                <= Duration::<u32, 1, 1_000>::from_ticks(5)
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(5)
                == Duration::<u32, 1, 1_000>::from_ticks(5)
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(5)
                != Duration::<u32, 1, 1_000>::from_ticks(4)
        );
    }

    #[cfg(not(feature = "certified_subset"))]
    #[test]
    fn duration_compare_u32_u64() {
        // Same fraction
        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(2) > Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(2) >= Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(1) >= Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(1) < Duration::<u64, 1, 1_000>::from_ticks(2)
        );
        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(1) <= Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(1) <= Duration::<u64, 1, 1_000>::from_ticks(2)
        );
        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(1) == Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(1) != Duration::<u64, 1, 1_000>::from_ticks(2)
        );

        // Different fraction
        assert!(
            Duration::<u32, 1, 10_000>::from_ticks(11) > Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 10_000>::from_ticks(11) >= Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 10_000>::from_ticks(10) >= Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 10_000>::from_ticks(11) < Duration::<u64, 1, 1_000>::from_ticks(2)
        );
        assert!(
            Duration::<u32, 1, 10_000>::from_ticks(1) <= Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 10_000>::from_ticks(10) <= Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 10_000>::from_ticks(10) == Duration::<u64, 1, 1_000>::from_ticks(1)
        );
        assert!(
            Duration::<u32, 1, 10_000>::from_ticks(9) != Duration::<u64, 1, 1_000>::from_ticks(2)
        );

        // From instants
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(5)
                > Duration::<u64, 1, 1_000>::from_ticks(4)
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(5)
                >= Duration::<u64, 1, 1_000>::from_ticks(4)
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(5)
                >= Duration::<u64, 1, 1_000>::from_ticks(5)
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(5)
                < Duration::<u64, 1, 1_000>::from_ticks(6)
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(5)
                <= Duration::<u64, 1, 1_000>::from_ticks(6)
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(5)
                <= Duration::<u64, 1, 1_000>::from_ticks(5)
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(5)
                == Duration::<u64, 1, 1_000>::from_ticks(5)
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(5)
                != Duration::<u64, 1, 1_000>::from_ticks(4)
        );
    }

    #[test]
    fn duration_duration_math_u32() {
        use crate::ExtU32;

        // Same base
        let sum: Duration<u32, 1, 1_000> =
            Duration::<u32, 1, 1_000>::from_ticks(10) + Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(sum == Duration::<u32, 1, 1_000>::from_ticks(11));

        let mut sum = Duration::<u32, 1, 1_000>::from_ticks(10);
        sum += Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(sum == Duration::<u32, 1, 1_000>::from_ticks(11));

        let diff: Duration<u32, 1, 1_000> =
            Duration::<u32, 1, 1_000>::from_ticks(10) - Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(diff == Duration::<u32, 1, 1_000>::from_ticks(9));

        let mut diff = Duration::<u32, 1, 1_000>::from_ticks(10);
        diff -= Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(diff == Duration::<u32, 1, 1_000>::from_ticks(9));

        // Different base
        let sum: Duration<u32, 1, 10_000> = Duration::<u32, 1, 10_000>::from_ticks(10)
            + Duration::<u32, 1, 1_000>::from_ticks(1).convert();
        assert!(sum == Duration::<u32, 1, 1_000>::from_ticks(2));

        let mut sum = Duration::<u32, 1, 1_000>::from_ticks(1);
        sum += Duration::<u32, 1, 10_000>::from_ticks(10).convert();
        assert!(sum == Duration::<u32, 1, 1_000>::from_ticks(2));

        let diff: Duration<u32, 1, 10_000> = Duration::<u32, 1, 10_000>::from_ticks(10)
            - Duration::<u32, 1, 1_000>::from_ticks(1).convert();
        assert!(diff == Duration::<u32, 1, 10_000>::from_ticks(0));

        let mut diff = Duration::<u32, 1, 1_000>::from_ticks(1);
        diff -= Duration::<u32, 1, 10_000>::from_ticks(10).convert();
        assert!(diff == Duration::<u32, 1, 1_000>::from_ticks(0));

        // Short hand vs u32 (should not need `.into()`)
        let sum = Duration::<u32, 1, 10_000>::from_ticks(10) + 1.millis();
        assert!(sum == Duration::<u32, 1, 10_000>::from_ticks(20));

        let mut sum = Duration::<u32, 1, 10_000>::from_ticks(10);
        sum += 1.millis();
        assert!(sum == Duration::<u32, 1, 10_000>::from_ticks(20));

        // Fixed in v0.3.2
        let d: Duration<u32, 1, 1_000> = Duration::<u32, 1, 32_768>::from_ticks(42949672).convert();
        assert!(d.ticks() == 1_310_719);

        // Division and multiplication by integers
        let mul: Duration<u32, 1, 1_000> = Duration::<u32, 1, 1_000>::from_ticks(10) * 2;
        assert!(mul == Duration::<u32, 1, 1_000>::from_ticks(20));

        let mut mul = Duration::<u32, 1, 1_000>::from_ticks(10);
        mul *= 2;
        assert!(mul == Duration::<u32, 1, 1_000>::from_ticks(20));

        let div: Duration<u32, 1, 1_000> = Duration::<u32, 1, 1_000>::from_ticks(10) / 2;
        assert!(div == Duration::<u32, 1, 1_000>::from_ticks(5));

        let mut div = Duration::<u32, 1, 1_000>::from_ticks(10);
        div /= 2;
        assert!(div == Duration::<u32, 1, 1_000>::from_ticks(5));

        assert!(
            Duration::<u32, 1, 100>::from_ticks(5) / Duration::<u32, 1, 1_000>::from_ticks(2)
                == 25
        );

        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(2) / Duration::<u32, 1, 100>::from_ticks(5)
                == 0
        );

        assert!(
            Duration::<u32, 1, 1_000>::from_ticks(500) / Duration::<u32, 1, 100>::from_ticks(5)
                == 10
        );
    }

    #[cfg(not(feature = "certified_subset"))]
    #[test]
    fn duration_duration_math_u64() {
        use crate::ExtU64;

        // Same base
        let sum: Duration<u64, 1, 1_000> =
            Duration::<u64, 1, 1_000>::from_ticks(10) + Duration::<u64, 1, 1_000>::from_ticks(1);
        assert!(sum == Duration::<u64, 1, 1_000>::from_ticks(11));

        let mut sum = Duration::<u64, 1, 1_000>::from_ticks(10);
        sum += Duration::<u64, 1, 1_000>::from_ticks(1);
        assert!(sum == Duration::<u64, 1, 1_000>::from_ticks(11));

        let diff: Duration<u64, 1, 1_000> =
            Duration::<u64, 1, 1_000>::from_ticks(10) - Duration::<u64, 1, 1_000>::from_ticks(1);
        assert!(diff == Duration::<u64, 1, 1_000>::from_ticks(9));

        let mut diff = Duration::<u64, 1, 1_000>::from_ticks(10);
        diff -= Duration::<u64, 1, 1_000>::from_ticks(1);
        assert!(diff == Duration::<u64, 1, 1_000>::from_ticks(9));

        // Different base
        let sum: Duration<u64, 1, 10_000> = Duration::<u64, 1, 10_000>::from_ticks(10)
            + Duration::<u64, 1, 1_000>::from_ticks(1).convert();
        assert!(sum == Duration::<u64, 1, 1_000>::from_ticks(2));

        let mut sum = Duration::<u64, 1, 1_000>::from_ticks(1);
        sum += Duration::<u64, 1, 10_000>::from_ticks(10).convert();
        assert!(sum == Duration::<u64, 1, 1_000>::from_ticks(2));

        let diff: Duration<u64, 1, 10_000> = Duration::<u64, 1, 10_000>::from_ticks(10)
            - Duration::<u64, 1, 1_000>::from_ticks(1).convert();
        assert!(diff == Duration::<u64, 1, 1_000>::from_ticks(0));

        let mut diff = Duration::<u64, 1, 1_000>::from_ticks(1);
        diff -= Duration::<u64, 1, 10_000>::from_ticks(10).convert();
        assert!(diff == Duration::<u64, 1, 1_000>::from_ticks(0));

        // Short hand vs u64 (should not need `.into()`)
        let sum = Duration::<u64, 1, 10_000>::from_ticks(10) + 1.millis();
        assert!(sum == Duration::<u64, 1, 10_000>::from_ticks(20));

        let mut sum = Duration::<u64, 1, 10_000>::from_ticks(10);
        sum += 1.millis();
        assert!(sum == Duration::<u64, 1, 10_000>::from_ticks(20));

        // Division and multiplication by integers
        let mul: Duration<u64, 1, 1_000> = Duration::<u64, 1, 1_000>::from_ticks(10) * 2;
        assert!(mul == Duration::<u64, 1, 1_000>::from_ticks(20));

        let mut mul = Duration::<u64, 1, 1_000>::from_ticks(10);
        mul *= 2;
        assert!(mul == Duration::<u64, 1, 1_000>::from_ticks(20));

        let div: Duration<u64, 1, 1_000> = Duration::<u64, 1, 1_000>::from_ticks(10) / 2;
        assert!(div == Duration::<u64, 1, 1_000>::from_ticks(5));

        let mut div = Duration::<u64, 1, 1_000>::from_ticks(10);
        div /= 2;
        assert!(div == Duration::<u64, 1, 1_000>::from_ticks(5));

        assert!(
            Duration::<u64, 1, 1_00>::from_ticks(5) / Duration::<u64, 1, 1_000>::from_ticks(2)
                == 25
        );

        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(2) / Duration::<u64, 1, 1_00>::from_ticks(5)
                == 0
        );

        assert!(
            Duration::<u64, 1, 1_000>::from_ticks(500) / Duration::<u64, 1, 1_00>::from_ticks(5)
                == 10
        );
    }

    #[cfg(not(feature = "certified_subset"))]
    #[test]
    fn duration_duration_math_u64_u32() {
        // Same base
        let sum: Duration<u64, 1, 1_000> =
            Duration::<u64, 1, 1_000>::from_ticks(10) + Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(sum == Duration::<u64, 1, 1_000>::from_ticks(11));

        let mut sum = Duration::<u64, 1, 1_000>::from_ticks(10);
        sum += Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(sum == Duration::<u64, 1, 1_000>::from_ticks(11));

        let diff: Duration<u64, 1, 1_000> =
            Duration::<u64, 1, 1_000>::from_ticks(10) - Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(diff == Duration::<u64, 1, 1_000>::from_ticks(9));

        let mut diff = Duration::<u64, 1, 1_000>::from_ticks(10);
        diff -= Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(diff == Duration::<u64, 1, 1_000>::from_ticks(9));

        // Different base
        let sum: Duration<u64, 1, 10_000> = Duration::<u64, 1, 10_000>::from_ticks(10)
            + Duration::<u32, 1, 1_000>::from_ticks(1).convert();
        assert!(sum == Duration::<u64, 1, 1_000>::from_ticks(2));

        let mut sum = Duration::<u64, 1, 1_000>::from_ticks(1);
        sum += Duration::<u32, 1, 10_000>::from_ticks(10).convert();
        assert!(sum == Duration::<u64, 1, 1_000>::from_ticks(2));

        let diff: Duration<u64, 1, 10_000> = Duration::<u64, 1, 10_000>::from_ticks(10)
            - Duration::<u32, 1, 1_000>::from_ticks(1).convert();
        assert!(diff == Duration::<u64, 1, 1_000>::from_ticks(0));

        let mut diff = Duration::<u64, 1, 1_000>::from_ticks(1);
        diff -= Duration::<u32, 1, 10_000>::from_ticks(10).convert();
        assert!(diff == Duration::<u64, 1, 1_000>::from_ticks(0));
    }

    #[cfg(not(feature = "certified_subset"))]
    #[test]
    fn duration_shorthands_u32() {
        use crate::{ExtU32, ExtU32Ceil};

        let d: Duration<u32, 1, 10_000> = 100_000_000.nanos();
        assert!(d.ticks() == 1_000);

        let d: Duration<u32, 1, 1_000_000> = 40_000.nanos_at_least();
        assert!(d.ticks() == 40);

        let d: Duration<u32, 1, 1_000_000> = 40_075.nanos_at_least();
        assert!(d.ticks() == 41);

        let d: Duration<u32, 1, 1_000> = 4001.micros_at_least();
        assert!(d.ticks() == 5);

        let d: Duration<u32, 1, 10_000> = 100_000.micros();
        assert!(d.ticks() == 1_000);

        let d: Duration<u32, 1, 10_000> = 1.millis();
        assert!(d.ticks() == 10);

        let d: Duration<u32, 1, 10_000> = 1.secs();
        assert!(d.ticks() == 10_000);

        let d: Duration<u32, 1, 10_000> = 1.minutes();
        assert!(d.ticks() == 600_000);

        let d: Duration<u32, 1, 10_000> = 1.hours();
        assert!(d.ticks() == 36_000_000);

        let d = Duration::<u32, 1, 10_000>::millis(10);
        assert!(d.ticks() == 100);

        let d = Duration::<u32, 1, 10_000>::Hz(200);
        assert!(d.ticks() == 50);

        let d = Duration::<u32, 1, 1>::from_ticks(2);
        assert!(d.to_secs() == 2);
        assert!(d.to_nanos() == 2_000_000_000);

        let d = Duration::<u32, 1, 1_000_000_000>::from_ticks(2_000_000_000);
        assert!(d.to_secs() == 2);
        assert!(d.to_nanos() == 2_000_000_000);

        let d = Duration::<u32, 1, 10_000>::from_ticks(100);
        assert!(d.to_nanos() == 10_000_000);

        let d = Duration::<u32, 1, 10_000>::from_ticks(100);
        assert!(d.to_micros() == 10_000);

        let d = Duration::<u32, 1, 10_000>::from_ticks(100);
        assert!(d.to_millis() == 10);

        let d = Duration::<u32, 1, 10_000>::from_ticks(100_000);
        assert!(d.to_secs() == 10);

        let d = Duration::<u32, 1, 10_000>::from_ticks(1_800_000);
        assert!(d.to_minutes() == 3);

        let d = Duration::<u32, 1, 10_000>::from_ticks(180_000_000);
        assert!(d.to_hours() == 5);
    }

    #[cfg(not(feature = "certified_subset"))]
    #[test]
    fn duration_shorthands_u64() {
        use crate::{ExtU64, ExtU64Ceil};

        let d: Duration<u64, 1, 10_000> = 100_000_000.nanos();
        assert!(d.ticks() == 1_000);

        let d: Duration<u64, 1, 1_000_000> = 40_000.nanos_at_least();
        assert!(d.ticks() == 40);

        let d: Duration<u64, 1, 1_000_000> = 40_075.nanos_at_least();
        assert!(d.ticks() == 41);

        let d: Duration<u64, 1, 1_000> = 4001.micros_at_least();
        assert!(d.ticks() == 5);

        let d: Duration<u64, 1, 10_000> = 100_000.micros();
        assert!(d.ticks() == 1_000);

        let d: Duration<u64, 1, 10_000> = 1.millis();
        assert!(d.ticks() == 10);

        let d: Duration<u64, 1, 10_000> = 1.secs();
        assert!(d.ticks() == 10_000);

        let d: Duration<u64, 1, 10_000> = 1.minutes();
        assert!(d.ticks() == 600_000);

        let d: Duration<u64, 1, 10_000> = 1.hours();
        assert!(d.ticks() == 36_000_000);

        let d = Duration::<u64, 1, 10_000>::millis(10);
        assert!(d.ticks() == 100);

        let d = Duration::<u64, 1, 10_000>::Hz(200);
        assert!(d.ticks() == 50);

        let d = Duration::<u32, 1, 1>::from_ticks(2);
        assert!(d.to_secs() == 2);
        assert!(d.to_nanos() == 2_000_000_000);

        let d = Duration::<u32, 1, 1_000_000_000>::from_ticks(2_000_000_000);
        assert!(d.to_secs() == 2);
        assert!(d.to_nanos() == 2_000_000_000);

        let d = Duration::<u64, 1, 10_000>::from_ticks(100);
        assert!(d.to_nanos() == 10_000_000);

        let d = Duration::<u64, 1, 10_000>::from_ticks(100);
        assert!(d.to_micros() == 10_000);

        let d = Duration::<u64, 1, 10_000>::from_ticks(100);
        assert!(d.to_millis() == 10);

        let d = Duration::<u64, 1, 10_000>::from_ticks(100_000);
        assert!(d.to_secs() == 10);

        let d = Duration::<u64, 1, 10_000>::from_ticks(1_800_000);
        assert!(d.to_minutes() == 3);

        let d = Duration::<u64, 1, 10_000>::from_ticks(180_000_000);
        assert!(d.to_hours() == 5);
    }

    #[test]
    fn duration_is_zero() {
        let d = Duration::<u64, 1, 1_000>::from_ticks(0);
        assert!(d.is_zero() == true);
        let d = Duration::<u64, 1, 1_000>::from_ticks(1);
        assert!(d.is_zero() == false);
        let d = Duration::<u32, 1, 1_000>::from_ticks(0);
        assert!(d.is_zero() == true);
        let d = Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(d.is_zero() == false);
    }

    ////////////////////////////////////////////////////////////////////////////////
    //
    // Instant tests
    //
    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn instant_compare_u32() {
        // Wrapping
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(1)
                > Instant::<u32, 1, 1_000>::from_ticks(u32::MAX)
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(u32::MAX - 1)
                < Instant::<u32, 1, 1_000>::from_ticks(u32::MAX)
        );

        // Non-wrapping
        assert!(Instant::<u32, 1, 1_000>::from_ticks(2) > Instant::<u32, 1, 1_000>::from_ticks(1));
        assert!(Instant::<u32, 1, 1_000>::from_ticks(2) >= Instant::<u32, 1, 1_000>::from_ticks(1));
        assert!(Instant::<u32, 1, 1_000>::from_ticks(1) >= Instant::<u32, 1, 1_000>::from_ticks(1));
        assert!(Instant::<u32, 1, 1_000>::from_ticks(1) < Instant::<u32, 1, 1_000>::from_ticks(2));
        assert!(Instant::<u32, 1, 1_000>::from_ticks(1) <= Instant::<u32, 1, 1_000>::from_ticks(1));
        assert!(Instant::<u32, 1, 1_000>::from_ticks(1) <= Instant::<u32, 1, 1_000>::from_ticks(2));
        assert!(Instant::<u32, 1, 1_000>::from_ticks(1) == Instant::<u32, 1, 1_000>::from_ticks(1));
        assert!(Instant::<u32, 1, 1_000>::from_ticks(1) != Instant::<u32, 1, 1_000>::from_ticks(2));

        // Checked duration since non-wrapping
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(1)
                .checked_duration_since(Instant::<u32, 1, 1_000>::from_ticks(1))
                == Some(Duration::<u32, 1, 1_000>::from_ticks(0))
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(2)
                .checked_duration_since(Instant::<u32, 1, 1_000>::from_ticks(1))
                == Some(Duration::<u32, 1, 1_000>::from_ticks(1))
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(2)
                .checked_duration_since(Instant::<u32, 1, 1_000>::from_ticks(3))
                == None
        );

        // Checked duration since wrapping
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(2)
                .checked_duration_since(Instant::<u32, 1, 1_000>::from_ticks(u32::MAX))
                == Some(Duration::<u32, 1, 1_000>::from_ticks(3))
        );
        assert!(
            Instant::<u32, 1, 1_000>::from_ticks(2)
                .checked_duration_since(Instant::<u32, 1, 1_000>::from_ticks(u32::MAX - 1))
                == Some(Duration::<u32, 1, 1_000>::from_ticks(4))
        );
    }

    #[test]
    fn instant_compare_u64() {
        // Wrapping
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(1)
                > Instant::<u64, 1, 1_000>::from_ticks(u64::MAX)
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(u64::MAX - 1)
                < Instant::<u64, 1, 1_000>::from_ticks(u64::MAX)
        );

        // Non-wrapping
        assert!(Instant::<u64, 1, 1_000>::from_ticks(2) > Instant::<u64, 1, 1_000>::from_ticks(1));
        assert!(Instant::<u64, 1, 1_000>::from_ticks(2) >= Instant::<u64, 1, 1_000>::from_ticks(1));
        assert!(Instant::<u64, 1, 1_000>::from_ticks(1) >= Instant::<u64, 1, 1_000>::from_ticks(1));
        assert!(Instant::<u64, 1, 1_000>::from_ticks(1) < Instant::<u64, 1, 1_000>::from_ticks(2));
        assert!(Instant::<u64, 1, 1_000>::from_ticks(1) <= Instant::<u64, 1, 1_000>::from_ticks(1));
        assert!(Instant::<u64, 1, 1_000>::from_ticks(1) <= Instant::<u64, 1, 1_000>::from_ticks(2));
        assert!(Instant::<u64, 1, 1_000>::from_ticks(1) == Instant::<u64, 1, 1_000>::from_ticks(1));
        assert!(Instant::<u64, 1, 1_000>::from_ticks(1) != Instant::<u64, 1, 1_000>::from_ticks(2));

        // Checked duration since non-wrapping
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(1)
                .checked_duration_since(Instant::<u64, 1, 1_000>::from_ticks(1))
                == Some(Duration::<u64, 1, 1_000>::from_ticks(0))
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(2)
                .checked_duration_since(Instant::<u64, 1, 1_000>::from_ticks(1))
                == Some(Duration::<u64, 1, 1_000>::from_ticks(1))
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(2)
                .checked_duration_since(Instant::<u64, 1, 1_000>::from_ticks(3))
                == None
        );

        // Checked duration since wrapping
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(2)
                .checked_duration_since(Instant::<u64, 1, 1_000>::from_ticks(u64::MAX))
                == Some(Duration::<u64, 1, 1_000>::from_ticks(3))
        );
        assert!(
            Instant::<u64, 1, 1_000>::from_ticks(2)
                .checked_duration_since(Instant::<u64, 1, 1_000>::from_ticks(u64::MAX - 1))
                == Some(Duration::<u64, 1, 1_000>::from_ticks(4))
        );
    }

    #[test]
    fn instant_duration_math_u32() {
        use crate::ExtU32;

        // Instant - Instant, Same base
        let diff: Duration<u32, 1, 1_000> =
            Instant::<u32, 1, 1_000>::from_ticks(10) - Instant::<u32, 1, 1_000>::from_ticks(1);
        assert!(diff == Duration::<u32, 1, 1_000>::from_ticks(9));

        // Instant +- Duration, Same base
        let sum: Instant<u32, 1, 1_000> =
            Instant::<u32, 1, 1_000>::from_ticks(10) + Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(sum == Instant::<u32, 1, 1_000>::from_ticks(11));

        let mut sum = Instant::<u32, 1, 1_000>::from_ticks(10);
        sum += Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(sum == Instant::<u32, 1, 1_000>::from_ticks(11));

        let diff: Instant<u32, 1, 1_000> =
            Instant::<u32, 1, 1_000>::from_ticks(10) - Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(diff == Instant::<u32, 1, 1_000>::from_ticks(9));

        let mut diff = Instant::<u32, 1, 1_000>::from_ticks(10);
        diff -= Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(diff == Instant::<u32, 1, 1_000>::from_ticks(9));

        // Instant +- Duration, Different base
        let sum: Instant<u32, 1, 10_000> = Instant::<u32, 1, 10_000>::from_ticks(10)
            + Duration::<u32, 1, 1_000>::from_ticks(1).convert();
        assert!(sum == Instant::<u32, 1, 10_000>::from_ticks(20));

        let mut sum = Instant::<u32, 1, 10_000>::from_ticks(10);
        sum += Duration::<u32, 1, 1_000>::from_ticks(1).convert();
        assert!(sum == Instant::<u32, 1, 10_000>::from_ticks(20));

        let diff: Instant<u32, 1, 10_000> = Instant::<u32, 1, 10_000>::from_ticks(10)
            - Duration::<u32, 1, 1_000>::from_ticks(1).convert();
        assert!(diff == Instant::<u32, 1, 10_000>::from_ticks(0));

        let mut diff = Instant::<u32, 1, 10_000>::from_ticks(10);
        diff -= Duration::<u32, 1, 1_000>::from_ticks(1).convert();
        assert!(diff == Instant::<u32, 1, 10_000>::from_ticks(0));

        // Instant + Extension trait
        let sum: Instant<u32, 1, 10_000> = Instant::<u32, 1, 10_000>::from_ticks(10) + 1.millis();
        assert!(sum == Instant::<u32, 1, 10_000>::from_ticks(20));

        // Instant - Extension trait
        let diff: Instant<u32, 1, 10_000> = Instant::<u32, 1, 10_000>::from_ticks(10) - 1.millis();
        assert!(diff == Instant::<u32, 1, 10_000>::from_ticks(0));
    }

    #[cfg(not(feature = "certified_subset"))]
    #[test]
    fn instant_duration_math_u64() {
        use crate::ExtU64;

        // Instant - Instant, Same base
        let diff: Duration<u64, 1, 1_000> =
            Instant::<u64, 1, 1_000>::from_ticks(10) - Instant::<u64, 1, 1_000>::from_ticks(1);
        assert!(diff == Duration::<u64, 1, 1_000>::from_ticks(9));

        // Instant +- Duration, Same base
        let sum: Instant<u64, 1, 1_000> =
            Instant::<u64, 1, 1_000>::from_ticks(10) + Duration::<u64, 1, 1_000>::from_ticks(1);
        assert!(sum == Instant::<u64, 1, 1_000>::from_ticks(11));

        let mut sum = Instant::<u64, 1, 1_000>::from_ticks(10);
        sum += Duration::<u64, 1, 1_000>::from_ticks(1);
        assert!(sum == Instant::<u64, 1, 1_000>::from_ticks(11));

        let diff: Instant<u64, 1, 1_000> =
            Instant::<u64, 1, 1_000>::from_ticks(10) - Duration::<u64, 1, 1_000>::from_ticks(1);
        assert!(diff == Instant::<u64, 1, 1_000>::from_ticks(9));

        let mut diff = Instant::<u64, 1, 1_000>::from_ticks(10);
        diff -= Duration::<u64, 1, 1_000>::from_ticks(1);
        assert!(diff == Instant::<u64, 1, 1_000>::from_ticks(9));

        // Instant +- Duration, Different base
        let sum: Instant<u64, 1, 10_000> = Instant::<u64, 1, 10_000>::from_ticks(10)
            + Duration::<u64, 1, 1_000>::from_ticks(1).convert();
        assert!(sum == Instant::<u64, 1, 10_000>::from_ticks(20));

        let mut sum = Instant::<u64, 1, 10_000>::from_ticks(10);
        sum += Duration::<u64, 1, 1_000>::from_ticks(1).convert();
        assert!(sum == Instant::<u64, 1, 10_000>::from_ticks(20));

        let diff: Instant<u64, 1, 10_000> = Instant::<u64, 1, 10_000>::from_ticks(10)
            - Duration::<u64, 1, 1_000>::from_ticks(1).convert();
        assert!(diff == Instant::<u64, 1, 10_000>::from_ticks(0));

        let mut diff = Instant::<u64, 1, 10_000>::from_ticks(10);
        diff -= Duration::<u64, 1, 1_000>::from_ticks(1).convert();
        assert!(diff == Instant::<u64, 1, 10_000>::from_ticks(0));

        // Instant + Extension trait
        let sum: Instant<u64, 1, 10_000> = Instant::<u64, 1, 10_000>::from_ticks(10) + 1.millis();
        assert!(sum == Instant::<u64, 1, 10_000>::from_ticks(20));

        // Instant - Extension trait
        let diff: Instant<u64, 1, 10_000> = Instant::<u64, 1, 10_000>::from_ticks(10) - 1.millis();
        assert!(diff == Instant::<u64, 1, 10_000>::from_ticks(0));
    }

    #[cfg(not(feature = "certified_subset"))]
    #[test]
    fn instant_duration_math_u64_u32() {
        // Instant +- Duration, Same base
        let sum: Instant<u64, 1, 1_000> =
            Instant::<u64, 1, 1_000>::from_ticks(10) + Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(sum == Instant::<u64, 1, 1_000>::from_ticks(11));

        let mut sum = Instant::<u64, 1, 1_000>::from_ticks(10);
        sum += Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(sum == Instant::<u64, 1, 1_000>::from_ticks(11));

        let diff: Instant<u64, 1, 1_000> =
            Instant::<u64, 1, 1_000>::from_ticks(10) - Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(diff == Instant::<u64, 1, 1_000>::from_ticks(9));

        let mut diff = Instant::<u64, 1, 1_000>::from_ticks(10);
        diff -= Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(diff == Instant::<u64, 1, 1_000>::from_ticks(9));

        // Instant +- Duration, Different base
        let sum: Instant<u64, 1, 10_000> = Instant::<u64, 1, 10_000>::from_ticks(10)
            + Duration::<u32, 1, 1_000>::from_ticks(1).convert();
        assert!(sum == Instant::<u64, 1, 10_000>::from_ticks(20));

        let mut sum = Instant::<u64, 1, 10_000>::from_ticks(10);
        sum += Duration::<u32, 1, 1_000>::from_ticks(1).convert();
        assert!(sum == Instant::<u64, 1, 10_000>::from_ticks(20));

        let diff: Instant<u64, 1, 10_000> = Instant::<u64, 1, 10_000>::from_ticks(10)
            - Duration::<u32, 1, 1_000>::from_ticks(1).convert();
        assert!(diff == Instant::<u64, 1, 10_000>::from_ticks(0));

        let mut diff = Instant::<u64, 1, 10_000>::from_ticks(10);
        diff -= Duration::<u32, 1, 1_000>::from_ticks(1).convert();
        assert!(diff == Instant::<u64, 1, 10_000>::from_ticks(0));
    }

    ////////////////////////////////////////////////////////////////////////////////
    //
    // Rate tests
    //
    ////////////////////////////////////////////////////////////////////////////////

    fn take_khz(r: Rate<u32, 1_000, 1>) -> Rate<u32, 1_000, 1> {
        r
    }

    #[test]
    fn rate_functions() {
        assert!(
            take_khz(Rate::<u32, 10_000, 1>::from_raw(1).convert())
                == Rate::<u32, 1_000, 1>::from_raw(10)
        );
    }

    #[test]
    fn rate_compare_u32() {
        // Same fraction
        assert!(Rate::<u32, 1_000, 1>::from_raw(2) > Rate::<u32, 1_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(2) >= Rate::<u32, 1_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(1) >= Rate::<u32, 1_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(1) < Rate::<u32, 1_000, 1>::from_raw(2));
        assert!(Rate::<u32, 1_000, 1>::from_raw(1) <= Rate::<u32, 1_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(1) <= Rate::<u32, 1_000, 1>::from_raw(2));
        assert!(Rate::<u32, 1_000, 1>::from_raw(1) == Rate::<u32, 1_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(1) != Rate::<u32, 1_000, 1>::from_raw(2));

        // Different fraction
        assert!(Rate::<u32, 1_000, 1>::from_raw(11) > Rate::<u32, 10_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(11) >= Rate::<u32, 10_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(10) >= Rate::<u32, 10_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(11) < Rate::<u32, 10_000, 1>::from_raw(2));
        assert!(Rate::<u32, 1_000, 1>::from_raw(1) <= Rate::<u32, 10_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(10) <= Rate::<u32, 10_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(10) == Rate::<u32, 10_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(9) != Rate::<u32, 10_000, 1>::from_raw(2));
    }

    #[test]
    fn rate_compare_u64() {
        // Same fraction
        assert!(Rate::<u64, 1_000, 1>::from_raw(2) > Rate::<u64, 1_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(2) >= Rate::<u64, 1_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(1) >= Rate::<u64, 1_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(1) < Rate::<u64, 1_000, 1>::from_raw(2));
        assert!(Rate::<u64, 1_000, 1>::from_raw(1) <= Rate::<u64, 1_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(1) <= Rate::<u64, 1_000, 1>::from_raw(2));
        assert!(Rate::<u64, 1_000, 1>::from_raw(1) == Rate::<u64, 1_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(1) != Rate::<u64, 1_000, 1>::from_raw(2));

        // Different fraction
        assert!(Rate::<u64, 1_000, 1>::from_raw(11) > Rate::<u64, 10_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(11) >= Rate::<u64, 10_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(10) >= Rate::<u64, 10_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(11) < Rate::<u64, 10_000, 1>::from_raw(2));
        assert!(Rate::<u64, 1_000, 1>::from_raw(1) <= Rate::<u64, 10_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(10) <= Rate::<u64, 10_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(10) == Rate::<u64, 10_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(9) != Rate::<u64, 10_000, 1>::from_raw(2));
    }

    #[test]
    fn rate_compare_u64_u32() {
        // Same fraction
        assert!(Rate::<u64, 1_000, 1>::from_raw(2) > Rate::<u32, 1_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(2) >= Rate::<u32, 1_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(1) >= Rate::<u32, 1_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(1) < Rate::<u32, 1_000, 1>::from_raw(2));
        assert!(Rate::<u64, 1_000, 1>::from_raw(1) <= Rate::<u32, 1_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(1) <= Rate::<u32, 1_000, 1>::from_raw(2));
        assert!(Rate::<u64, 1_000, 1>::from_raw(1) == Rate::<u32, 1_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(1) != Rate::<u32, 1_000, 1>::from_raw(2));

        // Different fraction
        assert!(Rate::<u64, 1_000, 1>::from_raw(11) > Rate::<u32, 10_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(11) >= Rate::<u32, 10_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(10) >= Rate::<u32, 10_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(11) < Rate::<u32, 10_000, 1>::from_raw(2));
        assert!(Rate::<u64, 1_000, 1>::from_raw(1) <= Rate::<u32, 10_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(10) <= Rate::<u32, 10_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(10) == Rate::<u32, 10_000, 1>::from_raw(1));
        assert!(Rate::<u64, 1_000, 1>::from_raw(9) != Rate::<u32, 10_000, 1>::from_raw(2));
    }

    #[test]
    fn rate_compare_u32_u64() {
        // Same fraction
        assert!(Rate::<u32, 1_000, 1>::from_raw(2) > Rate::<u64, 1_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(2) >= Rate::<u64, 1_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(1) >= Rate::<u64, 1_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(1) < Rate::<u64, 1_000, 1>::from_raw(2));
        assert!(Rate::<u32, 1_000, 1>::from_raw(1) <= Rate::<u64, 1_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(1) <= Rate::<u64, 1_000, 1>::from_raw(2));
        assert!(Rate::<u32, 1_000, 1>::from_raw(1) == Rate::<u64, 1_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(1) != Rate::<u64, 1_000, 1>::from_raw(2));

        // Different fraction
        assert!(Rate::<u32, 1_000, 1>::from_raw(11) > Rate::<u64, 10_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(11) >= Rate::<u64, 10_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(10) >= Rate::<u64, 10_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(11) < Rate::<u64, 10_000, 1>::from_raw(2));
        assert!(Rate::<u32, 1_000, 1>::from_raw(1) <= Rate::<u64, 10_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(10) <= Rate::<u64, 10_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(10) == Rate::<u64, 10_000, 1>::from_raw(1));
        assert!(Rate::<u32, 1_000, 1>::from_raw(9) != Rate::<u64, 10_000, 1>::from_raw(2));
    }

    #[test]
    fn rate_rate_math_u32() {
        use crate::RateExtU32;

        // Same base
        let sum: Rate<u32, 1_000, 1> =
            Rate::<u32, 1_000, 1>::from_raw(10) + Rate::<u32, 1_000, 1>::from_raw(1);
        assert!(sum == Rate::<u32, 1_000, 1>::from_raw(11));

        let diff: Rate<u32, 1_000, 1> =
            Rate::<u32, 1_000, 1>::from_raw(10) - Rate::<u32, 1_000, 1>::from_raw(1);
        assert!(diff == Rate::<u32, 1_000, 1>::from_raw(9));

        // Different base
        let sum: Rate<u32, 10_000, 1> =
            Rate::<u32, 10_000, 1>::from_raw(10) + Rate::<u32, 1_000, 1>::from_raw(10).convert();
        assert!(sum == Rate::<u32, 10_000, 1>::from_raw(11));

        let diff: Rate<u32, 10_000, 1> =
            Rate::<u32, 10_000, 1>::from_raw(10) - Rate::<u32, 1_000, 1>::from_raw(10).convert();
        assert!(diff == Rate::<u32, 10_000, 1>::from_raw(9));

        // Short hand vs u32 (should not need `.into()`)
        let sum = Rate::<u32, 1_000, 1>::from_raw(1) + 1.MHz();
        assert!(sum == Rate::<u32, 1_000, 1>::from_raw(1001));

        assert!(Rate::<u32, 1_000, 1>::from_raw(5) / Rate::<u32, 100, 1>::from_raw(2) == 25);

        assert!(Rate::<u32, 100, 1>::from_raw(2) / Rate::<u32, 1_000, 1>::from_raw(5) == 0);

        assert!(Rate::<u32, 100, 1>::from_raw(500) / Rate::<u32, 1_000, 1>::from_raw(5) == 10);
    }

    #[cfg(not(feature = "certified_subset"))]
    #[test]
    fn rate_rate_math_u64() {
        use crate::RateExtU64;

        // Same base
        let sum: Rate<u64, 1_000, 1> =
            Rate::<u64, 1_000, 1>::from_raw(10) + Rate::<u64, 1_000, 1>::from_raw(1);
        assert!(sum == Rate::<u64, 1_000, 1>::from_raw(11));

        let diff: Rate<u64, 1_000, 1> =
            Rate::<u64, 1_000, 1>::from_raw(10) - Rate::<u64, 1_000, 1>::from_raw(1);
        assert!(diff == Rate::<u64, 1_000, 1>::from_raw(9));

        // Different base
        let sum: Rate<u64, 10_000, 1> =
            Rate::<u64, 10_000, 1>::from_raw(10) + Rate::<u64, 1_000, 1>::from_raw(10).convert();
        assert!(sum == Rate::<u64, 10_000, 1>::from_raw(11));

        let diff: Rate<u64, 10_000, 1> =
            Rate::<u64, 10_000, 1>::from_raw(10) - Rate::<u64, 1_000, 1>::from_raw(10).convert();
        assert!(diff == Rate::<u64, 10_000, 1>::from_raw(9));

        // Short hand vs u64 (should not need `.into()`)
        let sum = Rate::<u64, 1_000, 1>::from_raw(1) + 1.MHz();
        assert!(sum == Rate::<u64, 1_000, 1>::from_raw(1001));

        assert!(Rate::<u64, 1_000, 1>::from_raw(5) / Rate::<u64, 100, 1>::from_raw(2) == 25);

        assert!(Rate::<u64, 100, 1>::from_raw(2) / Rate::<u64, 1_000, 1>::from_raw(5) == 0);

        assert!(Rate::<u64, 100, 1>::from_raw(500) / Rate::<u64, 1_000, 1>::from_raw(5) == 10);
    }

    #[test]
    fn rate_rate_math_u64_u32() {
        // Same base
        let sum: Rate<u64, 1_000, 1> =
            Rate::<u64, 1_000, 1>::from_raw(10) + Rate::<u32, 1_000, 1>::from_raw(1);
        assert!(sum == Rate::<u64, 1_000, 1>::from_raw(11));

        let diff: Rate<u64, 1_000, 1> =
            Rate::<u64, 1_000, 1>::from_raw(10) - Rate::<u32, 1_000, 1>::from_raw(1);
        assert!(diff == Rate::<u64, 1_000, 1>::from_raw(9));

        // Different base
        let sum: Rate<u64, 10_000, 1> =
            Rate::<u64, 10_000, 1>::from_raw(10) + Rate::<u32, 1_000, 1>::from_raw(10).convert();
        assert!(sum == Rate::<u64, 10_000, 1>::from_raw(11));

        let diff: Rate<u64, 10_000, 1> =
            Rate::<u64, 10_000, 1>::from_raw(10) - Rate::<u32, 1_000, 1>::from_raw(10).convert();
        assert!(diff == Rate::<u64, 10_000, 1>::from_raw(9));
    }

    #[test]
    fn rate_shorthands_u32() {
        use crate::RateExtU32;

        let r: Rate<u32, 1, 1> = 1.Hz();
        assert!(r.raw() == 1);

        let r: Rate<u32, 1, 1> = 1.kHz();
        assert!(r.raw() == 1_000);

        let r: Rate<u32, 1, 1> = 1.MHz();
        assert!(r.raw() == 1_000_000);

        let r = Rate::<u32, 1, 1>::kHz(20);
        assert!(r.raw() == 20_000);

        let r = Rate::<u32, 1, 1>::micros(50);
        assert!(r.raw() == 20_000);
    }

    #[cfg(not(feature = "certified_subset"))]
    #[test]
    fn rate_shorthands_u64() {
        use crate::RateExtU64;

        let r: Rate<u64, 1, 1> = 1.Hz();
        assert!(r.raw() == 1);

        let r: Rate<u64, 1, 1> = 1.kHz();
        assert!(r.raw() == 1_000);

        let r: Rate<u64, 1, 1> = 1.MHz();
        assert!(r.raw() == 1_000_000);

        let r = Rate::<u64, 1, 1>::kHz(20);
        assert!(r.raw() == 20_000);

        let r = Rate::<u64, 1, 1>::micros(50);
        assert!(r.raw() == 20_000);
    }

    #[test]
    fn rate_duration_conversion() {
        let r = Rate::<u32, 1_000, 1>::from_raw(1);
        let d: Duration<u32, 1, 1_000_000> = r.into_duration();
        assert!(d.ticks() == 1_000);
        let d2 = Duration::<u32, 1, 1_000_000>::from_rate(r);
        assert!(d2.ticks() == 1_000);

        let r = Rate::<u64, 1_000, 1>::from_raw(1);
        let d: Duration<u64, 1, 1_000_000> = r.into_duration();
        assert!(d.ticks() == 1_000);
        let d2 = Duration::<u64, 1, 1_000_000>::from_rate(r);
        assert!(d2.ticks() == 1_000);
    }

    #[test]
    fn rate_alias() {
        assert!(TimerRate::<u32, 1>::from_raw(1) == TimerRateU32::<1>::from_raw(1));
        assert!(TimerRate::<u64, 1>::from_raw(1) == TimerRateU64::<1>::from_raw(1));
        assert!(Hertz::<u32>::from_raw(1) == TimerRateU32::<1>::from_raw(1));
        assert!(HertzU32::from_raw(1) == TimerRateU32::<1>::from_raw(1));
        assert!(HertzU64::from_raw(1) == TimerRateU64::<1>::from_raw(1));
        assert!(Kilohertz::<u32>::from_raw(1) == TimerRateU32::<1_000>::from_raw(1));
        assert!(KilohertzU32::from_raw(1) == TimerRateU32::<1_000>::from_raw(1));
        assert!(KilohertzU64::from_raw(1) == TimerRateU64::<1_000>::from_raw(1));
        assert!(Megahertz::<u32>::from_raw(1) == TimerRateU32::<1_000_000>::from_raw(1));
        assert!(MegahertzU32::from_raw(1) == TimerRateU32::<1_000_000>::from_raw(1));
        assert!(MegahertzU64::from_raw(1) == TimerRateU64::<1_000_000>::from_raw(1));
    }

    ////////////////////////////////////////////////////////////////////////////////
    //
    // Rate coverage tests
    //
    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn rate_raw_u64() {
        let r = Rate::<u64, 1, 1_000>::from_raw(42);
        assert!(r.raw() == 42);
    }

    #[test]
    fn rate_checked_add_same_base_overflow_u32() {
        let r1 = Rate::<u32, 1, 1_000>::from_raw(u32::MAX);
        let r2 = Rate::<u32, 1, 1_000>::from_raw(1);
        assert!(r1.checked_add(r2).is_none());
    }

    #[test]
    fn rate_checked_add_same_base_overflow_u64() {
        let r1 = Rate::<u64, 1, 1_000>::from_raw(u64::MAX);
        let r2 = Rate::<u64, 1, 1_000>::from_raw(1);
        assert!(r1.checked_add(r2).is_none());
    }

    #[test]
    fn rate_checked_add_different_base() {
        // 100 Hz + 1 kHz in Hz base = 100 + 1000 = 1100
        let r1 = Rate::<u32, 1, 1>::from_raw(100);
        let r2 = Rate::<u32, 1_000, 1>::from_raw(1);
        let sum = r1.checked_add(r2);
        assert!(sum.is_some());
        assert!(sum.unwrap().raw() == 1100);

        // u64 variant
        let r1 = Rate::<u64, 1, 1>::from_raw(100);
        let r2 = Rate::<u64, 1_000, 1>::from_raw(1);
        let sum = r1.checked_add(r2);
        assert!(sum.is_some());
        assert!(sum.unwrap().raw() == 1100);
    }

    #[test]
    fn rate_checked_add_different_base_mul_overflow() {
        // self=Rate<_, 1, 1_000>, other=Rate<_, 1, 1>
        // LD_TIMES_RN = (DENOM * O_NOM) / DIVISOR = (1_000 * 1) / 1 = 1000
        // other.raw.checked_mul(1000) overflows when other.raw = MAX
        let r1 = Rate::<u32, 1, 1_000>::from_raw(0);
        let r2 = Rate::<u32, 1, 1>::from_raw(u32::MAX);
        assert!(r1.checked_add(r2).is_none());

        let r1 = Rate::<u64, 1, 1_000>::from_raw(0);
        let r2 = Rate::<u64, 1, 1>::from_raw(u64::MAX);
        assert!(r1.checked_add(r2).is_none());
    }

    #[test]
    fn rate_checked_add_different_base_add_overflow() {
        let r1 = Rate::<u32, 1, 1>::from_raw(u32::MAX);
        let r2 = Rate::<u32, 1_000, 1>::from_raw(1);
        assert!(r1.checked_add(r2).is_none());

        let r1 = Rate::<u64, 1, 1>::from_raw(u64::MAX);
        let r2 = Rate::<u64, 1_000, 1>::from_raw(1);
        assert!(r1.checked_add(r2).is_none());
    }

    #[test]
    fn rate_checked_sub_same_base_underflow_u32() {
        let r1 = Rate::<u32, 1, 1_000>::from_raw(0);
        let r2 = Rate::<u32, 1, 1_000>::from_raw(1);
        assert!(r1.checked_sub(r2).is_none());
    }

    #[test]
    fn rate_checked_sub_same_base_underflow_u64() {
        let r1 = Rate::<u64, 1, 1_000>::from_raw(0);
        let r2 = Rate::<u64, 1, 1_000>::from_raw(1);
        assert!(r1.checked_sub(r2).is_none());
    }

    #[test]
    fn rate_checked_sub_different_base() {
        // 1100 Hz - 1 kHz = 100 Hz
        let r1 = Rate::<u32, 1, 1>::from_raw(1100);
        let r2 = Rate::<u32, 1_000, 1>::from_raw(1);
        let diff = r1.checked_sub(r2);
        assert!(diff.is_some());
        assert!(diff.unwrap().raw() == 100);

        let r1 = Rate::<u64, 1, 1>::from_raw(1100);
        let r2 = Rate::<u64, 1_000, 1>::from_raw(1);
        let diff = r1.checked_sub(r2);
        assert!(diff.is_some());
        assert!(diff.unwrap().raw() == 100);
    }

    #[test]
    fn rate_checked_sub_different_base_mul_overflow() {
        let r1 = Rate::<u32, 1, 1_000>::from_raw(u32::MAX);
        let r2 = Rate::<u32, 1, 1>::from_raw(u32::MAX);
        assert!(r1.checked_sub(r2).is_none());

        let r1 = Rate::<u64, 1, 1_000>::from_raw(u64::MAX);
        let r2 = Rate::<u64, 1, 1>::from_raw(u64::MAX);
        assert!(r1.checked_sub(r2).is_none());
    }

    #[test]
    fn rate_checked_sub_different_base_sub_underflow() {
        let r1 = Rate::<u32, 1, 1>::from_raw(0);
        let r2 = Rate::<u32, 1_000, 1>::from_raw(1);
        assert!(r1.checked_sub(r2).is_none());

        let r1 = Rate::<u64, 1, 1>::from_raw(0);
        let r2 = Rate::<u64, 1_000, 1>::from_raw(1);
        assert!(r1.checked_sub(r2).is_none());
    }

    #[test]
    fn rate_const_partial_cmp_overflow() {
        let r1 = Rate::<u32, 1, 1>::from_raw(u32::MAX);
        let r2 = Rate::<u32, 1, 1_000>::from_raw(u32::MAX);
        assert!(r1.const_partial_cmp(r2).is_none());

        let r1 = Rate::<u64, 1, 1>::from_raw(u64::MAX);
        let r2 = Rate::<u64, 1, 1_000>::from_raw(u64::MAX);
        assert!(r1.const_partial_cmp(r2).is_none());
    }

    #[test]
    fn rate_const_eq_overflow() {
        let r1 = Rate::<u32, 1, 1>::from_raw(u32::MAX);
        let r2 = Rate::<u32, 1, 1_000>::from_raw(u32::MAX);
        assert!(!r1.const_eq(r2));

        let r1 = Rate::<u64, 1, 1>::from_raw(u64::MAX);
        let r2 = Rate::<u64, 1, 1_000>::from_raw(u64::MAX);
        assert!(!r1.const_eq(r2));
    }

    #[test]
    fn rate_const_try_from_overflow() {
        let r1 = Rate::<u32, 1, 1>::from_raw(u32::MAX);
        let r2 = Rate::<u32, 1, 1_000>::const_try_from(r1);
        assert!(r2.is_none());

        let r1 = Rate::<u64, 1, 1>::from_raw(u64::MAX);
        let r2 = Rate::<u64, 1, 1_000>::const_try_from(r1);
        assert!(r2.is_none());
    }

    #[test]
    fn rate_try_from_duration_zero() {
        let d = Duration::<u32, 1, 1_000>::from_ticks(0);
        let r = Rate::<u32, 1, 1>::try_from_duration(d);
        assert!(r.is_none());

        let d = Duration::<u64, 1, 1_000>::from_ticks(0);
        let r = Rate::<u64, 1, 1>::try_from_duration(d);
        assert!(r.is_none());
    }

    #[test]
    fn rate_to_and_from_hz_khz_mhz() {
        let r = Rate::<u32, 1_000_000, 1>::from_raw(1);
        assert!(r.to_Hz() == 1_000_000);
        assert!(r.to_kHz() == 1_000);
        assert!(r.to_MHz() == 1);

        let r = Rate::<u64, 1_000_000, 1>::from_raw(1);
        assert!(r.to_Hz() == 1_000_000);
        assert!(r.to_kHz() == 1_000);
        assert!(r.to_MHz() == 1);

        let r = Rate::<u32, 1, 1>::Hz(1_000);
        assert!(r.raw() == 1_000);
        let r = Rate::<u32, 1, 1>::kHz(1);
        assert!(r.raw() == 1_000);
        let r = Rate::<u32, 1, 1>::MHz(1);
        assert!(r.raw() == 1_000_000);

        let r = Rate::<u64, 1, 1>::Hz(1_000);
        assert!(r.raw() == 1_000);
        let r = Rate::<u64, 1, 1>::kHz(1);
        assert!(r.raw() == 1_000);
        let r = Rate::<u64, 1, 1>::MHz(1);
        assert!(r.raw() == 1_000_000);
    }

    #[test]
    #[should_panic(expected = "Convert failed!")]
    fn rate_convert_panic_u32() {
        let r = Rate::<u32, 1, 1>::from_raw(u32::MAX);
        let _: Rate<u32, 1, 1_000> = r.convert();
    }

    #[test]
    #[should_panic(expected = "Convert failed!")]
    fn rate_convert_panic_u64() {
        let r = Rate::<u64, 1, 1>::from_raw(u64::MAX);
        let _: Rate<u64, 1, 1_000> = r.convert();
    }

    #[test]
    #[should_panic(expected = "Into duration failed")]
    fn rate_into_duration_panic_u32() {
        let r = Rate::<u32, 1, 1>::from_raw(0);
        let _: Duration<u32, 1, 1_000> = r.into_duration();
    }

    #[test]
    #[should_panic(expected = "Into duration failed")]
    fn rate_into_duration_panic_u64() {
        let r = Rate::<u64, 1, 1>::from_raw(0);
        let _: Duration<u64, 1, 1_000> = r.into_duration();
    }

    #[test]
    #[should_panic(expected = "From duration failed")]
    fn rate_from_duration_panic_u32() {
        let d = Duration::<u32, 1, 1_000>::from_ticks(0);
        let _ = Rate::<u32, 1, 1>::from_duration(d);
    }

    #[test]
    #[should_panic(expected = "From duration failed")]
    fn rate_from_duration_panic_u64() {
        let d = Duration::<u64, 1, 1_000>::from_ticks(0);
        let _ = Rate::<u64, 1, 1>::from_duration(d);
    }

    #[test]
    fn rate_const_try_from_same_base() {
        let r1 = Rate::<u32, 1, 1_000>::from_raw(42);
        let r2 = Rate::<u32, 1, 1_000>::const_try_from(r1);
        assert!(r2.is_some());
        assert!(r2.unwrap().raw() == 42);

        let r1 = Rate::<u64, 1, 1_000>::from_raw(42);
        let r2 = Rate::<u64, 1, 1_000>::const_try_from(r1);
        assert!(r2.is_some());
        assert!(r2.unwrap().raw() == 42);
    }

    #[test]
    fn rate_ord_cmp() {
        let r1 = Rate::<u32, 1, 1_000>::from_raw(1);
        let r2 = Rate::<u32, 1, 1_000>::from_raw(2);
        assert!(r1.cmp(&r2) == core::cmp::Ordering::Less);
        assert!(r2.cmp(&r1) == core::cmp::Ordering::Greater);
        assert!(r1.cmp(&r1) == core::cmp::Ordering::Equal);

        let r1 = Rate::<u64, 1, 1_000>::from_raw(1);
        let r2 = Rate::<u64, 1, 1_000>::from_raw(2);
        assert!(r1.cmp(&r2) == core::cmp::Ordering::Less);
        assert!(r2.cmp(&r1) == core::cmp::Ordering::Greater);
        assert!(r1.cmp(&r1) == core::cmp::Ordering::Equal);
    }

    #[test]
    fn rate_add_assign() {
        let mut r1 = Rate::<u32, 1, 1_000>::from_raw(1);
        r1 += Rate::<u32, 1, 1_000>::from_raw(2);
        assert!(r1.raw() == 3);

        let mut r1 = Rate::<u64, 1, 1_000>::from_raw(1);
        r1 += Rate::<u64, 1, 1_000>::from_raw(2);
        assert!(r1.raw() == 3);
    }

    #[test]
    fn rate_u32_mul_rate() {
        let r = Rate::<u32, 1, 1_000>::from_raw(5);
        let r2 = 3u32 * r;
        assert!(r2.raw() == 15);

        let r = Rate::<u64, 1, 1_000>::from_raw(5);
        let r2 = 3u32 * r;
        assert!(r2.raw() == 15);
    }

    #[test]
    fn rate_mul_u32() {
        let r = Rate::<u32, 1, 1_000>::from_raw(5);
        let r2 = r * 3u32;
        assert!(r2.raw() == 15);

        let r = Rate::<u64, 1, 1_000>::from_raw(5);
        let r2 = r * 3u32;
        assert!(r2.raw() == 15);
    }

    #[test]
    fn rate_mul_assign() {
        let mut r = Rate::<u32, 1, 1_000>::from_raw(5);
        r *= 3u32;
        assert!(r.raw() == 15);

        let mut r = Rate::<u64, 1, 1_000>::from_raw(5);
        r *= 3u32;
        assert!(r.raw() == 15);
    }

    #[test]
    fn rate_div_u32() {
        let r = Rate::<u32, 1, 1_000>::from_raw(15);
        let r2 = r / 3u32;
        assert!(r2.raw() == 5);

        let r = Rate::<u64, 1, 1_000>::from_raw(15);
        let r2 = r / 3u32;
        assert!(r2.raw() == 5);
    }

    #[test]
    fn rate_div_assign() {
        let mut r = Rate::<u32, 1, 1_000>::from_raw(15);
        r /= 3u32;
        assert!(r.raw() == 5);

        let mut r = Rate::<u64, 1, 1_000>::from_raw(15);
        r /= 3u32;
        assert!(r.raw() == 5);
    }

    #[test]
    fn rate_nanos_and_millis() {
        let r = Rate::<u32, 1, 1>::nanos(1_000_000);
        assert!(r.raw() == 1_000);

        let r = Rate::<u64, 1, 1>::nanos(1_000_000);
        assert!(r.raw() == 1_000);

        let r = Rate::<u32, 1, 1>::millis(1);
        assert!(r.raw() == 1_000);

        let r = Rate::<u64, 1, 1>::millis(1);
        assert!(r.raw() == 1_000);
    }

    #[test]
    #[should_panic(expected = "Sub failed!")]
    fn rate_ops_sub_panic_u32() {
        let r1 = Rate::<u32, 1, 1_000>::from_raw(0);
        let r2 = Rate::<u32, 1, 1_000>::from_raw(1);
        let _ = r1 - r2;
    }

    #[test]
    #[should_panic(expected = "Sub failed!")]
    fn rate_ops_sub_panic_u64() {
        let r1 = Rate::<u64, 1, 1_000>::from_raw(0);
        let r2 = Rate::<u64, 1, 1_000>::from_raw(1);
        let _ = r1 - r2;
    }

    #[test]
    #[should_panic(expected = "Add failed!")]
    fn rate_ops_add_panic_u32() {
        let r1 = Rate::<u32, 1, 1_000>::from_raw(u32::MAX);
        let r2 = Rate::<u32, 1, 1_000>::from_raw(1);
        let _ = r1 + r2;
    }

    #[test]
    #[should_panic(expected = "Add failed!")]
    fn rate_ops_add_panic_u64() {
        let r1 = Rate::<u64, 1, 1_000>::from_raw(u64::MAX);
        let r2 = Rate::<u64, 1, 1_000>::from_raw(1);
        let _ = r1 + r2;
    }

    #[test]
    fn rate_try_from_duration_ok() {
        let d = Duration::<u32, 1, 1_000>::from_ticks(2);
        let r = Rate::<u32, 1, 1>::try_from_duration(d);
        assert!(r.is_some());
        assert!(r.unwrap().raw() == 500);

        let d = Duration::<u64, 1, 1_000>::from_ticks(2);
        let r = Rate::<u64, 1, 1>::try_from_duration(d);
        assert!(r.is_some());
        assert!(r.unwrap().raw() == 500);
    }

    #[test]
    fn duration_try_into_rate() {
        let d = Duration::<u32, 1, 1_000>::from_ticks(2);
        let r: Option<Rate<u32, 1, 1>> = d.try_into_rate();
        assert!(r.is_some());
        assert!(r.unwrap().raw() == 500);

        let d = Duration::<u64, 1, 1_000>::from_ticks(2);
        let r: Option<Rate<u64, 1, 1>> = d.try_into_rate();
        assert!(r.is_some());
        assert!(r.unwrap().raw() == 500);

        let d = Duration::<u32, 1, 1_000>::from_ticks(0);
        let r: Option<Rate<u32, 1, 1>> = d.try_into_rate();
        assert!(r.is_none());

        let d = Duration::<u64, 1, 1_000>::from_ticks(0);
        let r: Option<Rate<u64, 1, 1>> = d.try_into_rate();
        assert!(r.is_none());
    }

    #[test]
    fn duration_into_rate() {
        let d = Duration::<u32, 1, 1_000>::from_ticks(2);
        let r: Rate<u32, 1, 1> = d.into_rate();
        assert!(r.raw() == 500);

        let d = Duration::<u64, 1, 1_000>::from_ticks(2);
        let r: Rate<u64, 1, 1> = d.into_rate();
        assert!(r.raw() == 500);
    }

    #[test]
    #[should_panic(expected = "Into rate failed")]
    fn duration_into_rate_panic_u32() {
        let d = Duration::<u32, 1, 1_000>::from_ticks(0);
        let _: Rate<u32, 1, 1> = d.into_rate();
    }

    #[test]
    #[should_panic(expected = "Into rate failed")]
    fn duration_into_rate_panic_u64() {
        let d = Duration::<u64, 1, 1_000>::from_ticks(0);
        let _: Rate<u64, 1, 1> = d.into_rate();
    }

    #[test]
    fn duration_from_hz_khz_mhz() {
        // 1 Hz = 1_000 ms
        let d = Duration::<u32, 1, 1_000>::Hz(1);
        assert!(d.ticks() == 1_000);

        let d = Duration::<u64, 1, 1_000>::Hz(1);
        assert!(d.ticks() == 1_000);

        // 1 kHz = 1_000 us
        let d = Duration::<u32, 1, 1_000_000>::kHz(1);
        assert!(d.ticks() == 1_000);

        let d = Duration::<u64, 1, 1_000_000>::kHz(1);
        assert!(d.ticks() == 1_000);

        // 1 MHz = 1_000 ns
        let d = Duration::<u32, 1, 1_000_000_000>::MHz(1);
        assert!(d.ticks() == 1_000);

        let d = Duration::<u64, 1, 1_000_000_000>::MHz(1);
        assert!(d.ticks() == 1_000);
    }

    #[test]
    #[should_panic(expected = "From rate failed")]
    fn duration_from_rate_panic_u32() {
        let r = Rate::<u32, 1, 1>::from_raw(0);
        let _ = Duration::<u32, 1, 1_000>::from_rate(r);
    }

    #[test]
    #[should_panic(expected = "From rate failed")]
    fn duration_from_rate_panic_u64() {
        let r = Rate::<u64, 1, 1>::from_raw(0);
        let _ = Duration::<u64, 1, 1_000>::from_rate(r);
    }

    #[test]
    fn rate_from_u32_to_u64() {
        let r32 = Rate::<u32, 1, 1_000>::from_raw(42);
        let r64: Rate<u64, 1, 1_000> = r32.into();
        assert!(r64.raw() == 42);
    }

    #[test]
    fn rate_try_from_u64_to_u32() {
        use core::convert::TryFrom;

        let r64 = Rate::<u64, 1, 1_000>::from_raw(42);
        let r32 = Rate::<u32, 1, 1_000>::try_from(r64);
        assert!(r32.is_ok());
        assert!(r32.unwrap().raw() == 42);

        let r64_big = Rate::<u64, 1, 1_000>::from_raw(u64::MAX);
        let r32_fail = Rate::<u32, 1, 1_000>::try_from(r64_big);
        assert!(r32_fail.is_err());
    }

    #[test]
    fn rate_cross_type_sub_assign() {
        let mut r = Rate::<u64, 1, 1_000>::from_raw(10);
        r -= Rate::<u32, 1, 1_000>::from_raw(3);
        assert!(r.raw() == 7);
    }

    #[test]
    fn rate_cross_type_add_assign() {
        let mut r = Rate::<u64, 1, 1_000>::from_raw(10);
        r += Rate::<u32, 1, 1_000>::from_raw(3);
        assert!(r.raw() == 13);
    }

    #[test]
    #[should_panic(expected = "Sub failed!")]
    fn rate_cross_type_sub_panic() {
        let r1 = Rate::<u64, 1, 1_000>::from_raw(0);
        let r2 = Rate::<u32, 1, 1_000>::from_raw(1);
        let _ = r1 - r2;
    }

    #[test]
    #[should_panic(expected = "Add failed!")]
    fn rate_cross_type_add_panic() {
        let r1 = Rate::<u64, 1, 1_000>::from_raw(u64::MAX);
        let r2 = Rate::<u32, 1, 1_000>::from_raw(1);
        let _ = r1 + r2;
    }

    #[test]
    fn rate_cross_type_partial_cmp() {
        let r64 = Rate::<u64, 1, 1_000>::from_raw(10);
        let r32 = Rate::<u32, 1, 1_000>::from_raw(5);
        assert!(r64 > r32);
        assert!(!(r64 < r32));
        assert!(!(r64 == r32));

        let r32_2 = Rate::<u32, 1, 1_000>::from_raw(10);
        assert!(r64 == r32_2);
    }

    #[test]
    fn rate_cross_type_partial_cmp_u32_vs_u64() {
        let r32 = Rate::<u32, 1, 1_000>::from_raw(10);
        let r64 = Rate::<u64, 1, 1_000>::from_raw(5);
        assert!(r32 > r64);
        assert!(!(r32 < r64));
        assert!(!(r32 == r64));

        let r64_2 = Rate::<u64, 1, 1_000>::from_raw(10);
        assert!(r32 == r64_2);
    }

    ////////////////////////////////////////////////////////////////////////////////
    //
    // Per-instantiation Rate coverage tests
    //
    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn rate_checked_add_u32_1_1_diff_1000_1_mul_overflow() {
        // <u32,1,1>::checked_add::<1000,1> — LD_TIMES_RN=1000, mul overflow not hit
        let r1 = Rate::<u32, 1, 1>::from_raw(0);
        let r2 = Rate::<u32, 1_000, 1>::from_raw(u32::MAX);
        assert!(r1.checked_add(r2).is_none());
    }

    #[test]
    fn rate_checked_sub_u32_1_1_diff_1000_1_mul_overflow() {
        let r1 = Rate::<u32, 1, 1>::from_raw(u32::MAX);
        let r2 = Rate::<u32, 1_000, 1>::from_raw(u32::MAX);
        assert!(r1.checked_sub(r2).is_none());
    }

    #[test]
    fn rate_checked_add_u32_1_1000_diff_1_1_success() {
        // <u32,1,1000>::checked_add::<1,1> — only mul overflow hit, need success path
        let r1 = Rate::<u32, 1, 1_000>::from_raw(0);
        let r2 = Rate::<u32, 1, 1>::from_raw(1);
        let result = r1.checked_add(r2);
        assert!(result.is_some());

        // Add overflow after successful mul
        let r1 = Rate::<u32, 1, 1_000>::from_raw(u32::MAX);
        let r2 = Rate::<u32, 1, 1>::from_raw(1);
        assert!(r1.checked_add(r2).is_none());
    }

    #[test]
    fn rate_checked_sub_u32_1_1000_diff_1_1_success() {
        let r1 = Rate::<u32, 1, 1_000>::from_raw(u32::MAX);
        let r2 = Rate::<u32, 1, 1>::from_raw(1);
        let result = r1.checked_sub(r2);
        assert!(result.is_some());

        // Sub underflow after successful mul
        let r1 = Rate::<u32, 1, 1_000>::from_raw(0);
        let r2 = Rate::<u32, 1, 1>::from_raw(1);
        assert!(r1.checked_sub(r2).is_none());
    }

    #[test]
    fn rate_checked_add_u32_10000_1_same_base_overflow() {
        // <u32,10000,1>::checked_add::<10000,1> — same base, overflow not hit
        let r1 = Rate::<u32, 10_000, 1>::from_raw(u32::MAX);
        let r2 = Rate::<u32, 10_000, 1>::from_raw(1);
        assert!(r1.checked_add(r2).is_none());
    }

    #[test]
    fn rate_checked_sub_u32_10000_1_same_base_underflow() {
        let r1 = Rate::<u32, 10_000, 1>::from_raw(0);
        let r2 = Rate::<u32, 10_000, 1>::from_raw(1);
        assert!(r1.checked_sub(r2).is_none());
    }

    #[test]
    fn rate_checked_add_u32_1000_1_same_base_overflow() {
        // <u32,1000,1>::checked_add::<1000,1> — same base, overflow not hit
        let r1 = Rate::<u32, 1_000, 1>::from_raw(u32::MAX);
        let r2 = Rate::<u32, 1_000, 1>::from_raw(1);
        assert!(r1.checked_add(r2).is_none());
    }

    #[test]
    fn rate_checked_sub_u32_1000_1_same_base_underflow() {
        let r1 = Rate::<u32, 1_000, 1>::from_raw(0);
        let r2 = Rate::<u32, 1_000, 1>::from_raw(1);
        assert!(r1.checked_sub(r2).is_none());
    }

    #[test]
    fn rate_checked_add_u64_1_1_diff_1000_1_mul_overflow() {
        // <u64,1,1>::checked_add::<1000,1> — mul overflow not hit
        let r1 = Rate::<u64, 1, 1>::from_raw(0);
        let r2 = Rate::<u64, 1_000, 1>::from_raw(u64::MAX);
        assert!(r1.checked_add(r2).is_none());
    }

    #[test]
    fn rate_checked_sub_u64_1_1_diff_1000_1_mul_overflow() {
        let r1 = Rate::<u64, 1, 1>::from_raw(u64::MAX);
        let r2 = Rate::<u64, 1_000, 1>::from_raw(u64::MAX);
        assert!(r1.checked_sub(r2).is_none());
    }

    #[test]
    fn rate_checked_add_u64_1_1000_diff_1_1_success() {
        // <u64,1,1000>::checked_add::<1,1> — only mul overflow hit, need success path
        let r1 = Rate::<u64, 1, 1_000>::from_raw(0);
        let r2 = Rate::<u64, 1, 1>::from_raw(1);
        let result = r1.checked_add(r2);
        assert!(result.is_some());

        // Add overflow after successful mul
        let r1 = Rate::<u64, 1, 1_000>::from_raw(u64::MAX);
        let r2 = Rate::<u64, 1, 1>::from_raw(1);
        assert!(r1.checked_add(r2).is_none());
    }

    #[test]
    fn rate_checked_sub_u64_1_1000_diff_1_1_success() {
        let r1 = Rate::<u64, 1, 1_000>::from_raw(u64::MAX);
        let r2 = Rate::<u64, 1, 1>::from_raw(1);
        let result = r1.checked_sub(r2);
        assert!(result.is_some());

        let r1 = Rate::<u64, 1, 1_000>::from_raw(0);
        let r2 = Rate::<u64, 1, 1>::from_raw(1);
        assert!(r1.checked_sub(r2).is_none());
    }

    #[test]
    fn rate_checked_add_u64_10000_1_same_base_overflow() {
        let r1 = Rate::<u64, 10_000, 1>::from_raw(u64::MAX);
        let r2 = Rate::<u64, 10_000, 1>::from_raw(1);
        assert!(r1.checked_add(r2).is_none());
    }

    #[test]
    fn rate_checked_sub_u64_10000_1_same_base_underflow() {
        let r1 = Rate::<u64, 10_000, 1>::from_raw(0);
        let r2 = Rate::<u64, 10_000, 1>::from_raw(1);
        assert!(r1.checked_sub(r2).is_none());
    }

    #[test]
    fn rate_checked_add_u64_1000_1_same_base_overflow() {
        let r1 = Rate::<u64, 1_000, 1>::from_raw(u64::MAX);
        let r2 = Rate::<u64, 1_000, 1>::from_raw(1);
        assert!(r1.checked_add(r2).is_none());
    }

    #[test]
    fn rate_checked_sub_u64_1000_1_same_base_underflow() {
        let r1 = Rate::<u64, 1_000, 1>::from_raw(0);
        let r2 = Rate::<u64, 1_000, 1>::from_raw(1);
        assert!(r1.checked_sub(r2).is_none());
    }

    ////////////////////////////////////////////////////////////////////////////////
    //
    // Additional coverage tests
    //
    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn duration_shorthand_to_and_from_u32() {
        let d = Duration::<u32, 1, 1>::from_ticks(2);
        assert!(d.to_secs() == 2);
        assert!(d.to_nanos() == 2_000_000_000);

        let d = Duration::<u32, 1, 10_000>::from_ticks(100);
        assert!(d.to_nanos() == 10_000_000);
        assert!(d.to_micros() == 10_000);
        assert!(d.to_millis() == 10);

        let d = Duration::<u32, 1, 10_000>::from_ticks(100_000);
        assert!(d.to_secs() == 10);

        let d = Duration::<u32, 1, 10_000>::from_ticks(1_800_000);
        assert!(d.to_minutes() == 3);

        let d = Duration::<u32, 1, 10_000>::from_ticks(180_000_000);
        assert!(d.to_hours() == 5);

        let d = Duration::<u32, 1, 10_000>::millis(10);
        assert!(d.ticks() == 100);

        let d = Duration::<u32, 1, 10_000>::nanos(100_000_000);
        assert!(d.ticks() == 1_000);

        let d = Duration::<u32, 1, 10_000>::micros(100_000);
        assert!(d.ticks() == 1_000);

        let d = Duration::<u32, 1, 10_000>::secs(1);
        assert!(d.ticks() == 10_000);

        let d = Duration::<u32, 1, 10_000>::minutes(1);
        assert!(d.ticks() == 600_000);

        let d = Duration::<u32, 1, 10_000>::hours(1);
        assert!(d.ticks() == 36_000_000);
    }

    #[test]
    fn duration_shorthand_at_least_u32() {
        let d = Duration::<u32, 1, 1_000_000>::nanos_at_least(40_000);
        assert!(d.ticks() == 40);

        let d = Duration::<u32, 1, 1_000_000>::nanos_at_least(40_075);
        assert!(d.ticks() == 41);

        let d = Duration::<u32, 1, 1_000>::micros_at_least(4001);
        assert!(d.ticks() == 5);

        let d = Duration::<u32, 1, 1_000>::micros_at_least(4000);
        assert!(d.ticks() == 4);

        let d = Duration::<u32, 1, 1>::millis_at_least(1001);
        assert!(d.ticks() == 2);

        let d = Duration::<u32, 1, 1>::millis_at_least(1000);
        assert!(d.ticks() == 1);

        let d = Duration::<u32, 60, 1>::secs_at_least(61);
        assert!(d.ticks() == 2);

        let d = Duration::<u32, 60, 1>::secs_at_least(60);
        assert!(d.ticks() == 1);

        let d = Duration::<u32, 3600, 1>::minutes_at_least(61);
        assert!(d.ticks() == 2);

        let d = Duration::<u32, 3600, 1>::minutes_at_least(60);
        assert!(d.ticks() == 1);

        let d = Duration::<u32, 3600, 1>::hours_at_least(1);
        assert!(d.ticks() == 1);

        // Hit the ceiling-rounding branch: 1 hour in 7-second units = 3600/7 = 514.28 -> 515
        let d = Duration::<u32, 7, 1>::hours_at_least(1);
        assert!(d.ticks() == 515);
        // Hit the exact branch for the same instantiation
        let d = Duration::<u32, 7, 1>::hours_at_least(0);
        assert!(d.ticks() == 0);
    }

    #[test]
    fn duration_shorthand_to_and_from_u64() {
        let d = Duration::<u64, 1, 1>::from_ticks(2);
        assert!(d.to_secs() == 2);
        assert!(d.to_nanos() == 2_000_000_000);

        let d = Duration::<u64, 1, 10_000>::from_ticks(100);
        assert!(d.to_nanos() == 10_000_000);
        assert!(d.to_micros() == 10_000);
        assert!(d.to_millis() == 10);

        let d = Duration::<u64, 1, 10_000>::from_ticks(100_000);
        assert!(d.to_secs() == 10);

        let d = Duration::<u64, 1, 10_000>::from_ticks(1_800_000);
        assert!(d.to_minutes() == 3);

        let d = Duration::<u64, 1, 10_000>::from_ticks(180_000_000);
        assert!(d.to_hours() == 5);

        let d = Duration::<u64, 1, 10_000>::millis(10);
        assert!(d.ticks() == 100);

        let d = Duration::<u64, 1, 10_000>::nanos(100_000_000);
        assert!(d.ticks() == 1_000);

        let d = Duration::<u64, 1, 10_000>::micros(100_000);
        assert!(d.ticks() == 1_000);

        let d = Duration::<u64, 1, 10_000>::secs(1);
        assert!(d.ticks() == 10_000);

        let d = Duration::<u64, 1, 10_000>::minutes(1);
        assert!(d.ticks() == 600_000);

        let d = Duration::<u64, 1, 10_000>::hours(1);
        assert!(d.ticks() == 36_000_000);
    }

    #[test]
    fn duration_shorthand_at_least_u64() {
        let d = Duration::<u64, 1, 1_000_000>::nanos_at_least(40_000);
        assert!(d.ticks() == 40);

        let d = Duration::<u64, 1, 1_000_000>::nanos_at_least(40_075);
        assert!(d.ticks() == 41);

        let d = Duration::<u64, 1, 1_000>::micros_at_least(4001);
        assert!(d.ticks() == 5);

        let d = Duration::<u64, 1, 1_000>::micros_at_least(4000);
        assert!(d.ticks() == 4);

        let d = Duration::<u64, 1, 1>::millis_at_least(1001);
        assert!(d.ticks() == 2);

        let d = Duration::<u64, 1, 1>::millis_at_least(1000);
        assert!(d.ticks() == 1);

        let d = Duration::<u64, 60, 1>::secs_at_least(61);
        assert!(d.ticks() == 2);

        let d = Duration::<u64, 60, 1>::secs_at_least(60);
        assert!(d.ticks() == 1);

        let d = Duration::<u64, 3600, 1>::minutes_at_least(61);
        assert!(d.ticks() == 2);

        let d = Duration::<u64, 3600, 1>::minutes_at_least(60);
        assert!(d.ticks() == 1);

        let d = Duration::<u64, 3600, 1>::hours_at_least(1);
        assert!(d.ticks() == 1);

        // Hit the ceiling-rounding branch: 1 hour in 7-second units = 3600/7 = 514.28 -> 515
        let d = Duration::<u64, 7, 1>::hours_at_least(1);
        assert!(d.ticks() == 515);
        let d = Duration::<u64, 7, 1>::hours_at_least(0);
        assert!(d.ticks() == 0);
    }

    #[test]
    fn duration_ext_u32_trait() {
        use crate::ExtU32;

        let d: Duration<u32, 1, 10_000> = 100_000_000u32.nanos();
        assert!(d.ticks() == 1_000);

        let d: Duration<u32, 1, 10_000> = 100_000u32.micros();
        assert!(d.ticks() == 1_000);

        let d: Duration<u32, 1, 10_000> = 1u32.secs();
        assert!(d.ticks() == 10_000);

        let d: Duration<u32, 1, 10_000> = 1u32.minutes();
        assert!(d.ticks() == 600_000);

        let d: Duration<u32, 1, 10_000> = 1u32.hours();
        assert!(d.ticks() == 36_000_000);
    }

    #[test]
    fn duration_checked_add_different_base() {
        // Happy path: different base addition
        let d1 = Duration::<u32, 1, 10_000>::from_ticks(10);
        let d2 = Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(d1.checked_add(d2) == Some(Duration::<u32, 1, 10_000>::from_ticks(20)));

        // checked_mul overflow: LD_TIMES_RN=1_000_000 so u32::MAX * 1_000_000 overflows
        let d1 = Duration::<u32, 1, 1_000_000>::from_ticks(1);
        let d2 = Duration::<u32, 1, 1>::from_ticks(u32::MAX);
        assert!(d1.checked_add(d2) == None);

        // checked_add overflow: conversion succeeds but sum doesn't fit
        let d1 = Duration::<u32, 1, 10_000>::from_ticks(u32::MAX);
        let d2 = Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(d1.checked_add(d2) == None);
    }

    #[test]
    fn duration_checked_sub_different_base() {
        // Happy path: different base subtraction
        let d1 = Duration::<u32, 1, 10_000>::from_ticks(20);
        let d2 = Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(d1.checked_sub(d2) == Some(Duration::<u32, 1, 10_000>::from_ticks(10)));

        // checked_sub underflow
        let d1 = Duration::<u32, 1, 10_000>::from_ticks(0);
        let d2 = Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(d1.checked_sub(d2) == None);

        // checked_mul overflow: LD_TIMES_RN=1_000_000 so u32::MAX * 1_000_000 overflows
        let d1 = Duration::<u32, 1, 1_000_000>::from_ticks(1);
        let d2 = Duration::<u32, 1, 1>::from_ticks(u32::MAX);
        assert!(d1.checked_sub(d2) == None);
    }

    #[test]
    fn duration_const_partial_cmp_overflow() {
        let d1 = Duration::<u32, 1, 1_000>::from_ticks(u32::MAX);
        let d2 = Duration::<u32, 1, 10_000>::from_ticks(u32::MAX);
        assert!(d1.const_partial_cmp(d2) == None);
    }

    #[test]
    fn duration_const_partial_cmp_u32_1_10000_vs_1_1000_overflow() {
        // Instantiation: <u32,1,10000>::const_partial_cmp::<1,1000> — overflow (line 228) not hit.
        let d1 = Duration::<u32, 1, 10_000>::from_ticks(u32::MAX);
        let d2 = Duration::<u32, 1, 1_000>::from_ticks(u32::MAX);
        assert!(d1.const_partial_cmp(d2).is_none());
    }

    #[test]
    fn duration_const_partial_cmp_u32_1_1000_vs_1_10000_success() {
        // Instantiation: <u32,1,1000>::const_partial_cmp::<1,10000> — success (line 226) not hit.
        let d1 = Duration::<u32, 1, 1_000>::from_ticks(1);
        let d2 = Duration::<u32, 1, 10_000>::from_ticks(1);
        assert!(d1.const_partial_cmp(d2) == Some(core::cmp::Ordering::Greater));
    }

    #[test]
    fn duration_const_partial_cmp_u64_1_10000_vs_1_1000_overflow() {
        // Instantiation: <u64,1,10000>::const_partial_cmp::<1,1000> — overflow (line 228) not hit.
        let d1 = Duration::<u64, 1, 10_000>::from_ticks(u64::MAX);
        let d2 = Duration::<u64, 1, 1_000>::from_ticks(u64::MAX);
        assert!(d1.const_partial_cmp(d2).is_none());
    }

    #[test]
    fn duration_const_eq_overflow() {
        let d1 = Duration::<u32, 1, 1_000>::from_ticks(u32::MAX);
        let d2 = Duration::<u32, 1, 10_000>::from_ticks(u32::MAX);
        assert!(!d1.const_eq(d2));
    }

    #[test]
    fn duration_const_try_from_same_base() {
        let d1 = Duration::<u32, 1, 1_000>::from_ticks(42);
        let d2 = Duration::<u32, 1, 1_000>::const_try_from(d1);
        assert!(d2.unwrap().ticks() == 42);
    }

    #[test]
    fn duration_const_try_from_overflow() {
        let d1 = Duration::<u32, 1, 100>::from_ticks(u32::MAX);
        let d2: Option<Duration<u32, 1, 1_000>> = d1.const_try_into();
        assert!(d2 == None);

        let d1 = Duration::<u32, 1, 100>::from_ticks(u32::MAX);
        let d2: Option<Duration<u32, 1, 1_000_000>> = d1.const_try_into();
        assert!(d2 == None);
    }

    #[test]
    fn duration_integer_mul_duration() {
        let d = Duration::<u32, 1, 1_000>::from_ticks(10);
        let result = 3u32 * d;
        assert!(result == Duration::<u32, 1, 1_000>::from_ticks(30));
    }

    #[cfg(not(feature = "certified_subset"))]
    #[test]
    fn duration_try_from_u64_to_u32() {
        use core::convert::TryFrom;

        let d64 = Duration::<u64, 1, 1_000>::from_ticks(42);
        let d32 = Duration::<u32, 1, 1_000>::try_from(d64);
        assert!(d32.unwrap().ticks() == 42);

        let d64_big = Duration::<u64, 1, 1_000>::from_ticks(u64::MAX);
        let d32_big = Duration::<u32, 1, 1_000>::try_from(d64_big);
        assert!(d32_big.is_err());
    }

    #[test]
    fn instant_ticks() {
        let i = Instant::<u32, 1, 1_000>::from_ticks(123);
        assert!(i.ticks() == 123);

        let i = Instant::<u64, 1, 1_000>::from_ticks(456);
        assert!(i.ticks() == 456);
    }

    #[test]
    fn instant_duration_since_epoch() {
        let i = Instant::<u32, 1, 1_000>::from_ticks(11);
        assert!(i.duration_since_epoch().ticks() == 11);

        let i = Instant::<u64, 1, 1_000>::from_ticks(99);
        assert!(i.duration_since_epoch().ticks() == 99);
    }

    #[test]
    fn instant_checked_sub_duration_different_base() {
        let i = Instant::<u32, 1, 10_000>::from_ticks(20);
        let d = Duration::<u32, 1, 1_000>::from_ticks(1);
        let result = i.checked_sub_duration(d);
        assert!(result.unwrap().ticks() == 10);

        // checked_mul overflow: LD_TIMES_RN=1_000_000, u32::MAX * 1_000_000 overflows
        let i = Instant::<u32, 1, 1_000_000>::from_ticks(10);
        let d = Duration::<u32, 1, 1>::from_ticks(u32::MAX);
        assert!(i.checked_sub_duration(d) == None);
    }

    #[test]
    fn instant_checked_add_duration_different_base() {
        let i = Instant::<u32, 1, 10_000>::from_ticks(10);
        let d = Duration::<u32, 1, 1_000>::from_ticks(1);
        let result = i.checked_add_duration(d);
        assert!(result.unwrap().ticks() == 20);

        // checked_mul overflow: LD_TIMES_RN=1_000_000, u32::MAX * 1_000_000 overflows
        let i = Instant::<u32, 1, 1_000_000>::from_ticks(10);
        let d = Duration::<u32, 1, 1>::from_ticks(u32::MAX);
        assert!(i.checked_add_duration(d) == None);
    }

    #[test]
    fn duration_checked_add_same_base_overflow() {
        let d1 = Duration::<u32, 1, 1_000>::from_ticks(u32::MAX);
        let d2 = Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(d1.checked_add(d2) == None);
    }

    #[test]
    fn duration_checked_sub_same_base_underflow() {
        let d1 = Duration::<u32, 1, 1_000>::from_ticks(0);
        let d2 = Duration::<u32, 1, 1_000>::from_ticks(1);
        assert!(d1.checked_sub(d2) == None);
    }

    #[test]
    fn duration_checked_add_different_base_u64() {
        let d1 = Duration::<u64, 1, 10_000>::from_ticks(10);
        let d2 = Duration::<u64, 1, 1_000>::from_ticks(1);
        assert!(d1.checked_add(d2) == Some(Duration::<u64, 1, 10_000>::from_ticks(20)));

        // checked_mul overflow on u64
        let d1 = Duration::<u64, 1, 1_000_000>::from_ticks(1);
        let d2 = Duration::<u64, 1, 1>::from_ticks(u64::MAX);
        assert!(d1.checked_add(d2) == None);

        // checked_add overflow: conversion ok but sum overflows u64
        let d1 = Duration::<u64, 1, 10_000>::from_ticks(u64::MAX);
        let d2 = Duration::<u64, 1, 1_000>::from_ticks(1);
        assert!(d1.checked_add(d2) == None);
    }

    #[test]
    fn duration_checked_sub_different_base_u64() {
        let d1 = Duration::<u64, 1, 10_000>::from_ticks(20);
        let d2 = Duration::<u64, 1, 1_000>::from_ticks(1);
        assert!(d1.checked_sub(d2) == Some(Duration::<u64, 1, 10_000>::from_ticks(10)));

        // checked_sub underflow
        let d1 = Duration::<u64, 1, 10_000>::from_ticks(0);
        let d2 = Duration::<u64, 1, 1_000>::from_ticks(1);
        assert!(d1.checked_sub(d2) == None);

        // checked_mul overflow on u64
        let d1 = Duration::<u64, 1, 1_000_000>::from_ticks(1);
        let d2 = Duration::<u64, 1, 1>::from_ticks(u64::MAX);
        assert!(d1.checked_sub(d2) == None);
    }

    // Per-instantiation coverage: ensure every monomorphized checked_add/checked_sub
    // has all branches covered.

    #[test]
    fn duration_checked_add_u32_1_10000_same_base_overflow() {
        // Instantiation: <u32, 1, 10000>::checked_add::<1, 10000> — same base overflow
        let d1 = Duration::<u32, 1, 10_000>::from_ticks(u32::MAX);
        let d2 = Duration::<u32, 1, 10_000>::from_ticks(1);
        assert!(d1.checked_add(d2).is_none());
    }

    #[test]
    fn duration_checked_sub_u32_1_10000_same_base_underflow() {
        let d1 = Duration::<u32, 1, 10_000>::from_ticks(0);
        let d2 = Duration::<u32, 1, 10_000>::from_ticks(1);
        assert!(d1.checked_sub(d2).is_none());
    }

    #[test]
    fn duration_checked_add_u32_1_10000_diff_base_mul_overflow() {
        // Instantiation: <u32, 1, 10000>::checked_add::<1, 1000> — mul overflow
        // LD_TIMES_RN for <1,10000,1,1000>: (10000 * 1) / gcd(10000*1, 1000*1)= 10000/1000=10
        // Need other.ticks * 10 > u32::MAX => other.ticks > u32::MAX/10
        let d1 = Duration::<u32, 1, 10_000>::from_ticks(0);
        let d2 = Duration::<u32, 1, 1_000>::from_ticks(u32::MAX);
        assert!(d1.checked_add(d2).is_none());
    }

    #[test]
    fn duration_checked_sub_u32_1_10000_diff_base_mul_overflow() {
        let d1 = Duration::<u32, 1, 10_000>::from_ticks(u32::MAX);
        let d2 = Duration::<u32, 1, 1_000>::from_ticks(u32::MAX);
        assert!(d1.checked_sub(d2).is_none());
    }

    #[test]
    fn duration_checked_add_u32_1_1000_diff_base() {
        // Instantiation: <u32, 1, 1000>::checked_add::<1, 1000> only had same-base calls.
        // Force a different-base call to cover lines 117-130 for this instantiation.
        // We need a different-base pair that still resolves to <u32, 1, 1000> as self.
        let d1 = Duration::<u32, 1, 1_000>::from_ticks(5);
        let d2 = Duration::<u32, 1, 10_000>::from_ticks(10);
        let result = d1.checked_add(d2);
        assert!(result.is_some());
        assert!(result.unwrap().ticks() == 6);

        // Overflow in the add after conversion
        let d1 = Duration::<u32, 1, 1_000>::from_ticks(u32::MAX);
        let d2 = Duration::<u32, 1, 10_000>::from_ticks(10);
        assert!(d1.checked_add(d2).is_none());

        // Overflow in the mul
        let d1 = Duration::<u32, 1, 1_000>::from_ticks(0);
        let d2 = Duration::<u32, 10_000, 1>::from_ticks(u32::MAX);
        assert!(d1.checked_add(d2).is_none());
    }

    #[test]
    fn duration_checked_sub_u32_1_1000_diff_base() {
        let d1 = Duration::<u32, 1, 1_000>::from_ticks(6);
        let d2 = Duration::<u32, 1, 10_000>::from_ticks(10);
        let result = d1.checked_sub(d2);
        assert!(result.is_some());
        assert!(result.unwrap().ticks() == 5);

        // Underflow after conversion
        let d1 = Duration::<u32, 1, 1_000>::from_ticks(0);
        let d2 = Duration::<u32, 1, 10_000>::from_ticks(10);
        assert!(d1.checked_sub(d2).is_none());

        // Overflow in the mul
        let d1 = Duration::<u32, 1, 1_000>::from_ticks(u32::MAX);
        let d2 = Duration::<u32, 10_000, 1>::from_ticks(u32::MAX);
        assert!(d1.checked_sub(d2).is_none());
    }

    #[test]
    fn duration_checked_add_u32_1_1000000_diff_base_success() {
        // Instantiation: <u32, 1, 1000000>::checked_add::<1, 1> — only mul overflow was hit.
        // Need a successful conversion path (lines 121-124).
        let d1 = Duration::<u32, 1, 1_000_000>::from_ticks(0);
        let d2 = Duration::<u32, 1, 1>::from_ticks(1);
        let result = d1.checked_add(d2);
        assert!(result.is_some());
        assert!(result.unwrap().ticks() == 1_000_000);

        // Add overflow after successful conversion
        let d1 = Duration::<u32, 1, 1_000_000>::from_ticks(u32::MAX);
        let d2 = Duration::<u32, 1, 1>::from_ticks(1);
        assert!(d1.checked_add(d2).is_none());
    }

    #[test]
    fn duration_checked_sub_u32_1_1000000_diff_base_success() {
        let d1 = Duration::<u32, 1, 1_000_000>::from_ticks(1_000_001);
        let d2 = Duration::<u32, 1, 1>::from_ticks(1);
        let result = d1.checked_sub(d2);
        assert!(result.is_some());
        assert!(result.unwrap().ticks() == 1);
    }

    #[test]
    fn duration_checked_add_u64_1_10000_diff_base_mul_overflow() {
        // Instantiation: <u64, 1, 10000>::checked_add::<1, 1000> — mul overflow not hit.
        let d1 = Duration::<u64, 1, 10_000>::from_ticks(0);
        let d2 = Duration::<u64, 1, 1_000>::from_ticks(u64::MAX);
        assert!(d1.checked_add(d2).is_none());
    }

    #[test]
    fn duration_checked_sub_u64_1_10000_diff_base_mul_overflow() {
        let d1 = Duration::<u64, 1, 10_000>::from_ticks(u64::MAX);
        let d2 = Duration::<u64, 1, 1_000>::from_ticks(u64::MAX);
        assert!(d1.checked_sub(d2).is_none());
    }

    #[test]
    fn duration_checked_add_u32_1_1000_diff_base_10000_1_success() {
        // Instantiation: <u32,1,1000>::checked_add::<10000,1> — only mul overflow hit.
        // Need success path (lines 121-124).
        let d1 = Duration::<u32, 1, 1_000>::from_ticks(0);
        let d2 = Duration::<u32, 10_000, 1>::from_ticks(1);
        let result = d1.checked_add(d2);
        assert!(result.is_some());

        // Also hit add overflow after successful mul (lines 125-126)
        let d1 = Duration::<u32, 1, 1_000>::from_ticks(u32::MAX);
        let d2 = Duration::<u32, 10_000, 1>::from_ticks(1);
        assert!(d1.checked_add(d2).is_none());
    }

    #[test]
    fn duration_checked_sub_u32_1_1000_diff_base_10000_1_success() {
        let d1 = Duration::<u32, 1, 1_000>::from_ticks(u32::MAX);
        let d2 = Duration::<u32, 10_000, 1>::from_ticks(1);
        let result = d1.checked_sub(d2);
        assert!(result.is_some());

        // Sub underflow after successful mul
        let d1 = Duration::<u32, 1, 1_000>::from_ticks(0);
        let d2 = Duration::<u32, 10_000, 1>::from_ticks(1);
        assert!(d1.checked_sub(d2).is_none());
    }

    #[test]
    fn duration_checked_add_u64_1_1000_same_base_success() {
        // Instantiation: <u64,1,1000>::checked_add::<1,1000> — same base success not hit.
        let d1 = Duration::<u64, 1, 1_000>::from_ticks(1);
        let d2 = Duration::<u64, 1, 1_000>::from_ticks(2);
        let result = d1.checked_add(d2);
        assert!(result.is_some());
        assert!(result.unwrap().ticks() == 3);
    }

    #[test]
    fn duration_checked_sub_u64_1_1000_same_base_success() {
        let d1 = Duration::<u64, 1, 1_000>::from_ticks(5);
        let d2 = Duration::<u64, 1, 1_000>::from_ticks(2);
        let result = d1.checked_sub(d2);
        assert!(result.is_some());
        assert!(result.unwrap().ticks() == 3);
    }

    #[test]
    fn duration_checked_add_u64_1_1000000_diff_base_1_1_success() {
        // Instantiation: <u64,1,1000000>::checked_add::<1,1> — only mul overflow hit.
        // Need success path (lines 121-124).
        let d1 = Duration::<u64, 1, 1_000_000>::from_ticks(0);
        let d2 = Duration::<u64, 1, 1>::from_ticks(1);
        let result = d1.checked_add(d2);
        assert!(result.is_some());
        assert!(result.unwrap().ticks() == 1_000_000);

        // Add overflow after successful mul
        let d1 = Duration::<u64, 1, 1_000_000>::from_ticks(u64::MAX);
        let d2 = Duration::<u64, 1, 1>::from_ticks(1);
        assert!(d1.checked_add(d2).is_none());
    }

    #[test]
    fn duration_checked_sub_u64_1_1000000_diff_base_1_1_success() {
        let d1 = Duration::<u64, 1, 1_000_000>::from_ticks(1_000_001);
        let d2 = Duration::<u64, 1, 1>::from_ticks(1);
        let result = d1.checked_sub(d2);
        assert!(result.is_some());
        assert!(result.unwrap().ticks() == 1);
    }

    #[test]
    fn duration_const_try_from_checked_mul_overflow_u64() {
        let d = Duration::<u64, 1, 1>::from_ticks(u64::MAX);
        let result: Option<Duration<u64, 1, 1_000_000>> = d.const_try_into();
        assert!(result == None);
    }

    #[test]
    fn instant_checked_sub_duration_different_base_u64() {
        let i = Instant::<u64, 1, 10_000>::from_ticks(20);
        let d = Duration::<u64, 1, 1_000>::from_ticks(1);
        let result = i.checked_sub_duration(d);
        assert!(result.unwrap().ticks() == 10);

        // checked_mul overflow on u64
        let i = Instant::<u64, 1, 1_000_000>::from_ticks(10);
        let d = Duration::<u64, 1, 1>::from_ticks(u64::MAX);
        assert!(i.checked_sub_duration(d) == None);
    }

    #[test]
    fn instant_checked_add_duration_different_base_u64() {
        let i = Instant::<u64, 1, 10_000>::from_ticks(10);
        let d = Duration::<u64, 1, 1_000>::from_ticks(1);
        let result = i.checked_add_duration(d);
        assert!(result.unwrap().ticks() == 20);

        // checked_mul overflow on u64
        let i = Instant::<u64, 1, 1_000_000>::from_ticks(10);
        let d = Duration::<u64, 1, 1>::from_ticks(u64::MAX);
        assert!(i.checked_add_duration(d) == None);
    }

    #[test]
    fn instant_const_cmp_exact_half_range() {
        // wrapping_sub == MAX/2 exactly hits the Equal branch in const_cmp
        let half = u32::MAX / 2;
        let i1 = Instant::<u32, 1, 1_000>::from_ticks(half);
        let i2 = Instant::<u32, 1, 1_000>::from_ticks(0);
        assert!(i1.const_cmp(i2) == Ordering::Equal);

        let half_64 = u64::MAX / 2;
        let i1 = Instant::<u64, 1, 1_000>::from_ticks(half_64);
        let i2 = Instant::<u64, 1, 1_000>::from_ticks(0);
        assert!(i1.const_cmp(i2) == Ordering::Equal);
    }

    ////////////////////////////////////////////////////////////////////////////////
    //
    // Panic branch tests (#[should_panic])
    //
    ////////////////////////////////////////////////////////////////////////////////

    // duration.rs:405 — convert() overflow
    #[test]
    #[should_panic(expected = "Convert failed!")]
    fn duration_convert_panic_u32() {
        let d = Duration::<u32, 1, 1>::from_ticks(u32::MAX);
        let _: Duration<u32, 1, 1_000> = d.convert();
    }

    #[test]
    #[should_panic(expected = "Convert failed!")]
    fn duration_convert_panic_u64() {
        let d = Duration::<u64, 1, 1>::from_ticks(u64::MAX);
        let _: Duration<u64, 1, 1_000> = d.convert();
    }

    // duration.rs:481 — Duration - Duration underflow
    #[test]
    #[should_panic(expected = "Sub failed!")]
    fn duration_sub_panic_u32() {
        let _ = Duration::<u32, 1, 1_000>::from_ticks(0) - Duration::<u32, 1, 1_000>::from_ticks(1);
    }

    #[test]
    #[should_panic(expected = "Sub failed!")]
    fn duration_sub_panic_u64() {
        let _ = Duration::<u64, 1, 1_000>::from_ticks(0) - Duration::<u64, 1, 1_000>::from_ticks(1);
    }

    // duration.rs:508 — Duration + Duration overflow
    #[test]
    #[should_panic(expected = "Add failed!")]
    fn duration_add_panic_u32() {
        let _ = Duration::<u32, 1, 1_000>::from_ticks(u32::MAX)
            + Duration::<u32, 1, 1_000>::from_ticks(1);
    }

    #[test]
    #[should_panic(expected = "Add failed!")]
    fn duration_add_panic_u64() {
        let _ = Duration::<u64, 1, 1_000>::from_ticks(u64::MAX)
            + Duration::<u64, 1, 1_000>::from_ticks(1);
    }

    #[cfg(not(feature = "certified_subset"))]
    #[test]
    #[should_panic(expected = "Sub failed!")]
    fn duration_sub_u64_u32_panic() {
        let _ = Duration::<u64, 1, 1_000>::from_ticks(0) - Duration::<u32, 1, 1_000>::from_ticks(1);
    }

    #[cfg(not(feature = "certified_subset"))]
    #[test]
    #[should_panic(expected = "Add failed!")]
    fn duration_add_u64_u32_panic() {
        let _ = Duration::<u64, 1, 1_000>::from_ticks(u64::MAX)
            + Duration::<u32, 1, 1_000>::from_ticks(1);
    }

    // instant.rs:245 — Instant - Instant when other > self
    #[test]
    #[should_panic(expected = "Sub failed!")]
    fn instant_sub_instant_panic_u32() {
        let _ = Instant::<u32, 1, 1_000>::from_ticks(0) - Instant::<u32, 1, 1_000>::from_ticks(1);
    }

    #[test]
    #[should_panic(expected = "Sub failed!")]
    fn instant_sub_instant_panic_u64() {
        let _ = Instant::<u64, 1, 1_000>::from_ticks(0) - Instant::<u64, 1, 1_000>::from_ticks(1);
    }

    // instant.rs:264, 296 — Instant +/- Duration panic branches are unreachable:
    // The operator impls constrain both sides to same NOM/DENOM, so checked_sub_duration
    // and checked_add_duration always take the SAME_BASE path which uses wrapping_sub/add
    // and always returns Some.
    //
    // instant.rs:380, 412 — Instant<u64> +/- Duration<u32> cross-type panic branches are
    // also unreachable: .into() preserves the base, so SAME_BASE is always true.
}

#![cfg_attr(not(test), no_std)]
#![doc = include_str!("../README.md")]

use core::{
    borrow::Borrow,
    fmt::{Display, Formatter},
    ops::RangeInclusive,
};

/// Re-export of [`sanctum_u64_ratio`]
pub mod ratio {
    pub use sanctum_u64_ratio::*;
}

mod aft_bef_fee;

pub use aft_bef_fee::*;

use ratio::*;

/// A fee applied as a ratio to an amount
///
/// Invariant: encapsulated ratio is `<= 1.0`
///
/// Fields are private to enforce this invariant
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Fee<D>(D);

/// Displayed as `Fee({self.0})`
impl<D: Display> Display for Fee<D> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Fee({})", self.0))
    }
}

impl<D> Fee<D> {
    #[inline]
    pub const fn as_inner_ref(&self) -> &D {
        &self.0
    }
}

impl<D: Copy> Fee<D> {
    #[inline]
    pub const fn to_inner(self) -> D {
        self.0
    }
}

impl<D> AsRef<D> for Fee<D> {
    #[inline]
    fn as_ref(&self) -> &D {
        self.as_inner_ref()
    }
}

impl<D> Borrow<D> for Fee<D> {
    #[inline]
    fn borrow(&self) -> &D {
        self.as_inner_ref()
    }
}

macro_rules! impl_fee_ratio {
    ($N:ty, $D:ty) => {
        impl Fee<Ceil<Ratio<$N, $D>>> {
            /// # Returns
            /// `None` if `fee_ratio` is not valid (`>1.0`)
            #[inline]
            pub const fn new(fee_ratio: Ratio<$N, $D>) -> Option<Self> {
                if !fee_ratio.is_zero()
                    && fee_ratio.n as <Ratio<$N, $D> as ArithTypes>::Max
                        > fee_ratio.d as <Ratio<$N, $D> as ArithTypes>::Max
                {
                    None
                } else {
                    Some(Self(Ceil(fee_ratio)))
                }
            }

            /// # Safety
            /// - `fee_ratio` must be valid (`<= 1.0`)
            #[inline]
            pub const unsafe fn new_unchecked(fee_ratio: Ratio<$N, $D>) -> Self {
                Self(Ceil(fee_ratio))
            }

            /// # Params
            /// - `amount`: the token amount before fees
            ///
            /// # Returns
            /// `None` on overflow
            #[inline]
            pub const fn apply(&self, amount: u64) -> Option<AftFee> {
                let fee = match self.0.apply(amount) {
                    None => return None,
                    Some(f) => f,
                };
                BefFee(amount).with_fee(fee)
            }

            /// # Params
            /// - `rem`: the remaining token amount after fees were levied
            ///
            /// # Returns
            /// The range of possible `amount` values that was fed into [`Self::apply`]
            /// to output a [`AftFee`] with the same `rem`.
            ///
            /// Returns `None` on overflow or if `self` is an invalid fee ratio (>=1.0)
            #[inline]
            pub const fn reverse_from_rem(&self, rem: u64) -> Option<RangeInclusive<u64>> {
                if self.0 .0.is_zero() {
                    Some(rem..=rem)
                } else {
                    Floor(self.one_minus_fee_ratio()).reverse(rem)
                }
            }

            /// # Params
            /// - `fee`: the fee amount that was levied
            ///
            /// # Returns
            /// The range of possible `amount` values that was fed into [`Self::apply`]
            /// to output a [`AftFee`] with the same `fee`
            #[inline]
            pub const fn reverse_from_fee(&self, fee: u64) -> Option<RangeInclusive<u64>> {
                let Self(r) = self;
                if r.0.is_one() {
                    Some(fee..=fee)
                } else {
                    r.reverse(fee)
                }
            }

            /// # Returns
            /// `1.0` - self's ratio
            #[inline]
            pub const fn one_minus_fee_ratio(
                &self,
            ) -> Ratio<<Ratio<$N, $D> as ArithTypes>::Max, <Ratio<$N, $D> as ArithTypes>::Max> {
                let Self(Ceil(ratio)) = self;
                if ratio.is_zero() {
                    return Ratio::<
                        <Ratio<$N, $D> as ArithTypes>::Max,
                        <Ratio<$N, $D> as ArithTypes>::Max,
                    >::ONE;
                }
                let d = ratio.d as <Ratio<$N, $D> as ArithTypes>::Max;
                let n = ratio.n as <Ratio<$N, $D> as ArithTypes>::Max;
                // unchecked-arith: d >= n guaranteed at construction time
                Ratio { n: d - n, d }
            }
        }

        impl Fee<Floor<Ratio<$N, $D>>> {
            /// # Returns
            /// `None` if `fee_ratio` is not valid (`>1.0`)
            #[inline]
            pub const fn new(fee_ratio: Ratio<$N, $D>) -> Option<Self> {
                if !fee_ratio.is_zero()
                    && fee_ratio.n as <Ratio<$N, $D> as ArithTypes>::Max
                        > fee_ratio.d as <Ratio<$N, $D> as ArithTypes>::Max
                {
                    None
                } else {
                    Some(Self(Floor(fee_ratio)))
                }
            }

            /// # Safety
            /// - `fee_ratio` must be valid (`<= 1.0`)
            #[inline]
            pub const unsafe fn new_unchecked(fee_ratio: Ratio<$N, $D>) -> Self {
                Self(Floor(fee_ratio))
            }

            /// # Params
            /// - `amount`: the token amount before fees
            #[inline]
            pub const fn apply(&self, amount: u64) -> Option<AftFee> {
                let fee = match self.0.apply(amount) {
                    None => return None,
                    Some(f) => f,
                };
                BefFee(amount).with_fee(fee)
            }

            /// # Params
            /// - `rem`: the remaining token amount after fees were levied
            ///
            /// # Returns
            /// The range of possible `amount` values that was fed into [`Self::apply`]
            /// to output a [`AftFee`] with the same `rem`
            #[inline]
            pub const fn reverse_from_rem(&self, rem: u64) -> Option<RangeInclusive<u64>> {
                if self.0 .0.is_zero() {
                    Some(rem..=rem)
                } else {
                    Ceil(self.one_minus_fee_ratio()).reverse(rem)
                }
            }

            /// # Params
            /// - `fee`: the fee amount that was levied
            ///
            /// # Returns
            /// The range of possible `amount` values that was fed into [`Self::apply`]
            /// to output a [`AftFee`] with the same `fee`
            #[inline]
            pub const fn reverse_from_fee(&self, fee: u64) -> Option<RangeInclusive<u64>> {
                let Self(r) = self;
                if r.0.is_one() {
                    Some(fee..=fee)
                } else {
                    r.reverse(fee)
                }
            }

            /// # Returns
            /// `1.0` - self's ratio
            #[inline]
            pub const fn one_minus_fee_ratio(
                &self,
            ) -> Ratio<<Ratio<$N, $D> as ArithTypes>::Max, <Ratio<$N, $D> as ArithTypes>::Max> {
                let Self(Floor(ratio)) = self;
                if ratio.is_zero() {
                    return Ratio::<
                        <Ratio<$N, $D> as ArithTypes>::Max,
                        <Ratio<$N, $D> as ArithTypes>::Max,
                    >::ONE;
                }
                let d = ratio.d as <Ratio<$N, $D> as ArithTypes>::Max;
                let n = ratio.n as <Ratio<$N, $D> as ArithTypes>::Max;
                // unchecked-arith: d >= n guaranteed at construction time
                Ratio { n: d - n, d }
            }
        }
    };
}

impl_fee_ratio!(u8, u8);
impl_fee_ratio!(u8, u16);
impl_fee_ratio!(u8, u32);
impl_fee_ratio!(u8, u64);

impl_fee_ratio!(u16, u8);
impl_fee_ratio!(u16, u16);
impl_fee_ratio!(u16, u32);
impl_fee_ratio!(u16, u64);

impl_fee_ratio!(u32, u8);
impl_fee_ratio!(u32, u16);
impl_fee_ratio!(u32, u32);
impl_fee_ratio!(u32, u64);

impl_fee_ratio!(u64, u8);
impl_fee_ratio!(u64, u16);
impl_fee_ratio!(u64, u32);
impl_fee_ratio!(u64, u64);

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    macro_rules! test_suite {
        (
            $N:ty, $D:ty,
            $test:ident
        ) => {
            impl Fee<Floor<Ratio<$N, $D>>> {
                prop_compose! {
                    fn prop_floor_ceil()
                        (d in any::<$D>())
                        (
                            n in 0..=(
                                if d as <Ratio<$N, $D> as ArithTypes>::Max
                                    > <$N>::MAX as <Ratio<$N, $D> as ArithTypes>::Max {
                                    <$N>::MAX
                                } else {
                                    d as $N
                                }
                            ),
                            d in Just(d)
                        ) -> (Self, Fee<Ceil<Ratio<$N, $D>>>) {
                            let ratio = Ratio { n, d };
                            (
                                Self::new(ratio).unwrap(),
                                Fee::<Ceil::<Ratio<$N, $D>>>::new(ratio).unwrap(),
                            )
                        }
                }
            }

            proptest! {
                #[test]
                fn $test(
                    (floor, ceil) in Fee::<Floor::<Ratio<$N, $D>>>::prop_floor_ceil(),
                    bef: u64,
                ) {
                    // FLOOR TESTS
                    let floor_aaf = floor.apply(bef).unwrap();
                    prop_assert_eq!(floor_aaf.bef_fee(), bef);
                    // boundary cases
                    if floor.0.0.is_zero() {
                        prop_assert_eq!(floor_aaf.rem(), bef);
                        prop_assert_eq!(floor_aaf.fee(), 0);
                    } else if floor.0.0.is_one() {
                        prop_assert_eq!(floor_aaf.rem(), 0);
                        prop_assert_eq!(floor_aaf.fee(), bef);
                    }
                    // round-trip from rem
                    let floor_rev_rem = floor.reverse_from_rem(floor_aaf.rem()).unwrap();
                    for bound in [*floor_rev_rem.start(), *floor_rev_rem.end()] {
                        let rt = floor.apply(bound).unwrap();
                        prop_assert_eq!(rt.rem(), floor_aaf.rem());
                        if floor.0.0.is_one() {
                            // special-case: rem should be 0 and
                            // floor_rev_rem should be 0..=u64::MAX
                            prop_assert_eq!(rt.fee(), bound);
                        }
                        // else difference in fee is not well-bounded
                    }
                    // smaller valid reversal not in range should not exist
                    if *floor_rev_rem.start() > 0 {
                        let rt = floor.apply(*floor_rev_rem.start() - 1).unwrap();
                        prop_assert!(floor_aaf.rem() != rt.rem());
                    }
                    // larger valid reversal not in range should not exist
                    if *floor_rev_rem.end() < u64::MAX {
                        let rt = floor.apply(*floor_rev_rem.end() + 1).unwrap();
                        prop_assert!(floor_aaf.rem() != rt.rem());
                    }
                    // round-trip from fee
                    let floor_rev_fee = floor.reverse_from_fee(floor_aaf.fee()).unwrap();
                    for bound in [*floor_rev_fee.start(), *floor_rev_fee.end()] {
                        let rt = floor.apply(bound).unwrap();
                        prop_assert_eq!(rt.fee(), floor_aaf.fee());
                        if floor.0.0.is_zero() {
                            // special-case: fee should be 0 and
                            // floor_rev_fee should be 0..=u64::MAX
                            prop_assert_eq!(rt.rem(), bound);
                        }
                        // else difference in rem is not well-bounded
                    }
                    // smaller valid reversal not in range should not exist
                    if *floor_rev_fee.start() > 0 {
                        let rt = floor.apply(*floor_rev_fee.start() - 1).unwrap();
                        prop_assert!(floor_aaf.fee() != rt.fee());
                    }
                    // larger valid reversal not in range should not exist
                    if *floor_rev_fee.end() < u64::MAX {
                        let rt = floor.apply(*floor_rev_fee.end() + 1).unwrap();
                        prop_assert!(floor_aaf.fee() != rt.fee());
                    }


                    // CEIL TESTS
                    let ceil_aaf = ceil.apply(bef).unwrap();
                    prop_assert_eq!(ceil_aaf.bef_fee(), bef);
                    // boundary cases
                    if ceil.0.0.is_zero() {
                        prop_assert_eq!(ceil_aaf.rem(), bef);
                        prop_assert_eq!(ceil_aaf.fee(), 0);
                    } else if ceil.0.0.is_one() {
                        prop_assert_eq!(ceil_aaf.rem(), 0);
                        prop_assert_eq!(ceil_aaf.fee(), bef);
                    }
                    // round-trip from rem
                    let ceil_rev_rem = ceil.reverse_from_rem(ceil_aaf.rem()).unwrap();
                    for bound in [*ceil_rev_rem.start(), *ceil_rev_rem.end()] {
                        let rt = ceil.apply(bound).unwrap();
                        prop_assert_eq!(rt.rem(), ceil_aaf.rem());
                        if ceil.0.0.is_one() {
                            // special-case: rem should be 0 and
                            // ceil_rev_rem should be 0..=u64::MAX
                            prop_assert_eq!(rt.fee(), bound);
                        }
                        // else difference in fee is not well-bounded
                    }
                    // smaller valid reversal not in range should not exist
                    if *ceil_rev_rem.start() > 0 {
                        let rt = ceil.apply(*ceil_rev_rem.start() - 1).unwrap();
                        prop_assert!(ceil_aaf.rem() != rt.rem());
                    }
                    // larger valid reversal not in range should not exist
                    if *ceil_rev_rem.end() < u64::MAX {
                        let rt = ceil.apply(*ceil_rev_rem.end() + 1).unwrap();
                        prop_assert!(ceil_aaf.rem() != rt.rem());
                    }
                    // round-trip from fee
                    let ceil_rev_fee = ceil.reverse_from_fee(ceil_aaf.fee()).unwrap();
                    for bound in [*ceil_rev_fee.start(), *ceil_rev_fee.end()] {
                        let rt = ceil.apply(bound).unwrap();
                        prop_assert_eq!(rt.fee(), ceil_aaf.fee());
                        if ceil.0.0.is_zero() {
                            // special-case: fee should be 0 and
                            // ceil_rev_fee should be 0..=u64::MAX
                            prop_assert_eq!(rt.rem(), bound);
                        }
                        // else difference in rem is not well-bounded
                    }
                    // smaller valid reversal not in range should not exist
                    if *ceil_rev_fee.start() > 0 {
                        let rt = ceil.apply(*ceil_rev_fee.start() - 1).unwrap();
                        prop_assert!(ceil_aaf.fee() != rt.fee());
                    }
                    // larger valid reversal not in range should not exist
                    if *ceil_rev_fee.end() < u64::MAX {
                        let rt = ceil.apply(*ceil_rev_fee.end() + 1).unwrap();
                        prop_assert!(ceil_aaf.fee() != rt.fee());
                    }


                    // COMBINED TESTS
                    // floor's rem should be larger than ceil's by at most 1
                    prop_assert!(
                        floor_aaf.rem() - ceil_aaf.rem() <= 1
                    );
                    // floor's fee should be smaller than ceil's by at most 1
                    prop_assert!(
                        ceil_aaf.fee() - floor_aaf.fee() <= 1
                    );
                }
            }
        };
    }

    test_suite!(u8, u8, fee_tests_u8_u8);
    test_suite!(u8, u16, fee_tests_u8_u16);
    test_suite!(u8, u32, fee_tests_u8_u32);
    test_suite!(u8, u64, fee_tests_u8_u64);

    test_suite!(u16, u8, fee_tests_u16_u8);
    test_suite!(u16, u16, fee_tests_u16_u16);
    test_suite!(u16, u32, fee_tests_u16_u32);
    test_suite!(u16, u64, fee_tests_u16_u64);

    test_suite!(u32, u8, fee_tests_u32_u8);
    test_suite!(u32, u16, fee_tests_u32_u16);
    test_suite!(u32, u32, fee_tests_u32_u32);
    test_suite!(u32, u64, fee_tests_u32_u64);

    test_suite!(u64, u8, fee_tests_u64_u8);
    test_suite!(u64, u16, fee_tests_u64_u16);
    test_suite!(u64, u32, fee_tests_u64_u32);
    test_suite!(u64, u64, fee_tests_u64_u64);
}

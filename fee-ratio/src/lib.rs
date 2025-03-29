#![cfg_attr(not(test), no_std)]
#![doc = include_str!("../README.md")]

use core::{
    fmt::{Display, Formatter},
    ops::RangeInclusive,
};

/// Re-export of [`sanctum_token_ratio`]
pub mod ratio {
    pub use sanctum_token_ratio::*;
}

mod aft_bef_fee;

pub use aft_bef_fee::*;

use ratio::*;

/// A fee applied as a ratio <1.0
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Fee<D>(pub D);

/// Displayed as `FeeRatio({self.0})`
impl<D: Display> Display for Fee<D> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("FeeRatio({})", self.0))
    }
}

impl<D> Fee<D> {
    /// Convenience constructor for better compatibility with type aliases
    #[inline]
    pub const fn new(div_ratio: D) -> Self {
        Self(div_ratio)
    }
}

macro_rules! impl_fee_ratio {
    ($N:ty, $D:ty) => {
        impl Fee<Ceil<Ratio<$N, $D>>> {
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
            #[inline]
            pub const fn reverse_from_rem(&self, rem: u64) -> Option<RangeInclusive<u64>> {
                if self.0 .0.is_zero() {
                    Some(rem..=rem)
                } else {
                    let one_minus = match self.one_minus_fee_ratio() {
                        None => return None,
                        Some(r) => r,
                    };
                    Floor(one_minus).reverse(rem)
                }
            }

            /// # Params
            /// - `fee`: the fee amount that was levied
            #[inline]
            pub const fn reverse_from_fee(&self, fee: u64) -> Option<RangeInclusive<u64>> {
                let Self(Ceil(ratio)) = self;
                if ratio.is_one() {
                    Some(fee..=fee)
                } else {
                    Ceil(*ratio).reverse(fee)
                }
            }

            /// # Returns
            /// `1.0` - self's ratio, else `None` if self's ratio is `>1.0` and is hence not a valid fee
            #[inline]
            pub const fn one_minus_fee_ratio(&self)
                -> Option<Ratio<<Ratio<$N, $D> as ArithTypes>::Max, <Ratio<$N, $D> as ArithTypes>::Max>>
            {
                let Self(Ceil(ratio)) = self;
                if ratio.is_zero() {
                    return Some(
                        Ratio::<<Ratio<$N, $D> as ArithTypes>::Max, <Ratio<$N, $D> as ArithTypes>::Max>::ONE
                    );
                }
                let d = ratio.d as <Ratio<$N, $D> as ArithTypes>::Max;
                let n = ratio.n as <Ratio<$N, $D> as ArithTypes>::Max;
                let n = match d.checked_sub(n) {
                    None => return None,
                    Some(n) => n,
                };
                Some(Ratio { n, d })
            }
        }

        impl Fee<Floor<Ratio<$N, $D>>> {
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
            #[inline]
            pub const fn reverse_from_rem(&self, rem: u64) -> Option<RangeInclusive<u64>> {
                if self.0 .0.is_zero() {
                    Some(rem..=rem)
                } else {
                    let one_minus = match self.one_minus_fee_ratio() {
                        None => return None,
                        Some(r) => r,
                    };
                    Ceil(one_minus).reverse(rem)
                }
            }

            /// # Params
            /// - `fee`: the fee amount that was levied
            #[inline]
            pub const fn reverse_from_fee(&self, fee: u64) -> Option<RangeInclusive<u64>> {
                let Self(Floor(ratio)) = self;
                if ratio.is_one() {
                    Some(fee..=fee)
                } else {
                    Floor(*ratio).reverse(fee)
                }
            }

            /// # Returns
            /// `1.0` - self's ratio, else `None` if self's ratio is `>1.0` and is hence not a valid fee
            #[inline]
            pub const fn one_minus_fee_ratio(&self)
                -> Option<Ratio<<Ratio<$N, $D> as ArithTypes>::Max, <Ratio<$N, $D> as ArithTypes>::Max>>
            {
                let Self(Floor(ratio)) = self;
                if ratio.is_zero() {
                    return Some(
                        Ratio::<<Ratio<$N, $D> as ArithTypes>::Max, <Ratio<$N, $D> as ArithTypes>::Max>::ONE
                    );
                }
                let d = ratio.d as <Ratio<$N, $D> as ArithTypes>::Max;
                let n = ratio.n as <Ratio<$N, $D> as ArithTypes>::Max;
                let n = match d.checked_sub(n) {
                    None => return None,
                    Some(n) => n,
                };
                Some(Ratio { n, d })
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

#![cfg_attr(not(test), no_std)]

use core::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use core::fmt::{Display, Formatter};
use core::hash::{Hash, Hasher};

mod ceil;
mod floor;

pub(crate) mod utils;

pub use ceil::*;
pub use floor::*;

/// A ratio that is applied to a u64 token amount.
///
/// A zero denominator ratio (`self.d == 0`) is treated as the zero ratio.
///
/// Must use with [`crate::CeilDiv`] or [`crate::FloorDiv`]
/// for application on [`u64`]s
#[derive(Debug, Copy, Clone)]
pub struct Ratio<N, D> {
    /// Numerator
    pub n: N,

    /// Denominator
    pub d: D,
}

macro_rules! impl_gcd {
    ($f:ident, $T:ty) => {
        // holy shit you can have recursive const fns now
        /// Takes 1 fewer iteration if a > b compared to b > a.
        ///
        /// Never returns 0 unless both args are 0
        #[inline]
        const fn $f(a: $T, b: $T) -> $T {
            if b > 0 {
                $f(b, a % b)
            } else {
                a
            }
        }
    };
}

impl_gcd!(gcd_u8, u8);
impl_gcd!(gcd_u16, u16);
impl_gcd!(gcd_u32, u32);
impl_gcd!(gcd_u64, u64);

/// Associated types of a [`Ratio`] for use in arithmetic operations
///
/// (because inherent associated types are still unstable)
pub trait ArithTypes {
    /// The smaller bitwidth type between `Ratio::N` and `Ratio::D`
    type Min;

    /// The larger bitwidth type between `Ratio::N` and `Ratio::D`
    type Max;

    /// The unsigned type that `Ratio::N` and `Ratio::D` must be
    /// bit-extended (cast) into to avoid overflows on multiplication
    /// of `Ratio::N` and `Ratio::D`
    type Ext;
}

impl<N, D> Ratio<N, D> {
    /// Convenience constructor for better compatibility with type aliases
    #[inline]
    pub const fn new(n: N, d: D) -> Self {
        Self { n, d }
    }
}

macro_rules! impl_ratio {
    ($N:ty, $D:ty, [$gcd:expr, $MIN: ty, $MAX:ty, $EXT:ty]) => {
        impl ArithTypes for Ratio<$N, $D> {
            type Min = $MIN;
            type Max = $MAX;
            type Ext = $EXT;
        }

        impl Ratio<$N, $D> {
            pub const ZERO: Self = Self { n: 0, d: 0 };
            pub const ONE: Self = Self { n: 1, d: 1 };

            /// Returns true if this ratio represents `0.0`
            /// i.e. applying it to any value should output 0
            #[inline]
            pub const fn is_zero(&self) -> bool {
                self.n == 0 || self.d == 0
            }

            /// Returns true if this ratio represents `1.0`
            /// i.e. `numerator == denominator` and applying it
            /// to any value should output the same value
            #[inline]
            pub const fn is_one(&self) -> bool {
                type Max = <Ratio<$N, $D> as ArithTypes>::Max;

                !self.is_zero() && self.n as Max == self.d as Max
            }

            #[inline]
            pub const fn const_cmp(&self, other: &Self) -> Ordering {
                type Ext = <Ratio<$N, $D> as ArithTypes>::Ext;

                match (self.is_zero(), other.is_zero()) {
                    (true, true) => return Ordering::Equal,
                    (true, false) => return Ordering::Less,
                    (false, true) => return Ordering::Greater,
                    (false, false) => (),
                };

                let lhs = (self.n as Ext) * (other.d as Ext);
                let rhs = (other.n as Ext) * (self.d as Ext);
                if lhs == rhs {
                    Ordering::Equal
                } else if lhs < rhs {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }

            /// Returns the fraction's lowest form.
            ///
            /// This is `0/0` if [`Self::is_zero()`]
            #[inline]
            pub const fn lowest_form(
                &self,
            ) -> Ratio<<Self as ArithTypes>::Max, <Self as ArithTypes>::Max> {
                type Max = <Ratio<$N, $D> as ArithTypes>::Max;

                if self.is_zero() {
                    return Ratio::<Max, Max>::ZERO;
                }
                let n = self.n as Max;
                let d = self.d as Max;
                // usually the denominator is larger, so put it first
                let gcd = $gcd(d, n);
                // division-safety: gcd is never 0 due to early return above
                Ratio {
                    n: n / gcd,
                    d: d / gcd,
                }
            }
        }

        impl Default for Ratio<$N, $D> {
            #[inline]
            fn default() -> Self {
                Self::ZERO
            }
        }

        impl PartialEq for Ratio<$N, $D> {
            #[inline]
            fn eq(&self, rhs: &Self) -> bool {
                self.const_cmp(rhs).is_eq()
            }
        }

        impl Eq for Ratio<$N, $D> {}

        impl PartialOrd for Ratio<$N, $D> {
            #[inline]
            fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
                Some(self.cmp(rhs))
            }
        }

        impl Ord for Ratio<$N, $D> {
            #[inline]
            fn cmp(&self, rhs: &Self) -> Ordering {
                self.const_cmp(rhs)
            }
        }

        /// To ensure that the
        /// `k1 == k2 -> hash(k1) == hash(k2)`
        /// invariant is not violated, we need to hash the fraction's lowest form
        /// `<https://doc.rust-lang.org/std/hash/trait.Hash.html#hash-and-eq>`
        impl Hash for Ratio<$N, $D> {
            #[inline]
            fn hash<H>(&self, state: &mut H)
            where
                H: Hasher,
            {
                let Ratio { n, d } = self.lowest_form();
                n.hash(state);
                d.hash(state);
            }
        }

        /// Displayed as `{numerator}/{denominator}`
        impl Display for Ratio<$N, $D> {
            #[inline]
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                f.write_fmt(format_args!("{}/{}", self.n, self.d))
            }
        }
    };
}

impl_ratio!(u8, u8, [gcd_u8, u8, u8, u16]);
impl_ratio!(u8, u16, [gcd_u16, u8, u16, u32]);
impl_ratio!(u8, u32, [gcd_u32, u8, u32, u64]);
impl_ratio!(u8, u64, [gcd_u64, u8, u64, u128]);

impl_ratio!(u16, u8, [gcd_u16, u8, u16, u32]);
impl_ratio!(u16, u16, [gcd_u16, u16, u16, u32]);
impl_ratio!(u16, u32, [gcd_u32, u16, u32, u64]);
impl_ratio!(u16, u64, [gcd_u64, u16, u64, u128]);

impl_ratio!(u32, u8, [gcd_u32, u8, u32, u64]);
impl_ratio!(u32, u16, [gcd_u32, u16, u32, u64]);
impl_ratio!(u32, u32, [gcd_u32, u32, u32, u64]);
impl_ratio!(u32, u64, [gcd_u64, u32, u64, u128]);

impl_ratio!(u64, u8, [gcd_u64, u8, u64, u128]);
impl_ratio!(u64, u16, [gcd_u64, u16, u64, u128]);
impl_ratio!(u64, u32, [gcd_u64, u32, u64, u128]);
impl_ratio!(u64, u64, [gcd_u64, u64, u64, u128]);

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    macro_rules! zero_eq {
        ($N:ty, $D:ty, $zero:ident) => {
            proptest! {
                #[test]
                fn $zero(a: $N, b: $D) {
                    type NR = Ratio<$N, $D>;
                    type DR = Ratio<$D, $N>;
                    type Max = <NR as ArithTypes>::Max;

                    for nr in [
                        NR::new(a, 0),
                        NR::new(0, b),
                    ] {
                        prop_assert_eq!(NR::ZERO, nr, "{} != 0", nr);
                    }

                    for dr in [
                        DR::new(b, 0),
                        DR::new(0, a),
                    ] {
                        prop_assert_eq!(DR::ZERO, dr, "{} != 0", dr);
                    }

                    for lowest_form in [
                        DR::new(0, a).lowest_form(),
                        NR::new(a, 0).lowest_form(),
                        NR::new(0, b).lowest_form(),
                        DR::new(b, 0).lowest_form(),
                    ] {
                        prop_assert_eq!(
                            lowest_form,
                            Ratio::<Max, Max>::ZERO,
                            "{} != 0", lowest_form,
                        );
                    }
                }
            }
        };
    }

    macro_rules! lowest_form_ord_iff_ord {
        ($N: ty, $D:ty, $lowest_form:ident) => {
            proptest! {
                #[test]
                fn $lowest_form(a: $N, b: $D, c: $N, d: $D) {
                    type R = Ratio<$N, $D>;

                    let [(r1, l1), (r2, l2)] = [(a, b), (c, d)]
                        .map(|(n, d)| {
                            let r = R::new(n, d);
                            (r, r.lowest_form())
                        });
                    prop_assert_eq!(
                        r1.const_cmp(&r2),
                        l1.const_cmp(&l2),
                        "{}, {}, {}, {}",
                        r1, l1, r2, l2,
                    );
                }
            }
        };
    }

    macro_rules! ord {
        ($T:ty, $ord:ident) => {
            proptest! {
                #[test]
                fn $ord(common in 1..=<$T>::MAX, a in 1..=<$T>::MAX, b in 1..=<$T>::MAX) {
                    type R = Ratio<$T, $T>;

                    if a == b {
                        prop_assert_eq!(
                            R::new(a, common),
                            R::new(b, common),
                        );
                        prop_assert_eq!(
                            R::new(common, a),
                            R::new(common, b),
                        );
                        return Ok(());
                    }

                    let (smaller, larger) = if a < b {
                        (a, b)
                    } else {
                        (b, a)
                    };
                    let s = R::new(smaller, common);
                    let l = R::new(larger, common);
                    prop_assert!(s < l, "common d {s}, {l}");

                    let s = R::new(common, larger);
                    let l = R::new(common, smaller);
                    prop_assert!(s < l, "common n {s}, {l}");
                }
            }
        };
    }

    ord!(u8, ord_u8);
    ord!(u16, ord_u16);
    ord!(u32, ord_u32);
    ord!(u64, ord_u64);

    zero_eq!(u8, u8, zero_eq_u8_u8);
    zero_eq!(u8, u16, zero_eq_u8_u16);
    zero_eq!(u8, u32, zero_eq_u8_u32);
    zero_eq!(u8, u64, zero_eq_u8_u64);

    zero_eq!(u16, u8, zero_eq_u16_u8);
    zero_eq!(u16, u16, zero_eq_u16_u16);
    zero_eq!(u16, u32, zero_eq_u16_u32);
    zero_eq!(u16, u64, zero_eq_u16_u64);

    zero_eq!(u32, u8, zero_eq_u32_u8);
    zero_eq!(u32, u16, zero_eq_u32_u16);
    zero_eq!(u32, u32, zero_eq_u32_u32);
    zero_eq!(u32, u64, zero_eq_u32_u64);

    zero_eq!(u64, u8, zero_eq_u64_u8);
    zero_eq!(u64, u16, zero_eq_u64_u16);
    zero_eq!(u64, u32, zero_eq_u64_u32);
    zero_eq!(u64, u64, zero_eq_u64_u64);

    lowest_form_ord_iff_ord!(u8, u8, lowest_form_iff_u8_u8);
    lowest_form_ord_iff_ord!(u8, u16, lowest_form_iff_u8_u16);
    lowest_form_ord_iff_ord!(u8, u32, lowest_form_iff_u8_u32);
    lowest_form_ord_iff_ord!(u8, u64, lowest_form_iff_u8_u64);

    lowest_form_ord_iff_ord!(u16, u8, lowest_form_iff_u16_u8);
    lowest_form_ord_iff_ord!(u16, u16, lowest_form_iff_u16_u16);
    lowest_form_ord_iff_ord!(u16, u32, lowest_form_iff_u16_u32);
    lowest_form_ord_iff_ord!(u16, u64, lowest_form_iff_u16_u64);

    lowest_form_ord_iff_ord!(u32, u8, lowest_form_iff_u32_u8);
    lowest_form_ord_iff_ord!(u32, u16, lowest_form_iff_u32_u16);
    lowest_form_ord_iff_ord!(u32, u32, lowest_form_iff_u32_u32);
    lowest_form_ord_iff_ord!(u32, u64, lowest_form_iff_u32_u64);

    lowest_form_ord_iff_ord!(u64, u8, lowest_form_iff_u64_u8);
    lowest_form_ord_iff_ord!(u64, u16, lowest_form_iff_u64_u16);
    lowest_form_ord_iff_ord!(u64, u32, lowest_form_iff_u64_u32);
    lowest_form_ord_iff_ord!(u64, u64, lowest_form_iff_u64_u64);
}

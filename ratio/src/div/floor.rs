use core::{
    fmt::{Display, Formatter},
    ops::RangeInclusive,
};

use crate::{utils::u128_to_u64_checked, Ratio};

/// A ratio `(n/d)` floor-applied to a u64 `x`. Output = `floor(xn/d)`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Floor<R>(pub R);

/// Displayed as `Floor({{self.0})`
impl<R: Display> Display for Floor<R> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Floor({})", self.0))
    }
}

impl<R> Floor<R> {
    /// Convenience constructor for better compatibility with type aliases
    #[inline]
    pub const fn new(r: R) -> Self {
        Self(r)
    }
}

macro_rules! impl_floor_div {
    ($N:ty, $D:ty) => {
        impl Floor<Ratio<$N, $D>> {
            /// # Returns
            ///
            /// `floor(amt * self.0.n / self.0.d)`
            ///
            /// ## Special Case Returns
            /// - `0` if `self.0.is_zero()`
            /// - `None` if `result > u64::MAX`
            #[inline]
            pub const fn apply(&self, amount: u64) -> Option<u64> {
                if self.0.is_zero() {
                    return Some(0);
                }
                let Ratio { n, d } = self.0;
                let d = d as u128;
                let n = n as u128;
                let x = amount as u128;
                // unchecked-arith: mul will not overflow because
                // both x and n are <= u64::MAX
                let xn = x * n;
                // unchecked-arith: ratio is not 0 so d != 0
                let res = xn / d;
                u128_to_u64_checked(res)
            }

            /// # Returns
            ///
            /// `min..=max` the range of possible values that were fed into `self.apply()`
            /// to get output `amt_after_apply`.
            ///
            /// `min` and `max` are saturated at `0` and `u64::MAX`.
            ///
            /// `min` rounds up.
            /// - Example: if the actual range has `min = 14.6`, then the range returned will be `15..=xx`
            ///
            /// `max` rounds down.
            /// - Example: if the actual range has `max = 14.6`, then the range returned will be `xx..=14`
            ///
            /// ## Special Case Returns
            ///
            /// - `0..=u64::MAX` if `self.0.is_zero()` and `amt_after_apply == 0`
            /// - `None` if `self.0.is_zero()` but `amt_after_apply != 0`
            /// - `None` if `min > u64::MAX`
            ///
            /// # Derivation
            ///
            /// ```md
            /// let x = input amount we are trying to find
            /// y = amt_after_apply
            /// n = numerator
            /// d = denominator
            ///
            /// y = floor(xn / d)
            /// y <= xn / d < y + 1
            ///
            /// LHS (min):
            /// dy <= xn
            /// dy / n <= x
            ///
            /// RHS (max):
            /// xn < d(y+1)
            /// xn < dy + d
            /// x < (dy + d) / n
            /// ```
            #[inline]
            pub const fn reverse(&self, amt_after_apply: u64) -> Option<RangeInclusive<u64>> {
                if self.0.is_zero() {
                    return if amt_after_apply == 0 {
                        Some(0..=u64::MAX)
                    } else {
                        None
                    };
                }

                let Ratio { n, d } = self.0;
                let d = d as u128;
                let n = n as u128;
                let y = amt_after_apply as u128;

                // unchecked-arith: mul will not overflow because
                // both d and y are <= u64::MAX
                let dy = d * y;
                // unchecked-arith: ratio is not 0 so n != 0
                let min = dy.div_ceil(n);
                let min = match u128_to_u64_checked(min) {
                    None => return None,
                    Some(r) => r,
                };

                // unchecked-arith: even if d = y = u64::MAX, does not overflow u128
                let dy_plus_d = dy + d;
                // unchecked-arith: ratio is not 0 so n != 0
                let max = dy_plus_d / n;
                let rem = dy_plus_d % n;
                let max = if rem == 0 {
                    // range-exclusive, so must - 1
                    max.saturating_sub(1)
                } else {
                    max
                };
                let max = match u128_to_u64_checked(max) {
                    // saturation
                    None => u64::MAX,
                    Some(r) => r,
                };

                Some(min..=max)
            }
        }
    };
}

impl_floor_div!(u8, u8);
impl_floor_div!(u8, u16);
impl_floor_div!(u8, u32);
impl_floor_div!(u8, u64);

impl_floor_div!(u16, u8);
impl_floor_div!(u16, u16);
impl_floor_div!(u16, u32);
impl_floor_div!(u16, u64);

impl_floor_div!(u32, u8);
impl_floor_div!(u32, u16);
impl_floor_div!(u32, u32);
impl_floor_div!(u32, u64);

impl_floor_div!(u64, u8);
impl_floor_div!(u64, u16);
impl_floor_div!(u64, u32);
impl_floor_div!(u64, u64);

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    macro_rules! test_suite {
        (
            $N:ty, $D:ty,
            $nonzero_tests:ident,
            $zero_tests:ident
        ) => {
            impl Floor<Ratio<$N, $D>> {
                prop_compose! {
                    /// max_limit is the max number that ratio can be applied to without overflowing u64
                    pub(crate) fn prop_ratio_gte_one_and_overflow_max_limit()
                        (ratio in <Ratio<$N, $D>>::prop_gte_one())-> (u64, Self) {
                            // let x be max limit
                            // floor(xn/d) <= u64::MAX
                            // xn/d < u64::MAX + 1
                            // x <= d(1 + u64::MAX) / n
                            let max_limit =  (1 + u64::MAX as u128) * ratio.d as u128 / ratio.n as u128;
                            (max_limit.try_into().unwrap_or(u64::MAX), Self(ratio))
                    }
                }

                prop_compose! {
                    pub(crate) fn prop_ratio_gte_one_amt_no_overflow()
                        ((maxlimit, ratio) in Self::prop_ratio_gte_one_and_overflow_max_limit())
                        (amt in 0..=maxlimit, maxlimit in Just(maxlimit), ratio in Just(ratio)) -> (u64, u64, Self) {
                            (amt, maxlimit, ratio)
                        }
                }

                prop_compose! {
                    /// max_limit is the max number that ratio can be reversed on without overflowing u64
                    pub(crate) fn prop_ratio_lte_one_and_rev_overflow_max_limit()
                        (ratio in <Ratio<$N, $D>>::prop_lte_one())-> (u64, Self) {
                            // max limit is exceeded when min of range exceeds u64::MAX
                            //
                            // let y be max limit
                            // dy / n <= u64::MAX
                            // y <= u64::MAX * n / d
                            let max_limit = (u64::MAX as u128 * ratio.n as u128) / ratio.d as u128;
                            (u64::try_from(max_limit).unwrap(), Self(ratio))
                    }
                }

                prop_compose! {
                    pub(crate) fn prop_ratio_lte_one_rev_no_overflow()
                        ((maxlimit, ratio) in Self::prop_ratio_lte_one_and_rev_overflow_max_limit())
                        (amt in 0..=maxlimit, maxlimit in Just(maxlimit), ratio in Just(ratio)) -> (u64, u64, Self) {
                            (amt, maxlimit, ratio)
                        }
                }
            }

            proptest! {
                #[test]
                fn $nonzero_tests(
                    (amt, amt_max, gte) in Floor::<Ratio<$N, $D>>::prop_ratio_gte_one_amt_no_overflow(),
                    (_aaf, aaf_max, lte) in Floor::<Ratio<$N, $D>>::prop_ratio_lte_one_rev_no_overflow(),
                    any_u64: u64,
                ) {
                    // gte one round trip
                    let app = gte.apply(amt).unwrap();
                    let rt = gte.reverse(app).unwrap();
                    prop_assert!(rt.start() <= rt.end(), "gte one minmax {:?}", rt);
                    prop_assert!(
                        amt - *rt.start() <= 1 &&
                        *rt.end() - amt <= 1,
                        "gte one rt {} {:?}", amt, rt,
                    );
                    // make sure that applying the ratio again
                    // yields the same result
                    let app_min = gte.apply(*rt.start()).unwrap();
                    let app_max = gte.apply(*rt.end()).unwrap();
                    prop_assert!(
                        app == app_min &&
                        app == app_max,
                        "gte one rt {} {:?}", amt, rt,
                    );
                    // smaller valid reversal not in range should not exist
                    if *rt.start() > 0 {
                        let app_smaller = gte.apply(*rt.start() - 1).unwrap();
                        prop_assert!(app != app_smaller);
                    }
                    // larger valid reversal not in range should not exist
                    if *rt.end() < u64::MAX {
                        if let Some(app_larger) = gte.apply(*rt.end() + 1) {
                            prop_assert!(app != app_larger);
                        }
                    }

                    // gte overflow
                    if amt_max < u64::MAX {
                        prop_assert!(gte.apply(amt_max + 1).is_none());
                    }

                    // gte reverse zero is zero
                    // this is true because gte 1 means applying on anything
                    // should always at least return 1 unless its 0
                    let rev_zero = gte.reverse(0).unwrap();
                    prop_assert_eq!(rev_zero.clone(), 0..=0, "gte rev zero {:?}", rev_zero);


                    // lte one round trip
                    let amt = any_u64;
                    let app = lte.apply(amt).unwrap();
                    let rt = lte.reverse(app).unwrap();
                    prop_assert!(rt.start() <= rt.end(), "lte one minmax {:?}", rt);
                    // range is variable due to floor, will not be
                    // well-bounded like gte one
                    prop_assert!(*rt.start() <= amt);
                    prop_assert!(amt <= *rt.end());
                    // but make sure that applying the ratio again
                    // yields the same result
                    let app_min = lte.apply(*rt.start()).unwrap();
                    let app_max = lte.apply(*rt.end()).unwrap();
                    prop_assert!(
                        app == app_min &&
                        app == app_max,
                        "lte one rt {} {:?}", amt, rt,
                    );
                    // smaller valid reversal not in range should not exist
                    if *rt.start() > 0 {
                        let app_smaller = lte.apply(*rt.start() - 1).unwrap();
                        prop_assert!(app != app_smaller);
                    }
                    // larger valid reversal not in range should not exist
                    if *rt.end() < u64::MAX {
                        let app_larger = lte.apply(*rt.end() + 1).unwrap();
                        prop_assert!(app != app_larger);
                    }

                    // lte overflow
                    if aaf_max < u64::MAX {
                        prop_assert!(lte.reverse(aaf_max + 1).is_none());
                    }
                }
            }

            proptest! {
                #[test]
                fn $zero_tests(
                    zer in <Ratio<$N, $D>>::prop_zero(),
                    amt: u64,
                ) {
                    let zer = Floor(zer);
                    prop_assert_eq!(zer.apply(amt).unwrap(), 0);
                    if amt != 0 {
                        prop_assert!(zer.reverse(amt).is_none());
                    }
                    prop_assert_eq!(zer.reverse(0).unwrap(), 0..=u64::MAX);
                }
            }
        }
    }

    test_suite!(u8, u8, floor_u8_u8_nonzero_tests, floor_u8_u8_zero_tests);
    test_suite!(u8, u16, floor_u8_u16_nonzero_tests, floor_u8_u16_zero_tests);
    test_suite!(u8, u32, floor_u8_u32_nonzero_tests, floor_u8_u32_zero_tests);
    test_suite!(u8, u64, floor_u8_u64_nonzero_tests, floor_u8_u64_zero_tests);

    test_suite!(u16, u8, floor_u16_u8_nonzero_tests, floor_u16_u8_zero_tests);
    test_suite!(
        u16,
        u16,
        floor_u16_u16_nonzero_tests,
        floor_u16_u16_zero_tests
    );
    test_suite!(
        u16,
        u32,
        floor_u16_u32_nonzero_tests,
        floor_u16_u32_zero_tests
    );
    test_suite!(
        u16,
        u64,
        floor_u16_u64_nonzero_tests,
        floor_u16_u64_zero_tests
    );

    test_suite!(u32, u8, floor_u32_u8_nonzero_tests, floor_u32_u8_zero_tests);
    test_suite!(
        u32,
        u16,
        floor_u32_u16_nonzero_tests,
        floor_u32_u16_zero_tests
    );
    test_suite!(
        u32,
        u32,
        floor_u32_u32_nonzero_tests,
        floor_u32_u32_zero_tests
    );
    test_suite!(
        u32,
        u64,
        floor_u32_u64_nonzero_tests,
        floor_u32_u64_zero_tests
    );

    test_suite!(u64, u8, floor_u64_u8_nonzero_tests, floor_u64_u8_zero_tests);
    test_suite!(
        u64,
        u16,
        floor_u64_u16_nonzero_tests,
        floor_u64_u16_zero_tests
    );
    test_suite!(
        u64,
        u32,
        floor_u64_u32_nonzero_tests,
        floor_u64_u32_zero_tests
    );
    test_suite!(
        u64,
        u64,
        floor_u64_u64_nonzero_tests,
        floor_u64_u64_zero_tests
    );
}

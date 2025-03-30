#[inline]
pub(crate) const fn u128_to_u64_checked(x: u128) -> Option<u64> {
    if x > u64::MAX as u128 {
        None
    } else {
        Some(x as u64)
    }
}

#[cfg(test)]
pub mod test_utils {
    use proptest::prelude::*;
    use proptest::strategy::Union;

    use crate::{ArithTypes, Ratio};

    macro_rules! ratio_cases {
        (
            $N:ty, $D:ty
        ) => {
            impl Ratio<$N, $D> {
                prop_compose! {
                    pub fn prop_gte_one()
                        (d in 1..=<Ratio<$N, $D> as ArithTypes>::Min::MAX)
                        (n in d as $N..=<$N>::MAX, d in Just(d as $D)) -> Ratio<$N, $D> {
                            Ratio { n, d }
                        }
                }

                prop_compose! {
                    /// nonzero
                    pub fn prop_lte_one()
                        (d in 1..=<$D>::MAX)
                        (
                            n in 1..=(
                                if d as <Ratio<$N, $D> as ArithTypes>::Max
                                    > <$N>::MAX as <Ratio<$N, $D> as ArithTypes>::Max
                                {
                                    <$N>::MAX
                                } else {
                                    d as $N
                                }
                            ),
                            d in Just(d)
                        )
                        -> Ratio<$N, $D> {
                            Ratio { n, d }
                        }
                }

                prop_compose! {
                    pub fn prop_zero()
                        (n in any::<$N>(), d in any::<$D>())
                        (r in Union::new([
                            Just(Ratio { n: 0, d, }).boxed(),
                            Just(Ratio { n, d: 0 }).boxed()
                        ]))-> Ratio<$N, $D> {
                            r
                        }
                }
            }
        };
    }

    ratio_cases!(u8, u8);
    ratio_cases!(u8, u16);
    ratio_cases!(u8, u32);
    ratio_cases!(u8, u64);

    ratio_cases!(u16, u8);
    ratio_cases!(u16, u16);
    ratio_cases!(u16, u32);
    ratio_cases!(u16, u64);

    ratio_cases!(u32, u8);
    ratio_cases!(u32, u16);
    ratio_cases!(u32, u32);
    ratio_cases!(u32, u64);

    ratio_cases!(u64, u8);
    ratio_cases!(u64, u16);
    ratio_cases!(u64, u32);
    ratio_cases!(u64, u64);
}

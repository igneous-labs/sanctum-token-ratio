mod ceil;
mod floor;

pub use ceil::*;
pub use floor::*;

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::Ratio;

    use super::*;

    macro_rules! test_suite {
        (
            $N:ty, $D:ty,
            $test:ident
        ) => {
            proptest! {
                #[test]
                fn $test(
                    (amt, _amt_max, gte) in Ceil::<Ratio<$N, $D>>::prop_ratio_gte_one_amt_no_overflow(),
                    (aaf, _aaf_max, lte) in Floor::<Ratio<$N, $D>>::prop_ratio_lte_one_rev_no_overflow(),
                    any_u64: u64,
                ) {
                    let gte_ceil = gte;
                    let gte_floor = Floor(gte_ceil.0);
                    let lte_floor = lte;
                    let lte_ceil = Ceil(lte_floor.0);

                    // ceil should be at most floor + 1
                    for (amt, ceil, floor) in [
                        (amt, gte_ceil, gte_floor),
                        (any_u64, lte_ceil, lte_floor),
                    ] {
                        let app_ceil = ceil.apply(amt).unwrap();
                        let app_floor = floor.apply(amt).unwrap();
                        prop_assert!(
                            app_ceil - app_floor <= 1,
                            "floor > ceil {}. {} {} | {} {}",
                            amt, ceil, app_ceil, floor, app_floor,
                        );
                    }

                    // reverse ceiling should be <= reverse floor
                    for (aaf, ceil, floor) in [
                        (aaf, lte_ceil, lte_floor),
                        (any_u64, gte_ceil, gte_floor),
                    ] {
                        match (ceil.reverse(aaf), floor.reverse(aaf)) {
                            (Some(rev_ceil), Some(rev_floor)) => {
                                prop_assert!(
                                    rev_ceil.start() <= rev_floor.start() &&
                                    rev_ceil.end() <= rev_floor.end(),
                                    "rev_ceil > rev_floor {aaf}. {ceil} {rev_ceil:?} | {floor} {rev_floor:?}",
                                );
                            }
                            (None, None) => (),
                            (None, Some(f)) => {
                                // assert that ceil is indeed unattainable:
                                // - f should be single value
                                // - ceil.apply(f - 1) < aaf
                                // - ceil.apply(f) != aaf
                                // - ceil.apply(f + 1) > aaf
                                assert!(f.start() == f.end());
                                assert!(ceil.apply(f.start() - 1).unwrap() < aaf);
                                assert!(ceil.apply(*f.start()).unwrap() != aaf);
                                assert!(ceil.apply(f.start() + 1).unwrap() > aaf);
                            }
                            (Some(c), None) => {
                                // assert that floor is indeed unattainable:
                                // - c should be single value
                                // - floor.apply(c - 1) < aaf
                                // - floor.apply(c) != aaf
                                // - floor.apply(c + 1) > aaf
                                assert!(c.start() == c.end());
                                assert!(floor.apply(c.start() - 1).unwrap() < aaf);
                                assert!(floor.apply(*c.start()).unwrap() != aaf);
                                assert!(floor.apply(c.start() + 1).unwrap() > aaf);
                            }
                        }
                    }
                }
            }
        };
    }

    test_suite!(u8, u8, floor_ceil_cmp_u8_u8);
    test_suite!(u8, u16, floor_ceil_cmp_u8_u16);
    test_suite!(u8, u32, floor_ceil_cmp_u8_u32);
    test_suite!(u8, u64, floor_ceil_cmp_u8_u64);

    test_suite!(u16, u8, floor_ceil_cmp_u16_u8);
    test_suite!(u16, u16, floor_ceil_cmp_u16_u16);
    test_suite!(u16, u32, floor_ceil_cmp_u16_u32);
    test_suite!(u16, u64, floor_ceil_cmp_u16_u64);

    test_suite!(u32, u8, floor_ceil_cmp_u32_u8);
    test_suite!(u32, u16, floor_ceil_cmp_u32_u16);
    test_suite!(u32, u32, floor_ceil_cmp_u32_u32);
    test_suite!(u32, u64, floor_ceil_cmp_u32_u64);

    test_suite!(u64, u8, floor_ceil_cmp_u64_u8);
    test_suite!(u64, u16, floor_ceil_cmp_u64_u16);
    test_suite!(u64, u32, floor_ceil_cmp_u64_u32);
    test_suite!(u64, u64, floor_ceil_cmp_u64_u64);
}

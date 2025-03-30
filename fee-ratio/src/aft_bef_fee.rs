/// A token amount after the levying of fees and the amount of fees levied.
///
/// invariant: `self.rem() + self.fee() = self.bef_fee()`.
///
/// Fields are private to ensure invariant is never violated.
///
/// Use [`BefFee`] to build this struct
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct AftFee {
    rem: u64,
    fee: u64,
}

impl AftFee {
    /// The remaining token amount after fees have been levied
    #[inline]
    pub const fn rem(&self) -> u64 {
        self.rem
    }

    /// The fee amount that was levied
    #[inline]
    pub const fn fee(&self) -> u64 {
        self.fee
    }

    /// The original token amount before levying of fees.
    ///
    /// `self.rem() + self.fee()`
    #[inline]
    pub const fn bef_fee(&self) -> u64 {
        self.rem + self.fee
    }

    /// # Safety
    /// - `rem + fee` must not overflow
    #[inline]
    pub const unsafe fn new_unchecked(rem: u64, fee: u64) -> Self {
        Self { rem, fee }
    }
}

/// A token amount before the levying of fees
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct BefFee(pub u64);

impl BefFee {
    /// # Params
    /// - `fee`: the fee amount charged to be subtracted
    ///    from the encapsulated token amount
    ///
    /// # Returns
    /// The constructed [`AftFee`] or `None` if `fee > self.0`
    #[inline]
    pub const fn with_fee(self, fee: u64) -> Option<AftFee> {
        let rem = match self.0.checked_sub(fee) {
            None => return None,
            Some(r) => r,
        };
        Some(AftFee { rem, fee })
    }

    /// # Params
    /// - `rem`: the remaining amount after subtracting the fee charged
    ///   from the encapsulated token amount
    ///
    /// # Returns
    /// The constructed [`AftFee`] or `None` if `rem > self.0`.
    #[inline]
    pub const fn with_rem(self, rem: u64) -> Option<AftFee> {
        let fee = match self.0.checked_sub(rem) {
            None => return None,
            Some(r) => r,
        };
        Some(AftFee { rem, fee })
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    prop_compose! {
        fn cap_sample_out_of_range()
            (cap in any::<u64>())
            (
                sample in 0..=cap,
                out_of_range in cap.saturating_add(1)..=u64::MAX,
                cap in Just(cap)
            ) -> (u64, u64, Option<u64>) {
                (cap, sample, (cap != u64::MAX).then_some(out_of_range))
            }
    }

    proptest! {
        #[test]
        fn aft_bef_fee_invariant(
            (bef_fee, sample, _oor) in cap_sample_out_of_range()
        ) {
            let bef = BefFee(bef_fee);
            for a in [bef.with_fee(sample).unwrap(), bef.with_rem(sample).unwrap()] {
                prop_assert_eq!(a.fee() + a.rem(), a.bef_fee());
                prop_assert_eq!(a.bef_fee(), bef_fee);
            }
        }
    }

    proptest! {
        #[test]
        fn out_of_range_none(
            (bef_fee, _sample, oor) in cap_sample_out_of_range()
        ) {
            let bef = BefFee(bef_fee);
            if let Some(oor) = oor {
                for opt in [bef.with_fee(oor), bef.with_rem(oor)] {
                    prop_assert!(opt.is_none());
                }
            }
        }
    }
}

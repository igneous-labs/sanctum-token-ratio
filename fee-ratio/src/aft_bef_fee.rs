/// A token amount after the levying of fees and the amount of fees levied.
///
/// invariant: `self.rem() + self.fee() = self.before_fee()`.
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
    /// The constructed [`AftFee`] or `None` if `fee_charged > self.0`
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

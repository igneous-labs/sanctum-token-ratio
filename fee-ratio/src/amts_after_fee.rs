/// invariant: `self.rem() + self.fee() = self.before_fee()`.
///
/// Fields are private to ensure invariant is never violated.
///
/// Use [`AfterFeeBuilder`] to build this struct
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct AfterFee {
    rem: u64,
    fee: u64,
}

impl AfterFee {
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
    pub const fn before_fee(&self) -> u64 {
        self.rem + self.fee
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct AfterFeeBuilder(u64);

impl AfterFeeBuilder {
    /// Constructs a new `AfterFeeBuilder` from a
    /// token amount before the levying of the fee
    #[inline]
    pub const fn new(amount: u64) -> Self {
        Self(amount)
    }

    /// Returns the token amount before fee encapsulated by this builder
    #[inline]
    pub const fn before_fee(&self) -> u64 {
        self.0
    }

    /// # Params
    /// - `fee`: the fee amount charged to be subtracted
    ///    from the encapsulated token amount
    ///
    /// # Returns
    /// The constructed [`AfterFee`] or `None` if `fee_charged > amount`
    #[inline]
    pub const fn with_fee(self, fee: u64) -> Option<AfterFee> {
        let rem = match self.checked_sub(fee) {
            None => return None,
            Some(r) => r,
        };
        Some(AfterFee { rem, fee })
    }

    /// # Params
    /// - `rem`: the remaining amount after subtracting the fee charged
    ///   from the encapsulated token amount
    ///
    /// # Returns
    /// The constructed [`AfterFee`] or `None` if `rem > amount`.
    #[inline]
    pub const fn with_rem(self, rem: u64) -> Option<AfterFee> {
        let fee = match self.checked_sub(rem) {
            None => return None,
            Some(r) => r,
        };
        Some(AfterFee { rem, fee })
    }

    #[inline]
    const fn checked_sub(&self, amt: u64) -> Option<u64> {
        self.0.checked_sub(amt)
    }
}

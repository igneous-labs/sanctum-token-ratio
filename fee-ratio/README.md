# sanctum-fee-ratio

Abstractions over fees applied as ratios to `u64` quantities.

## Rationale

Token amounts on Solana are most commonly `u64`s. A very common operation is applying a fee to a token amount. These fees are usually calculated by multiplying the token amount with a ratio <=1.0. The remaining amount after substracting this product is the amount after fees. This library seeks to provide generalized code that can be reused across multiple such contexts.

## Example Usage

### Fee Application

```rust
use sanctum_fee_ratio::{Fee, ratio::{Ceil, Ratio}};

type FeeCeil = Fee<Ceil<Ratio<u16, u16>>>;
struct BpsFeeCeil(FeeCeil);

impl BpsFeeCeil {
    pub fn new(bps: u16) -> Option<Self> {
        FeeCeil::new(Ratio { n: bps, d: 10_000 }).map(Self)
    }
}

let four_bps_fee = BpsFeeCeil::new(4).unwrap();
let bef_fee = 1_000_000_001;
let aft_fee = four_bps_fee.0.apply(bef_fee).unwrap();

assert_eq!(aft_fee.rem(), 999_600_000);
assert_eq!(aft_fee.fee(), 400_001);
// bef_fee() performs rem() + fee() to get original amount before fee
assert_eq!(aft_fee.bef_fee(), bef_fee);
```

### Fee Reversal

Uses [`sanctum_token_ratio`]'s `reverse()` functionality to obtain a range of amount before fees from quantities after fees.

#### From `rem`

Calling `.apply()` to any number in the returned range is guaranteed to return an [`AftFee`] with the same `rem()`, but it might not have the same `fee()`

```rust
use sanctum_fee_ratio::{Fee, ratio::{Ceil, Ratio}};

type FeeCeil = Fee<Ceil<Ratio<u64, u64>>>;

let fee = FeeCeil::new(Ratio { n: 1, d: 10 }).unwrap();
let bef_fee = 1_000_000_001;
let aft_fee = fee.apply(bef_fee).unwrap();
assert_eq!(aft_fee.rem(), 900_000_000);
assert_eq!(aft_fee.fee(), 100_000_001);

let range = fee.reverse_from_rem(aft_fee.rem()).unwrap();

// min results in a different fee, but max doesnt

let min = fee.apply(*range.start()).unwrap();
assert_eq!(min.rem(), aft_fee.rem());
assert_eq!(min.fee(), 100_000_000);

let max = fee.apply(*range.end()).unwrap();
assert_eq!(max.rem(), aft_fee.rem());
assert_eq!(max.fee(), 100_000_001);
```

#### From `fee`

Calling `.apply()` to any number in the returned range is guaranteed to return an [`AftFee`] with the same `fee()`, but it might not have the same `rem()`

```rust
use sanctum_fee_ratio::{Fee, ratio::{Floor, Ratio}};

type FeeFloor = Fee<Floor<Ratio<u64, u64>>>;

let fee = FeeFloor::new(Ratio { n: 1, d: 10 }).unwrap();
let bef_fee = 1_000_000_001;
let aft_fee = fee.apply(bef_fee).unwrap();
assert_eq!(aft_fee.rem(), 900_000_001);
assert_eq!(aft_fee.fee(), 100_000_000);

let range = fee.reverse_from_fee(aft_fee.fee()).unwrap();

// both min and max result in a different `rem()`

let min = fee.apply(*range.start()).unwrap();
assert_eq!(min.rem(), 900_000_000);
assert_eq!(min.fee(), aft_fee.fee());

let max = fee.apply(*range.end()).unwrap();
assert_eq!(max.rem(), 900_000_009);
assert_eq!(max.fee(), aft_fee.fee());
```

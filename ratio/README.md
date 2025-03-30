# sanctum-token-ratio

A library for applying ratios to `u64` quantities.

## Rationale

Token amounts on Solana are most commonly `u64`s. A very common operation is applying ratios to a token amount e.g. determining proportional ownership of a pool. This library seeks to provide generalized code that can be reused across multiple such contexts.

## Example Usage

### Ratio Application

```rust
use sanctum_u64_ratio::{Ratio, Ceil, Floor};

let ratio: Ratio<u8, u16> = Ratio {
    n: 1,
    d: 10_000
};

assert_eq!(Floor(ratio).apply(10_001), Some(1));
assert_eq!(Ceil(ratio).apply(10_001), Some(2));
```

### Ratio Reversal

You can also reverse ratios, which returns the range of possible values that were fed into `.apply()`.

The ranges are inclusive, so any number in the returned range should result in the same value if you feed it into `.apply()` again.

However, the range might not be complete due to integer rounding. There may exist decimal numbers not in the range that will also result in the same `.apply()` result.

```rust
use sanctum_u64_ratio::{Ratio, Ceil, Floor};

let ratio: Ratio<u8, u16> = Ratio {
    n: 1,
    d: 10_000
};

let floor = Floor(ratio);
let floor_range = floor.reverse(1);
assert_eq!(floor_range, Some(10_000..=19_999));
assert_eq!(floor.apply(10_000), Some(1));
assert_eq!(floor.apply(19_999), Some(1));

let ceil = Ceil(ratio);
let ceil_range = ceil.reverse(2);
assert_eq!(ceil_range, Some(10_001..=20_000));
assert_eq!(ceil.apply(10_001), Some(2));
assert_eq!(ceil.apply(20_000), Some(2));
```

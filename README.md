# Sanctum Token Ratio

Token amounts on solana are most commonly `u64`s. A very common operation is applying ratios to a token amount e.g. determining proportional ownership of a pool. This workspace contains the following 2 `no-std` libraries for working with such operations:

- `sanctum-u64-ratio` for applying and reversing ratios to `u64` amounts, with options for controlling whether to `floor()` or `ceil()`
- `sanctum-fee-ratio` for applying and reversing fees that are expressed as ratios to `u64` amounts. Builds off `sanctum-u64-ratio`

See individual crates for more docs.

## MSRV

`rustc 1.79.0`, same vers as that used in `cargo-build-sbf v2`

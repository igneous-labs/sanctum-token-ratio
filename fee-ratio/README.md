# sanctum-fee-ratio

Abstractions over fees applied as ratios to `u64` quantities.

## Rationale

Token amounts on Solana are most commonly `u64`s. A very common operation is applying a fee to a token amount. These fees are usually saved as a ratio thats <1.0 that is applied to the token amount. The remaining amount after substracting this product is the amount after fees. This library seeks to provide generalized code that can be reused across multiple such contexts.

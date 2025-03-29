# Doc

Random notes

## Equivalence In Application of Fees

In general there are 2 ways of applying fee ratios:

1. take `fees_charged = amount * fee_num / fee_denom` and `amount_after_fee = amount - fee_charged`
2. take `amount_after_fee = amount * (fee_denom - fee_num) / fee_num` and `fee_charged = amount - amount_after_fee`

Where the division in both cases can either be floor or ceil.

They are actually equivalent, the ceiling div of one is the floor div of the other.

Let `n` be a `fee_num` and `d` be a `fee_denom` e.g. n = 1, d = 10 means a 10% fee.

Let `y` be amount after fees, `x` be amount before fees, `f` be fee amount

### Show ceil of method 1 is equivalent to floor of method 2

floor method 2:

```md
y = floor(x(d - n)/d)
```

ceil method 1:

```md
f = ceil(xn/d)
y = x - ceil(xn/d)
ceil(xn/d) = x - y
xn/d <= x - y < xn/d + 1

LHS:
y <= x(1 - n/d)

RHS:
x(1 - n/d) - 1 < y

x(1 - n/d) - 1 < y <= x(1 - n/d)
x(d - n)/d - 1 < y <= x(d - n)/d
y = floor(x(d - n)/d)
```

### Show floor of method 1 is equivalent to ceil of method 2

ceil method 2:

```md
y = ceil(x(d - n)/d)
```

floor method 1:

```md
f = floor(xn/d)
y = x - floor(xn/d)
floor(xn/d) = x - y
x - y <= xn/d < x - y + 1

LHS:
x(1 - n/d) <= y

RHS:
y < x(1 - n/d) + 1

x(1 - n/d) <= y < x(1 - n/d) + 1
x(d - n)/d <= y < x(d - n)/d + 1
y = ceil(x(d - n)/d)
```

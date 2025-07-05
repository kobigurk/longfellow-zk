#!/usr/bin/env python3

# Check the result we got
result_low = 0x3fffffffffffffff
result_high = 0x00000ffffffffffc
result = result_low | (result_high << 64)

print(f"Result = 0x{result:032x}")
print(f"Result = {result}")

# Compare with R
R = (1 << 128) % ((1 << 128) - (1 << 108) + 1)
print(f"\nR = 0x{R:032x}")
print(f"R = {R}")

if result == R:
    print("\n✅ Result equals R")
else:
    print("\n❌ Result does not equal R")
    print(f"Difference: {result - R}")

# Let's also check what we got before subtraction
before_sub_low = 0x4000000000000000
before_sub_high = 0xfffffffffffffffc
before_sub = before_sub_low | (before_sub_high << 64)

p = (1 << 128) - (1 << 108) + 1
print(f"\n\nBefore subtraction: 0x{before_sub:032x}")
print(f"p = 0x{p:032x}")
print(f"Before - p = 0x{(before_sub - p):032x}")

# Is before_sub close to p?
print(f"\nbefore_sub / p = {before_sub / p}")
print(f"before_sub mod p = {before_sub % p}")
print(f"before_sub mod p = 0x{(before_sub % p):032x}")
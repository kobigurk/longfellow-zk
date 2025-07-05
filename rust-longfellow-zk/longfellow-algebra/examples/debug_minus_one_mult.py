#!/usr/bin/env python3

# Debug (-1) * (-1) multiplication

p = 2**128 - 2**108 + 1
print(f"p = {p}")
print(f"p hex = 0x{p:032x}")

minus_one = p - 1
print(f"-1 = p-1 = {minus_one}")
print(f"-1 hex = 0x{minus_one:032x}")

# Compute (-1) * (-1) mod p
result = (minus_one * minus_one) % p
print(f"(-1) * (-1) mod p = {result}")
print(f"Result hex = 0x{result:032x}")

# Check intermediate values
product = minus_one * minus_one
print(f"\nIntermediate product (before mod):")
print(f"(-1) * (-1) = {product}")
print(f"Product hex = 0x{product:064x}")

# Check the division
quotient = product // p
remainder = product % p
print(f"\nDivision check:")
print(f"quotient = {quotient}")
print(f"remainder = {remainder}")
print(f"quotient * p + remainder = {quotient * p + remainder}")
print(f"Matches original product: {quotient * p + remainder == product}")

# The result should be 1
print(f"\nExpected result: 1")
print(f"Actual result: {result}")
print(f"Correct: {result == 1}")

# Let's also check what the product looks like in 128-bit limbs
print(f"\nProduct as 128-bit limbs:")
low_128 = product & ((1 << 128) - 1)
high_128 = product >> 128
print(f"Low 128 bits: 0x{low_128:032x}")
print(f"High 128 bits: 0x{high_128:032x}")

# And the modulus
print(f"\nModulus as 128-bit:")
print(f"p: 0x{p:032x}")

# Check if this is causing overflow in Rust
print(f"\nChecking for potential overflow issues:")
print(f"Product requires {product.bit_length()} bits")
print(f"2^256 = {2**256}")
print(f"Product < 2^256: {product < 2**256}")

if product >= 2**256:
    print("⚠️  Product exceeds 256 bits - potential overflow in Rust!")
else:
    print("✓ Product fits in 256 bits")
#!/usr/bin/env python3

p = (1 << 128) - (1 << 108) + 1
print(f"p = {p}")
print(f"p = 0x{p:032x}")

# Check divisibility
print(f"\np mod 2 = {p % 2}")
print(f"p mod 3 = {p % 3}")
print(f"p mod 6 = {p % 6}")

print(f"\n(p-1) mod 2 = {(p-1) % 2}")
print(f"(p-1) mod 3 = {(p-1) % 3}")
print(f"(p-1) mod 6 = {(p-1) % 6}")

# Since (p-1) mod 6 = 0, we have p ≡ 1 (mod 6)

# Check the order of 6 in the multiplicative group
# The order is the smallest k such that 6^k ≡ 1 (mod p)

print("\nChecking powers of 6:")
val = 1
for i in range(1, 20):
    val = (val * 6) % p
    print(f"6^{i} mod p = {val}")
    if val == 1:
        print(f"Order of 6 is {i}")
        break

# Let's also check what 6R mod p is in Montgomery form
R = (1 << 128) % p
print(f"\nR = {R}")
print(f"6R mod p = {(6 * R) % p}")
print(f"6R mod p = 0x{(6 * R) % p:032x}")

# And check the inverse
six_inv = pow(6, -1, p)
print(f"\n6^(-1) mod p = {six_inv}")
print(f"6^(-1) mod p = 0x{six_inv:032x}")

# In Montgomery form
six_inv_R = (six_inv * R) % p
print(f"\n6^(-1) * R mod p = {six_inv_R}")
print(f"6^(-1) * R mod p = 0x{six_inv_R:032x}")

# Check if there's something special about the product
print(f"\n6 * 6^(-1) mod p = {(6 * six_inv) % p}")

# Check 6R * (6^(-1))R in Montgomery
six_R = (6 * R) % p
product_before_reduction = (six_R * six_inv_R) % p
print(f"\n(6R) * (6^(-1)R) mod p = {product_before_reduction}")
print(f"(6R) * (6^(-1)R) mod p = 0x{product_before_reduction:032x}")

# This should give R^2, which after Montgomery reduction gives R
R2 = (R * R) % p
print(f"\nR^2 mod p = {R2}")
print(f"R^2 mod p = 0x{R2:032x}")

if product_before_reduction == R2:
    print("\n✅ (6R) * (6^(-1)R) = R^2 (correct before reduction)")
#!/usr/bin/env python3

# Trace Montgomery multiplication of 6 * 6^(-1)

p = (1 << 128) - (1 << 108) + 1
R = (1 << 128) % p

print(f"p = 0x{p:032x}")
print(f"R = 0x{R:032x}")

# In Montgomery form
six_R = (6 * R) % p
six_inv = pow(6, -1, p)
six_inv_R = (six_inv * R) % p

print(f"\n6R mod p = 0x{six_R:032x}")
print(f"6^(-1)R mod p = 0x{six_inv_R:032x}")

# Montgomery multiplication: (6R) * (6^(-1)R) * R^(-1) mod p
# This should give R (which represents 1)

# First compute (6R) * (6^(-1)R) mod p
product = (six_R * six_inv_R) % p
print(f"\n(6R) * (6^(-1)R) mod p = 0x{product:032x}")

# This should be R^2
R2 = (R * R) % p
print(f"R^2 mod p = 0x{R2:032x}")
print(f"Match: {product == R2}")

# Now Montgomery reduce R^2 to get R
# Montgomery reduction of x computes x * R^(-1) mod p
# So R^2 * R^(-1) = R mod p

# But wait, let's check something else
# What if the multiplication is producing exactly p?

# Check if 6R * (6^(-1)R) could equal p
print(f"\n\nChecking if multiplication could produce p:")
print(f"p = {p}")
print(f"6R * 6^(-1)R = {six_R * six_inv_R}")
print(f"(6R * 6^(-1)R) // p = {(six_R * six_inv_R) // p}")
print(f"(6R * 6^(-1)R) mod p = {(six_R * six_inv_R) % p}")

# Actually, let's trace the Montgomery multiplication manually
print("\n\nManual Montgomery multiplication trace:")

# Convert to limbs
def to_limbs(x):
    return [x & 0xffffffffffffffff, (x >> 64) & 0xffffffffffffffff]

six_R_limbs = to_limbs(six_R)
six_inv_R_limbs = to_limbs(six_inv_R)

print(f"6R limbs = [{six_R_limbs[0]:016x}, {six_R_limbs[1]:016x}]")
print(f"6^(-1)R limbs = [{six_inv_R_limbs[0]:016x}, {six_inv_R_limbs[1]:016x}]")

# Multiply to get 4 limbs
prod_limbs = [0, 0, 0, 0]
for i in range(2):
    for j in range(2):
        val = six_R_limbs[i] * six_inv_R_limbs[j]
        prod_limbs[i+j] += val & 0xffffffffffffffff
        prod_limbs[i+j+1] += val >> 64

# Handle carries
for i in range(3):
    if prod_limbs[i] >= (1 << 64):
        prod_limbs[i+1] += prod_limbs[i] >> 64
        prod_limbs[i] &= 0xffffffffffffffff

print(f"\nProduct limbs (before reduction):")
for i in range(4):
    if prod_limbs[i] != 0:
        print(f"  limbs[{i}] = 0x{prod_limbs[i]:016x}")

# This should match R^2
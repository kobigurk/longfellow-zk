#!/usr/bin/env python3

# Test Montgomery arithmetic for Fp128

# Field: p = 2^128 - 2^108 + 1
p = (1 << 128) - (1 << 108) + 1
print(f"p = 2^128 - 2^108 + 1")
print(f"p = 0x{p:032x}")
print(f"p = {p}")

# Montgomery parameter R = 2^128 mod p
R = (1 << 128) % p
print(f"\nR = 2^128 mod p")
print(f"R = 0x{R:032x}")
print(f"R = {R}")
print(f"R = 2^108 - 1 = {(1 << 108) - 1}")
assert R == (1 << 108) - 1

# R^2 mod p
R2 = (R * R) % p
print(f"\nR^2 mod p = 0x{R2:032x}")
print(f"R^2 mod p = {R2}")

# Montgomery form of 1 is R
print(f"\nMontgomery form of 1 = R = 0x{R:032x}")

# To convert from Montgomery form, we compute a * R^(-1) mod p
# Find R^(-1) mod p using extended GCD
def egcd(a, b):
    if a == 0:
        return b, 0, 1
    gcd, x1, y1 = egcd(b % a, a)
    x = y1 - (b // a) * x1
    y = x1
    return gcd, x, y

gcd, x, _ = egcd(R, p)
R_inv = x % p
print(f"\nR^(-1) mod p = 0x{R_inv:032x}")

# Verify R * R^(-1) = 1 mod p
assert (R * R_inv) % p == 1

# from_montgomery(R) should give 1
result = (R * R_inv) % p
print(f"\nfrom_montgomery(R) = R * R^(-1) mod p = {result}")
assert result == 1

# Now let's check what -p^(-1) mod 2^64 should be
# First find p^(-1) mod 2^64
p_low = p & ((1 << 64) - 1)
print(f"\np mod 2^64 = {p_low}")

# Since p ≡ 1 (mod 2^64), we have p^(-1) ≡ 1 (mod 2^64)
# Therefore -p^(-1) ≡ -1 ≡ 2^64 - 1 (mod 2^64)
mprime = (1 << 64) - 1
print(f"-p^(-1) mod 2^64 = 0x{mprime:016x}")

# Test Montgomery reduction manually
print("\n\nManual Montgomery reduction test:")
print("Computing from_montgomery(R)...")

# Input: R
t = [0] * 4
t[0] = R & ((1 << 64) - 1)
t[1] = (R >> 64) & ((1 << 64) - 1)
print(f"Initial t = [0x{t[0]:016x}, 0x{t[1]:016x}, 0x{t[2]:016x}, 0x{t[3]:016x}]")

# Montgomery reduction
for i in range(2):
    m = (t[i] * mprime) & ((1 << 64) - 1)
    print(f"\nIteration {i}: m = 0x{m:016x}")
    
    # Add m * p to t
    carry = 0
    temp = m * p
    for j in range(4):
        if i + j < 4:
            val = t[i + j] + ((temp >> (64 * j)) & ((1 << 64) - 1)) + carry
            t[i + j] = val & ((1 << 64) - 1)
            carry = val >> 64
    
    print(f"After iteration {i}: t = [0x{t[0]:016x}, 0x{t[1]:016x}, 0x{t[2]:016x}, 0x{t[3]:016x}]")

# Result is in upper half
result_low = t[2]
result_high = t[3]
result = (result_high << 64) | result_low
print(f"\nResult = 0x{result:032x}")
print(f"Result = {result}")

# Check if this equals 1
if result == 1:
    print("✅ SUCCESS: from_montgomery(R) = 1")
else:
    print("❌ FAILED: from_montgomery(R) != 1")
    # Check if it needs reduction
    if result >= p:
        result_reduced = result - p
        print(f"After reduction: {result_reduced}")
        if result_reduced == 1:
            print("✅ After reduction, result = 1")
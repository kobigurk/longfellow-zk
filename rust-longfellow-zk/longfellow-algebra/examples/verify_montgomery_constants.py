#!/usr/bin/env python3

# Verify Montgomery constants are correct

p = 2**128 - 2**108 + 1

print(f"p = {p}")
print(f"p hex = 0x{p:032x}")

# R should be 2^128 mod p
R = (2**128) % p
print(f"R = 2^128 mod p = {R}")
print(f"R hex = 0x{R:032x}")

# R^2 should be (2^128)^2 mod p = 2^256 mod p  
R2 = (2**256) % p
print(f"R^2 = 2^256 mod p = {R2}")
print(f"R^2 hex = 0x{R2:032x}")

# INV should be the modular inverse of p mod 2^64
# We need p * INV ≡ -1 (mod 2^64)
# Since p = 2^128 - 2^108 + 1, we have p mod 2^64 = 1
# So we need 1 * INV ≡ -1 (mod 2^64)
# Therefore INV = 2^64 - 1 = 0xFFFFFFFFFFFFFFFF

INV = 2**64 - 1
print(f"INV = 2^64 - 1 = {INV}")
print(f"INV hex = 0x{INV:016x}")

# Verify INV is correct
p_low = p % (2**64)
product = (p_low * INV) % (2**64)
expected = (2**64 - 1) % (2**64)  # -1 mod 2^64

print(f"\nVerification:")
print(f"p mod 2^64 = {p_low}")
print(f"(p mod 2^64) * INV mod 2^64 = {product}")
print(f"Expected (-1 mod 2^64) = {expected}")
print(f"INV correct: {product == expected}")

# Now let's verify the constants from Rust
print(f"\nRust constants (from fp128.rs):")
rust_r = [0xFFFFFFFFFFFFFFFF, 0x00000FFFFFFFFFFF]
rust_r2 = [0xFFFEFFFFEFFFFF01, 0x000FDFFFFEFFFFEF]
rust_inv = 0xFFFFFFFFFFFFFFFF

rust_r_value = rust_r[0] | (rust_r[1] << 64)
rust_r2_value = rust_r2[0] | (rust_r2[1] << 64)

print(f"Rust R = {rust_r_value}")
print(f"Rust R^2 = {rust_r2_value}")
print(f"Rust INV = 0x{rust_inv:016x}")

print(f"\nComparisons:")
print(f"R correct: {rust_r_value == R}")
print(f"R^2 correct: {rust_r2_value == R2}")
print(f"INV correct: {rust_inv == INV}")

if rust_r_value != R:
    print(f"R ERROR: Expected {R}, got {rust_r_value}")
if rust_r2_value != R2:
    print(f"R^2 ERROR: Expected {R2}, got {rust_r2_value}")
if rust_inv != INV:
    print(f"INV ERROR: Expected {INV}, got {rust_inv}")

# Test Montgomery multiplication manually
print(f"\n=== Manual Montgomery test ===")

# Test omega * omega where omega = 164956748514267535023998284330560247862
omega_regular = 164956748514267535023998284330560247862

# Step 1: Convert to Montgomery form
omega_montgomery = (omega_regular * R) % p
print(f"omega regular = {omega_regular}")
print(f"omega Montgomery = {omega_montgomery}")

# Step 2: Multiply Montgomery forms  
product_naive = (omega_montgomery * omega_montgomery) % p
print(f"(omega_M * omega_M) mod p = {product_naive}")

# Step 3: Apply Montgomery reduction (divide by R)
omega2_montgomery = (product_naive * pow(R, p-2, p)) % p
print(f"Montgomery reduction result = {omega2_montgomery}")

# This should be omega^2 in Montgomery form
expected_omega2_regular = (omega_regular * omega_regular) % p
expected_omega2_montgomery = (expected_omega2_regular * R) % p

print(f"Expected omega^2 regular = {expected_omega2_regular}")
print(f"Expected omega^2 Montgomery = {expected_omega2_montgomery}")

print(f"Manual Montgomery result correct: {omega2_montgomery == expected_omega2_montgomery}")
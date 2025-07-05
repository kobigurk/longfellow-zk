#!/usr/bin/env python3

# Compute correct Montgomery constants for p = 2^128 - 2^108 + 1

def extended_gcd(a, b):
    if a == 0:
        return b, 0, 1
    gcd, x1, y1 = extended_gcd(b % a, a)
    x = y1 - (b // a) * x1
    y = x1
    return gcd, x, y

def mod_inverse(a, m):
    gcd, x, _ = extended_gcd(a, m)
    if gcd != 1:
        return None
    return x % m

# Define p = 2^128 - 2^108 + 1
p = (1 << 128) - (1 << 108) + 1
print(f"p = {p}")
print(f"p hex = 0x{p:032x}")

# Compute Montgomery constants
# R = 2^128 mod p
R = pow(2, 128, p)
print(f"R = {R}")
print(f"R hex = 0x{R:032x}")

# R^2 = 2^256 mod p
R2 = pow(2, 256, p)
print(f"R^2 = {R2}")
print(f"R^2 hex = 0x{R2:032x}")

# INV = (-p)^(-1) mod 2^64
# This is the modular inverse of -p modulo 2^64
inv_p = mod_inverse((-p) % (1 << 64), 1 << 64)
print(f"INV = {inv_p}")
print(f"INV hex = 0x{inv_p:016x}")

# Verify the constants
print(f"\nVerification:")
print(f"R * R^(-1) mod p = {(R * pow(R, -1, p)) % p} (should be 1)")
print(f"R^2 * R^(-2) mod p = {(R2 * pow(R2, -1, p)) % p} (should be 1)")

# Check INV: (-p) * INV ≡ 1 (mod 2^64)
check_inv = ((-p) * inv_p) % (1 << 64)
print(f"(-p) * INV mod 2^64 = {check_inv} (should be 1)")

# Let's also check what the current Rust constants are
print(f"\nCurrent Rust constants:")
print(f"R from Rust:")
print(f"  limbs[0] = 0xFFFFFFFFFFFFFFFF")
print(f"  limbs[1] = 0x00000FFFFFFFFFFF")
rust_r = 0xFFFFFFFFFFFFFFFF | (0x00000FFFFFFFFFFF << 64)
print(f"  as u128 = {rust_r}")
print(f"  matches computed R: {rust_r == R}")

print(f"R^2 from Rust:")
print(f"  limbs[0] = 0xFFFEFFFFEFFFFF01")
print(f"  limbs[1] = 0x000FDFFFFEFFFFEF")
rust_r2 = 0xFFFEFFFFEFFFFF01 | (0x000FDFFFFEFFFFEF << 64)
print(f"  as u128 = {rust_r2}")
print(f"  matches computed R^2: {rust_r2 == R2}")

print(f"INV from Rust: 0xFFFFFFFFFFFFFFFF")
print(f"  matches computed INV: {0xFFFFFFFFFFFFFFFF == inv_p}")

if 0xFFFFFFFFFFFFFFFF != inv_p:
    print(f"❌ INV constant is wrong in Rust!")
    print(f"  Should be: 0x{inv_p:016x}")
    print(f"  Current:   0xFFFFFFFFFFFFFFFF")
else:
    print(f"✅ INV constant is correct")
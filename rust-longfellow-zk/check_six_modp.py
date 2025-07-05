#!/usr/bin/env python3

# Check if 6 has special properties in this field
p = (1 << 128) - (1 << 108) + 1

print(f"p = 0x{p:032x}")
print(f"p = {p}")

# Check if 6 divides p-1 or p+1
print(f"\n(p-1) mod 6 = {(p-1) % 6}")
print(f"(p+1) mod 6 = {(p+1) % 6}")

# Compute 6^(-1) mod p
from math import gcd

g = gcd(6, p)
print(f"\ngcd(6, p) = {g}")

if g == 1:
    # Extended GCD to find inverse
    def egcd(a, b):
        if a == 0:
            return b, 0, 1
        gcd, x1, y1 = egcd(b % a, a)
        x = y1 - (b // a) * x1
        y = x1
        return gcd, x, y
    
    _, six_inv, _ = egcd(6, p)
    six_inv = six_inv % p
    
    print(f"\n6^(-1) mod p = {six_inv}")
    print(f"6^(-1) mod p = 0x{six_inv:032x}")
    
    # Verify
    product = (6 * six_inv) % p
    print(f"\n6 * 6^(-1) mod p = {product}")
    
    # Also check what our Rust code computed
    rust_six_inv = 0xd5554800000000000000000000000001
    print(f"\nRust computed 6^(-1) = 0x{rust_six_inv:032x}")
    
    product_rust = (6 * rust_six_inv) % p
    print(f"6 * (Rust 6^(-1)) mod p = {product_rust}")
else:
    print(f"6 is not invertible mod p!")
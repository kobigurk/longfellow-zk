#!/usr/bin/env python3

# Test the specific multiplication that's failing in Rust

p = 2**128 - 2**108 + 1
omega = 164956748514267535023998284330560247862

print(f"Testing omega * omega in Python:")
print(f"omega = {omega}")
print(f"p = {p}")

# Compute omega^2 in regular arithmetic
omega_squared = (omega * omega) % p
print(f"omega^2 = {omega_squared}")

# Convert to hex for comparison
print(f"omega^2 hex = 0x{omega_squared:032x}")

# Let's also compute this using Montgomery arithmetic manually
# to see what the correct Montgomery steps should be

R = 2**128 % p  # Montgomery constant R
R_inv = pow(R, p-2, p)  # R^(-1) mod p

print(f"\nMontgomery constants:")
print(f"R = 2^128 mod p = {R}")
print(f"R hex = 0x{R:032x}")

# In Montgomery form, omega is stored as omega * R mod p
omega_montgomery = (omega * R) % p
print(f"omega in Montgomery form = {omega_montgomery}")
print(f"omega Montgomery hex = 0x{omega_montgomery:032x}")

# Montgomery multiplication: (a*R) * (b*R) = (a*b*R^2)
# Then we need to divide by R to get (a*b*R)
omega_squared_montgomery_naive = (omega_montgomery * omega_montgomery) % p
print(f"\nNaive (omega*R)^2 = {omega_squared_montgomery_naive}")
print(f"Naive hex = 0x{omega_squared_montgomery_naive:032x}")

# Montgomery reduction: divide by R
omega_squared_montgomery = (omega_squared_montgomery_naive * R_inv) % p
print(f"After Montgomery reduction = {omega_squared_montgomery}")
print(f"Reduction hex = 0x{omega_squared_montgomery:032x}")

# This should equal omega^2 * R mod p
expected_montgomery_result = (omega_squared * R) % p
print(f"Expected Montgomery result = {expected_montgomery_result}")
print(f"Expected hex = 0x{expected_montgomery_result:032x}")

print(f"Montgomery reduction correct: {omega_squared_montgomery == expected_montgomery_result}")

# Convert Rust result to compare
rust_result = 78586892784590695660420324926014672584
rust_regular = (rust_result * R_inv) % p
print(f"\nRust result (Montgomery): {rust_result}")
print(f"Rust result hex = 0x{rust_result:032x}")
print(f"Rust result (regular) = {rust_regular}")
print(f"Rust regular hex = 0x{rust_regular:032x}")

print(f"Rust matches expected: {rust_result == expected_montgomery_result}")

# Let's check if the Rust result is off by some factor
if rust_result != expected_montgomery_result:
    ratio1 = expected_montgomery_result * pow(rust_result, p-2, p) % p
    ratio2 = rust_result * pow(expected_montgomery_result, p-2, p) % p
    print(f"Expected / Rust = {ratio1}")
    print(f"Rust / Expected = {ratio2}")
    
    # Check if it's off by R or R^2
    if ratio1 == R:
        print("Rust result is missing a factor of R")
    elif ratio1 == (R * R) % p:
        print("Rust result is missing a factor of R^2")
    elif ratio2 == R:
        print("Rust result has an extra factor of R")
    elif ratio2 == (R * R) % p:
        print("Rust result has an extra factor of R^2")
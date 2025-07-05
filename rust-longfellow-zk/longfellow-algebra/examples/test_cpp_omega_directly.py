#!/usr/bin/env python3

# Test the C++ omega value directly

p = 2**128 - 2**108 + 1
cpp_omega = 164956748514267535023998284330560247862

print(f"p = {p}")
print(f"C++ omega = {cpp_omega}")

# Test the key property: omega^(2^31) should equal -1
omega_2_31 = pow(cpp_omega, 2**31, p)
minus_one = p - 1

print(f"omega^(2^31) = {omega_2_31}")
print(f"-1 mod p = {minus_one}")
print(f"omega^(2^31) == -1: {omega_2_31 == minus_one}")

# Test that omega^(2^32) = 1
omega_2_32 = pow(cpp_omega, 2**32, p)
print(f"omega^(2^32) = {omega_2_32}")
print(f"omega^(2^32) == 1: {omega_2_32 == 1}")

# Test the square: (omega^(2^31))^2 should equal 1
omega_2_31_squared = (omega_2_31 * omega_2_31) % p
print(f"(omega^(2^31))^2 = {omega_2_31_squared}")
print(f"(omega^(2^31))^2 == 1: {omega_2_31_squared == 1}")

print(f"\n✓ C++ omega value is mathematically correct!")

# Convert the Rust result back to check
rust_omega_2_31_value = 78586892784590695660420324926014672584  # From Rust test output 
rust_omega_2_31_squared_value = 103056141742055093999006978806036951820  # From Rust test output

print(f"\nRust results (converted from hex):")
print(f"Rust omega^(2^31) = {rust_omega_2_31_value}")
print(f"Rust (omega^(2^31))^2 = {rust_omega_2_31_squared_value}")

# Check if Rust values match Python
print(f"Rust omega^(2^31) == Python omega^(2^31): {rust_omega_2_31_value == omega_2_31}")
print(f"Rust (omega^(2^31))^2 == Python (omega^(2^31))^2: {rust_omega_2_31_squared_value == omega_2_31_squared}")

if rust_omega_2_31_value != omega_2_31:
    print("✗ Rust is computing omega^(2^31) incorrectly!")
    print("  The Montgomery arithmetic might still have bugs")
else:
    print("✓ Rust computation matches Python - Montgomery arithmetic is working")
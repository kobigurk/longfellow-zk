#!/usr/bin/env python3

# Verify the exact value and test it step by step

p = 2**128 - 2**108 + 1
omega_32 = 124138436495952958347847942047415585016

print(f"p = {p}")
print(f"omega_32 = {omega_32}")
print(f"omega_32 < p? {omega_32 < p}")

# Check the bytes we're using
omega_bytes = omega_32.to_bytes(16, 'little')
print(f"omega_32 as bytes: {list(omega_bytes)}")

# Test step by step computation
print("\nStep by step verification in Python:")

# Test small powers manually
omega_2 = (omega_32 * omega_32) % p
omega_4 = (omega_2 * omega_2) % p
omega_8 = (omega_4 * omega_4) % p

print(f"omega^2 = {omega_2}")
print(f"omega^4 = {omega_4}")
print(f"omega^8 = {omega_8}")

# Compare with pow function
omega_2_pow = pow(omega_32, 2, p)
omega_4_pow = pow(omega_32, 4, p)
omega_8_pow = pow(omega_32, 8, p)

print(f"\nUsing pow function:")
print(f"omega^2 = {omega_2_pow}")
print(f"omega^4 = {omega_4_pow}")
print(f"omega^8 = {omega_8_pow}")

print(f"\nMatch? {omega_2 == omega_2_pow and omega_4 == omega_4_pow and omega_8 == omega_8_pow}")

# Test the critical power: 2^31
power_31 = 2**31
omega_2_31 = pow(omega_32, power_31, p)
print(f"\nomega^(2^31) = {omega_2_31}")
print(f"omega^(2^31) as hex = 0x{omega_2_31:032x}")

# This should be -1 if omega is a primitive 2nd root of unity
minus_one = p - 1
print(f"-1 (mod p) = {minus_one}")
print(f"-1 as hex = 0x{minus_one:032x}")
print(f"omega^(2^31) == -1? {omega_2_31 == minus_one}")

# Test the full power: 2^32
power_32 = 2**32
omega_2_32 = pow(omega_32, power_32, p)
print(f"\nomega^(2^32) = {omega_2_32}")
print(f"omega^(2^32) == 1? {omega_2_32 == 1}")

# Double-check our field arithmetic by testing that 2^31 * 2 = 2^32
power_31_doubled = (omega_2_31 * omega_2_31) % p
print(f"\n(omega^(2^31))^2 = {power_31_doubled}")
print(f"(omega^(2^31))^2 == omega^(2^32)? {power_31_doubled == omega_2_32}")

# Convert the results to little-endian bytes for comparison with Rust
omega_2_31_bytes = omega_2_31.to_bytes(16, 'little')
omega_2_32_bytes = omega_2_32.to_bytes(16, 'little')

print(f"\nomega^(2^31) as bytes: {list(omega_2_31_bytes)}")
print(f"omega^(2^32) as bytes: {list(omega_2_32_bytes)}")

# Convert back from Rust format to check
# The Rust values from our previous test:
rust_2_31_hex = "0xf6368a5307bf86dc412d37ea354eddc0"
rust_2_32_hex = "0xd8b2f2c8306439c74f52924647e5bf61"

rust_2_31_int = int(rust_2_31_hex, 16)
rust_2_32_int = int(rust_2_32_hex, 16)

print(f"\nRust omega^(2^31) = {rust_2_31_int}")
print(f"Python omega^(2^31) = {omega_2_31}")
print(f"Match? {rust_2_31_int == omega_2_31}")

print(f"\nRust omega^(2^32) = {rust_2_32_int}")
print(f"Python omega^(2^32) = {omega_2_32}")
print(f"Match? {rust_2_32_int == omega_2_32}")
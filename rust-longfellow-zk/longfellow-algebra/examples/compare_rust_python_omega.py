#!/usr/bin/env python3

# Compare Rust and Python results for the C++ omega value

p = 2**128 - 2**108 + 1
cpp_omega = 164956748514267535023998284330560247862

print(f"p = {p}")
print(f"C++ omega = {cpp_omega}")

# Test the computation that Rust is doing
print("\n=== Python computation ===")

# Test small powers first to see where divergence starts
python_powers = {}
for k in [1, 2, 4, 8, 16, 32]:
    result = pow(cpp_omega, k, p)
    python_powers[k] = result
    print(f"omega^{k} = {result}")
    print(f"omega^{k} hex = 0x{result:032x}")

# Test the problematic large power
omega_2_31_python = pow(cpp_omega, 2**31, p)
print(f"\nomega^(2^31) = {omega_2_31_python}")
print(f"omega^(2^31) hex = 0x{omega_2_31_python:032x}")

# Test (omega^(2^31))^2
omega_2_31_squared_python = (omega_2_31_python * omega_2_31_python) % p
print(f"(omega^(2^31))^2 = {omega_2_31_squared_python}")
print(f"(omega^(2^31))^2 hex = 0x{omega_2_31_squared_python:032x}")

print(f"\n-1 = {p-1}")
print(f"-1 hex = 0x{p-1:032x}")

print(f"\nomega^(2^31) == -1: {omega_2_31_python == p-1}")
print(f"(omega^(2^31))^2 == 1: {omega_2_31_squared_python == 1}")

print("\n=== Rust values for comparison ===")
# From the Rust output we saw earlier:
rust_omega_2_31 = 0x256fddde2611717e23957bc4aff2d35e
rust_omega_2_31_squared = 0x33adf0beac9fbf599e02a8eaa6d4192c

print(f"Rust omega^(2^31) = {rust_omega_2_31}")
print(f"Rust (omega^(2^31))^2 = {rust_omega_2_31_squared}")

print(f"\nComparison:")
print(f"omega^(2^31) matches: {omega_2_31_python == rust_omega_2_31}")
print(f"(omega^(2^31))^2 matches: {omega_2_31_squared_python == rust_omega_2_31_squared}")

if omega_2_31_python != rust_omega_2_31:
    print(f"\n✗ Rust omega^(2^31) computation is incorrect!")
    print(f"  Expected: {omega_2_31_python}")
    print(f"  Got: {rust_omega_2_31}")
    print(f"  This indicates a bug in Rust Montgomery arithmetic or power function")
else:
    print(f"\n✓ Rust omega^(2^31) computation is correct")
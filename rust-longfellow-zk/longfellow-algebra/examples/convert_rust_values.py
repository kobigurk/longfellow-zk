#!/usr/bin/env python3

# Convert the Rust hex outputs to decimal for comparison

# From the Rust test output:
# omega^(2^31) = FpGeneric(Nat(0x256fddde2611717e23957bc4aff2d35e))
# (omega^(2^31))^2 = FpGeneric(Nat(0x33adf0beac9fbf599e02a8eaa6d4192c))
# -1 = FpGeneric(Nat(0xfffff000000000000000000000000000))

rust_omega_2_31_hex = "0x256fddde2611717e23957bc4aff2d35e"
rust_omega_2_31_squared_hex = "0x33adf0beac9fbf599e02a8eaa6d4192c"
rust_minus_one_hex = "0xfffff000000000000000000000000000"

rust_omega_2_31_value = int(rust_omega_2_31_hex, 16)
rust_omega_2_31_squared_value = int(rust_omega_2_31_squared_hex, 16)
rust_minus_one_value = int(rust_minus_one_hex, 16)

print(f"Rust omega^(2^31) = {rust_omega_2_31_value}")
print(f"Rust (omega^(2^31))^2 = {rust_omega_2_31_squared_value}")
print(f"Rust -1 = {rust_minus_one_value}")

# Compare with mathematical values
p = 2**128 - 2**108 + 1
cpp_omega = 164956748514267535023998284330560247862

omega_2_31_correct = pow(cpp_omega, 2**31, p)
omega_2_31_squared_correct = (omega_2_31_correct * omega_2_31_correct) % p
minus_one_correct = p - 1

print(f"\nMathematically correct values:")
print(f"omega^(2^31) = {omega_2_31_correct}")
print(f"(omega^(2^31))^2 = {omega_2_31_squared_correct}")
print(f"-1 = {minus_one_correct}")

print(f"\nComparisons:")
print(f"omega^(2^31) correct: {rust_omega_2_31_value == omega_2_31_correct}")
print(f"(omega^(2^31))^2 correct: {rust_omega_2_31_squared_value == omega_2_31_squared_correct}")
print(f"-1 correct: {rust_minus_one_value == minus_one_correct}")

if rust_minus_one_value != minus_one_correct:
    print(f"\n✗ Rust -1 value is wrong!")
    print(f"  This suggests a fundamental bug in the field representation")
elif rust_omega_2_31_value != omega_2_31_correct:
    print(f"\n✗ Rust omega^(2^31) is wrong!")
    print(f"  This suggests a bug in the power function or Montgomery arithmetic")
elif rust_omega_2_31_squared_value != omega_2_31_squared_correct:
    print(f"\n✗ Rust (omega^(2^31))^2 is wrong!")
    print(f"  This suggests a bug in multiplication")
else:
    print(f"\n✓ All Rust values are mathematically correct!")
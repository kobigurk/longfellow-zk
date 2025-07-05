#!/usr/bin/env python3

# Compute omega^(2^31) step by step to compare with Rust

p = 2**128 - 2**108 + 1
omega = 164956748514267535023998284330560247862

print(f"p = {p}")
print(f"omega = {omega}")

# Compute omega^(2^31) using binary exponentiation step by step
exp = 2**31
print(f"Computing omega^{exp} using binary exponentiation")
print(f"exp = {exp} = 0b{exp:032b}")

result = 1
base = omega

# Process each bit of the exponent
for bit_pos in range(32):  # 2^31 has bit 31 set
    bit = (exp >> bit_pos) & 1
    print(f"Bit {bit_pos}: {bit}")
    
    if bit == 1:
        print(f"  Multiplying result by base: {result} * {base}")
        result = (result * base) % p
        print(f"  New result: {result}")
        print(f"  New result hex: 0x{result:032x}")
    
    if bit_pos < 31:  # Don't square after the last bit
        print(f"  Squaring base: {base}^2")
        base = (base * base) % p
        print(f"  New base: {base}")
        print(f"  New base hex: 0x{base:032x}")
    
    print()

print(f"Final result: {result}")
print(f"Final result hex: 0x{result:032x}")

print(f"\n-1 = {p-1}")
print(f"-1 hex = 0x{p-1:032x}")

print(f"\nResult == -1: {result == p-1}")

# Let's also compute omega^(2^30) and see if (omega^(2^30))^2 = omega^(2^31)
omega_2_30 = pow(omega, 2**30, p)
omega_2_30_squared = (omega_2_30 * omega_2_30) % p

print(f"\nVerification:")
print(f"omega^(2^30) = {omega_2_30}")
print(f"omega^(2^30) hex = 0x{omega_2_30:032x}")
print(f"(omega^(2^30))^2 = {omega_2_30_squared}")
print(f"(omega^(2^30))^2 hex = 0x{omega_2_30_squared:032x}")
print(f"Matches direct computation: {omega_2_30_squared == result}")
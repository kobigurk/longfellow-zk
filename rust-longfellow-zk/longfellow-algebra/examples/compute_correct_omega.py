#!/usr/bin/env python3

# Compute the correct omega value that should be a primitive 2^32 root of unity

p = 2**128 - 2**108 + 1

print(f"p = {p}")
print(f"p hex = 0x{p:032x}")

# For omega to be a primitive 2^32 root of unity:
# 1. omega^(2^32) = 1 (mod p)  
# 2. omega^(2^31) = -1 (mod p) (since omega^(2^32) = (omega^(2^31))^2 = 1)
# 3. omega^k ≠ 1 for k < 2^32

# First check if 2^32 divides p-1
p_minus_1 = p - 1
print(f"p-1 = {p_minus_1}")

# Factor p-1 to see if 2^32 divides it
factors_of_2 = 0
temp = p_minus_1
while temp % 2 == 0:
    temp //= 2
    factors_of_2 += 1

print(f"p-1 = 2^{factors_of_2} * {temp}")

if factors_of_2 >= 32:
    print(f"✓ 2^32 divides p-1 (p-1 has 2^{factors_of_2} as factor)")
else:
    print(f"✗ 2^32 does NOT divide p-1 (p-1 only has 2^{factors_of_2} as factor)")
    print("  Cannot have a primitive 2^32 root of unity!")
    exit(1)

# Compute a primitive 2^32 root of unity
# If g is a primitive root mod p, then omega = g^((p-1)/2^32) is a primitive 2^32 root of unity

# Use a simple generator: find a small number that generates the multiplicative group
def is_primitive_root(g, p):
    """Check if g is a primitive root modulo p"""
    if pow(g, (p-1)//2, p) == 1:  # g^((p-1)/2) should be ≠ 1 for primitive root
        return False
    
    # Check a few small factors
    for factor in [2, 3, 5, 7, 11, 13, 17, 19]:
        if (p-1) % factor == 0:
            if pow(g, (p-1)//factor, p) == 1:
                return False
    return True

# Find a primitive root
primitive_root = None
for g in range(2, 100):
    if is_primitive_root(g, p):
        primitive_root = g
        break

if primitive_root is None:
    print("Could not find primitive root in range 2-99")
    # Use a known approach: for p = 2^n - 2^k + 1, usually 3 or 5 work
    primitive_root = 3
    
print(f"Using primitive root candidate: {primitive_root}")

# Compute omega = g^((p-1)/2^32)
exponent = (p - 1) // (2**32)
omega = pow(primitive_root, exponent, p)

print(f"omega = {primitive_root}^((p-1)/2^32) mod p = {omega}")
print(f"omega hex = 0x{omega:032x}")

# Verify this is a 2^32 root of unity
omega_to_2_32 = pow(omega, 2**32, p)
print(f"omega^(2^32) mod p = {omega_to_2_32}")

if omega_to_2_32 == 1:
    print("✓ omega^(2^32) = 1 (correct)")
else:
    print("✗ omega^(2^32) ≠ 1 (incorrect)")

# Check that omega^(2^31) = -1
omega_to_2_31 = pow(omega, 2**31, p)
minus_one = p - 1
print(f"omega^(2^31) mod p = {omega_to_2_31}")
print(f"-1 mod p = {minus_one}")

if omega_to_2_31 == minus_one:
    print("✓ omega^(2^31) = -1 (correct)")
else:
    print("✗ omega^(2^31) ≠ -1 (incorrect)")

# Check that it's primitive (omega^k ≠ 1 for k < 2^32)
print("\nChecking primitivity (omega^k ≠ 1 for proper divisors of 2^32):")
for k in [1, 2, 4, 8, 16, 2**16, 2**20, 2**24, 2**28, 2**30]:
    omega_k = pow(omega, k, p)
    if omega_k == 1:
        print(f"✗ omega^{k} = 1 (not primitive)")
    else:
        print(f"✓ omega^{k} ≠ 1")

# Convert to little-endian bytes
omega_bytes = omega.to_bytes(16, 'little')
print(f"\nomega as little-endian bytes: {list(omega_bytes)}")
print(f"omega as little-endian bytes hex: {omega_bytes.hex()}")

# Compare with C++ value
cpp_omega = 164956748514267535023998284330560247862
print(f"\nC++ omega = {cpp_omega}")
print(f"Computed omega = {omega}")
print(f"Same value: {cpp_omega == omega}")

if cpp_omega != omega:
    print(f"\n✗ C++ omega is incorrect!")
    print(f"  C++ omega^(2^31) = {pow(cpp_omega, 2**31, p)}")
    print(f"  Should be -1 = {minus_one}")
    print(f"  Matches: {pow(cpp_omega, 2**31, p) == minus_one}")
else:
    print(f"\n✓ C++ omega is correct!")
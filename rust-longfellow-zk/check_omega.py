p = (1 << 128) - (1 << 108) + 1
print(f"p = {p}")
print(f"p = 0x{p:032x}")
print(f"p - 1 = {p - 1}")

# Factor p - 1
p_minus_1 = p - 1
print(f"\np - 1 = 2^108 * {(p - 1) >> 108}")

# The omega_32 value from C++ comment: 164956748514267535023998284330560247862
omega_32_from_comment = 164956748514267535023998284330560247862
print(f"\nomega_32 from comment = {omega_32_from_comment}")
print(f"omega_32 from comment = 0x{omega_32_from_comment:032x}")

# The bytes in the code (little-endian)
omega_32_bytes = [0x36, 0x0c, 0xda, 0x62, 0xfe, 0xea, 0x28, 0x7c,
                  0xce, 0x03, 0x89, 0x3f, 0xf2, 0x73, 0x50, 0x01]
omega_32_from_bytes = int.from_bytes(omega_32_bytes, 'little')
print(f"\nomega_32 from bytes = {omega_32_from_bytes}")
print(f"omega_32 from bytes = 0x{omega_32_from_bytes:032x}")

# Check if they match
print(f"\nDo they match? {omega_32_from_comment == omega_32_from_bytes}")

# Verify it's a 2^32 root of unity
print(f"\nChecking if it's a 2^32 root of unity:")
omega = omega_32_from_comment
print(f"omega^(2^32) mod p = {pow(omega, 1 << 32, p)}")
print(f"omega^(2^31) mod p = {pow(omega, 1 << 31, p)}")

# To get a 2nd root of unity, we need omega^(2^31)
omega_2 = pow(omega, 1 << 31, p)
print(f"\nomega_2 = omega^(2^31) = {omega_2}")
print(f"omega_2 = 0x{omega_2:032x}")
print(f"omega_2^2 mod p = {(omega_2 * omega_2) % p}")

# Is omega_2 = -1?
minus_one = p - 1
print(f"\n-1 mod p = {minus_one}")
print(f"omega_2 == -1? {omega_2 == minus_one}")
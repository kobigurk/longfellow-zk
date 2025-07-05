#!/usr/bin/env python3

# Compute larger powers of omega to find where Rust diverges

p = 2**128 - 2**108 + 1
omega = 164956748514267535023998284330560247862

print("Computing larger powers of omega:")

# Compute powers by repeated squaring
current = omega
power = 1

for i in range(15):  # Up to omega^(2^14) = omega^16384
    print(f"omega^{power} = 0x{current:032x}")
    current = (current * current) % p
    power *= 2

# Also compute specific powers that we tested
test_powers = [1024, 2048, 4096, 8192]
for exp in test_powers:
    result = pow(omega, exp, p)
    print(f"omega^{exp} = 0x{result:032x}")
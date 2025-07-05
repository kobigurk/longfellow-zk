#!/usr/bin/env python3

# Recalculate the 2^32 root of unity for p = 2^128 - 2^108 + 1

p = 2**128 - 2**108 + 1
print(f"p = {p}")
print(f"p in hex = 0x{p:032x}")

# p - 1 should be divisible by 2^32 for a 2^32 root of unity to exist
p_minus_1 = p - 1
print(f"p - 1 = {p_minus_1}")

# Factor p - 1
# p - 1 = 2^128 - 2^108 + 1 - 1 = 2^128 - 2^108 = 2^108 * (2^20 - 1)
print(f"p - 1 = 2^108 * (2^20 - 1)")
print(f"2^108 = {2**108}")
print(f"2^20 - 1 = {2**20 - 1}")

# Check if p - 1 is divisible by 2^32
factor_2_32 = 2**32
quotient, remainder = divmod(p_minus_1, factor_2_32)
print(f"\n(p - 1) / 2^32 = {quotient}, remainder = {remainder}")

if remainder == 0:
    print("✓ p - 1 is divisible by 2^32, so 2^32 roots of unity exist")
    
    # Find a generator (primitive root)
    # We'll try small values
    for g in range(2, 100):
        # Check if g^((p-1)/2) ≠ 1 (Legendre symbol test for primitivity)
        if pow(g, (p - 1) // 2, p) != 1:
            # Check if g has order p - 1
            if pow(g, p - 1, p) == 1:
                print(f"Found potential generator: g = {g}")
                
                # Compute g^((p-1)/2^32) mod p
                exponent = (p - 1) // (2**32)
                omega_32 = pow(g, exponent, p)
                print(f"omega_32 = {g}^{exponent} mod p = {omega_32}")
                
                # Verify it's a 2^32 root of unity
                test = pow(omega_32, 2**32, p)
                print(f"omega_32^(2^32) mod p = {test}")
                
                if test == 1:
                    print("✓ This is a valid 2^32 root of unity")
                    
                    # Check if it's primitive (order exactly 2^32)
                    primitive = True
                    for k in range(1, 32):
                        if pow(omega_32, 2**k, p) == 1:
                            print(f"omega_32^(2^{k}) = 1, so order is 2^{k}, not 2^32")
                            primitive = False
                            break
                    
                    if primitive:
                        print(f"✓ This is a primitive 2^32 root of unity")
                        
                        # Convert to bytes
                        omega_bytes = omega_32.to_bytes(16, 'little')
                        print(f"omega_32 as little-endian bytes: {list(omega_bytes)}")
                        print(f"omega_32 as hex bytes: {omega_bytes.hex()}")
                        
                        # The value I had before
                        old_value = 124138436495952958347847942047415585016
                        print(f"\nComparison with old value:")
                        print(f"Old value = {old_value}")
                        print(f"New value = {omega_32}")
                        print(f"Same? {old_value == omega_32}")
                        
                        break
                else:
                    print("✗ Not a 2^32 root of unity")
            
            # Only check first few generators
            if g > 20:
                break
else:
    print("✗ p - 1 is not divisible by 2^32, so 2^32 roots of unity don't exist")
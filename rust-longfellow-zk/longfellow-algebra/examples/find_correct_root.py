#!/usr/bin/env python3

# Find the correct primitive 2^32 root of unity more carefully

p = 2**128 - 2**108 + 1
print(f"p = {p}")
print(f"p in hex = 0x{p:032x}")

# Verify field structure
p_minus_1 = p - 1
print(f"p - 1 = {p_minus_1}")

# p - 1 = 2^128 - 2^108 = 2^108 * (2^20 - 1)
factor_2_108 = 2**108
factor_2_20_minus_1 = 2**20 - 1

print(f"p - 1 = 2^108 * (2^20 - 1)")
print(f"2^108 = {factor_2_108}")
print(f"2^20 - 1 = {factor_2_20_minus_1}")
print(f"Product check: {factor_2_108 * factor_2_20_minus_1 == p_minus_1}")

# The maximum possible order of a 2-group element is 2^108
# But we want a 2^32 root of unity, so we need 2^32 to divide p-1
# Since p-1 = 2^108 * (odd number), we have 2^108 divides p-1
# So 2^32 certainly divides p-1

print(f"\nFor 2^32 roots of unity to exist, 2^32 must divide p-1")
quotient_32 = (p - 1) // (2**32)
remainder_32 = (p - 1) % (2**32)
print(f"(p-1) / 2^32 = {quotient_32}, remainder = {remainder_32}")

if remainder_32 == 0:
    print("✓ 2^32 divides p-1")
    
    # Now find a primitive root more carefully
    # We need an element g such that ord(g) = p-1
    print("\nSearching for primitive roots:")
    
    for g in range(2, 50):
        # Quick check: g^((p-1)/2) should be -1 for primitive roots
        legendre = pow(g, (p - 1) // 2, p)
        if legendre == p - 1:  # -1 mod p
            print(f"g = {g} passes Legendre test")
            
            # More thorough check: g should have order exactly p-1
            # Check that g^(2^k) ≠ 1 for k < 108
            is_primitive = True
            
            # Check small prime factors of p-1
            # p-1 = 2^108 * (2^20 - 1)
            # 2^20 - 1 = 1048575 = 3 * 5^2 * 11 * 31 * 41
            
            prime_factors = [3, 5, 11, 31, 41]
            for prime in prime_factors:
                if pow(g, (p - 1) // prime, p) == 1:
                    print(f"  g = {g} has order divisible by {prime}, not primitive")
                    is_primitive = False
                    break
            
            # Check powers of 2
            if is_primitive:
                for k in range(1, 108):
                    if pow(g, (p - 1) // (2**k), p) == 1:
                        print(f"  g = {g} has order divisible by 2^{k}, not primitive")
                        is_primitive = False
                        break
            
            if is_primitive:
                print(f"✓ g = {g} appears to be a primitive root")
                
                # Compute omega_32 = g^((p-1)/2^32)
                exponent = (p - 1) // (2**32)
                omega_32 = pow(g, exponent, p)
                print(f"omega_32 = {g}^{exponent} mod p = {omega_32}")
                
                # Verify it's a primitive 2^32 root of unity
                print("\nVerifying omega_32:")
                
                # Check omega_32^(2^32) = 1
                test_full = pow(omega_32, 2**32, p)
                print(f"omega_32^(2^32) = {test_full}")
                if test_full == 1:
                    print("✓ omega_32^(2^32) = 1")
                else:
                    print("✗ omega_32^(2^32) ≠ 1")
                    continue
                
                # Check omega_32^(2^k) ≠ 1 for k < 32 (primitivity)
                primitive_32 = True
                for k in range(1, 32):
                    test_k = pow(omega_32, 2**k, p)
                    if test_k == 1:
                        print(f"omega_32^(2^{k}) = 1, so order is 2^{k}, not 2^32")
                        primitive_32 = False
                        break
                
                if primitive_32:
                    print("✓ omega_32 is a primitive 2^32 root of unity")
                    
                    # Check the specific value: omega_32^(2^31) should be -1
                    omega_31 = pow(omega_32, 2**31, p)
                    print(f"omega_32^(2^31) = {omega_31}")
                    if omega_31 == p - 1:
                        print("✓ omega_32^(2^31) = -1")
                    else:
                        print(f"✗ omega_32^(2^31) = {omega_31} ≠ -1")
                    
                    # Convert to bytes for Rust
                    omega_bytes = omega_32.to_bytes(16, 'little')
                    print(f"\nCorrect omega_32 bytes: {list(omega_bytes)}")
                    print(f"Hex: {omega_bytes.hex()}")
                    
                    # Compare with our current value
                    current_value = 124138436495952958347847942047415585016
                    if omega_32 == current_value:
                        print(f"✓ Matches our current value {current_value}")
                    else:
                        print(f"✗ Different from our current value {current_value}")
                        print(f"  New value: {omega_32}")
                    
                    break
                else:
                    print("✗ omega_32 is not primitive")
        
        # Only check first few candidates
        if g > 10:
            break
else:
    print("✗ 2^32 does not divide p-1")
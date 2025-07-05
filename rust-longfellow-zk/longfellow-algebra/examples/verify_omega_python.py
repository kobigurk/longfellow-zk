#!/usr/bin/env python3

# Verify if our omega value is actually correct in Python

p = 2**128 - 2**108 + 1
omega_32 = 124138436495952958347847942047415585016

print(f"Testing omega = {omega_32}")
print(f"p = {p}")

# Check that omega^(2^32) = 1
omega_power_32 = pow(omega_32, 2**32, p)
print(f"omega^(2^32) = {omega_power_32}")
print(f"omega^(2^32) == 1? {omega_power_32 == 1}")

# Check that omega^(2^31) = -1
omega_power_31 = pow(omega_32, 2**31, p)
print(f"omega^(2^31) = {omega_power_31}")
print(f"omega^(2^31) == p-1? {omega_power_31 == p-1}")

# Double-check: (omega^(2^31))^2 should equal omega^(2^32)
omega_31_squared = (omega_power_31 * omega_power_31) % p
print(f"(omega^(2^31))^2 = {omega_31_squared}")
print(f"(omega^(2^31))^2 == omega^(2^32)? {omega_31_squared == omega_power_32}")

# If these don't match, then the omega value is wrong
if omega_power_32 != 1:
    print("\n❌ ERROR: omega is not a 2^32 root of unity!")
    print("The Python calculation was wrong.")
    
    # Try to find the correct omega using a known method
    print("\nTrying to find a correct 2^32 root of unity...")
    
    # Use a different approach: find any element of order 2^32
    # We know that there are φ(2^32) = 2^31 such elements
    
    # Try powers of a simple primitive root
    for g in [2, 3, 5, 7, 11, 13, 17, 19]:
        print(f"\nTrying generator g = {g}")
        
        # Check if g is actually a generator by testing g^((p-1)/2) = -1
        test = pow(g, (p-1)//2, p)
        if test == p - 1:
            print(f"  g = {g} passes generator test")
            
            # Compute candidate omega = g^((p-1)/2^32)
            exp = (p - 1) // (2**32)
            candidate = pow(g, exp, p)
            
            # Test if it's a 2^32 root of unity
            test_32 = pow(candidate, 2**32, p)
            test_31 = pow(candidate, 2**31, p)
            
            print(f"  candidate = {candidate}")
            print(f"  candidate^(2^32) = {test_32}")
            print(f"  candidate^(2^31) = {test_31}")
            
            if test_32 == 1 and test_31 == p - 1:
                print(f"  ✅ Found correct omega = {candidate}")
                
                # Convert to bytes
                omega_bytes = candidate.to_bytes(16, 'little')
                print(f"  Correct bytes: {list(omega_bytes)}")
                
                # Compare with current value
                if candidate == omega_32:
                    print(f"  Same as current value")
                else:
                    print(f"  DIFFERENT from current value {omega_32}")
                break
            else:
                print(f"  ❌ Not a valid omega")
        else:
            print(f"  g = {g} is not a generator")
else:
    print("\n✅ omega is a valid 2^32 root of unity in Python")
    if omega_power_31 == p - 1:
        print("✅ omega is a primitive 2^32 root of unity in Python")
        print("The issue must be in the Rust implementation")
    else:
        print("❌ omega is not primitive (omega^(2^31) ≠ -1)")
#!/usr/bin/env python3

# Test the C++ omega value

p = 2**128 - 2**108 + 1
cpp_omega = 164956748514267535023998284330560247862

print(f"Testing C++ omega = {cpp_omega}")
print(f"p = {p}")

# Test if this is a 2^32 root of unity
omega_2_32 = pow(cpp_omega, 2**32, p)
print(f"cpp_omega^(2^32) = {omega_2_32}")
print(f"cpp_omega^(2^32) == 1? {omega_2_32 == 1}")

# Test if this is primitive
omega_2_31 = pow(cpp_omega, 2**31, p)
print(f"cpp_omega^(2^31) = {omega_2_31}")
print(f"cpp_omega^(2^31) == p-1? {omega_2_31 == p - 1}")

if omega_2_32 == 1:
    print("✅ C++ omega is a 2^32 root of unity")
    
    if omega_2_31 == p - 1:
        print("✅ C++ omega is a primitive 2^32 root of unity")
        
        # Convert to bytes for Rust
        cpp_omega_bytes = cpp_omega.to_bytes(16, 'little')
        print(f"C++ omega bytes: {list(cpp_omega_bytes)}")
        print(f"Hex: {cpp_omega_bytes.hex()}")
        
    else:
        print("❌ C++ omega is not primitive")
        
        # Find what order it actually has
        for k in range(1, 32):
            test = pow(cpp_omega, 2**k, p)
            if test == 1:
                print(f"C++ omega has order 2^{k}")
                break
else:
    print("❌ C++ omega is not a 2^32 root of unity")
    
    # Test smaller orders
    for k in range(1, 33):
        test = pow(cpp_omega, 2**k, p)
        if test == 1:
            print(f"C++ omega has order 2^{k}")
            break
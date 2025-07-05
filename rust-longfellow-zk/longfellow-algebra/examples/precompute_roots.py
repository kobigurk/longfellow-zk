#!/usr/bin/env python3

# Pre-compute all the root of unity values we need

p = 2**128 - 2**108 + 1
cpp_omega_32 = 164956748514267535023998284330560247862

print("Pre-computing roots of unity for common sizes:")

# Common sizes needed for FFT: powers of 2 up to 2^20
sizes = [2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536]

roots = {}

for n in sizes:
    log_n = n.bit_length() - 1  # log2(n)
    exponent = 2**(32 - log_n)
    omega_n = pow(cpp_omega_32, exponent, p)
    
    roots[n] = omega_n
    
    print(f"omega_{n} = cpp_omega_32^(2^{32-log_n}) = {omega_n}")
    
    # Verify it's correct
    test = pow(omega_n, n, p)
    if test == 1:
        print(f"  ✓ omega_{n}^{n} = 1")
    else:
        print(f"  ✗ omega_{n}^{n} = {test} ≠ 1")
    
    # Check if it's primitive
    primitive = True
    for k in range(1, log_n + 1):
        if pow(omega_n, n // (2**k), p) == 1:
            print(f"  omega_{n} has order {n // (2**k)}, not {n}")
            primitive = False
            break
    
    if primitive:
        print(f"  ✓ omega_{n} is primitive")
    
    # Convert to bytes for Rust
    omega_bytes = omega_n.to_bytes(16, 'little')
    print(f"  bytes: {list(omega_bytes)}")
    print()

# Generate Rust code
print("Rust lookup table:")
print("match n {")
for n in sizes:
    omega_bytes = roots[n].to_bytes(16, 'little')
    bytes_str = ", ".join(f"0x{b:02x}" for b in omega_bytes)
    print(f"    {n} => Some(Self::from_bytes_le(&[{bytes_str}]).ok()?),")

print("    _ => None,")
print("}")
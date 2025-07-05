#!/usr/bin/env python3

# Compare intermediate steps between Python and Rust

p = 2**128 - 2**108 + 1
omega_32 = 124138436495952958347847942047415585016

print("Computing powers step by step in Python:")

current = omega_32
for k in range(1, 32):
    current = (current * current) % p
    power = 2**k
    print(f"omega^(2^{k}) = {current}")
    
    # Convert to little-endian bytes for comparison with Rust
    if k >= 28:  # Only print bytes for the last few steps
        current_bytes = current.to_bytes(16, 'little')
        print(f"  as hex: 0x{current:032x}")
        print(f"  as bytes: {list(current_bytes)}")

print("\nFrom Rust debug output, we had:")
rust_results = [
    (28, "0xae4675622738b7fbf5e17029ac971d5a"),
    (29, "0xae52b544cb71ec4e56447ba86c6446ac"), 
    (30, "0xd232834fc3440945f4c8225eb4a90196"),
    (31, "0xf6368a5307bf86dc412d37ea354eddc0"),
]

print("Comparing last few steps:")
for k, rust_hex in rust_results:
    rust_int = int(rust_hex, 16)
    
    # Compute Python result for this k
    python_current = omega_32
    for i in range(k):
        python_current = (python_current * python_current) % p
    
    print(f"k={k}:")
    print(f"  Python: 0x{python_current:032x}")
    print(f"  Rust:   {rust_hex}")
    print(f"  Match:  {python_current == rust_int}")
    
    if python_current != rust_int:
        print(f"  ❌ DIVERGENCE at k={k}!")
        break
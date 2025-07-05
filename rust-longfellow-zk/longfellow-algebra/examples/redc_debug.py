#!/usr/bin/env python3

# Debug Montgomery REDC step by step

def debug_redc(t, p, inv):
    """
    Debug Montgomery REDC algorithm step by step
    t: input value (product of two Montgomery form numbers)
    p: modulus
    inv: (-p)^(-1) mod 2^64
    """
    print(f"=== REDC DEBUG ===")
    print(f"Input t = {t}")
    print(f"t hex = 0x{t:064x}")
    print(f"p = {p}")
    print(f"p hex = 0x{p:032x}")
    print(f"inv = {inv}")
    print(f"inv hex = 0x{inv:016x}")
    
    # Convert to limbs for easier debugging
    t_limbs = []
    temp = t
    for i in range(4):
        t_limbs.append(temp & ((1 << 64) - 1))
        temp >>= 64
    
    p_limbs = []
    temp = p
    for i in range(2):
        p_limbs.append(temp & ((1 << 64) - 1))
        temp >>= 64
    
    print(f"t_limbs = {[hex(x) for x in t_limbs]}")
    print(f"p_limbs = {[hex(x) for x in p_limbs]}")
    
    # REDC algorithm
    for i in range(2):
        print(f"\n--- Step {i} ---")
        
        # Calculate m = t[i] * inv mod 2^64
        m = (t_limbs[i] * inv) & ((1 << 64) - 1)
        print(f"m = t[{i}] * inv mod 2^64 = 0x{t_limbs[i]:016x} * 0x{inv:016x} = 0x{m:016x}")
        
        # Add m * p to t starting at position i
        carry = 0
        for j in range(2):
            # Calculate m * p[j]
            prod = m * p_limbs[j]
            print(f"  m * p[{j}] = 0x{m:016x} * 0x{p_limbs[j]:016x} = 0x{prod:032x}")
            
            # Add to t[i+j] with carry
            old_val = t_limbs[i + j]
            new_val = old_val + prod + carry
            t_limbs[i + j] = new_val & ((1 << 64) - 1)
            carry = new_val >> 64
            
            print(f"  t[{i+j}]: 0x{old_val:016x} + 0x{prod:032x} + carry(0x{carry:016x}) = 0x{new_val:032x}")
            print(f"    -> t[{i+j}] = 0x{t_limbs[i+j]:016x}, carry = 0x{carry:016x}")
        
        # Add carry to t[i+2]
        if i + 2 < len(t_limbs):
            old_val = t_limbs[i + 2]
            t_limbs[i + 2] = (old_val + carry) & ((1 << 64) - 1)
            print(f"  t[{i+2}]: 0x{old_val:016x} + carry(0x{carry:016x}) = 0x{t_limbs[i+2]:016x}")
        
        print(f"After step {i}: t_limbs = {[hex(x) for x in t_limbs]}")
    
    # Extract result
    result = t_limbs[2] | (t_limbs[3] << 64)
    print(f"\nResult before final reduction: {result}")
    print(f"Result hex: 0x{result:032x}")
    
    # Final reduction
    if result >= p:
        result -= p
        print(f"Final reduction applied: {result}")
    
    return result

# Test with (-1) * (-1)
p = (1 << 128) - (1 << 108) + 1
r = pow(2, 128, p)
inv = pow((-p) % (1 << 64), -1, 1 << 64)

print("Testing (-1) * (-1)")
minus_one_regular = p - 1
minus_one_montgomery = (minus_one_regular * r) % p
t = minus_one_montgomery * minus_one_montgomery

print(f"minus_one_regular = {minus_one_regular}")
print(f"minus_one_montgomery = {minus_one_montgomery}")
print(f"t = minus_one_montgomery^2 = {t}")

result = debug_redc(t, p, inv)

# Convert back to regular form
result_regular = (result * pow(r, -1, p)) % p
print(f"\nFinal result in regular form: {result_regular}")
print(f"Expected: 1")
print(f"Correct: {result_regular == 1}")

# Let's also do a sanity check with regular arithmetic
regular_result = (minus_one_regular * minus_one_regular) % p
print(f"\nSanity check - regular arithmetic: ({minus_one_regular} * {minus_one_regular}) mod {p} = {regular_result}")
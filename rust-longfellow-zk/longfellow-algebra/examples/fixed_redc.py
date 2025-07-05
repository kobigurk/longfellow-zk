#!/usr/bin/env python3

# Fixed Montgomery REDC algorithm

def fixed_redc(t, p, inv):
    """
    Fixed Montgomery REDC algorithm
    """
    print(f"=== FIXED REDC ===")
    print(f"Input t = {t}")
    print(f"t hex = 0x{t:064x}")
    
    # Convert to limbs
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
    
    # REDC algorithm - fixed version
    for i in range(2):
        print(f"\n--- Step {i} ---")
        
        # Calculate m = t[i] * inv mod 2^64
        m = (t_limbs[i] * inv) & ((1 << 64) - 1)
        print(f"m = 0x{m:016x}")
        
        # Add m * p to t starting at position i
        carry = 0
        for j in range(2):
            # Calculate m * p[j] as full 128-bit result
            prod = m * p_limbs[j]
            prod_lo = prod & ((1 << 64) - 1)
            prod_hi = prod >> 64
            
            print(f"  m * p[{j}] = 0x{prod:032x} = (hi=0x{prod_hi:016x}, lo=0x{prod_lo:016x})")
            
            # Add to t[i+j] with carry
            old_val = t_limbs[i + j]
            sum_val = old_val + prod_lo + carry
            t_limbs[i + j] = sum_val & ((1 << 64) - 1)
            carry = (sum_val >> 64) + prod_hi
            
            print(f"  t[{i+j}]: 0x{old_val:016x} + 0x{prod_lo:016x} + carry(0x{carry - prod_hi:016x}) = 0x{sum_val:032x}")
            print(f"    -> t[{i+j}] = 0x{t_limbs[i+j]:016x}, new_carry = 0x{carry:016x}")
        
        # Add final carry to t[i+2]
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

result = fixed_redc(t, p, inv)

# Convert back to regular form
result_regular = (result * pow(r, -1, p)) % p
print(f"\nFinal result in regular form: {result_regular}")
print(f"Expected: 1")
print(f"Correct: {result_regular == 1}")

# Test with 2 * 3
print("\n" + "="*50)
print("Testing 2 * 3")
two_montgomery = (2 * r) % p
three_montgomery = (3 * r) % p
t_2_3 = two_montgomery * three_montgomery

result_2_3 = fixed_redc(t_2_3, p, inv)
result_2_3_regular = (result_2_3 * pow(r, -1, p)) % p
print(f"\nFinal result in regular form: {result_2_3_regular}")
print(f"Expected: 6")
print(f"Correct: {result_2_3_regular == 6}")
#!/usr/bin/env python3

# Correct Montgomery multiplication reference implementation

def montgomery_mult(a, b, p, r, inv):
    """
    Correct Montgomery multiplication
    a, b: operands in Montgomery form
    p: modulus 
    r: Montgomery constant (2^128 mod p)
    inv: modular inverse of (-p) mod 2^64
    """
    # Step 1: Regular multiplication
    t = a * b
    
    # Step 2: Montgomery reduction (REDC algorithm)
    # Split t into 64-bit limbs
    limbs = []
    temp = t
    for i in range(4):  # Need 4 limbs for 256-bit intermediate
        limbs.append(temp & ((1 << 64) - 1))
        temp >>= 64
    
    # Pad with zeros if needed
    while len(limbs) < 4:
        limbs.append(0)
    
    print(f"Input t = {t}")
    print(f"t hex = 0x{t:064x}")
    print(f"Initial limbs: {[hex(x) for x in limbs]}")
    
    # Split p into 64-bit limbs
    p_limbs = []
    temp_p = p
    for i in range(2):  # p is 128 bits = 2 limbs
        p_limbs.append(temp_p & ((1 << 64) - 1))
        temp_p >>= 64
    
    print(f"p_limbs: {[hex(x) for x in p_limbs]}")
    
    # REDC algorithm
    for i in range(2):  # N=2 limbs
        # m = t[i] * inv mod 2^64
        m = (limbs[i] * inv) & ((1 << 64) - 1)
        print(f"  i={i}: m = 0x{m:016x}")
        
        # Add m * p to limbs starting at position i
        carry = 0
        for j in range(2):  # N=2 limbs of modulus
            prod = m * p_limbs[j]
            prod_lo = prod & ((1 << 64) - 1)
            prod_hi = prod >> 64
            
            # Add to limbs[i+j] with carry
            sum_val = limbs[i + j] + prod_lo + carry
            limbs[i + j] = sum_val & ((1 << 64) - 1)
            carry = (sum_val >> 64) + prod_hi
        
        # Propagate carry to position i+2
        limbs[i + 2] = (limbs[i + 2] + carry) & ((1 << 64) - 1)
        
        print(f"    After step {i}: limbs = {[hex(x) for x in limbs]}")
    
    # Extract result from upper limbs [2, 3]
    result = limbs[2] | (limbs[3] << 64)
    
    print(f"Before final reduction: result = {result}, 0x{result:032x}")
    
    # Final conditional subtraction
    if result >= p:
        result -= p
    
    print(f"Final result: {result}, 0x{result:032x}")
    return result

# Test with p = 2^128 - 2^108 + 1
p = (1 << 128) - (1 << 108) + 1
r = pow(2, 128, p)  # 2^128 mod p
inv = pow((-p) % (1 << 64), -1, 1 << 64)  # (-p)^(-1) mod 2^64

print(f"p = {p}")
print(f"p hex = 0x{p:032x}")
print(f"r = {r}")
print(f"r hex = 0x{r:032x}")
print(f"inv = {inv}")
print(f"inv hex = 0x{inv:016x}")

# Test case: (-1) * (-1)
print("\n=== Test case: (-1) * (-1) ===")
minus_one_regular = p - 1
minus_one_montgomery = (minus_one_regular * r) % p

print(f"Regular -1 = {minus_one_regular}")
print(f"Montgomery -1 = {minus_one_montgomery}")

result = montgomery_mult(minus_one_montgomery, minus_one_montgomery, p, r, inv)

# Convert back to regular form
result_regular = (result * pow(r, -1, p)) % p
print(f"Result in regular form: {result_regular}")
print(f"Expected: 1")
print(f"Correct: {result_regular == 1}")

# Test case: 2 * 3
print("\n=== Test case: 2 * 3 ===")
two_regular = 2
three_regular = 3
two_montgomery = (two_regular * r) % p
three_montgomery = (three_regular * r) % p

print(f"Regular 2 = {two_regular}")
print(f"Montgomery 2 = {two_montgomery}")
print(f"Regular 3 = {three_regular}")
print(f"Montgomery 3 = {three_montgomery}")

result_2_3 = montgomery_mult(two_montgomery, three_montgomery, p, r, inv)
result_2_3_regular = (result_2_3 * pow(r, -1, p)) % p

print(f"Result in regular form: {result_2_3_regular}")
print(f"Expected: 6")
print(f"Correct: {result_2_3_regular == 6}")
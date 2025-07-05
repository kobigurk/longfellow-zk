#!/usr/bin/env python3

# Reference Montgomery multiplication implementation to compare against

def montgomery_mult(a, b, p, r, inv):
    """
    Reference Montgomery multiplication
    a, b: operands in Montgomery form
    p: modulus 
    r: Montgomery constant (2^k mod p)
    inv: modular inverse of p mod 2^64
    """
    # Step 1: Regular multiplication
    t = a * b
    
    # Step 2: Montgomery reduction (CHES algorithm)
    # Split t into limbs (64-bit each for our case)
    limbs = []
    temp = t
    while temp > 0:
        limbs.append(temp & ((1 << 64) - 1))
        temp >>= 64
    
    # Pad to at least 4 limbs (2*N for N=2)
    while len(limbs) < 4:
        limbs.append(0)
    
    print(f"Input t = {t}")
    print(f"t hex = 0x{t:064x}")
    print(f"Initial limbs: {[hex(x) for x in limbs]}")
    
    # REDC algorithm
    for i in range(2):  # N=2 limbs
        m = (limbs[i] * inv) & ((1 << 64) - 1)  # mod 2^64
        print(f"  i={i}: m = 0x{m:016x}")
        
        # Add m * p to current position
        carry = 0
        p_limbs = [p & ((1 << 64) - 1), (p >> 64) & ((1 << 64) - 1)]
        
        for j in range(2):  # N=2 limbs of modulus
            prod = m * p_limbs[j]
            prod_lo = prod & ((1 << 64) - 1)
            prod_hi = prod >> 64
            
            # Add to limbs[i+j] with carry
            sum_val = limbs[i + j] + prod_lo + carry
            limbs[i + j] = sum_val & ((1 << 64) - 1)
            carry = (sum_val >> 64) + prod_hi
        
        # Propagate carry
        if i + 2 < len(limbs):
            limbs[i + 2] += carry
        else:
            limbs.append(carry)
        
        print(f"    After step {i}: limbs = {[hex(x) for x in limbs]}")
    
    # Extract result from upper limbs
    result = 0
    for i in range(2, min(4, len(limbs))):
        result |= limbs[i] << (64 * (i - 2))
    
    print(f"Before final reduction: result = {result}, 0x{result:032x}")
    
    # Final conditional subtraction
    if result >= p:
        result -= p
    
    print(f"Final result: {result}, 0x{result:032x}")
    return result

# Test with p = 2^128 - 2^108 + 1
p = (1 << 128) - (1 << 108) + 1
r = pow(2, 128, p)  # 2^128 mod p
inv = pow(p, -1, 1 << 64)  # p^(-1) mod 2^64

print(f"p = {p}")
print(f"p hex = 0x{p:032x}")
print(f"r = {r}")
print(f"r hex = 0x{r:032x}")
print(f"inv = {inv}")
print(f"inv hex = 0x{inv:016x}")

# Test case 1: (-1) * (-1)
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

# Test case 2: omega * omega
print("\n=== Test case: omega * omega ===")
omega_regular = 164956748514267535023998284330560247862
omega_montgomery = (omega_regular * r) % p

print(f"Regular omega = {omega_regular}")
print(f"Montgomery omega = {omega_montgomery}")

result_omega = montgomery_mult(omega_montgomery, omega_montgomery, p, r, inv)
result_omega_regular = (result_omega * pow(r, -1, p)) % p

expected_omega2 = (omega_regular * omega_regular) % p
print(f"Result omega^2 regular: {result_omega_regular}")
print(f"Expected omega^2: {expected_omega2}")
print(f"Correct: {result_omega_regular == expected_omega2}")
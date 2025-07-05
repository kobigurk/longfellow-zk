#!/usr/bin/env python3

# Textbook Montgomery REDC algorithm from the original paper

def textbook_redc(T, N, N_prime, radix_bits=64):
    """
    Textbook Montgomery REDC algorithm
    T: input (should be < R*N where R = radix^n)
    N: modulus 
    N_prime: modular inverse of -N mod radix
    radix_bits: bits per "digit" (64 for our case)
    """
    radix = 1 << radix_bits
    n = 2  # N has 2 limbs
    
    print(f"=== TEXTBOOK REDC ===")
    print(f"T = {T}")
    print(f"N = {N}")
    print(f"N_prime = {N_prime}")
    print(f"radix = {radix}")
    print(f"n = {n}")
    
    # Convert T to base-radix representation
    T_digits = []
    temp = T
    for i in range(2 * n):  # Need 2*n digits for the result
        T_digits.append(temp % radix)
        temp //= radix
    
    print(f"T_digits = {[hex(x) for x in T_digits]}")
    
    # Convert N to base-radix representation  
    N_digits = []
    temp = N
    for i in range(n):
        N_digits.append(temp % radix)
        temp //= radix
    
    print(f"N_digits = {[hex(x) for x in N_digits]}")
    
    # Montgomery REDC algorithm
    for i in range(n):
        print(f"\n--- Iteration {i} ---")
        
        # Step 1: m = T[i] * N' mod radix
        m = (T_digits[i] * N_prime) % radix
        print(f"m = T[{i}] * N' mod radix = {T_digits[i]} * {N_prime} mod {radix} = {m}")
        
        # Step 2: T = T + m * N * radix^i
        # This is equivalent to adding m*N to T starting at position i
        carry = 0
        for j in range(n):
            # Calculate m * N[j] + T[i+j] + carry
            prod = m * N_digits[j]
            sum_val = T_digits[i + j] + prod + carry
            T_digits[i + j] = sum_val % radix
            carry = sum_val // radix
            
            print(f"  T[{i+j}] = ({T_digits[i+j] + prod - sum_val % radix} + {prod} + {carry - sum_val // radix}) mod {radix} = {T_digits[i+j]}")
            print(f"  carry = {carry}")
        
        # Propagate carry
        k = i + n
        while carry > 0 and k < len(T_digits):
            sum_val = T_digits[k] + carry
            T_digits[k] = sum_val % radix
            carry = sum_val // radix
            k += 1
            
        print(f"After iteration {i}: T_digits = {[hex(x) for x in T_digits]}")
    
    # Extract result T / radix^n
    result = 0
    for i in range(n, 2 * n):
        if i < len(T_digits):
            result += T_digits[i] * (radix ** (i - n))
    
    print(f"\nResult before final reduction: {result}")
    print(f"Result hex: 0x{result:032x}")
    
    # Final conditional subtraction
    if result >= N:
        result -= N
        print(f"Final reduction applied: {result}")
    
    return result

# Test with our values
p = (1 << 128) - (1 << 108) + 1
r = pow(2, 128, p)
inv = pow((-p) % (1 << 64), -1, 1 << 64)

print("Testing (-1) * (-1)")
minus_one_regular = p - 1
minus_one_montgomery = (minus_one_regular * r) % p
t = minus_one_montgomery * minus_one_montgomery

result = textbook_redc(t, p, inv)

# Convert back to regular form
result_regular = (result * pow(r, -1, p)) % p
print(f"\nFinal result in regular form: {result_regular}")
print(f"Expected: 1")
print(f"Correct: {result_regular == 1}")

# Let's also test with a simple case: 1 * 1
print("\n" + "="*50)
print("Testing 1 * 1 (sanity check)")
one_montgomery = r % p  # 1 in Montgomery form
t_one = one_montgomery * one_montgomery

result_one = textbook_redc(t_one, p, inv)
result_one_regular = (result_one * pow(r, -1, p)) % p
print(f"\nFinal result in regular form: {result_one_regular}")
print(f"Expected: 1")
print(f"Correct: {result_one_regular == 1}")
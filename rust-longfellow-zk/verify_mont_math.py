p = (1 << 128) - (1 << 108) + 1
R = (1 << 128) % p
INV = 0xFFFFFFFFFFFFFFFF

print(f"p = 0x{p:032x}")
print(f"R = 0x{R:032x}")
print(f"R² mod p = 0x{(R * R) % p:032x}")
print(f"INV = 0x{INV:016x}")

# 6 in Montgomery form
six_mont = (6 * R) % p
print(f"\n6R mod p = 0x{six_mont:032x}")

# 6^(-1) in regular form
six_inv = pow(6, -1, p)
print(f"6^(-1) mod p = 0x{six_inv:032x}")

# 6^(-1) in Montgomery form
six_inv_mont = (six_inv * R) % p
print(f"6^(-1) * R mod p = 0x{six_inv_mont:032x}")

# Product in Montgomery form
product_mont = (six_mont * six_inv_mont) % p
print(f"\n(6R) * (6^(-1)R) mod p = 0x{product_mont:032x}")

# This should equal R² mod p
r_squared = (R * R) % p
print(f"R² mod p = 0x{r_squared:032x}")
print(f"Are they equal? {product_mont == r_squared}")

# Now apply Montgomery reduction to get R
# REDC(product_mont) should give R

def montgomery_reduce(T, p, inv, bits=128):
    """Montgomery reduction: compute T * R^(-1) mod p where R = 2^bits"""
    result = T
    for i in range(2):  # 2 limbs
        # Get the i-th limb
        limb_mask = (1 << 64) - 1
        t_i = (result >> (64 * i)) & limb_mask
        
        # m = t_i * inv mod 2^64
        m = (t_i * inv) & limb_mask
        
        # result = (result + m * p * 2^(64*i)) // 2^64
        result = result + m * p * (1 << (64 * i))
    
    # Divide by R = 2^128
    result = result >> 128
    
    # Conditional subtraction
    if result >= p:
        result -= p
    
    return result

# Test Montgomery reduction
reduced = montgomery_reduce(product_mont, p, INV)
print(f"\nMontgomery reduce((6R)(6^(-1)R)) = 0x{reduced:032x}")
print(f"Expected R = 0x{R:032x}")
print(f"Are they equal? {reduced == R}")

# What went wrong?
print(f"\nDifference: {reduced - R}")

# Let's also check the value we actually got
actual_result = 0x00000ffffffffffc3fffffffffffffff
print(f"\nActual result from code = 0x{actual_result:032x}")
print(f"Actual - R = {actual_result - R}")

# Check if the actual result is somehow related to p
print(f"\nActual result / p = {actual_result / p}")
print(f"Actual result mod p = {actual_result % p}")
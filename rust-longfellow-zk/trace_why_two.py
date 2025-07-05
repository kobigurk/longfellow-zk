p = (1 << 128) - (1 << 108) + 1
R = (1 << 128) % p
print(f"p = 0x{p:032x}")
print(f"R = 0x{R:032x}")

# We're getting 2 instead of 1
# In Montgomery form:
# 1 is represented as R
# 2 is represented as 2R mod p

two_mont = (2 * R) % p
print(f"\n2 in Montgomery form = 0x{two_mont:032x}")

# So if we're getting 2, that means we're getting 2R instead of R
# This suggests we're doubling the result somewhere

# Let's trace through what should happen:
# 6 * 6^(-1) in Montgomery form = (6R)(6^(-1)R) = R^2 mod p
r_squared = (R * R) % p
print(f"\nR² mod p = 0x{r_squared:032x}")

# After Montgomery reduction: REDC(R²) = R² * R^(-1) mod p = R
print(f"REDC(R²) should give R = 0x{R:032x}")

# But we're getting 2R mod p
print(f"We're getting 2R mod p = 0x{two_mont:032x}")

# Why would we get 2R?
# One possibility: we're adding R when we shouldn't
print("\nPossible issue: when we have overflow and add R, we might be adding it to a non-zero value")
print("If the lower limbs are 0 and we have overflow=1, result should be R")
print("But if lower limbs already contain 1, then 1 + R = R + 1 ≈ R (since 1 << R)")
#!/usr/bin/env python3

# Convert R^2 to limbs
r2 = 0x000fdffffeffffeffffeffffefffff01

print(f"R^2 = 0x{r2:032x}")

# Extract limbs (little-endian)
limb0 = r2 & 0xffffffffffffffff
limb1 = (r2 >> 64) & 0xffffffffffffffff

print(f"\nLimbs (little-endian):")
print(f"limb[0] = 0x{limb0:016x}")
print(f"limb[1] = 0x{limb1:016x}")

# Verify
reconstructed = limb0 | (limb1 << 64)
print(f"\nReconstructed: 0x{reconstructed:032x}")
print(f"Match: {reconstructed == r2}")
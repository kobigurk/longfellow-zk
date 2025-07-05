#!/usr/bin/env python3

# Verify R^2 calculation
p = (1 << 128) - (1 << 108) + 1
R = (1 << 128) % p

print(f"p = 0x{p:032x}")
print(f"R = 0x{R:032x}")
print(f"R = {R}")

# Calculate R^2 mod p
R2 = (R * R) % p
print(f"\nR^2 mod p = 0x{R2:032x}")
print(f"R^2 = {R2}")

# Convert to limbs
r2_low = R2 & 0xffffffffffffffff
r2_high = (R2 >> 64) & 0xffffffffffffffff

print(f"\nR^2 limbs:")
print(f"  limbs[0] = 0x{r2_low:016x}")
print(f"  limbs[1] = 0x{r2_high:016x}")

# Check our current value
our_r2_low = 0xFFFEFFFFEFFFFF01
our_r2_high = 0x000FDFFFFEFFFFEF

print(f"\nOur R^2 limbs:")
print(f"  limbs[0] = 0x{our_r2_low:016x}")
print(f"  limbs[1] = 0x{our_r2_high:016x}")

if r2_low == our_r2_low and r2_high == our_r2_high:
    print("\n✅ Our R^2 is correct")
else:
    print("\n❌ Our R^2 is wrong!")

# Let's also verify what R * R gives us before reduction
r_low = 0xffffffffffffffff
r_high = 0x00000fffffffffff

# Multiply as 128-bit value
r_value = r_low | (r_high << 64)
r_squared = r_value * r_value

print(f"\n\nR * R (before mod) = 0x{r_squared:064x}")

# Split into limbs
limbs = []
temp = r_squared
for i in range(4):
    limbs.append(temp & 0xffffffffffffffff)
    temp >>= 64

print("\nR * R limbs (before reduction):")
for i, limb in enumerate(limbs):
    print(f"  limbs[{i}] = 0x{limb:016x}")
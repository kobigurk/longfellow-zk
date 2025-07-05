print("Checking carry overflow in iteration 1:")
print()

# Values from the trace
t3_before = 0x00002ffffdffffff
carry = 0xffffd00002000001  # This is the carry value

print(f"t[3] before = 0x{t3_before:016x}")
print(f"carry = 0x{carry:016x}")

# What should happen
sum_full = t3_before + carry
print(f"\nt[3] + carry = 0x{sum_full:032x}")

# What happens with 64-bit arithmetic
sum_64bit = (t3_before + carry) & 0xffffffffffffffff
carry_out = 1 if sum_full > 0xffffffffffffffff else 0

print(f"\nWith 64-bit arithmetic:")
print(f"sum = 0x{sum_64bit:016x}")
print(f"carry_out = {carry_out}")

# The issue: we have a carry_out=1 but no t[4] to put it in!
print(f"\nThe carry_out=1 needs to go somewhere!")
print(f"But extended.len() = 4, so there's no t[4]")

# Let's see what the actual values should be
print("\n--- What should happen ---")
# After iteration 0: t = [0, 0x0000200000000000, 0xffffffffffffffff, 0x00002ffffdffffff]
# Iteration 1: k = 0xffffe00000000000
# Add k*p to t[1..]:
#   t[1] += k*p[0] = 0x0000200000000000 + 0xffffe00000000000 = 0 (carry 1)
#   t[2] += k*p[1] + carry = 0xffffffffffffffff + 0 + 1 = 0 (carry 1)
#   t[3] += carry = 0x00002ffffdffffff + 0xffffd00002000000 = ???

# The multiplication k * p[1]
k = 0xffffe00000000000
p1 = 0xfffff00000000000

lo = (k * p1) & 0xffffffffffffffff
hi = (k * p1) >> 64

print(f"k * p[1]:")
print(f"  k = 0x{k:016x}")
print(f"  p[1] = 0x{p1:016x}")

# Let's compute this properly
import math
prod = k * p1
print(f"  product = 0x{prod:032x}")
print(f"  lo = 0x{lo:016x}")
print(f"  hi = 0x{hi:016x}")

# So the carry going into t[3] is hi + 1 = 0xffffd00002000001
print(f"\nCarry into t[3] = 0x{hi + 1:016x}")
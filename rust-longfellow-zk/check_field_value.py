p = (1 << 128) - (1 << 108) + 1
R = (1 << 128) % p

result_low = 0x3fffffffffffffff
result_high = 0x00000ffffffffffc
result = result_low | (result_high << 64)

print(f"p = 0x{p:032x}")
print(f"R = 0x{R:032x}")
print(f"result = 0x{result:032x}")

print(f"\np = {p}")
print(f"R = {R}")
print(f"result = {result}")

# Check modular arithmetic
print(f"\nresult mod p = {result % p}")
print(f"Is result ≡ 0 (mod p)? {result % p == 0}")
print(f"Is result ≡ R (mod p)? {result % p == R}")

# What's the difference?
print(f"\nresult - R = {result - R}")
print(f"(result - R) / p = {(result - R) / p}")

# Is result = p?
print(f"\nIs result == p? {result == p}")
print(f"Is result == p - 1? {result == p - 1}")

# Let's check if result + something = p
diff = p - result
print(f"\np - result = {diff}")
print(f"p - result = 0x{diff:032x}")
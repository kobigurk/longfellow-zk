result_low = 0x3fffffffffffffff
result_high = 0x00000ffffffffffc
result = result_low | (result_high << 64)
print(f'Result = 0x{result:032x}')
print(f'Result = {result}')

p = (1 << 128) - (1 << 108) + 1
print(f'\np = 0x{p:032x}')

# What is this result mod p?
print(f'\nresult mod p = {result % p}')
print(f'result mod p = 0x{result % p:032x}')

# Is it p-1?
print(f'\np - 1 = 0x{(p-1):032x}')
print(f'result == p-1? {result == p-1}')

# Is it p?
print(f'\nresult == p? {result == p}')
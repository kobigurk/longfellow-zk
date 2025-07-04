pub fn bit_reverse(n: usize, bits: usize) -> usize {
    let mut result = 0;
    let mut n = n;
    
    for _ in 0..bits {
        result = (result << 1) | (n & 1);
        n >>= 1;
    }
    
    result
}

pub fn bit_reverse_inplace<T>(data: &mut [T]) {
    let n = data.len();
    if n <= 1 {
        return;
    }
    
    let bits = (n.trailing_zeros()) as usize;
    
    for i in 0..n {
        let j = bit_reverse(i, bits);
        if i < j {
            data.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_reverse() {
        assert_eq!(bit_reverse(0b000, 3), 0b000);
        assert_eq!(bit_reverse(0b001, 3), 0b100);
        assert_eq!(bit_reverse(0b010, 3), 0b010);
        assert_eq!(bit_reverse(0b011, 3), 0b110);
        assert_eq!(bit_reverse(0b100, 3), 0b001);
        assert_eq!(bit_reverse(0b101, 3), 0b101);
        assert_eq!(bit_reverse(0b110, 3), 0b011);
        assert_eq!(bit_reverse(0b111, 3), 0b111);
    }

    #[test]
    fn test_bit_reverse_inplace() {
        let mut data = vec![0, 1, 2, 3, 4, 5, 6, 7];
        bit_reverse_inplace(&mut data);
        assert_eq!(data, vec![0, 4, 2, 6, 1, 5, 3, 7]);
    }
}
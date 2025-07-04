/// Start with a naive version, using 64 bit words an ceil(log(p))=32
const NR_WORDS: usize = 32;
const BASE: u128 = 1 << 64; // 2^64
struct Fe {
    words: [u64; NR_WORDS],
}

// TODO: Add a real prime here
const PRIME: Fe = Fe {
    words: [0; NR_WORDS],
};

/// Adds first and second and returns the result and a carry bit
fn add(first: [u64; NR_WORDS], second: [u64; NR_WORDS]) -> ([u64; NR_WORDS], bool) {
    let mut result = [0_u64; NR_WORDS];
    let mut carry = false;
    for i in 0..NR_WORDS {
        let sum = (first[i] as u128) + (second[i] as u128) + (carry as u128);
        result[i] = sum as u64; // computes sum % BASE
        carry = sum >= BASE;
    }
    (result, carry)
}

fn sub(first: [u64; NR_WORDS], second: [u64; NR_WORDS]) -> ([u64; NR_WORDS], bool) {
    let mut result = [0; NR_WORDS];
    let mut borrow = false;
    for i in 0..NR_WORDS {
        let eps= (first[i] as u128) < (second[i] as u128) + (borrow as u128);
        // Need eps*BASE to avoid overflow below 0. Rust panics in that case
        let diff = (eps as u128)*BASE + (first[i] as u128) - (second[i] as u128) - (borrow as u128);
        result[i] = diff as u64;
        borrow = eps;
    }
    (result, borrow)
}

fn add_mod_p(first: Fe, second: Fe) -> Fe {
    let (result, carry) = add(first.words, second.words);
    if carry {
        return Fe {
            words: sub(result, PRIME.words).0,
        };
    }
    Fe { words: result }
}

fn sub_mod_p(first: Fe, second: Fe) -> Fe {
    let (result, borrow) = sub(first.words, second.words);
    if borrow {
        return Fe {
            words: add(result, PRIME.words).0,
        };
    }
    Fe { words: result }
}

fn mul_operand_scanning(
    first: [u64; NR_WORDS],
    second: [u64; NR_WORDS],
) -> [u64; 2 * NR_WORDS - 1] {
    let mut result = [0; 2 * NR_WORDS - 1];
    for i in 0..NR_WORDS {
        let mut carry = 0;
        for j in 0..NR_WORDS {
            let prod = (first[i] as u128) * (second[j] as u128)
                + (result[i + j] as u128)
                + (carry as u128);
            result[i + j] = prod as u64; // get lower part
            carry = prod / BASE; // get higher part
        }
        result[i + NR_WORDS -1] = carry as u64;
    }
    result
}

fn mul_product_scanning(
    first: [u64; NR_WORDS],
    second: [u64; NR_WORDS],
) -> [u64; 2 * NR_WORDS - 1] {
    let (mut r0, mut r1, mut r2) = (0_u128, 0_u128, 0_u128);
    let mut result = [0; 2 * NR_WORDS - 1];
    // for each word of the result
    for k in 0..(2 * NR_WORDS - 1) {
        // all (i,j) with i+j=k
        for i in 0..k {
            let mut eps = false;
            let prod = (first[k - i] as u128) * (second[i] as u128);
            r0 = r0 + (prod % BASE); // r0 collects A[i]*B[j] lower parts
            eps = r0 >= BASE;
            r0 = r0 % BASE; // get rid of carry
            r1 = r1 + (prod / BASE) + (eps as u128); // r1 collects the higher part of A[i]*B[j].
            eps = r1 >= BASE;
            r1 = r1 % BASE; // get rid of carry
            r2 += (eps as u128); // r2 collects carries in r1.
        }
        // proceed to next word
        result[k] = r0 as u64;
        r0 = r1;
        r1 = r2;
        r2 = 0;
    }
    // highest order word is not in the loop
    result[NR_WORDS - 2] = r1 as u64;
    result
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_simple_add() {
        let mut first = [0; NR_WORDS];
        let mut second = [0; NR_WORDS];
        first[0] = 7;
        second[0] = 5;
        let third = add(first, second);
        assert_eq!(third.1, false); // no carry
        assert_eq!(third.0[0], 12);
        for i in 1..NR_WORDS {
            assert_eq!(third.0[i], 0);
        }
    }

    #[test]
    fn test_add_internal_carry() {
        let mut first = [0; NR_WORDS];
        let mut second = [0; NR_WORDS];
        first[7] = ((1u128 << 64) - 1) as u64;
        second[7] = 5;
        let third = add(first, second);
        assert_eq!(third.1, false); // no carry
        assert_eq!(third.0[7], 4);
        assert_eq!(third.0[8], 1);
        for i in 1..7 {
            assert_eq!(third.0[i], 0);
        }
        for i in 9..NR_WORDS {
            assert_eq!(third.0[i], 0);
        }
    }

    #[test]
    fn test_add_carry() {
        let first = [!0; NR_WORDS];
        let mut second = [0; NR_WORDS];
        second[0] = 1; // increments exactly by one
        let third = add(first, second);
        assert_eq!(third.1, true); // must have carry
        for i in 1..NR_WORDS {
            assert_eq!(third.0[i], 0);
        }
    }

    #[test]
    fn test_simple_sub() {
        let mut first = [0; NR_WORDS];
        let mut second = [0; NR_WORDS];
        first[0] = 7;
        second[0] = 5;
        let third = sub(first, second);
        assert_eq!(third.1, false); // no borrow 
        assert_eq!(third.0[0], 2);
        for i in 1..NR_WORDS {
            assert_eq!(third.0[i], 0);
        }
    }

    #[test]
    fn test_add_internal_borrow() {
        let mut first = [0; NR_WORDS];
        let mut second = [0; NR_WORDS];
        first[7] = 3;
        first[8] = 4;
        second[7] = 5;
        let third = sub(first, second);
        assert_eq!(third.1, false); // no borrow
        assert_eq!(third.0[7], !0-1);
        assert_eq!(third.0[8], 3);
        for i in 1..7 {
            assert_eq!(third.0[i], 0);
        }
        for i in 9..NR_WORDS {
            assert_eq!(third.0[i], 0);
        }
    }

    #[test]
    fn test_add_borrow() {
        let first = [0; NR_WORDS];
        let mut second = [0; NR_WORDS];
        second[0] = 1; // decrement exactly by one
        let third = sub(first, second);
        assert_eq!(third.1, true); // must have carry
        for i in 1..NR_WORDS {
            assert_eq!(third.0[i], !0);
        }
    }

    #[test]
    fn test_mul_simple() {
        let mut first = [0; NR_WORDS];
        let mut second = [0; NR_WORDS];
        first[0] = 5;
        second[0] = 7;
        let third = mul_operand_scanning(first, second);
        assert_eq!(third[0], 35);
        for i in 1..(2*NR_WORDS -1) {
            assert_eq!(third[i], 0);
        }
    }

    fn test_mul_neutral() {
        let mut first = [0; NR_WORDS];
        // ChatGPT gave me this number
        let second: [u64; 32] = [
        0x0123_4567_89AB_CDEF,
        0xFEDC_BA98_7654_3210,
        0x1111_2222_3333_4444,
        0x5555_6666_7777_8888,
        0x9999_AAAA_BBBB_CCCC,
        0xDDDD_EEEE_FFFF_0000,
        0x1357_9BDF_2468_ACE0,
        0x0F0F_F0F0_0F0F_F0F0,
        0xABCDEF01_23456789,
        0xCAFEBABE_DEADBEEF,
        0xDEAD_BEEF_CAFE_BABE,
        0x0001_0002_0003_0004,
        0x7FFF_FFFF_FFFF_FFFF,
        0x8000_0000_0000_0000,
        0x0000_0000_0000_0001,
        0xFFFF_FFFF_FFFF_FFFF,
        0x1234_5678_9ABC_DEF0,
        0x0BAD_C0DE_0BAD_C0DE,
        0xFEED_FACE_C0FF_EE00,
        0xDEAD_C0DE_FEED_BEEF,
        0xCAFED00D_BADC0DE1,
        0x0000_1111_2222_3333,
        0x4444_5555_6666_7777,
        0x8888_9999_AAAA_BBBB,
        0xCCCC_DDDD_EEEE_FFFF,
        0x1357_2468_ACE0_BDF1,
        0x1111_1111_1111_1111,
        0x2222_2222_2222_2222,
        0x3333_3333_3333_3333,
        0x4444_4444_4444_4444,
        0x5555_5555_5555_5555,
        0x6666_6666_6666_6666,
        ];
        let third = mul_operand_scanning(first, second);
        for i in 1..(2*NR_WORDS -1) {
            assert_eq!(third[i], 0);
        }
    }




}

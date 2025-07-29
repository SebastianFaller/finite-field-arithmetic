/// Start with a naive version, using 8 bit words an floor(log(p))+1=32
const NR_WORDS: usize = 32;
const BASE: u16 = 1 << 8; // 256
const WORD_LEN: usize = 8;
struct Fe {
    words: [u8; NR_WORDS],
}

// 2*255 - 19. Generated with precompute.py
const PRIME: Fe = Fe {
    words: [
        0xf6, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0x3f,
    ],
};

// computed with precompute.py
const BARRET_MU: Fe = Fe {
    words: [
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ],
};

/// Adds first and second and returns the result and a carry bit
fn add(first: [u8; NR_WORDS], second: [u8; NR_WORDS]) -> ([u8; NR_WORDS], bool) {
    let mut result = [0_u8; NR_WORDS];
    let mut carry = false;
    for i in 0..NR_WORDS {
        let sum = (first[i] as u16) + (second[i] as u16) + (carry as u16);
        result[i] = sum as u8; // computes sum % BASE
        carry = sum >= BASE;
    }
    (result, carry)
}

fn sub(first: [u8; NR_WORDS], second: [u8; NR_WORDS]) -> ([u8; NR_WORDS], bool) {
    let mut result = [0; NR_WORDS];
    let mut borrow = false;
    for i in 0..NR_WORDS {
        let eps = (first[i] as u16) < (second[i] as u16) + (borrow as u16);
        // Need eps*BASE to avoid overflow below 0. Rust panics in that case
        let diff = (eps as u16) * BASE + (first[i] as u16) - (second[i] as u16) - (borrow as u16);
        result[i] = diff as u8;
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

fn mul_operand_scanning(first: [u8; NR_WORDS], second: [u8; NR_WORDS]) -> [u8; 2 * NR_WORDS - 1] {
    let mut result = [0; 2 * NR_WORDS - 1];
    for i in 0..NR_WORDS {
        let mut carry = 0;
        for j in 0..NR_WORDS {
            let prod =
                (first[i] as u16) * (second[j] as u16) + (result[i + j] as u16) + (carry as u16);
            result[i + j] = prod as u8; // get lower part
            carry = prod >> WORD_LEN; // get higher part
        }
        result[i + NR_WORDS - 1] = carry as u8;
    }
    result
}

fn mul_product_scanning(first: [u8; NR_WORDS], second: [u8; NR_WORDS]) -> [u8; 2 * NR_WORDS - 1] {
    let (mut r0, mut r1, mut r2) = (0_u16, 0_u16, 0_u16);
    let mut result = [0; 2 * NR_WORDS - 1];
    // for each word of the result
    for k in 0..(2 * NR_WORDS - 1) {
        // all (i,j) with i+j=k
        for i in 0..k {
            let mut eps = false;
            let prod = (first[k - i] as u16) * (second[i] as u16);
            r0 = r0 + (prod % BASE); // r0 collects A[i]*B[j] lower parts
            eps = r0 >= BASE;
            r0 = r0 % BASE; // get rid of carry
            r1 = r1 + (prod / BASE) + (eps as u16); // r1 collects the higher part of A[i]*B[j].
            eps = r1 >= BASE;
            r1 = r1 % BASE; // get rid of carry
            r2 += (eps as u16); // r2 collects carries in r1.
        }
        // proceed to next word
        result[k] = r0 as u8;
        r0 = r1;
        r1 = r2;
        r2 = 0;
    }
    // highest order word is not in the loop
    result[NR_WORDS - 2] = r1 as u8;
    result
}

fn square(x: [u8; NR_WORDS]) -> [u8; 2 * NR_WORDS] {
    let mut result = [0_u8; 2 * NR_WORDS];
    for i in 0..NR_WORDS {
        let s = (x[i] as u16) * (x[i] as u16) + (result[2 * i] as u16);
        result[2 * i] = s as u8;
        let mut carry = (s >> WORD_LEN);
        for j in (i + 1)..(NR_WORDS - 1) {
            let r = (result[i + j] as u32) + 2 * (x[i] as u32) * (x[j] as u32) + (carry as u32);
            result[i + j] = r as u8;
            carry = (r >> WORD_LEN) as u16;
        }
        result[i + NR_WORDS] = carry as u8;
    }
    result
}

// Shifts the array a of length a_length to the right by k words and
fn word_shift(a: &[u8], a_length: usize, k: usize) -> [u8; NR_WORDS] {
    assert!(k <= a_length);
    let mut result = [0; NR_WORDS];
    for i in 0..(a_length - k) {
        result[i] = a[i + k];
    }
    result
}

// Returns the remainder of a when dividing a by b*k.
// TODO make inplace
fn mod_shift(a: &[u8], a_length: usize, k: usize) -> [u8; NR_WORDS] {
    assert!(k <= a_length);
    let mut result = [0; NR_WORDS];
    for i in 0..k {
        result[i] = a[i];
    }
    result
}

fn barret_red(a: Fe) -> Fe {
    let k = NR_WORDS / 2;
    let q1 = word_shift(&a.words, NR_WORDS, k - 1);
    let q2 = mul_operand_scanning(q1, BARRET_MU.words);
    let q3 = word_shift(&q2, 2 * NR_WORDS, k + 1);
    let r1 = mod_shift(&a.words, NR_WORDS, k + 1);
    let qm = mul_operand_scanning(q3, PRIME.words);
    let r2 = mod_shift(&qm, 2 * NR_WORDS, k + 1);
    let (mut r, borrow_r) = sub(r1, r2);
    if borrow_r {
        r[k + 1] += 1; // r = r + b^{k+1}
    }
    let mut borrow_rr = false;
    let mut rr = r;
    (rr, borrow_rr) = sub(r, PRIME.words);
    if borrow_rr {
        // meaning r < PRIME
        Fe { words: r }
    } else {
        r = rr;
        (rr, borrow_rr) = sub(r, PRIME.words);
        if borrow_rr {
            Fe { words: r }
        } else {
            Fe { words: rr } // never more than two subtractions necessary
        }
    }
}

fn main() {
    println!("Hello, world!");
    // let x = [
        // 0x04, 0x82, 0x2e, 0xcb, 0xef, 0x66, 0xf5, 0x7e, 0x9f, 0x86, 0xcc, 0xe2, 0xc3, 0xd7, 0x9d,
    //     0x51, 0xa1, 0x40, 0x0d, 0x79, 0x3f, 0x31, 0xdb, 0xee, 0x83, 0xf3, 0x91, 0xc2, 0xae, 0xfb,
    //     0x15, 0x39,
    // ];
    // println!("square(x) {:?}", square(x))
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
        first[7] = ((1u16 << 8) - 1) as u8;
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
        assert_eq!(third.0[7], !0 - 1);
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
        for i in 1..(2 * NR_WORDS - 1) {
            assert_eq!(third[i], 0);
        }
    }

    #[test]
    fn test_mul_neutral() {
        let mut first = [0; NR_WORDS];
        // ChatGPT gave me this number
        let second: [u8; 32] = [
            0x3A, 0xF1, 0x7C, 0xB8, 0x45, 0x2D, 0xE9, 0x01, 0x9F, 0xAC, 0xD3, 0x0B, 0x66, 0x7A,
            0x20, 0xC4, 0x8E, 0x11, 0x5D, 0xBE, 0x39, 0x72, 0x04, 0xFA, 0x6B, 0x93, 0xCE, 0x28,
            0x10, 0x87, 0xD5, 0x4F,
        ];
        let third = mul_operand_scanning(first, second);
        for i in 1..(2 * NR_WORDS - 1) {
            assert_eq!(third[i], 0);
        }
    }

    #[test]
    fn test_square() {
        let result = [
            0x10, 0x10, 0x78, 0x53, 0x1d, 0xbe, 0x5c, 0xdd, 0x7e, 0x56, 0xa2, 0x96, 0xeb, 0x1c,
            0x9e, 0xf4, 0x14, 0xc4, 0x84, 0xdc, 0x45, 0x42, 0xad, 0x1c, 0x70, 0x66, 0xa1, 0x4b,
            0x5e, 0xa0, 0xdf, 0xf4, 0x20, 0x98, 0x40, 0x36, 0x50, 0xb4, 0xc9, 0x61, 0x51, 0xe9,
            0xa8, 0x2c, 0x4d, 0x48, 0x9e, 0x6b, 0x01, 0xb3, 0xf7, 0x95, 0xe8, 0x11, 0xd0, 0xf9,
            0x39, 0x13, 0xc1, 0x14, 0xf7, 0xcb, 0xba, 0x0c,
        ];
        let x = [
            0x04, 0x82, 0x2e, 0xcb, 0xef, 0x66, 0xf5, 0x7e, 0x9f, 0x86, 0xcc, 0xe2, 0xc3, 0xd7,
            0x9d, 0x51, 0xa1, 0x40, 0x0d, 0x79, 0x3f, 0x31, 0xdb, 0xee, 0x83, 0xf3, 0x91, 0xc2,
            0xae, 0xfb, 0x15, 0x39,
        ];
        assert_eq!(square(x), result)
    }
}

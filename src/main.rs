/// Start with a naive version, using 64 bit words an ceil(log(p))=32
const NR_WORDS : usize = 32;
struct fe{
    words : [u64; NR_WORDS]
}

// TODO: Add a real prime here
const PRIME : fe = fe {
    words : [0;NR_WORDS]
};

/// Adds first and second and returns the result and a carry bit
fn add(first : fe, second : fe) -> (fe,bool) {
    let mut result = [0;NR_WORDS];
    let mut carry= false;
    for i in 0..NR_WORDS {
       (result[i],carry) = first.words[i].carrying_add(second.words[i],carry) ; 
    }
    (fe{words: result}, carry)
}

fn sub(first : fe, second : fe) -> (fe,bool) {
    let mut result = [0;NR_WORDS];
    let mut borrow = false;
    for i in 0..NR_WORDS {
        (result[i], borrow) = first.words[i].carrying_sub(second.words[i]);
    }
    (fe{words: result}, borrow)
}

fn add_mod_p(first : fe, second : fe) -> fe {
    let (result,carry) = add(first, second);
    if carry {
        return sub(result, PRIME).0
    }
    result
}

fn sub_mod_p(first : fe,second : fe) -> fe {
    let (result, borrow) = sub(first, second);
    if borrow {
        return add(result, PRIME).0
    }
    result
}

fn mul_operand_scanning(first : fe, second : fe) -> fe {
    let result = [0;2*NR_WORDS-1];
    for i in 0..NR_WORDS {
        let mut u = 0;
        let mut v = 0;
        for j in 0..NR_WORDS {
            (u,v) = first.words[i].carry_mul_add(second,result[i+j],u);
            result[i+j] = v;
        }
        result[i+NR_WORDS] = u;
    }
}

fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod test{
    use crate::test;

    #[test]
    fn testAdd(){

    }
}

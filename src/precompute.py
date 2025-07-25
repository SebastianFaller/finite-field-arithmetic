import math 

# This file contains static precomputation of constants.

## Print big numbers as rust arrays
def print_a_with_b_words_of_length_c(a,b,c):
    assert (2**c)**b > a
    s = c // 4 # number of leading zeros
    print(s)
    print("[")
    for i in range(b):
        word = ((a // ((2**c)**i)) % 2**c)
        print("    0x{0:0{size}x},".format(word,size=s))
    print("];")

### Some examples  
print_a_with_b_words_of_length_c(32,32,64)
print_a_with_b_words_of_length_c((2**255-20)//2,32,8)
# must output [0xf6, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x3f]; /* (p-1)/2 */

print_a_with_b_words_of_length_c((2**255-19),32,32)
# [0xffffffed,0xffffffff,0xffffffff,0xffffffff,0xffffffff,0xffffffff,0xffffffff,0x7fffffff,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,];



## Barrett constant precomputation
# Compute mu = floor(b^2k/m), where 2k is the number of words, b is (roughly) the word size and m is the modulus
def barrett_mu(word_size, nr_words, modulus):
    return word_size**nr_words // modulus
print(barrett_mu(2**8, 32, 2**255-19))
# print(barrett_mu(2**64, 32, 2**255-19))
print_a_with_b_words_of_length_c(barrett_mu(2**8, 32, 2**255-19),32,8)




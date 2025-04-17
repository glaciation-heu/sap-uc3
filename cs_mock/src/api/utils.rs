use num_bigint::BigInt;
use std::ops::{Mul, Add};
use num_integer::Integer;
use base64::{prelude::BASE64_STANDARD, Engine};

use super::amphora::OutputDeliveryObject;


static LIMB_WIDTH: usize = 8;
static WORD_WIDTH: usize = 2 * LIMB_WIDTH;
pub struct SharesDecoded {
    pub secrets: Vec<BigInt>,
    pub rs: Vec<BigInt>,
    pub us: Vec<BigInt>,
    pub vs: Vec<BigInt>,
    pub ws: Vec<BigInt>,
}

pub fn zipp_shares(shares: &Vec<OutputDeliveryObject>, r_inv: &BigInt, prime: &BigInt) -> SharesDecoded {
    SharesDecoded {
        secrets: zipp_values(
            shares.iter().map(|s| s.secret_shares.clone()).collect(),
            r_inv,
            prime,
        ),
        rs: zipp_values(
            shares.iter().map(|s| s.r_shares.clone()).collect(),
            r_inv,
            prime,
        ),
        us: zipp_values(
            shares.iter().map(|s| s.u_shares.clone()).collect(),
            r_inv,
            prime,
        ),
        vs: zipp_values(
            shares.iter().map(|s| s.v_shares.clone()).collect(),
            r_inv,
            prime,
        ),
        ws: zipp_values(
            shares.iter().map(|s| s.w_shares.clone()).collect(),
            r_inv,
            prime,
        ),
    }
}

fn zipp_values(shares: Vec<String>, r_inv: &BigInt, prime: &BigInt) -> Vec<BigInt> {
    // supplier
    let mut zipped_shares: Vec<BigInt> = Vec::new();

    // Getting list of tuple-list from different providers
    for (i, tuple_list) in shares.iter().enumerate() {
        // decode share from one provider
        let bytes: Vec<u8> = BASE64_STANDARD.decode(&tuple_list).unwrap();
        // Add shares to list
        for (j, share) in bytes.chunks(WORD_WIDTH).enumerate() {
            if i > 0 {
                // second round
                zipped_shares[j] =
                    zipped_shares
                        .get(j)
                        .unwrap()
                        .add(from_gfp(&share.to_vec(), r_inv, prime));
            } else {
                // first round
                zipped_shares.push(from_gfp(&share.to_vec(), r_inv, prime));
            }
        }
    }
    // finisher
    return zipped_shares.iter().map(|v| v.mod_floor(prime)).collect();
}

pub fn to_bigint_arr(value: &Vec<u8>, r_inv: &BigInt, prime: &BigInt) -> Vec<BigInt> {
    value.chunks(WORD_WIDTH).map(|s| from_gfp(&s.to_vec(), r_inv, prime)).collect()
}

pub fn to_gfp(value: &BigInt, r: &BigInt, prime: &BigInt) -> Vec<u8> {
    let mont_bytes = from_int_to_mont(value, r, prime);
    let inverted_mont = invert_limb_endianness(&mont_bytes);
    return swap_limbs(inverted_mont);
}

fn from_gfp(gfp: &Vec<u8>, r_inv: &BigInt, prime: &BigInt) -> BigInt {
    let inverted = invert_limb_endianness(gfp);
    let swapped = swap_limbs(inverted);
    return from_mont_to_int(swapped, r_inv, prime);
}

fn from_int_to_mont(num: &BigInt, r: &BigInt, prime: &BigInt) -> Vec<u8> {
    let mont_bytes = num.mul(r).mod_floor(prime).to_signed_bytes_be();
    let mut bytes: Vec<u8> = vec![0; WORD_WIDTH];

    let src_pos = if mont_bytes.len() > WORD_WIDTH {
        mont_bytes.len() - WORD_WIDTH
    } else {
        0
    };
    let dest_pos = if mont_bytes.len() >= WORD_WIDTH {
        0
    } else {
        WORD_WIDTH - mont_bytes.len()
    };
    let length = mont_bytes.len().min(WORD_WIDTH);

    array_copy(mont_bytes, src_pos, &mut bytes, dest_pos, length);

    return bytes;
}

fn from_mont_to_int(value: Vec<u8>, r_inv: &BigInt, prime: &BigInt) -> BigInt {
    let mut fixed: Vec<u8> = vec![0; WORD_WIDTH + 1];
    array_copy(value, 0, &mut fixed, 1, WORD_WIDTH);

    let x = BigInt::from_signed_bytes_be(&fixed);
    return x.mul(r_inv).mod_floor(prime);
}

fn invert_limb_endianness(input: &Vec<u8>) -> Vec<u8> {
    let mut fixed: Vec<u8> = vec![0; WORD_WIDTH];

    for i in 0..(WORD_WIDTH / LIMB_WIDTH) {
        let mut slice = Vec::from_iter(
            input[(i * LIMB_WIDTH)..(i * LIMB_WIDTH + LIMB_WIDTH)]
                .iter()
                .cloned(),
        );
        slice.reverse();
        array_copy(slice, 0, &mut fixed, i * LIMB_WIDTH, LIMB_WIDTH);
    }

    return fixed;
}

/// equivalent to Java's System.arraycopy(Object src,  int  srcPos, Object dest, int destPos, int length)
fn array_copy(src: Vec<u8>, src_pos: usize, dest: &mut Vec<u8>, dest_pos: usize, length: usize) {
    dest[dest_pos..(dest_pos + length)].copy_from_slice(&src[src_pos..(src_pos + length)])
}

fn swap_limbs(input: Vec<u8>) -> Vec<u8> {
    let mut swapped: Vec<u8> = vec![0; WORD_WIDTH];
    array_copy(input.clone(), 0, &mut swapped, LIMB_WIDTH, LIMB_WIDTH);
    array_copy(input.clone(), LIMB_WIDTH, &mut swapped, 0, LIMB_WIDTH);
    return swapped;
}

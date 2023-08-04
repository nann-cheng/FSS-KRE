pub mod prg;
pub mod idpf;
pub mod dpf;
pub mod dcf;
pub mod beavertuple;


#[macro_use]
extern crate lazy_static;
mod ring;
pub use crate::ring::RingElm;

mod binary;
pub use crate::binary::BinElm;


trait TupleMapToExt<T, U> {
    type Output;
    fn map<F: FnMut(&T) -> U>(&self, f: F) -> Self::Output;
}

type TupleMutIter<'a, T> =
    std::iter::Chain<std::iter::Once<(bool, &'a mut T)>, std::iter::Once<(bool, &'a mut T)>>;
 trait TupleExt<T> {
    fn map_mut<F: Fn(&mut T)>(&mut self, f: F);
    fn get(&self, val: bool) -> &T;
    fn get_mut(&mut self, val: bool) -> &mut T;
    fn iter_mut(&mut self) -> TupleMutIter<T>;
}

impl<T, U> TupleMapToExt<T, U> for (T, T) {
    type Output = (U, U);

    #[inline(always)]
    fn map<F: FnMut(&T) -> U>(&self, mut f: F) -> Self::Output {
        (f(&self.0), f(&self.1))
    }
}

impl<T> TupleExt<T> for (T, T) {
    #[inline(always)]
    fn map_mut<F: Fn(&mut T)>(&mut self, f: F) {
        f(&mut self.0);
        f(&mut self.1);
    }

    #[inline(always)]
    fn get(&self, val: bool) -> &T {
        match val {
            false => &self.0,
            true => &self.1,
        }
    }

    #[inline(always)]
    fn get_mut(&mut self, val: bool) -> &mut T {
        match val {
            false => &mut self.0,
            true => &mut self.1,
        }
    }

    fn iter_mut(&mut self) -> TupleMutIter<T> {
        std::iter::once((false, &mut self.0)).chain(std::iter::once((true, &mut self.1)))
    }
}




// Additive group, such as (Z_n, +)
pub trait Group {
    fn zero() -> Self;
    fn one() -> Self;
    fn negate(&mut self);
    fn add(&mut self, other: &Self);
    fn mul(&mut self, other: &Self);
    fn sub(&mut self, other: &Self);
}

pub trait Share: Group + prg::FromRng + Clone {
    fn random() -> Self {
        let mut out = Self::zero();
        out.randomize();
        out
    }

    fn share(&self) -> (Self, Self) {
        let mut s0 = Self::zero();
        s0.randomize();
        let mut s1 = self.clone();
        s1.sub(&s0);

        (s0, s1)
    }

    fn share_random() -> (Self, Self) {
        (Self::random(), Self::random())
    }
}

pub fn vec_bool_to_string(vec_bool: &Vec<bool>) -> String {
    let mut string = String::new();
    for i in 0..vec_bool.len() {
        if vec_bool[i]{
            string.push('1');
        }else {
            string.push('0');
        }
    }

    string
}

pub fn u32_to_bits_BE(nbits: usize, input: u32) -> String {
    assert!(nbits <= 32);

    let mut string = String::new();
    for i in 0..nbits {
        if (input & (1 << nbits-1-i)) != 0{
            string.push('1');
        }else {
            string.push('0');
        }
    }

    string
}

pub fn u32_to_bits(nbits: usize, input: u32) -> Vec<bool> {
    assert!(nbits <= 32);

    let mut out: Vec<bool> = Vec::new();
    for i in 0..nbits {
        let bit = (input & (1 << (nbits-1-i)) ) != 0;
        out.push(bit);
    }

    out.reverse();
    
    out
}

pub fn bits_to_u32(bits: &[bool]) -> u32 {
    assert!(bits.len() <= 32);
    let mut out = 0u32;

    for i in 0..bits.len() {
        let b32: u32 = bits[i].into();
        out |= b32 << i;
    }
    out
}

pub fn bits_to_u32_BE(bits: &[bool]) -> u32 {
    assert!(bits.len() <= 32);
    let mut out = 0u32;

    for i in 0..bits.len() {
        let b32: u32 = bits[i].into();
        out |= b32 << bits.len()-1-i;
    }
    out
}

pub fn bits_to_u8_BE(bits: &[bool]) -> u8 {
    assert!(bits.len() <= 32);
    let mut out = 0u8;

    for i in 0..bits.len() {
        let b8: u8 = bits[i].into();
        out |= b8 << bits.len()-1-i;
    }
    out
}

pub fn u64_to_bits(input: u64) -> Vec<bool> {

    let mut out: Vec<bool> = Vec::new();
    for i in 0..64 {
        let bit = (input & (1 << i)) != 0;
        out.push(bit);
    }
    out
}

pub fn string_to_bits(s: &str) -> Vec<bool> {
    let mut bits = vec![];
    let byte_vec = s.to_string().into_bytes();
    for byte in &byte_vec {
        let mut b = crate::u32_to_bits(8, (*byte).into());
        bits.append(&mut b);
    }
    bits
}

fn bits_to_u8(bits: &[bool]) -> u8 {
    assert_eq!(bits.len(), 8);
    let mut out = 0u8;
    for i in 0..8 {
        let b8: u8 = bits[i].into();
        out |= b8 << i;
    }

    out
}

pub fn bits_to_string(bits: &[bool]) -> String {
    assert!(bits.len() % 8 == 0);

    let mut out: String = "".to_string();
    let byte_len = bits.len() / 8;
    for b in 0..byte_len {
        let byte = &bits[8 * b..8 * (b + 1)];
        let ubyte = bits_to_u8(&byte);
        out.push_str(std::str::from_utf8(&[ubyte]).unwrap());
    }

    out
}

pub fn bits_Xor(left: &Vec<bool>, right:&Vec<bool>) -> Vec<bool>{

    assert_eq!(left.len(), right.len());

        let mut out = Vec::new();
        for i in 0..left.len(){
            out.push(left[i] ^ right[i]);
        }
        out
}

#[cfg(test)]
mod tests {
    /*use super::*;
    use super::prg::*;

    #[test]
    fn share() {
        let val = FieldElm::random();
        let (s0, s1) = val.share();
        let mut out = FieldElm::zero();
        out.add(&s0);
        out.add(&s1);
        assert_eq!(out, val);
    }


    #[test]
    fn to_bits() {
        let empty: Vec<bool> = vec![];
        assert_eq!(u32_to_bits(0, 7), empty);
        assert_eq!(u32_to_bits(1, 0), vec![false]);
        assert_eq!(u32_to_bits(2, 0), vec![false, false]);
        assert_eq!(u32_to_bits(2, 3), vec![true, true]);
        assert_eq!(u32_to_bits(2, 1), vec![true, false]);
        assert_eq!(u32_to_bits(12, 65535), vec![true; 12]);
    }

    #[test]
    fn to_U32() {
        assert_eq!(bits_to_u32_BE(&vec![true, true,false]), 6);
        assert_eq!(bits_to_u32_BE(&vec![true, false,false]), 4);
    }


    #[test]
    fn to_string() {
        let empty: Vec<bool> = vec![];
        assert_eq!(string_to_bits(""), empty);
        let avec = vec![true, false, false, false, false, true, true, false];
        assert_eq!(string_to_bits("a"), avec);

        let mut aaavec = vec![];
        for _i in 0..3 {
            aaavec.append(&mut avec.clone());
        }
        assert_eq!(string_to_bits("aaa"), aaavec);
    }

    #[test]
    fn to_from_string() {
        let s = "basfsdfwefwf";
        let bitvec = string_to_bits(s);
        let s2 = bits_to_string(&bitvec);

        assert_eq!(bitvec.len(), s.len() * 8);
        assert_eq!(s, s2);
    }*/
}

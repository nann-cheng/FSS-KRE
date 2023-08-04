
// use num::ToPrimitive;
use serde::Deserialize;
use serde::Serialize;
// use std::cmp::Ordering;
// use std::convert::TryInto;
// use std::u32;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BinElm {
    value: bool,
}

impl BinElm {
    pub fn to_vec(&self, len: usize) -> Vec<BinElm> {
        std::iter::repeat(self.clone()).take(len).collect()
    }

    pub fn print(&self){
        println!("I am {}", self.value);
    }

    pub fn to_Bool(&self) -> bool {
        self.value
    }
}

/*******/
impl From<bool> for BinElm {
    #[inline]
    fn from(inp: bool) -> Self {
        BinElm {
            value: inp,
        }
    }
}

// impl Ord for RingElm {
//     #[inline]
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.value.cmp(&other.value)
//     }
// }

// impl PartialOrd for RingElm {
//     #[inline]
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.value.cmp(&other.value))
//     }
// }

impl crate::Group for BinElm {
    #[inline]
    fn zero() -> Self {
        BinElm::from(false)
    }

    #[inline]
    fn one() -> Self {
        BinElm::from(true)
    }

    #[inline]
    fn add(&mut self, other: &Self) {
        self.value ^= other.value;
    }

    #[inline]
    fn sub(&mut self, other: &Self) {
        self.value ^= other.value;
    }

    #[inline]
    fn mul(&mut self, other: &Self) {
        self.value = self.value && other.value;
    }

    //  #[inline]
    // fn negate(&mut self) {
    //     self.value ^= true;
    // }

    #[inline]
    fn negate(&mut self) {
        // self.value
    }
}

impl crate::prg::FromRng for BinElm {
    #[inline]
    fn from_rng(&mut self, rng: &mut impl rand::Rng) {
        let rand_u32:u32  = rng.next_u32();
        self.value = rand_u32%2 == 1;
    }
}


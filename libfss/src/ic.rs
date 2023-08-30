use crate::prg::{PrgSeed,FixedKeyPrgStream};
use super::{bits_to_u32_BE,u32_to_bits_BE,RingElm,BinElm,dcf::*};
use crate::Group;
use std::mem;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ICKey{
    pub key_idx: bool,
    dcf_key: DCFKey<BinElm>,
    p: RingElm,
    q: RingElm,
    word: BinElm,
}

//TODO:Convert BinElm to a general type 
impl ICKey
{
    pub fn gen(alpha_bits: &[bool], p_bound:& RingElm, q_bound:& RingElm) -> (ICKey, ICKey) {
        let gamma_in = RingElm::from( bits_to_u32_BE(&alpha_bits) );

        let mut gamma = gamma_in.clone();
        gamma.sub(&RingElm::one());

        let mut gamma_bits = vec![false;32usize];
        let num_eval = gamma.to_u32();
        match num_eval {
            Some(numeric) => gamma_bits = u32_to_bits_BE(32usize,numeric),
            None      => println!( "u32 Conversion failed!!" ),
        }

        let beta = BinElm::from(true);
        let (key0, key1) = DCFKey::gen(&gamma_bits, &beta);

        let mut q_prime = q_bound.clone();
        q_prime.add(&RingElm::one());
        

        let mut alpha_p = p_bound.clone();
        alpha_p.add(&gamma_in);

        let mut alpha_q = q_bound.clone();
        alpha_q.add(&gamma_in);

        let mut alpha_q_prime = alpha_q.clone();
        alpha_q_prime.add(&RingElm::one());

        let root_seed = PrgSeed::random();
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&root_seed.key);
        let z_0_bits = stream.next_bits(1usize);
        let z_0 = BinElm::from( z_0_bits[0] );
        let mut z_1 = BinElm::zero();
        if alpha_p > alpha_q{
            z_1.add(&BinElm::one());
        }
        if &alpha_p > p_bound{
            z_1.sub(&BinElm::one());
        }
        if alpha_q_prime > q_prime{
            z_1.add(&BinElm::one());
        }
        if alpha_q == RingElm::from(u32::MAX){
            z_1.add(&BinElm::one());
        }
        z_1.sub(&z_0);

        (
            ICKey{
                key_idx: false,
                dcf_key: key0,
                p: p_bound.clone(),
                q: q_bound.clone(),
                word: z_0,
            },
            ICKey{
                key_idx: true,
                dcf_key: key1,
                p: p_bound.clone(),
                q: q_bound.clone(),
                word: z_1,
            }
        )
    }

    pub fn eval(&self, x:& RingElm) -> BinElm {
        let mut q_prime = self.q.clone();
        q_prime.add(&RingElm::one());

        let mut x_p = x.clone();
        x_p.add(&RingElm::from(u32::MAX));
        x_p.sub(&self.p);

        let mut x_q_prime = x.clone();
        x_q_prime.add(&RingElm::from(u32::MAX));
        x_q_prime.sub(&q_prime);

        let mut output_word:BinElm = BinElm::zero();
        output_word.add(&self.word);

        let mut x_p_bits = vec![false;32usize];
        let mut num_eval = x_p.to_u32();
        match num_eval {
            Some(numeric) => x_p_bits = u32_to_bits_BE(32usize,numeric),
            None      => println!( "u32 Conversion failed!!" ),
        }

        let mut x_q_prime_bits = vec![false;32usize];
        num_eval = x_q_prime.to_u32();
        match num_eval {
            Some(numeric) => x_q_prime_bits = u32_to_bits_BE(32usize,numeric),
            None      => println!( "u32 Conversion failed!!" ),
        }
        let duplicate_dcf = self.dcf_key.clone();

        let s_p = self.dcf_key.eval(&x_p_bits);
        let s_q_prime = duplicate_dcf.eval(&x_q_prime_bits);
        output_word.add(&s_q_prime);
        output_word.sub(&s_p);

        if self.key_idx{
            if x>&self.p{
                output_word.add(&BinElm::one());
            }

            if x>&q_prime{
                output_word.sub(&BinElm::one());
            }
        }

        output_word
    }


    pub fn key_size(&self) -> usize {
        let mut keySize = 0usize;
        keySize += mem::size_of_val(&self.key_idx);
        keySize += mem::size_of_val(&self.dcf_key);
        keySize += mem::size_of_val(&self.word);
        keySize
    }

}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ICCKey{
    pub key_idx: bool,
    dcf_key: DCFKey<RingElm>,
    p: RingElm,
    q: RingElm,
    word: RingElm,
}

impl ICCKey
{
    pub fn gen(alpha_bits: &[bool], p_bound:& RingElm, q_bound:& RingElm) -> (ICCKey, ICCKey) {
        let gamma_in = RingElm::from( bits_to_u32_BE(&alpha_bits) );

        let mut gamma = gamma_in.clone();
        gamma.sub(&RingElm::one());

        let mut gamma_bits = vec![false;32usize];
        let num_eval = gamma.to_u32();
        match num_eval {
            Some(numeric) => gamma_bits = u32_to_bits_BE(32usize,numeric),
            None      => println!( "u32 Conversion failed!!" ),
        }

        let beta = RingElm::one();
        let (key0, key1) = DCFKey::gen(&gamma_bits, &beta);

        let mut q_prime = q_bound.clone();
        q_prime.add(&RingElm::one());
        

        let mut alpha_p = p_bound.clone();
        alpha_p.add(&gamma_in);

        let mut alpha_q = q_bound.clone();
        alpha_q.add(&gamma_in);

        let mut alpha_q_prime = alpha_q.clone();
        alpha_q_prime.add(&RingElm::one());

        let root_seed = PrgSeed::random();
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&root_seed.key);
        let z_0_bits = stream.next_bits(1usize);
        let z_0 = RingElm::from(bits_to_u32_BE(&z_0_bits));
        let mut z_1 = RingElm::zero();
        if alpha_p > alpha_q{
            z_1.add(&RingElm::one());
        }
        if &alpha_p > p_bound{
            z_1.sub(&RingElm::one());
        }
        if alpha_q_prime > q_prime{
            z_1.add(&RingElm::one());
        }
        if alpha_q == RingElm::from(u32::MAX){
            z_1.add(&RingElm::one());
        }
        z_1.sub(&z_0);

        (
            ICCKey{
                key_idx: false,
                dcf_key: key0,
                p: p_bound.clone(),
                q: q_bound.clone(),
                word: z_0,
            },
            ICCKey{
                key_idx: true,
                dcf_key: key1,
                p: p_bound.clone(),
                q: q_bound.clone(),
                word: z_1,
            }
        )
    }

    pub fn eval(&self, x:& RingElm) -> RingElm {
        let mut q_prime = self.q.clone();
        q_prime.add(&RingElm::one());

        let mut x_p = x.clone();
        x_p.add(&RingElm::from(u32::MAX));
        x_p.sub(&self.p);

        let mut x_q_prime = x.clone();
        x_q_prime.add(&RingElm::from(u32::MAX));
        x_q_prime.sub(&q_prime);

        let mut output_word:RingElm = RingElm::zero();
        output_word.add(&self.word);

        let mut x_p_bits = vec![false;32usize];
        let mut num_eval = x_p.to_u32();
        match num_eval {
            Some(numeric) => x_p_bits = u32_to_bits_BE(32usize,numeric),
            None      => println!( "u32 Conversion failed!!" ),
        }

        let mut x_q_prime_bits = vec![false;32usize];
        num_eval = x_q_prime.to_u32();
        match num_eval {
            Some(numeric) => x_q_prime_bits = u32_to_bits_BE(32usize,numeric),
            None      => println!( "u32 Conversion failed!!" ),
        }
        let duplicate_dcf = self.dcf_key.clone();

        let s_p = self.dcf_key.eval(&x_p_bits);
        let s_q_prime = duplicate_dcf.eval(&x_q_prime_bits);
        output_word.add(&s_q_prime);
        output_word.sub(&s_p);

        if self.key_idx{
            if x>&self.p{
                output_word.add(&RingElm::one());
            }

            if x>&q_prime{
                output_word.sub(&RingElm::one());
            }
        }

        output_word
    }


    pub fn key_size(&self) -> usize {
        let mut keySize = 0usize;
        keySize += mem::size_of_val(&self.key_idx);
        keySize += mem::size_of_val(&self.dcf_key);
        keySize += mem::size_of_val(&self.word);
        keySize
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ring::*;
    use crate::binary::*;
    use crate::Group;

    #[test]
    fn evalCheck() {
        let seed = PrgSeed::one();
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);
        let alpha_bits = stream.next_bits(32usize);

        let p_bound = RingElm::zero();
        let q_bound = RingElm::from((1<<31)-1);
        //let q_bound = RingElm::from(4);

        // println!("u32 max is: {:?}",RingElm::from(u32::MAX) );
        println!("u32 u32_to_bits_BE test: {:?}", u32_to_bits_BE(32usize,4) );

        let (key0, key1) = ICCKey::gen(&alpha_bits,&p_bound, &q_bound);

        {   
            for i in 1..5{
                let mut alpha_numeric = RingElm::from(bits_to_u32_BE(&alpha_bits));

                alpha_numeric.sub(&RingElm::from(i));
                println!("a[{}] {:?}", i, alpha_numeric);

                println!("pass check {}",i);

                let mut evalResult = RingElm::zero();

                let word0 = key0.eval(&alpha_numeric);
                evalResult.add(&word0);

                let word1 = key1.eval(&alpha_numeric);
                evalResult.add(&word1);

                assert_eq!(evalResult, RingElm::zero());
            }
        }


        // {   
        //     for i in 6..10{
        //         let mut alpha_numeric = RingElm::from(bits_to_u32_BE(&alpha_bits));
        //         alpha_numeric.add(&RingElm::from(i));

        //         let mut evalResult = RingElm::zero();

        //         let word0 = key0.eval(&alpha_numeric);
        //         evalResult.add(&word0);

        //         let word1 = key1.eval(&alpha_numeric);
        //         evalResult.add(&word1);

        //         assert_eq!(evalResult, RingElm::zero());
        //     }
        // }

    }
}
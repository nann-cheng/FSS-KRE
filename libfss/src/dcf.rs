use crate::prg;
use crate::Group;
use serde::Deserialize;
use serde::Serialize;
use std::mem;
use crate::TupleExt;
use crate::TupleMapToExt;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CorWord<T>{
    seed: prg::PrgSeed,
    bits: (bool, bool),
    word: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DCFKey<T> {
    key_idx: bool,
    root_seed: prg::PrgSeed,
    cor_words: Vec<CorWord<T>>,
    word: T,
}

impl<T> DCFKey<T>
where
    T: prg::FromRng + Clone + Group + std::fmt::Debug
{
    fn gen_cor_word(bit: bool, bits: &mut (bool, bool), seeds: &mut (prg::PrgSeed, prg::PrgSeed), v_alpha: &mut T, beta: &T) -> CorWord<T>
    {
        let data = seeds.map(|s| s.long_expand());
        let keep = bit;
        let lose = !keep;

        let mut cw = CorWord {
            seed: data.0.seeds.get(lose) ^ data.1.seeds.get(lose),//done
            bits: (
                data.0.bits.0 ^ data.1.bits.0 ^ bit ^ true,
                data.0.bits.1 ^ data.1.bits.1 ^ bit,
            ),//done
            word: T::zero(),
        };
        // Deal with V_cw from here
        cw.word.sub(&v_alpha);
        cw.word.add(&data.1.veeds.get(lose).convert().word);
        cw.word.sub(&data.0.veeds.get(lose).convert().word);

        if bits.1{
            cw.word.negate();
        }
        if bit{
            if bits.1{
                cw.word.sub(&beta);
            }else{
                cw.word.add(&beta);
            }
        }
        v_alpha.sub(&data.1.veeds.get(keep).convert().word);
        v_alpha.add(&data.0.veeds.get(keep).convert().word);
        if bits.1{
            v_alpha.sub(&cw.word);
        }else{
            v_alpha.add(&cw.word);
        }

        for (b, seed) in seeds.iter_mut() {
            *seed = data.get(b).seeds.get(keep).clone();

            if *bits.get(b) {
                *seed = &*seed ^ &cw.seed;
            }

            let mut newbit = *data.get(b).bits.get(keep);
            if *bits.get(b) {
                newbit ^= cw.bits.get(keep);
            }

            *bits.get_mut(b) = newbit;
        }

        cw
    }

    pub fn gen(alpha_bits: &[bool], value:&T) -> (DCFKey<T>, DCFKey<T>) {
        // let root_seeds = (prg::PrgSeed::zero(), prg::PrgSeed::one());
        let root_seeds = (prg::PrgSeed::random(), prg::PrgSeed::random());
        let root_bits = (false, true);

        let mut seeds = root_seeds.clone();
        let mut bits = root_bits;
        let mut cor_words: Vec<CorWord<T>> = Vec::new();
        let mut lastWord:T = T::zero();
        let mut v_alpha:T = T::zero();

        for (i, &bit) in alpha_bits.iter().enumerate() {
            let cw = Self::gen_cor_word(bit, &mut bits, &mut seeds, &mut v_alpha, &value);
            cor_words.push(cw);
            if i==alpha_bits.len()-1{
                let converted = seeds.map(|s| s.convert());
                lastWord.add(&converted.1.word);
                lastWord.sub(&converted.0.word);
                lastWord.sub(&v_alpha);
                if bits.1 {
                    lastWord.negate();
                }
            }
        }

        (
            DCFKey::<T> {
                key_idx: false,
                root_seed: root_seeds.0,
                cor_words: cor_words.clone(),
                word: lastWord.clone(),
            },
            DCFKey::<T> {
                key_idx: true,
                root_seed: root_seeds.1,
                cor_words: cor_words,
                word:  lastWord,
            },
        )
    }

    pub fn eval(&self, idx: &Vec<bool>) -> T {
        debug_assert!(idx.len() <= self.domain_size());
        debug_assert!(!idx.is_empty());

        let mut seed: prg::PrgSeed = self.root_seed.clone();
      
        let mut t_bit:bool = self.key_idx;

        let mut v_word:T = T::zero();

        for level in 0..idx.len() {
            let bit = idx[level];
            let mut temp_word:T = T::zero();
            

            // Step 1: compute tau
            // 2 bis, 4 seeds
            /*** refresh seed and t value***/
            let tau = seed.long_expand();
            seed = tau.seeds.get(bit).clone();
            let veed = tau.veeds.get(bit).clone();
            if t_bit{
                seed = &seed ^ &self.cor_words[level].seed;
                let new_bit = *tau.bits.get(bit);
                t_bit = new_bit ^ self.cor_words[level].bits.get(bit);

                temp_word.add(&self.cor_words[level].word);
            }else{ //when t_bit is false, update seed and t_bit as orginal expanded tau value
                t_bit = *tau.bits.get(bit);
            }
            let tmp_converted = veed.convert::<T>();
            temp_word.add(&tmp_converted.word);
            if self.key_idx {
                temp_word.negate();
            }
            v_word.add(&temp_word);

            if level==idx.len()-1{
                let mut word:T = T::zero();

                let converted = seed.convert::<T>();
                word.add(&converted.word);
                if t_bit {
                    word.add(&self.word);
                }
                if self.key_idx {
                    word.negate();
                }
                v_word.add(&word);
            }
        }

        v_word
    }

    pub fn domain_size(&self) -> usize {
        self.cor_words.len()
    }

    pub fn key_size(&self) -> usize {
        let mut keySize = 0usize;
        keySize += mem::size_of_val(&self.key_idx);
        keySize += mem::size_of_val(&self.root_seed);
        keySize += mem::size_of_val(&*self.cor_words);
        keySize += mem::size_of_val(&self.word);
        // println!("cor_words is {}",mem::size_of_val(&*self.cor_words));
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
    fn evalCheck_RingElm() {
        // let mut alpha = vec![true];
        let mut alpha = vec![true,false,false];
        // let alpha = crate::u32_to_bits(3, 5);
        let beta = RingElm::from(117u32);
        let (key0, key1) = DCFKey::gen(&alpha, &beta);

        {   
            let less = vec![false,true,true];
            let mut evalResult = RingElm::zero();

            let word0 = key0.eval(&less);
            evalResult.add(&word0);

            let word1 = key1.eval(&less);
            evalResult.add(&word1);

            assert_eq!(evalResult, beta);
        }

        {
            let greater = vec![true,false,true];

            let mut evalResult = RingElm::zero();

            let word0 = key0.eval(&greater);
            evalResult.add(&word0);

            let word1 = key1.eval(&greater);
            evalResult.add(&word1);

            assert_eq!(evalResult, RingElm::zero());
        }
    }


    #[test]
    fn evalCheck_BinElm() {
        // let mut alpha = vec![true];
        let mut alpha = vec![true,false,false];
        // let alpha = crate::u32_to_bits(3, 5);
        let beta = BinElm::from(true);
        let (key0, key1) = DCFKey::gen(&alpha, &beta);

        {   
            let less = vec![false,true,true];
            let mut evalResult = BinElm::zero();

            let word0 = key0.eval(&less);
            evalResult.add(&word0);

            let word1 = key1.eval(&less);
            evalResult.add(&word1);

            assert_eq!(evalResult, beta);
        }

        {
            let greater = vec![true,false,true];

            let mut evalResult = BinElm::zero();

            let word0 = key0.eval(&greater);
            evalResult.add(&word0);

            let word1 = key1.eval(&greater);
            evalResult.add(&word1);

            assert_eq!(evalResult, BinElm::zero());
        }
    }
}
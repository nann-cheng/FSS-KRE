use crate::prg;
use crate::Group;
use serde::Deserialize;
use serde::Serialize;
use std::mem;
use crate::TupleExt;
use crate::TupleMapToExt;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CorWord{
    seed: prg::PrgSeed,
    bits: (bool, bool),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DPFKey<T> {
    key_idx: bool,
    root_seed: prg::PrgSeed,
    cor_words: Vec<CorWord>,
    word: T,
}

fn gen_cor_word(bit: bool, bits: &mut (bool, bool), seeds: &mut (prg::PrgSeed, prg::PrgSeed)) -> CorWord
{
    let data = seeds.map(|s| s.expand());
    let keep = bit;
    let lose = !keep;

    let mut cw = CorWord {
        seed: data.0.seeds.get(lose) ^ data.1.seeds.get(lose),
        bits: (
            data.0.bits.0 ^ data.1.bits.0 ^ bit ^ true,
            data.0.bits.1 ^ data.1.bits.1 ^ bit,
        ),
    };
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

impl<T> DPFKey<T>
where
    T: prg::FromRng + Clone + Group + std::fmt::Debug
{
    pub fn gen(alpha_bits: &[bool], value:&T) -> (DPFKey<T>, DPFKey<T>) {
        // let root_seeds = (prg::PrgSeed::zero(), prg::PrgSeed::one());
        let root_seeds = (prg::PrgSeed::random(), prg::PrgSeed::random());
        let root_bits = (false, true);

        let mut seeds = root_seeds.clone();
        let mut bits = root_bits;
        let mut cor_words: Vec<CorWord> = Vec::new();
        let mut lastWord:T = T::zero();

        for (i, &bit) in alpha_bits.iter().enumerate() {
            let cw = gen_cor_word(bit, &mut bits, &mut seeds);
            cor_words.push(cw);
            // Generate the last word
            if i==alpha_bits.len()-1{
                let converted = seeds.map(|s| s.convert());
                lastWord.add(&value);
                lastWord.sub(&converted.0.word);
                lastWord.add(&converted.1.word);
                if bits.1 {
                    lastWord.negate();
                }
            }
        }

        (
            DPFKey::<T> {
                key_idx: false,
                root_seed: root_seeds.0,
                cor_words: cor_words.clone(),
                word: lastWord.clone(),
            },
            DPFKey::<T> {
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
        let dir = self.key_idx;
        let mut t_bit:bool = self.key_idx;

        let mut word:T = T::zero();

        for level in 0..idx.len() {
            let bit = idx[level];

            // Step 1: compute tau
            // 2 bis, 2 seeds
            // let tau = seed.expand_dir(!dir, dir);
            let tau = seed.expand();
            seed = tau.seeds.get(bit).clone();
            if t_bit{
                seed = &seed ^ &self.cor_words[level].seed;
                let new_bit = *tau.bits.get(bit);
                t_bit = new_bit ^ self.cor_words[level].bits.get(bit);
                
            }else{ //when t_bit is false, update seed and t_bit as orginal expanded tau value
                t_bit = *tau.bits.get(bit);
            }


            if level==idx.len()-1{
                let converted = seed.convert::<T>();
                word.add(&converted.word);
                if t_bit {
                    word.add(&self.word);
                }

                if self.key_idx {
                    word.negate();
                }
            }
        }

        word
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
    use crate::Group;

    #[test]
    fn evalCheck() {
        // let mut alpha = vec![true];
        // let mut alpha = vec![true,false];
        let mut alpha = crate::u32_to_bits(3, 7);

        let beta = RingElm::from(117u32);
        let (dpf_key0, dpf_key1) = DPFKey::gen(&alpha, &beta);

        {
            let mut evalResult = RingElm::zero();

            let word0 = dpf_key0.eval(&alpha);
            evalResult.add(&word0);

            let word1 = dpf_key1.eval(&alpha);
            evalResult.add(&word1);

            assert_eq!(evalResult, beta);
        }

        {
            let mut evalResult = RingElm::zero();

            alpha[1] ^= true;

            let word0 = dpf_key0.eval(&alpha);
            evalResult.add(&word0);

            let word1 = dpf_key1.eval(&alpha);
            evalResult.add(&word1);

            assert_eq!(evalResult, RingElm::zero());
        }
    }
}
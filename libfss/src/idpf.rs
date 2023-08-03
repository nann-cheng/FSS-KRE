use crate::prg;
use crate::Group;
use serde::Deserialize;
use serde::Serialize;
use std::mem;

use crate::TupleExt;
use crate::TupleMapToExt;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CorWord<T> {
    seed: prg::PrgSeed,
    bits: (bool, bool),
    word: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IDPFKey<T> {
    key_idx: bool,
    root_seed: prg::PrgSeed,
    cor_words: Vec<CorWord<T>>,
}

#[derive(Clone)]
pub struct EvalState {
    level: usize,
    seed: prg::PrgSeed,
    bit: bool,
}

fn gen_cor_word<W>(bit: bool, value: W, bits: &mut (bool, bool), seeds: &mut (prg::PrgSeed, prg::PrgSeed)) -> CorWord<W>
    where W: prg::FromRng + Clone + Group + std::fmt::Debug
{
    let data = seeds.map(|s| s.expand());

    // If alpha[i] = 0:
    //   Keep = L,  Lose = R
    // Else
    //   Keep = R,  Lose = L
    let keep = bit;
    let lose = !keep;


    let mut cw = CorWord {
        seed: data.0.seeds.get(lose) ^ data.1.seeds.get(lose),
        bits: (
            data.0.bits.0 ^ data.1.bits.0 ^ bit ^ true,
            data.0.bits.1 ^ data.1.bits.1 ^ bit,
        ),
        word: W::zero(),
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
    let converted = seeds.map(|s| s.convert());
    cw.word = value;

    cw.word.sub(&converted.0.word);
    cw.word.add(&converted.1.word);

    if bits.1 {
        cw.word.negate();
    }

    seeds.0 = converted.0.seed;
    seeds.1 = converted.1.seed;

    cw
}


impl<T> IDPFKey<T> where T: prg::FromRng + Clone + Group + std::fmt::Debug
{
    pub fn gen(alpha_bits: &[bool], values: &[T]) -> (IDPFKey<T>, IDPFKey<T>) {
        debug_assert!(alpha_bits.len() == values.len() );

        let root_seeds = (prg::PrgSeed::random(), prg::PrgSeed::random());
        let root_bits = (false, true);

        let mut seeds = root_seeds.clone();
        let mut bits = root_bits;
        let mut cor_words: Vec<CorWord<T>> = Vec::new();

        for (i, &bit) in alpha_bits.iter().enumerate() {
            let cw = gen_cor_word::<T>(bit, values[i].clone(), &mut bits, &mut seeds);
            cor_words.push(cw);
        }

        (
            IDPFKey::<T> {
                key_idx: false,
                root_seed: root_seeds.0,
                cor_words: cor_words.clone(),
            },
            IDPFKey::<T> {
                key_idx: true,
                root_seed: root_seeds.1,
                cor_words,
            },
        )
    }

    pub fn eval_bit(&self, state: &EvalState, dir: bool) -> (EvalState, T) {
        let tau = state.seed.expand_dir(!dir, dir);
        let mut seed = tau.seeds.get(dir).clone();
        let mut new_bit = *tau.bits.get(dir);

        if state.bit {
            seed = &seed ^ &self.cor_words[state.level].seed;
            new_bit ^= self.cor_words[state.level].bits.get(dir);
        }

        let converted = seed.convert::<T>();
        seed = converted.seed;

        let mut word = converted.word;
        if new_bit {
            word.add(&self.cor_words[state.level].word);
        }

        if self.key_idx {
            word.negate()
        }

        (
            EvalState {
                level: state.level + 1,
                seed,
                bit: new_bit,
            },
            word,
        )
    }

    pub fn eval_init(&self) -> EvalState {
        EvalState {
            level: 0,
            seed: self.root_seed.clone(),
            bit: self.key_idx,
        }
    }

    pub fn eval(&self, idx: &Vec<bool>) -> T {
        debug_assert!(idx.len() <= self.domain_size());
        debug_assert!(!idx.is_empty());
        let mut out = vec![];
        let mut state = self.eval_init();

        for i in 0..idx.len() {
            let bit = idx[i];
            let (state_new, word) = self.eval_bit(&state, bit);
            out.push(word);
            state = state_new;
        }
        let word = out[idx.len()-1].clone();
        word
    }

    pub fn gen_from_str(s: &str) -> (Self, Self) {
        let bits = crate::string_to_bits(s);
        let values = vec![T::one(); bits.len()-1];

        IDPFKey::gen(&bits, &values)
    }

    pub fn domain_size(&self) -> usize {
        self.cor_words.len()
    }

    pub fn key_size(&self) -> usize {
        let mut keySize = 0usize;

        keySize += mem::size_of_val(&self.key_idx);
        // println!("key_idx is {}",mem::size_of_val(&self.key_idx));


        keySize += mem::size_of_val(&self.root_seed);
        // println!("root_seed is {}",mem::size_of_val(&self.root_seed));


        keySize += mem::size_of_val(&*self.cor_words);
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
        let nbits = 3usize;
        let alpha = crate::u32_to_bits(nbits, 7);

        let values = RingElm::from(1u32).to_vec(nbits);

        let (dpf_key0, dpf_key1) = IDPFKey::gen(&alpha, &values);

        let mut state0 = dpf_key0.eval_init();
        let mut state1 = dpf_key1.eval_init();

        let testNumber = crate::u32_to_bits(nbits, 7);

        //Prefix trial test
        for i in 0..nbits{
            let bit = testNumber[i];
            let (state_new0, word0) = dpf_key0.eval_bit(&state0, bit);
            state0 = state_new0;

            let (state_new1, word1) = dpf_key1.eval_bit(&state1, bit);
            state1 = state_new1;

            let mut sum = RingElm::zero();
            sum.add(&word0);
            sum.add(&word1);

            assert_eq!(sum, values[i]);
        }
    }
}
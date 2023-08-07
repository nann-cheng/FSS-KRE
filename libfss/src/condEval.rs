use crate::prg::{PrgSeed,FixedKeyPrgStream};
use super::{bits_to_u32_BE,u32_to_bits_BE,RingElm,BinElm,ic::*};
use crate::Group;
use serde::Deserialize;
use serde::Serialize;
use bincode::*;
use serde::de::DeserializeOwned;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CondEvalKey{
    cipher_0: Vec<u8>,
    cipher_1: Vec<u8>,
    sk_0: Vec<u8>,
    sk_1: Vec<u8>,
    pi: bool,
    alpha: RingElm,
}

// TODO: Not sure if memory size is as the same as serilization size
impl CondEvalKey
{
    pub fn gen() -> (CondEvalKey, CondEvalKey) {
        let mut condEvalK0 = CondEvalKey{
                cipher_0: Vec::<u8>::new(),
                cipher_1: Vec::<u8>::new(),
                sk_0: Vec::<u8>::new(),
                sk_1: Vec::<u8>::new(),
                pi: false,
                alpha: RingElm::zero(),
            };
        let mut condEvalK1 = condEvalK0.clone();

        //The KeyGen step in Table3 of condEval paper
        let root_seed = PrgSeed::random();
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&root_seed.key);

        let alpha_bits = stream.next_bits(32usize);
        let (p_bound,q_bound) = (RingElm::zero(), RingElm::from((1<<31)-1));
        let (mut key0,  mut key1) = ICKey::gen(&alpha_bits,&p_bound, &q_bound);
        key1.key_idx=false;

        
        let (mut key0_Bin, mut key1_Bin) = (bincode::serialize(&key0).expect("Serialize value error"), bincode::serialize(&key1).expect("Serialize value error"));

        println!("key0_Bin size is: {}",key0_Bin.len());
        println!("key0_Bin first byte is: {}",key1_Bin[0]);
        // let key0_Bin: Vec<u8>;
        // match bincode::serialize(&key0).expect("Serialize value error") {
        //     Ok(value) => key0_Bin = value,
        //     Err(e) => println!("Error reading file: {}", e),  // Or handle the error as needed
        // }

        let rnd_bits = stream.next_bits(3usize);
        let (t_bit,pi_0,pi_1) = (rnd_bits[0],rnd_bits[1],rnd_bits[2]);

        let perSk_len = key0_Bin.len()+1;
        let sk_bytes = stream.next_bytes(4*perSk_len);

        condEvalK0.sk_0 = sk_bytes[0..perSk_len].to_vec();
        condEvalK0.sk_1 = sk_bytes[perSk_len..2*perSk_len].to_vec();

        condEvalK1.sk_0 = sk_bytes[2*perSk_len..3*perSk_len].to_vec();
        condEvalK1.sk_1 = sk_bytes[3*perSk_len..].to_vec();

        condEvalK0.pi = pi_1;
        condEvalK1.pi = pi_0;
        
        let mut alphaNumeric = RingElm::from(bits_to_u32_BE(&alpha_bits));
        let alpha0 = RingElm::from(bits_to_u32_BE(&stream.next_bits(32usize)));

        condEvalK0.alpha = alpha0.clone();
        alphaNumeric.sub(&condEvalK0.alpha);
        condEvalK1.alpha = alphaNumeric;

        //The KeyEnc step in Table3 of condEval paper
        let mut concatenate0 = Vec::<u8>::new();
        if t_bit{
            concatenate0.append(&mut key1_Bin);
            concatenate0.push(1u8);
        }
        else{
            concatenate0.append(&mut key0_Bin);
            concatenate0.push(0u8);
        }
        let msg_00 = condEvalK1.sk_0.iter().zip(concatenate0.iter()).map(|(&a, &b)| a ^ b).collect();
        let msg_10 = condEvalK0.sk_0.iter().zip(concatenate0.iter()).map(|(&a, &b)| a ^ b).collect();


        let mut concatenate1 = Vec::<u8>::new();
        if t_bit{
            concatenate1.append(&mut key0_Bin);
            concatenate1.push(0u8);
        }
        else{
            concatenate1.append(&mut key1_Bin);
            concatenate1.push(1u8);
        }
        let msg_01 = condEvalK1.sk_1.iter().zip(concatenate1.iter()).map(|(&a, &b)| a ^ b).collect();
        let msg_11 = condEvalK0.sk_1.iter().zip(concatenate1.iter()).map(|(&a, &b)| a ^ b).collect();

        if pi_0{
            condEvalK0.cipher_0 = msg_01;
            condEvalK0.cipher_1 = msg_00;
        }
        else{
            condEvalK0.cipher_0 = msg_00;
            condEvalK0.cipher_1 = msg_01;
        }

        if pi_1{
            condEvalK1.cipher_0 = msg_11;
            condEvalK1.cipher_1 = msg_10;
        }
        else{
            condEvalK1.cipher_0 = msg_10;
            condEvalK1.cipher_1 = msg_11;
        }
        (condEvalK0,condEvalK1)
    }

    pub fn eval(&self, x:& RingElm,pointer:bool, sk:& Vec<u8>) -> BinElm {
        let correctCipher = if pointer{ &self.cipher_1} else {&self.cipher_0};
        let decrypted:Vec<u8> = correctCipher.iter().zip(sk.iter()).map(|(&a, &b)| a ^ b).collect();
        println!("sk size is: {}",sk.len());
        println!("correctCipher size is: {}",correctCipher.len());

        match bincode::deserialize(&decrypted[..decrypted.len()-1]){
            Ok(value) => {
                // m_fssKey = value;
                let mut m_fssKey:ICKey= value;
                let m_idx:u8 = decrypted[decrypted.len()-1];
                m_fssKey.key_idx = if m_idx==0{ false} else {true};
                m_fssKey.eval(x)
            },
            Err(e) => {
                println!("Error deserialize file: {}", e);
                BinElm::zero()
                },  // Or handle the error as needed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ring::*;
    use crate::binary::*;
    use crate::Group;

    #[test]
    fn evalCheck(){
        let (mut key0,mut key1) = CondEvalKey::gen();
        let mut x = RingElm::from((1<<31)+100);
        // let mut x = RingElm::from(100);
        x.add(&key0.alpha);
        x.add(&key1.alpha);

        // S_0 does as follows:
        let cmp_0 = BinElm::zero().to_Bool();
        let pointer = key0.pi ^ cmp_0;
        let mut concatenate0  = Vec::<u8>::new();
        concatenate0.append(if cmp_0{ &mut key0.sk_1 } else {&mut key0.sk_0});

        // S_1 does as follows:
        let cmp_1 = BinElm::zero().to_Bool();
        let pointer1 = key1.pi ^ cmp_1;
        let mut concatenate1  = Vec::<u8>::new();
        concatenate1.append(if cmp_1{&mut key1.sk_1 } else {&mut key1.sk_0});

        let mut evalResult = key0.eval(&x, pointer, &concatenate1);
        evalResult.add(&key1.eval(&x,pointer1, &concatenate0));
        
        assert_eq!(evalResult, BinElm::zero());
    }
}
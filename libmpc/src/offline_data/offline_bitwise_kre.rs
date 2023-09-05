use crate::offline_data::*;
use fss::condEval::CondEvalKey;
pub struct BitKreOffline{
    pub base: BasicOffline,
    pub condeval_k_share: Vec<CondEvalKey>,//CondEval keys for lessThan check
}

impl BitKreOffline{
    pub fn new() -> Self{
        Self{base: BasicOffline::new(),  condeval_k_share: Vec::new()}
    }

    pub fn loadData(&mut self,idx:&u8){
        self.base.loadData(idx);

        match read_file(&format!("../data/bitwise_kre_k{}.bin", idx)) {
            Ok(value) => self.condeval_k_share = value,
            Err(e) => println!("Error reading file: {}", e),  //Or handle the error as needed
        }
    }

    pub fn genData(&self, seed: &PrgSeed,input_size: usize, input_bits: usize){
        self.base.genData(&seed,input_size,input_bits, input_bits*4);
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);

        //Offline-Step-4. Random condEval keys
        let mut online_k0: Vec<CondEvalKey> = Vec::new();
        let mut online_k1: Vec<CondEvalKey> = Vec::new();
        for _ in 0..2*input_bits{
            let ( key0, key1) = CondEvalKey::gen();
            online_k0.push(key0);
            online_k1.push(key1);
        }
        write_file("../data/bitwise_kre_k0.bin", &online_k0);
        write_file("../data/bitwise_kre_k1.bin", &online_k1);
    }
}

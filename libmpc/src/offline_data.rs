use fss::beavertuple::BeaverTuple;
use fss::idpf::*;
use fss::dpf::*;
use fss::condEval::CondEvalKey;
use fss::RingElm;
use fss::BinElm;
use fss::Group;
use fss::Share;
use fss::prg::PrgSeed;
use fss::bits_to_u32;
use fss::bits_Xor;
use fss::prg::FixedKeyPrgStream;
use std::path::PathBuf;
use bincode::Error;
use std::fs::File;
use serde::Deserialize;
use serde::Serialize;
use std::io::Write;
use std::io::Read;
use serde::de::DeserializeOwned;

const NUMERIC_LEN:usize = 32;

fn write_file<T: serde::ser::Serialize>(path:&str, value:&T){
    let mut file = File::create(path).expect("create failed");
    file.write_all(&bincode::serialize(&value).expect("Serialize value error")).expect("Write key error.");
}

fn read_file<T: DeserializeOwned>(path: &str) -> Result<T, Error> {
    let mut file = std::fs::File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let value = bincode::deserialize(&buf)?;
    Ok(value)
}

pub struct BasicOffline {
    // seed: PrgSeed,
    pub k_share: Vec<IDPFKey<RingElm>>, //idpf keys
    pub a_share: Vec<bool>,  //alpha

    pub qa_share: Vec<RingElm>, //q arithmetical share
    pub qb_share: Vec<bool>, //q bool share

    pub beavers: Vec<BeaverTuple>,
}

impl BasicOffline{
    pub fn new() -> Self{
        Self{k_share: Vec::new(), a_share: Vec::new(), qa_share: Vec::new(), qb_share: Vec::new(), beavers: Vec::new()}
    }

    pub fn loadData(&mut self,idx:&u8){
        match read_file(&format!("../data/k{}.bin", idx)) {
            Ok(value) => self.k_share = value,
            Err(e) => println!("Error reading file: {}", e),  // Or handle the error as needed
        }

        match read_file(&format!("../data/a{}.bin", idx)) {
            Ok(value) => self.a_share = value,
            Err(e) => println!("Error reading file: {}", e),  // Or handle the error as needed
        }

        match read_file(&format!("../data/qa{}.bin", idx)) {
            Ok(value) => self.qa_share = value,
            Err(e) => println!("Error reading file: {}", e),  // Or handle the error as needed
        }

        match read_file(&format!("../data/qb{}.bin", idx)) {
            Ok(value) => self.qb_share = value,
            Err(e) => println!("Error reading file: {}", e),  // Or handle the error as needed
        }

        match read_file(&format!("../data/beaver{}.bin", idx)) {
            Ok(value) => self.beavers = value,
            Err(e) => println!("Error reading file: {}", e),  // Or handle the error as needed
        }
    }

    pub fn genData(&self,seed: &PrgSeed,input_size: usize, input_bits: usize, beaver_amount: usize){
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);
        //Offline-Step-1. Set IDPF Parameters
        let fix_betas = RingElm::from(1u32).to_vec(input_bits); //generate a series of 1 as beta
        let r_bits = stream.next_bits(input_bits*input_size);  
        //Offline-Step-2. Generate Random I-DPFs
        let mut dpf_0: Vec<IDPFKey<RingElm>> = Vec::new();
        let mut dpf_1: Vec<IDPFKey<RingElm>> = Vec::new();
        for i in 0..input_size{
            let alpha = &r_bits[i*input_bits..(i+1)*input_bits];
            let (k0, k1) = IDPFKey::gen(&alpha, &fix_betas);
            dpf_0.push(k0);
            dpf_1.push(k1);
        }
        write_file("../data/k0.bin", &dpf_0);
        write_file("../data/k1.bin", &dpf_1);
        let r_bits_0 = stream.next_bits(input_bits*input_size);
        let r_bits_1 = bits_Xor(&r_bits, &r_bits_0);
        write_file("../data/a0.bin", &r_bits_0);
        write_file("../data/a1.bin", &r_bits_1);
        //Offline-Step-3. Random daBits for masking
        let q_boolean = stream.next_bits(input_bits);
        // println!("q_boolean is: {} ",vec_bool_to_string(&q_boolean));
        let q_boolean_0 = stream.next_bits(input_bits);
        let q_boolean_1 = bits_Xor(&q_boolean, &q_boolean_0);
        write_file("../data/qb0.bin", &q_boolean_0);
        write_file("../data/qb1.bin", &q_boolean_1);
        let mut q_numeric = Vec::new();
        let mut q_numeric_0 = Vec::new();
        let mut q_numeric_1 = Vec::new();
        for i in 0..input_bits{
            let mut q_i = RingElm::zero();
            if q_boolean[i]{
                q_i = RingElm::from(1u32);
            }
            let (q_i_0,q_i_1) = q_i.share();
            q_numeric.push(q_i);
            q_numeric_0.push(q_i_0);
            q_numeric_1.push(q_i_1);
        }
        write_file("../data/qa0.bin", &q_numeric_0);
        write_file("../data/qa1.bin", &q_numeric_1);

        self.genBeaver(&seed, beaver_amount);
    }

    pub fn genBeaver(&self, seed: &PrgSeed, size:usize){
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);

        let mut beavertuples0: Vec<BeaverTuple> = Vec::new();
        let mut beavertuples1: Vec<BeaverTuple> = Vec::new();
        for i in 0..size{
            let rd_bits = stream.next_bits(NUMERIC_LEN*5);
            let a0 = RingElm::from( bits_to_u32(&rd_bits[..NUMERIC_LEN]) );
            let b0 = RingElm::from( bits_to_u32(&rd_bits[NUMERIC_LEN..2*NUMERIC_LEN]) );

            let a1 = RingElm::from( bits_to_u32(&rd_bits[2*NUMERIC_LEN..3*NUMERIC_LEN]) );
            let b1 = RingElm::from( bits_to_u32(&rd_bits[3*NUMERIC_LEN..4*NUMERIC_LEN]));

            let ab0 = RingElm::from( bits_to_u32(&rd_bits[4*NUMERIC_LEN..5*NUMERIC_LEN]) );

            let mut a = RingElm::zero();
            a.add(&a0);
            a.add(&a1);

            let mut b = RingElm::zero();
            b.add(&b0);
            b.add(&b1);

            let mut ab = RingElm::one();
            ab.mul(&a);
            ab.mul(&b);

            ab.sub(&ab0);

            let beaver0 = BeaverTuple{
                a: a0,
                b: b0,
                ab: ab0,
                delta_a:RingElm::zero(),
                delta_b:RingElm::zero(),
            };

            let beaver1 = BeaverTuple{
                a: a1,
                b: b1,
                ab: ab,
                delta_a:RingElm::zero(),
                delta_b:RingElm::zero(),
            };
            beavertuples0.push(beaver0);
            beavertuples1.push(beaver1);
        }
        write_file("../data/beaver0.bin", &beavertuples0);
        write_file("../data/beaver1.bin", &beavertuples1);
    }
}

pub struct BitMaxOffline{
    pub base: BasicOffline,
    pub zc_k_share: Vec<DPFKey<BinElm>>,//dpf keys for zero_check
    pub zc_a_share: Vec<RingElm>,
}

impl BitMaxOffline{
    pub fn new() -> Self{
        Self{base: BasicOffline::new(),  zc_k_share: Vec::new(), zc_a_share: Vec::new()}
    }

    pub fn loadData(&mut self,idx:&u8){
        self.base.loadData(idx);

        match read_file(&format!("../data/zc_a{}.bin", idx)) {
            Ok(value) => self.zc_a_share = value,
            Err(e) => println!("Error reading file: {}", e),  //Or handle the error as needed
        }

        match read_file(&format!("../data/zc_k{}.bin", idx)) {
            Ok(value) => self.zc_k_share = value,
            Err(e) => println!("Error reading file: {}", e),  //Or handle the error as needed
        }
    }

    pub fn genData(&self, seed: &PrgSeed,input_size: usize, input_bits: usize){
        self.base.genData(&seed,input_size,input_bits, input_bits*2);
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);

        //Offline-Step-4. Random DPFs for zeroCheck, input_bits required in total
        let mut zero_dpf_0: Vec<DPFKey<BinElm>> = Vec::new();
        let mut zero_dpf_1: Vec<DPFKey<BinElm>> = Vec::new();

        let mut zero_dpf_r0: Vec<RingElm> = Vec::new();
        let mut zero_dpf_r1: Vec<RingElm> = Vec::new();
        
        for _ in 0..input_bits{
            let zero_r_bits = stream.next_bits(NUMERIC_LEN*2);

            let mut numeric_zero_r_1 = RingElm::from( bits_to_u32(&zero_r_bits[..NUMERIC_LEN]) );
            let numeric_zero_r = RingElm::from( bits_to_u32(&zero_r_bits[..NUMERIC_LEN]) );

            println!("numeric_zero_r={:?}", numeric_zero_r);
            // println!("Vec<bool>: {:?}", zero_r_bits[..NUMERIC_LEN].to_vec());
            let numeric_zero_r_0 = RingElm::from( bits_to_u32(&zero_r_bits[NUMERIC_LEN..]) );
            numeric_zero_r_1.sub(&numeric_zero_r_0);
            // let zero_betas: Vec<BinElm> = BinElm::from(false).to_vec(NUMERIC_LEN);
            let zero_beta: BinElm = BinElm::one();
            let (k0, k1) = DPFKey::gen(&zero_r_bits[..NUMERIC_LEN], &zero_beta);

            zero_dpf_0.push(k0);
            zero_dpf_1.push(k1);
            zero_dpf_r0.push(numeric_zero_r_0);
            zero_dpf_r1.push(numeric_zero_r_1);
        }
        write_file("../data/zc_a0.bin", &zero_dpf_r0);
        write_file("../data/zc_a1.bin", &zero_dpf_r1);
        write_file("../data/zc_k0.bin", &zero_dpf_0);
        write_file("../data/zc_k1.bin", &zero_dpf_1);
    }
}


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
        self.base.genData(&seed,input_size,input_bits, input_bits*5);
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


#[cfg(test)]
mod tests {
    use crate::offline_data::BitMaxOffline;
    use crate::offline_data::BitKreOffline;
    use fss::prg::PrgSeed;

    #[test]
    fn io_check() {
        // let seed = PrgSeed::random();
        let seed = PrgSeed::one();

        // let mut bitMax = BitMaxOffline::new(0u8);
        // bitMax.genData(&seed,3usize,5usize);
        // bitMax.loadData();


        let mut bitKre = BitKreOffline::new();
        bitKre.genData(&seed,3usize,5usize);
        bitKre.loadData(&0u8);
    }
}
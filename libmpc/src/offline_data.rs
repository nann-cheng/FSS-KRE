use fss::beavertuple::BeaverTuple;
use fss::idpf::*;
use fss::dpf::*;
use fss::RingElm;
use fss::BinElm;
use fss::Group;
use fss::Share;
use fss::prg::PrgSeed;
use fss::{bits_to_u32,bits_Xor};
use fss::prg::FixedKeyPrgStream;
use bincode::Error;
use std::fs::File;
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

    pub fn genData(&self,seed: &PrgSeed,input_size: usize, input_bits: usize, beaver_amount: usize)->Vec<bool>{ //there, I think genData() should be a class method
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

        let mut beavertuples0: Vec<BeaverTuple> = Vec::new();
        let mut beavertuples1: Vec<BeaverTuple> = Vec::new();
        BeaverTuple::genBeaver(&mut beavertuples0, &mut beavertuples1, &seed, beaver_amount);
        write_file("../data/beaver0.bin", &beavertuples0);
        write_file("../data/beaver1.bin", &beavertuples1);

        q_boolean
    }
}


pub mod offline_bitwise_max;
pub mod offline_bitwise_kre;
pub mod offline_batch_max;
pub mod offline_batch_kre;
pub mod offline_ic_max;
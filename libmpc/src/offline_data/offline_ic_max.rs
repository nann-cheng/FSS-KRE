/*To implement the max by greater_than function: fzhang, update0920*/
/*There are two implementation all finished on 09/21/2023 by fzhang*/
use super::NUMERIC_LEN;
use fss::ic::*;
use fss::beavertuple::BeaverTuple;
use fss::{Group, RingElm};

use fss::dpf::*;

use super::{write_file, read_file};
use fss::{bits_to_u32, bits_to_u32_BE};
use fss::prg::FixedKeyPrgStream;
//use std::path::PathBuf;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaxOffline_IC{
    pub ic_alpha : Vec<RingElm>,
    pub ic_key: Vec<ICCKey>,
    pub beavers: Vec<BeaverTuple>,

    //the following items are used to extract the k-th max from the sorted list
    pub zc_alpha: Vec<RingElm>,
    pub zc_key: Vec<DPFKey<RingElm>>,//dpf keys for zero_check
} 

impl MaxOffline_IC{
    pub fn new() -> Self{
        Self{ic_alpha: Vec::<RingElm>::new(), ic_key: Vec::<ICCKey>::new(), beavers: Vec::<BeaverTuple>::new(),zc_alpha: Vec::<RingElm>::new(), zc_key: Vec::<DPFKey<RingElm>>::new()}
    }

    pub fn genData(stream: &mut FixedKeyPrgStream, ic_key_size: usize, cbeavers_num: usize, zc_key_size: usize){
        //let alpha_bits = stream.next_bits(32usize);
        let (p_bound,q_bound) = (RingElm::zero(), RingElm::from((1<<31)-1));
        let mut alpha0 = Vec::<RingElm>::new();
        let mut alpha1 = Vec::<RingElm>::new();
        let mut ic_key_0 = Vec::<ICCKey>::new();
        let mut ic_key_1 = Vec::<ICCKey>::new();
        for _ in 0..ic_key_size{
            let alpha_bits = stream.next_bits(NUMERIC_LEN);
            let (key0, key1) = ICCKey::gen(&alpha_bits,&p_bound, &q_bound);
            ic_key_0.push(key0);
            ic_key_1.push(key1);
            let alpha_bits_share = stream.next_bits(NUMERIC_LEN);
            let mut alpha_numeric = RingElm::from(bits_to_u32_BE(&alpha_bits));
            let mut alpha_share = RingElm::from(bits_to_u32_BE(&alpha_bits_share));
            alpha_numeric.sub(&alpha_share);
            alpha0.push(alpha_share);
            alpha1.push(alpha_numeric);
        }
        
        write_file("../data/ic_key0.bin", &ic_key_0);
        write_file("../data/ic_key1.bin", &ic_key_1);
        write_file("../data/ic_alpha0.bin", &alpha0);
        write_file("../data/ic_alpha1.bin", &alpha1);

        let mut beavertuples0: Vec<BeaverTuple> = Vec::new();
        let mut beavertuples1: Vec<BeaverTuple> = Vec::new();
        for i in 0..cbeavers_num{
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

        if zc_key_size > 0 { //
            let mut zero_dpf_0: Vec<DPFKey<RingElm>> = Vec::new();
            let mut zero_dpf_1: Vec<DPFKey<RingElm>> = Vec::new();

            let mut zero_dpf_r0: Vec<RingElm> = Vec::new();
            let mut zero_dpf_r1: Vec<RingElm> = Vec::new();
            for _ in 0..zc_key_size{
                let zero_r_bits = stream.next_bits(NUMERIC_LEN*2);
    
                let mut numeric_zero_r_1 = RingElm::from( bits_to_u32(&zero_r_bits[..NUMERIC_LEN]) );
                //let numeric_zero_r = RingElm::from( bits_to_u32(&zero_r_bits[..NUMERIC_LEN]) );
    
                //println!("numeric_zero_r={:?}", numeric_zero_r);
                // println!("Vec<bool>: {:?}", zero_r_bits[..NUMERIC_LEN].to_vec());
                let numeric_zero_r_0 = RingElm::from( bits_to_u32(&zero_r_bits[NUMERIC_LEN..]) );
                numeric_zero_r_1.sub(&numeric_zero_r_0);
                // let zero_betas: Vec<BinElm> = BinElm::from(false).to_vec(NUMERIC_LEN);
                let zero_beta: RingElm = RingElm::one();
                let (k0, k1) = DPFKey::gen(&zero_r_bits[..NUMERIC_LEN], &zero_beta);
    
                zero_dpf_0.push(k0);
                zero_dpf_1.push(k1);
                zero_dpf_r0.push(numeric_zero_r_0);
                zero_dpf_r1.push(numeric_zero_r_1);
            }
            write_file("../data/zc_alpha0.bin", &zero_dpf_r0);
            write_file("../data/zc_alpha1.bin", &zero_dpf_r1);
            write_file("../data/zc_key0.bin", &zero_dpf_0);
            write_file("../data/zc_key1.bin", &zero_dpf_1);
        }
    }

    pub fn loadData(&mut self, idx:&u8, extrainfo: bool){
        match read_file(&format!("../data/ic_alpha{}.bin", idx)) {
            Ok(value) => self.ic_alpha = value,
            Err(e) => println!("Error reading file: {}", e),  // Or handle the error as needed
        }
        
        match read_file(&format!("../data/ic_key{}.bin", idx)) {
            Ok(value) => self.ic_key = value,
            Err(e) => println!("Error reading file: {}", e),  // Or handle the error as needed
        }

        match read_file(&format!("../data/beaver{}.bin", idx)) {
            Ok(value) => self.beavers = value,
            Err(e) => println!("Error reading file: {}", e),  // Or handle the error as needed
        }

        if extrainfo == true{
            match read_file(&format!("../data/zc_key{}.bin", idx)) {
                Ok(value) => self.zc_key = value,
                Err(e) => println!("Error reading file: {}", e),  // Or handle the error as needed
            }
    
            match read_file(&format!("../data/zc_alpha{}.bin", idx)) {
                Ok(value) => self.zc_alpha = value,
                Err(e) => println!("Error reading file: {}", e),  // Or handle the error as needed
            }
        }
    }
}


#[cfg(test)]
mod test{
    use super::*;
    use fss::{prg::*, RingElm};
    use fss::u32_to_bits;
    #[test]
    fn gen_data_for_max_ic(){
        const INPUT_SIZE: usize = 10;
        //const INPUT_BITS: usize = 32;
        let (p_bound,q_bound) = (RingElm::zero(), RingElm::from((1<<31)-1));
        let seed = PrgSeed::zero();//Guarantee same input bits to ease the debug process
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);

        MaxOffline_IC::genData(&mut stream, INPUT_SIZE*(INPUT_SIZE - 1) / 2 , INPUT_SIZE * (INPUT_SIZE-1) / 2 + 2 * INPUT_SIZE, INPUT_SIZE);
        let mut offline0 = MaxOffline_IC::new();
        let mut offline1 = MaxOffline_IC::new();
        offline0.loadData(&0u8, false);
        offline1.loadData(&1u8, false);
        for i in 0..INPUT_SIZE{
            /*let alpha_bits = stream.next_bits(NUMERIC_LEN);
            let mut alpha_numeric = RingElm::from(bits_to_u32_BE(&alpha_bits));
            let (key0, key1) = ICCKey::gen(&alpha_bits,&p_bound, &q_bound);*/
            let key0 = &offline0.ic_key[i];
            let key1 = &offline1.ic_key[i];
            let alpha = offline0.ic_alpha[i] + offline1.ic_alpha[i];
            let x = RingElm::from(200); 
            let y = RingElm::from(199);

            let r0 = key0.eval(&(x - y + alpha));
            let r1 = key1.eval(&(x - y + alpha));
            //println!("r = {:?}", r0 + r1);
            assert_eq!(r0 + r1, RingElm::from(1));
        }

    }

    #[test]
    fn GreaterThan_works()
    // if x < y, return 0, else return 1; 
    {
        let (p_bound,q_bound) = (RingElm::zero(), RingElm::from((1<<31)-1));
        let seed = PrgSeed::zero();//Guarantee same input bits to ease the debug process
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);

        for _ in 0..100{
            let alpha_bits = stream.next_bits(NUMERIC_LEN);
            let mut alpha_numeric = RingElm::from(bits_to_u32_BE(&alpha_bits));
            let (key0, key1) = ICCKey::gen(&alpha_bits,&p_bound, &q_bound);

            let x = RingElm::from(189); 
            let y = RingElm::from(199);

            let r0 = key0.eval(&(x - y + alpha_numeric));
            let r1 = key1.eval(&(x - y + alpha_numeric));
            //println!("r = {:?}", r0 + r1);
            assert_eq!(r0 + r1, RingElm::from(0));
        }
    }

    #[test]
    fn ZeroCheckForRing_works(){ //if return 1 if x =0, else return 1; the evaluted point is at x + alpha
        let seed = PrgSeed::zero();//Guarantee same input bits to ease the debug process
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);
        
        let zero_r_bits = stream.next_bits(NUMERIC_LEN*2);
        let mut numeric_zero_r_1 = RingElm::from( bits_to_u32(&zero_r_bits[..NUMERIC_LEN]) );
        let numeric_zero_r = RingElm::from( bits_to_u32(&zero_r_bits[..NUMERIC_LEN]) );

        //println!("numeric_zero_r={:?}", numeric_zero_r);
        // println!("Vec<bool>: {:?}", zero_r_bits[..NUMERIC_LEN].to_vec());
        let numeric_zero_r_0 = RingElm::from( bits_to_u32(&zero_r_bits[NUMERIC_LEN..]) );
        numeric_zero_r_1.sub(&numeric_zero_r_0);
        // let zero_betas: Vec<BinElm> = BinElm::from(false).to_vec(NUMERIC_LEN);
        let zero_beta = RingElm::one();
        let (k0, k1) = DPFKey::gen(&zero_r_bits[..NUMERIC_LEN], &zero_beta);

        //eval
        let x_fznc = RingElm::from(0) + numeric_zero_r_0 + numeric_zero_r_1;
        let mut vec_eval = vec![false;32usize];
        let num_eval = x_fznc.to_u32();
        match num_eval {
            Some(numeric) => vec_eval = u32_to_bits(32usize,numeric),
            None      => println!( "u32 Conversion failed!!" ),
        }
        let r0 = k0.eval(&vec_eval);
        let r1 = k1.eval(&vec_eval);

        println!("r={:?}", r0+r1);

    }
}
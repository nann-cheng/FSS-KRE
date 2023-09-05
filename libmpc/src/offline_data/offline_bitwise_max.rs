use crate::offline_data::*;
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

            //println!("numeric_zero_r={:?}", numeric_zero_r);
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
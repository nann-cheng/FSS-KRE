use super::*;
use fss::ic::ICCKey;
use fss::prg::FixedKeyPrgStream;
use fss::{bits_to_u32, bits_to_u8_BE, u32_to_bits, u32_to_bits_BE};
use fss::mbeaver::*;
use fss::qmatrix::*;

pub struct BatchKreOffline {
    pub base: BasicOffline,
    pub let_k_share: Vec<ICCKey>, //dpf keys for less-than-equal
    pub let_a_share: Vec<RingElm>,
    pub qelmmatrix_share: Vec<QElmMatrix>, // The convert matrix that is in the form of 1-dim, when using it, the two indexs should be transformed into one-dim index
    pub qbeavers: Vec<Vec<BeaverTuple>>, // common beaver vetor for 2 terms product in binary ring
    pub cbeavers: Vec<Vec<BeaverTuple>>, // the i-dim beaver tuples where i from 2 to 2^\omega in order
    pub kbeavers: Vec<Vec<BeaverTuple>>,
}

impl BatchKreOffline {
    pub fn new() -> Self {
        Self {
            base: BasicOffline::new(),
            let_k_share: Vec::new(),
            let_a_share: Vec::new(),
            qelmmatrix_share: Vec::new(),
            qbeavers: Vec::new(),
            cbeavers: Vec::new(),
            kbeavers: Vec::new(),
        }
    }

    pub fn loadData(&mut self, idx: &u8) {
        self.base.loadData(idx);

        match read_file(&format!("../data/let_a{}.bin", idx)) {
            Ok(value) => self.let_a_share = value,
            Err(e) => println!("Error reading file: {}", e), //Or handle the error as needed
        }

        match read_file(&format!("../data/let_k{}.bin", idx)) {
            Ok(value) => self.let_k_share = value,
            Err(e) => println!("Error reading file: {}", e), //Or handle the error as needed
        }

        match read_file(&format!("../data/qelmmatrix{}.bin", idx)) {
            Ok(value) => self.qelmmatrix_share = value,
            Err(e) => println!("Error reading file: {}", e), //Or handle the error as needed
        }

        match read_file(&format!("../data/qbeavers{}.bin", idx)) {
            Ok(value) => self.qbeavers = value,
            Err(e) => println!("Error reading file: {}", e), //Or handle the error as needed
        }

        match read_file(&format!("../data/cbeavers{}.bin", idx)) {
            Ok(value) => self.cbeavers = value,
            Err(e) => println!("Error reading file: {}", e), //Or handle the error as needed
        }
        
        match read_file(&format!("../data/kbeavers{}.bin", idx)) {
            Ok(value) => self.kbeavers = value,
            Err(e) => println!("Error reading file: {}", e), //Or handle the error as needed
        }
    }

    pub fn genData(
        &self,
        seed: &PrgSeed,
        input_size: usize,
        input_bits: usize,
        batch_size: usize,
    ) {
        let q_boolean = self
            .base
            .genData(&seed, input_size, input_bits, input_bits * 2);
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);

        let every_batch_num: usize = 1 << batch_size; // the maximum of a batch
        let block_num = input_bits / batch_size; // the block number
        
        //Offline-Step-4. Random DPFs for zeroCheck, input_bits required in total
        let mut let_icc_0: Vec<ICCKey> = Vec::new();
        let mut let_icc_1: Vec<ICCKey> = Vec::new();

        let mut let_icc_r0: Vec<RingElm> = Vec::new();
        let mut let_icc_r1: Vec<RingElm> = Vec::new();

        for _ in 0..every_batch_num * block_num {
            // It needs call {\tao} f_znc in every block
            let let_r_bits = stream.next_bits(NUMERIC_LEN * 2);

            let mut numeric_zero_r_1 = RingElm::from(bits_to_u32(&let_r_bits[..NUMERIC_LEN]));
            let numeric_zero_r_0 = RingElm::from(bits_to_u32(&let_r_bits[NUMERIC_LEN..]));
            numeric_zero_r_1.sub(&numeric_zero_r_0);
            
            let p_bound = RingElm::zero();
            let q_bound = RingElm::from((1<<31)-1);
    
            let (k0, k1) = ICCKey::gen(&let_r_bits[..NUMERIC_LEN], &p_bound, &q_bound);

            let_icc_0.push(k0);
            let_icc_1.push(k1);
            let_icc_r0.push(numeric_zero_r_0);
            let_icc_r1.push(numeric_zero_r_1);
        }

        write_file("../data/let_a0.bin", &let_icc_r0);
        write_file("../data/let_a1.bin", &let_icc_r1);
        write_file("../data/let_k0.bin", &let_icc_0);
        write_file("../data/let_k1.bin", &let_icc_1);

        //Offline-Step-3.1 Q terms value generation

        let mut qelmmatrix_share0 = Vec::<QElmMatrix>::new();
        let mut qelmmatrix_share1 = Vec::<QElmMatrix>::new();

        for i in 0..block_num {
            //let q_matrix_i = f_conv_matrix(&self.base.qb_share[i*batch_size..(i+1)*batch_size].to_vec(), batch_size);
            let q_matrix_i = f_conv_matrix(
                &q_boolean[i * batch_size..(i + 1) * batch_size].to_vec(),
                batch_size,
            ); //changed 08-17
            let q_elm_matrix_i = QElmMatrix::convertFromQMatrix(q_matrix_i);
            let (qm0, qm1) = q_elm_matrix_i.split();
            qelmmatrix_share0.push(qm0);
            qelmmatrix_share1.push(qm1);
        }

        write_file("../data/qelmmatrix0.bin", &qelmmatrix_share0);
        write_file("../data/qelmmatrix1.bin", &qelmmatrix_share1);

        let mut qbeavers_1 = Vec::new();
        let mut qbeavers_2 = Vec::new();

        for i in 0..block_num {
            let mut qbeavers_1_t = Vec::<BeaverTuple>::new();
            let mut qbeavers_2_t = Vec::<BeaverTuple>::new();
    
            let qbeavers_num = every_batch_num * every_batch_num;
            BeaverTuple::genBeaver(&mut qbeavers_1_t, &mut qbeavers_2_t, &seed, qbeavers_num);
            
            qbeavers_1.push(qbeavers_1_t);
            qbeavers_2.push(qbeavers_2_t);
        }

        write_file("../data/qbeavers0.bin", &qbeavers_1);
        write_file("../data/qbeavers1.bin", &qbeavers_2);

        let mut cbeavers_1 = Vec::new();
        let mut cbeavers_2 = Vec::new();

        for i in 0..block_num{
            let mut cbeavers_1_t = Vec::<BeaverTuple>::new();
            let mut cbeavers_2_t = Vec::<BeaverTuple>::new();        
            
            let cbeavers_num = every_batch_num-1;
            BeaverTuple::genBeaver(&mut cbeavers_1_t, &mut cbeavers_2_t, &seed, cbeavers_num);
        
            cbeavers_1.push(cbeavers_1_t);
            cbeavers_2.push(cbeavers_2_t);
        }

        write_file("../data/cbeavers0.bin", &cbeavers_1);
        write_file("../data/cbeavers1.bin", &cbeavers_2);

        let mut kbeavers_1 = Vec::new();
        let mut kbeavers_2 = Vec::new();

        for i in 0..block_num{
            let mut kbeavers_1_t = Vec::<BeaverTuple>::new();
            let mut kbeavers_2_t = Vec::<BeaverTuple>::new();        
            
            let kbeavers_num = every_batch_num;
            BeaverTuple::genBeaver(&mut kbeavers_1_t, &mut kbeavers_2_t, &seed, kbeavers_num);
            
            kbeavers_1.push(kbeavers_1_t);
            kbeavers_2.push(kbeavers_2_t);
        }
        write_file("../data/kbeavers0.bin", &kbeavers_1);
        write_file("../data/kbeavers1.bin", &kbeavers_2);
    }
}
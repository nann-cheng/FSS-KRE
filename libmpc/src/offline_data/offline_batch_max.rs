use super::*;
use fss::{bits_to_u32, u32_to_bits_BE, u32_to_bits, bits_to_u8_BE};
use fss::prg::FixedKeyPrgStream;
//use std::path::PathBuf;
use serde::Deserialize;
use serde::Serialize;
use fss::mbeaver::MBeaver;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QMatrix{ //This structure is defined for the ConvMatrix 
    pub v: Vec<bool>,
    pub n: usize
} // The offline data used in every batch.

impl QMatrix{
    //type Output = bool;
    pub fn locate(&self, i: usize, j: usize) -> bool{
         self.v[i*self.n + j]
    }

    pub fn Mutlocate(&mut self, i: usize, j: usize) -> &mut bool{
        &mut self.v[i*self.n + j]
    }

    pub fn split(&self) ->(Self, Self){
        let seed = PrgSeed::random();
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);

        let bv1 = stream.next_bits(self.n * self.n);
        let mut bv0 = self.v.clone();

        for i in 0..self.n * self.n{
            bv0[i] = bv0[i] ^ bv1[i];
        }

        (Self{v: bv0, n: self.n}, Self{v: bv1, n: self.n})
    }

    pub fn print(&self){
        print!("[");
        for i in 0..self.n{
            for j in 0..self.n{
                if self.v[i*self.n+j]{
                    print!("1 ");
                }
                else{
                    print!("0 ");
                }
            }
            if i < self.n - 1{
                print!("\n ");
            }
        }
        println!("]");
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MBeaverBlock{
    pub n: usize, //batsize
    pub mbs: Vec<MBeaver>, // a MBeaver Serie for 2..n terms product
    //pub b2d: Vec<MBeaver>  // s common beaver vetor for 2 terms product
}

impl  MBeaverBlock{
    pub fn gen(dim: usize) -> Self{
        let mut mbss = Vec::<MBeaver>::new();
        for i in 2..=dim{
            let beaver = MBeaver::gen(i);
            mbss.push(beaver);
        }
        Self { n: dim, mbs:mbss }
    }

    pub fn split(&self) -> (Self, Self){
        let mut mbs1 = Vec::<MBeaver>::new();
        let mut mbs2 = Vec::<MBeaver>::new();

        for i in 2..=self.n{ //changed: form 2 to n 
            let (b1, b2) = self.mbs[i-2].split(); //changed: from i to i-2
            mbs1.push(b1);
            mbs2.push(b2);
        }
        (Self{n: self.n, mbs: mbs1},
         Self{n: self.n, mbs: mbs2})
    }    
} 


pub struct BatchMaxOffline{
    pub base: BasicOffline,
    //pub batch_size: usize,
    pub zc_k_share: Vec<DPFKey<BinElm>>,//dpf keys for zero_check
    pub zc_a_share: Vec<RingElm>,
    pub qmatrix_share: Vec<QMatrix>, // The convert matrix that is in the form of 1-dim, when using it, the two indexs should be transformed into one-dim index
    pub mbeavers: Vec<MBeaverBlock>, // the i-dim beaver tuples where i from 2 to 2^\omega in order
    pub binary_beavers: Vec<MBeaver>  // common beaver vetor for 2 terms product in binary ring
}

impl BatchMaxOffline{
    pub fn new() -> Self{
        Self{base: BasicOffline::new(), zc_k_share: Vec::new(), zc_a_share: Vec::new(), qmatrix_share: Vec::new(), mbeavers: Vec::new(), binary_beavers: Vec::new()}
    }

    pub fn loadData(&mut self, idx: &u8){
        self.base.loadData(idx);

        match read_file(&format!("../data/zc_a{}.bin", idx)) {
            Ok(value) => self.zc_a_share = value,
            Err(e) => println!("Error reading file: {}", e),  //Or handle the error as needed
        }

        match read_file(&format!("../data/zc_k{}.bin", idx)) {
            Ok(value) => self.zc_k_share = value,
            Err(e) => println!("Error reading file: {}", e),  //Or handle the error as needed
        }

        match read_file(&format!("../data/qmatrix{}.bin", idx)) {
            Ok(value) => self.qmatrix_share = value,
            Err(e) => println!("Error reading file: {}", e),  //Or handle the error as needed
        }

        match read_file(&format!("../data/mbeaver{}.bin", idx)) {
            Ok(value) => self.mbeavers = value,
            Err(e) => println!("Error reading file: {}", e),  //Or handle the error as needed
        }

        match read_file(&format!("../data/binary_beavers{}.bin", idx)) {
            Ok(value) => self.binary_beavers = value,
            Err(e) => println!("Error reading file: {}", e),  //Or handle the error as needed
        }
    }

    pub fn genData(&self, seed: &PrgSeed,input_size: usize, input_bits: usize, batch_size: usize, cbeavers_num: usize){
        let q_boolean = self.base.genData(&seed,input_size,input_bits, input_bits*2);
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);

        //Offline-Step-4. Random DPFs for zeroCheck, input_bits required in total
        let mut zero_dpf_0: Vec<DPFKey<BinElm>> = Vec::new();
        let mut zero_dpf_1: Vec<DPFKey<BinElm>> = Vec::new();

        let mut zero_dpf_r0: Vec<RingElm> = Vec::new();
        let mut zero_dpf_r1: Vec<RingElm> = Vec::new();
        
        let every_batch_num:usize = 1 << batch_size; // the maximum of a batch
         let block_num = input_bits / batch_size; // the block number 
         let remain_bits = input_bits % batch_size;
         
        for _ in 0..every_batch_num * block_num{ // It needs call {\tao} f_znc in every block
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

         //Offline-Step-3.1 Q terms value generation
         
         let mut qmatrix_share0 = Vec::<QMatrix>::new();
         let mut qmatrix_share1 = Vec::<QMatrix>::new();
         
         for i in 0..block_num{
            //let q_matrix_i = f_conv_matrix(&self.base.qb_share[i*batch_size..(i+1)*batch_size].to_vec(), batch_size);
            let q_matrix_i = f_conv_matrix(&q_boolean[i*batch_size..(i+1)*batch_size].to_vec(), batch_size); //changed 08-17
            let (qm0, qm1) = q_matrix_i.split();
            qmatrix_share0.push(qm0);
            qmatrix_share1.push(qm1);
         }
         

        write_file("../data/qmatrix0.bin", &qmatrix_share0);
        write_file("../data/qmatrix1.bin", &qmatrix_share1);

        let mut mbb0 = Vec::<MBeaverBlock>::new();
        let mut mbb1 = Vec::<MBeaverBlock>::new();
        for i in 0..block_num{
                let mbb_item_i = MBeaverBlock::gen(every_batch_num);
                let(mbb_i_0, mbb_i_1) = mbb_item_i.split();
                mbb0.push(mbb_i_0);
                mbb1.push(mbb_i_1);
            
        }
        write_file("../data/mbeaver0.bin", &mbb0);
        write_file("../data/mbeaver1.bin", &mbb1);

        let mut binary_beavers_1 = Vec::<MBeaver>::new();
        let mut binary_beavers_2 = Vec::<MBeaver>::new();
        for i in 0..cbeavers_num{
            let binary_beaver_i = MBeaver::gen(2);
            let (binary_beaver_i_1, binary_beaver_i_2) = binary_beaver_i.split();
            binary_beavers_1.push(binary_beaver_i_1);
            binary_beavers_2.push(binary_beaver_i_2);
        }

        write_file("../data/binary_beavers0.bin", &binary_beavers_1);
        write_file("../data/binary_beavers1.bin", &binary_beavers_1);
       
    }

}
//assume batch_size <= 8 
pub fn f_conv_matrix(q:&Vec<bool>, batch_size: usize) -> QMatrix{
    let every_batch_num: usize = 1 << batch_size;
    /*let q_num = bits_to_u8_BE(q); //indicate the location to get 1..1
    println!("q_num={}", q_num);
    let all_one_pos = q_num as usize;
    let  mut v = vec![false; (every_batch_num * every_batch_num) as usize];
    for i in 0..every_batch_num{
        v[i*every_batch_num + ((all_one_pos + i) % every_batch_num)] = true;
    }*/
    let mut const_bdc_bits = Vec::<bool>::new();
    for i in 0..every_batch_num{
        let cur_bits = u32_to_bits_BE(batch_size, (every_batch_num-1-i).try_into().unwrap()); 
        //convert int to {omega}-bits. q[0..{omega}]
        const_bdc_bits.extend(cur_bits);
    }

    let mut v = vec![false; every_batch_num * every_batch_num];
    for i in 0..every_batch_num{
        let mut pos_bits = vec![false; batch_size];
        for j in 0..batch_size{
            pos_bits[j] = q[j] ^ const_bdc_bits[i*batch_size + j];
        }
        let pos: usize = bits_to_u8_BE(&pos_bits).into();
        println!("i= {},pos = {}", i, pos);
        v[i*every_batch_num + (every_batch_num-pos-1)] = true; 
    }
    QMatrix{ v: v, n: every_batch_num}
} 


#[cfg(test)]
mod tests {
    use crate::offline_data::BitMaxOffline;
    use crate::offline_data::offline_batch_max::f_conv_matrix;
    use fss::prg::PrgSeed;

   // #[test]
    fn io_check() {
        let mut bitMax = BitMaxOffline::new();
        // let seed = PrgSeed::random();
        let seed = PrgSeed::one();

        bitMax.genData(&seed,3usize,5usize);
        bitMax.loadData(&0);
    }

    #[test]
    fn fconvmatrix_works(){
        let mut q = Vec::<bool>::new();
        q.push(false);
        q.push(false);
        //q.push(true);

        let m =  f_conv_matrix(&q, 2);
        //println!("{:?}", m);
        m.print();

        let (m1, m2) = m.split();

        assert_eq!(m1.n, m2.n);

        for i in 0..m1.n*m2.n{
            assert_eq!(m1.v[i] ^ m2.v[i], m.v[i]);
        }
    }

}
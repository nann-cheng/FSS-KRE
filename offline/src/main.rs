use idpf::*;
use idpf::prg::*;
use idpf::dpf::*;
use idpf::RingElm;
use idpf::BinElm;

use std::fs::File;
use bincode::{serialize, deserialize};
use std::io::prelude::*;
use beavertuple::BeaverTuple;

//const WAN_LATENCY: u128 = 60u128;
//const LAN_LATENCY: u128 = 1u128;


fn setup(input_size:usize, input_bits:usize){
    let seed = PrgSeed::one();
    let mut stream = FixedKeyPrgStream::new();
    stream.set_key(&seed.key);

    //Offline-Step-1. Set IDPF Parameters
    let fix_betas = RingElm::from(1u32).to_vec(input_bits); //generate a series of 1 as beta
    let r_bits = stream.next_bits(input_bits*input_size);  

    //Offline-Step-2. Generate Random I-DPFs
    let mut dpf_0: Vec<DPFKey<RingElm>> = Vec::new();
    let mut dpf_1: Vec<DPFKey<RingElm>> = Vec::new();
    for i in 0..input_size{
        let alpha = &r_bits[i*input_bits..(i+1)*input_bits];
        let (k0, k1) = DPFKey::gen(&alpha, &fix_betas);

        dpf_0.push(k0);
        dpf_1.push(k1);
    }
    let mut f_k0 = File::create("../data/k0.bin").expect("create failed");
    let mut f_k1 = File::create("../data/k1.bin").expect("create failed");
    f_k0.write_all(&bincode::serialize(&dpf_0).expect("Serialize key error")).expect("Write key error.");
    f_k1.write_all(&bincode::serialize(&dpf_1).expect("Serialize key error")).expect("Write key error.");

    let r_bits_0 = stream.next_bits(input_bits*input_size);
    let r_bits_1 = bits_Xor(&r_bits, &r_bits_0);
    
    let mut f_a0 = File::create("../data/a0.bin").expect("create failed");
    let mut f_a1 = File::create("../data/a1.bin").expect("create failed");
    f_a0.write_all(&bincode::serialize(&r_bits_0).expect("Serialize alpha error")).expect("Write alpha error.");
    f_a1.write_all(&bincode::serialize(&r_bits_1).expect("Serialize alpha error")).expect("Write alpha error.");

    //Offline-Step-3. Random daBits for masking
    let q_boolean = stream.next_bits(input_bits);
    // println!("q_boolean is: {} ",vec_bool_to_string(&q_boolean));
    let q_boolean_0 = stream.next_bits(input_bits);
    let q_boolean_1 = bits_Xor(&q_boolean, &q_boolean_0);
    let mut f_qb0 = File::create("../data/qb0.bin").expect("create failed");
    let mut f_qb1 = File::create("../data/qb1.bin").expect("create failed");
    f_qb0.write_all(&bincode::serialize(&q_boolean_0).expect("Serialize q-bool-share error")).expect("Write q-bool-share error.");
    f_qb1.write_all(&bincode::serialize(&q_boolean_1).expect("Serialize q-bool-share error")).expect("Write q-bool-share error.");


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
    let mut f_qa0 = File::create("../data/qa0.bin").expect("create failed");
    let mut f_qa1 = File::create("../data/qa1.bin").expect("create failed");
    f_qa0.write_all(&bincode::serialize(&q_numeric_0).expect("Serialize q-a-share error")).expect("Write q-a-share error.");
    f_qa1.write_all(&bincode::serialize(&q_numeric_1).expect("Serialize q-a-share error")).expect("Write q-a-share error.");

    //Offline-Step-4. Random DPFs for zeroCheck, input_bits required in total
    let mut zero_dpf_0: Vec<DPFKey<BinElm>> = Vec::new();
    let mut zero_dpf_1: Vec<DPFKey<BinElm>> = Vec::new();

    let mut zero_dpf_r0: Vec<RingElm> = Vec::new();
    let mut zero_dpf_r1: Vec<RingElm> = Vec::new();
    
    for _ in 0..input_bits{
        let zero_r_bits = stream.next_bits(NUMERIC_LEN*2);

        let mut numeric_zero_r_1 = RingElm::from( bits_to_u32(&zero_r_bits[..NUMERIC_LEN]) );
        // let numeric_zero_r = RingElm::from( bits_to_u32(&zero_r_bits[..NUMERIC_LEN]) );

        // println!("numeric_zero_r={:?}", numeric_zero_r);
        // println!("Vec<bool>: {:?}", zero_r_bits[..NUMERIC_LEN].to_vec());

        let numeric_zero_r_0 = RingElm::from( bits_to_u32(&zero_r_bits[NUMERIC_LEN..]) );
        numeric_zero_r_1.sub(&numeric_zero_r_0);
       

        let zero_betas: Vec<BinElm> = BinElm::from(false).to_vec(NUMERIC_LEN);
        let (k0, k1) = DPFKey::gen(&zero_r_bits[..NUMERIC_LEN], &zero_betas);

        // let mut partial_data: Vec<bool> = zero_r_bits[..NUMERIC_LEN].to_vec();
        // let k0Clone = k0.clone();
        // let k1Clone = k1.clone();
        // println!("partial_data={:?}", partial_data);
        // // partial_data[3] ^= true;

        // let y_fnzc0: BinElm = k0Clone.eval(&partial_data);
        // println!("y_fnzc0={:?}", y_fnzc0);
        // let mut y_fnzc1: BinElm = k1Clone.eval(&partial_data);
        // println!("y_fnzc1={:?}", y_fnzc1);
        // y_fnzc1.add(&y_fnzc0);
        // println!("y_fnzc={:?}", y_fnzc1);

        zero_dpf_0.push(k0);
        zero_dpf_1.push(k1);

        zero_dpf_r0.push(numeric_zero_r_0);
        zero_dpf_r1.push(numeric_zero_r_1);
    }
    let mut f_zc_a0 = File::create("../data/zc_a0.bin").expect("create failed");
    let mut f_zc_a1 = File::create("../data/zc_a1.bin").expect("create failed");
    let mut f_zc_k0 = File::create("../data/zc_k0.bin").expect("create failed");
    let mut f_zc_k1 = File::create("../data/zc_k1.bin").expect("create failed");
    f_zc_a0.write_all(&bincode::serialize(&zero_dpf_r0).expect("Serialize zc-key-share error")).expect("Write zc-key-share error.");
    f_zc_a1.write_all(&bincode::serialize(&zero_dpf_r1).expect("Serialize zc-key-share error")).expect("Write zc-key-share error.");
    f_zc_k0.write_all(&bincode::serialize(&zero_dpf_0).expect("Serialize zc-key-share error")).expect("Write zc-key-share error.");
    f_zc_k1.write_all(&bincode::serialize(&zero_dpf_1).expect("Serialize zc-key-share error")).expect("Write zc-key-share error.");
    // println!("{:.5?} seconds for offline phase.", start.elapsed());

    let mut beavertuples0: Vec<BeaverTuple> = Vec::new();
    let mut beavertuples1: Vec<BeaverTuple> = Vec::new();
    for i in 0..input_bits*2{
        let rd_bits = stream.next_bits(NUMERIC_LEN*5);
        let a0 = RingElm::from( bits_to_u32(&rd_bits[..NUMERIC_LEN]) );
        let b0 = RingElm::from( bits_to_u32(&rd_bits[NUMERIC_LEN..2*NUMERIC_LEN]) );

        let a1 = RingElm::from( bits_to_u32(&rd_bits[2*NUMERIC_LEN..3*NUMERIC_LEN]) );
        let b1 = RingElm::from( bits_to_u32(&rd_bits[3*NUMERIC_LEN..4*NUMERIC_LEN]));

        let ab0 = RingElm::from( bits_to_u32(&rd_bits[4*NUMERIC_LEN..5*NUMERIC_LEN]) );

        let mut a = RingElm::from(0);
        a.add(&a0);
        a.add(&a1);

        let mut b = RingElm::from(0);
        b.add(&b0);
        b.add(&b1);

        let mut ab = RingElm::from(1);
        ab.mul(&a);
        ab.mul(&b);

        ab.sub(&ab0);

        let beaver0 = BeaverTuple{
            a: a0,
            b: b0,
            ab: ab0
        };

        let beaver1 = BeaverTuple{
            a: a1,
            b: b1,
            ab: ab
        };
        beavertuples0.push(beaver0);
        beavertuples1.push(beaver1);
    }
    let mut f_beaver0 = File::create("../data/beaver0.bin").expect("create failed");
    let mut f_beaver1 = File::create("../data/beaver1.bin").expect("create failed");
    f_beaver0.write_all(&bincode::serialize(&beavertuples0).expect("Serialize beaver0 error")).expect("Write beaver0 error.");
    f_beaver1.write_all(&bincode::serialize(&beavertuples1).expect("Serialize beaver1 error")).expect("Write beaver1 error.");

}

fn main()
{
    setup(3, 5);
    // setup(10, 32);
}

#[cfg(test)]
mod tests {
    use idpf::*;
    use idpf::prg::*;
    use idpf::dpf::*;
    use idpf::RingElm;
    use std::fs::File;
    use bincode::{serialize, deserialize};
    use std::io::prelude::*;
    use idpf::beavertuple::*;
    
    #[test]
    fn it_works() {
        const input_size: usize = 10;
        const input_bits: usize = 32;

        let seed = PrgSeed::random();
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);
        
        let r = stream.next_bits(input_bits*input_size);  
        let betas = RingElm::from(1u32).to_vec(input_bits); 
        
        let mut fw1 = File::create("../test/a.bin").expect("create failed");
        let mut fw2 = File::create("../test/b.bin").expect("create failed");
        fw1.write_all(&bincode::serialize(&r).expect("Serialize key error")).expect("Write key error.");
        fw2.write_all(&bincode::serialize(&betas).expect("Serialize key error")).expect("Write key error.");

        let mut fr1 = File::open("../test/a.bin").expect("open failed");
        
        let mut buf = Vec::<u8>::new();
        fr1.read_to_end(&mut buf).expect("Read error!");
        let r_v: Vec<bool> = bincode::deserialize(&mut buf).expect("Deserialize Error");

        let mut fr2 = File::open("../test/b.bin").expect("open failed");
        buf.clear();
        fr2.read_to_end(&mut buf).expect("Read error!");
        let b_v: Vec<RingElm> = bincode::deserialize(&mut buf).expect("Deserialize Error");

        assert_eq!(r.len(), r_v.len());
        assert_eq!(betas.len(), b_v.len());

        let mut succ = true;
        for i in 0..r.len(){
            //assert_eq!(r[i], r_v[i]);
            if r[i] != r_v[i]{
                succ = false;
                break;
            }
        }
        assert_eq!(succ, true);

        succ = true;
        for i in 0..betas.len(){
            //assert_eq!(r[i], r_v[i]);
            if betas[i] != b_v[i]{
                succ = false;
                break;
            }
        }
        assert_eq!(succ, true);
    }

    #[test]
    fn beavers_works(){
        const NUMERIC_LEN: usize = 32usize;
        let seed = PrgSeed::random();
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);
        let mut beavertuples0: Vec<BeaverTuple> = Vec::new();
        let mut beavertuples1: Vec<BeaverTuple> = Vec::new();

        for _ in 0..3{
            let rd_bits = stream.next_bits(NUMERIC_LEN*5);
            let a0 = RingElm::from( bits_to_u32(&rd_bits[..NUMERIC_LEN]) );
            let b0 = RingElm::from( bits_to_u32(&rd_bits[NUMERIC_LEN..2*NUMERIC_LEN]) );

            let a1 = RingElm::from( bits_to_u32(&rd_bits[2*NUMERIC_LEN..3*NUMERIC_LEN]) );
            let b1 = RingElm::from( bits_to_u32(&rd_bits[3*NUMERIC_LEN..4*NUMERIC_LEN]));

            let ab0 = RingElm::from( bits_to_u32(&rd_bits[4*NUMERIC_LEN..5*NUMERIC_LEN]) );

            let mut a = RingElm::from(0u32);
            a.add(&a0);
            a.add(&a1);

            let mut b = RingElm::from(0u32);
            b.add(&b0);
            b.add(&b1);
            
            let mut ab = RingElm::from(1u32);
            ab.mul(&a);
            ab.mul(&b);
            println!("g:a={:?}, b={:?}, ab={:?}", a, b, ab);
            let org_ab = ab.clone();
            ab.sub(&ab0);
            
            let beaver0 = BeaverTuple{
                a: a0,
                b: b0,
                ab: ab0
            };

            let beaver1 = BeaverTuple{
                a: a1,
                b: b1,
                ab: ab
            };
            let mut c = RingElm::from(0);
            c.add(&beaver0.ab);
            c.add(&beaver1.ab);
            assert_eq!(org_ab, c);
            beavertuples0.push(beaver0);
            beavertuples1.push(beaver1);
        }
        let mut fw0 = File::create("../test/beaver0.bin").expect("create failed");
        let mut fw1 = File::create("../test/beaver1.bin").expect("create failed");
        fw0.write_all(&bincode::serialize(&beavertuples0).expect("Serialize beaver0 error")).expect("Write beaver0 error.");
        fw1.write_all(&bincode::serialize(&beavertuples1).expect("Serialize beaver1 error")).expect("Write beaver1 error.");

        let mut fr1 = File::open("../test/beaver0.bin").expect("open failed");
        
        let mut buf = Vec::<u8>::new();
        fr1.read_to_end(&mut buf).expect("Read error!");
        let bv0: Vec<BeaverTuple> = bincode::deserialize(&mut buf).expect("Deserialize Error");

        let mut fr2 = File::open("../test/beaver1.bin").expect("open failed");
        buf.clear();
        fr2.read_to_end(&mut buf).expect("Read error!");
        let bv1: Vec<BeaverTuple> = bincode::deserialize(&mut buf).expect("Deserialize Error");

        assert_eq!(beavertuples0.len(), bv0.len());
        assert_eq!(beavertuples1.len(), bv1.len());

        for i in 0..bv0.len(){
            //assert_eq!(r[i], r_v[i]);
            assert_eq!(beavertuples0[i].a, bv0[i].a);
            assert_eq!(beavertuples0[i].b, bv0[i].b);
            assert_eq!(beavertuples0[i].ab, bv0[i].ab);
        }
        
        for i in 0..bv1.len(){
            //assert_eq!(r[i], r_v[i]);
            assert_eq!(beavertuples1[i].a, bv1[i].a);
            assert_eq!(beavertuples1[i].b, bv1[i].b);
            assert_eq!(beavertuples1[i].ab, bv1[i].ab);
        }
        
        assert_eq!(bv0.len(), bv1.len());
        for i in 0..bv0.len(){
            //assert_eq!(r[i], r_v[i]);
            let mut a = RingElm::from(0u32);
            a.add(&bv0[i].a);
            a.add(&bv1[i].a);

            let mut b = RingElm::from(0u32);
            b.add(&bv0[i].b);
            b.add(&bv1[i].b);

            let mut ab = RingElm::from(0u32);
            ab.add(&bv0[i].ab);
            ab.add(&bv1[i].ab);

            let mut c = RingElm::from(1u32);
            c.mul(&a);
            c.mul(&b);
            println!("r:a0={:?}, b0={:?}, ab0={:?}", bv0[i].a, bv0[i].b, bv0[i].ab);
            println!("r:a1={:?}, b1={:?}, ab1={:?}", bv1[i].a, bv1[i].b, bv1[i].ab);
            println!("r:a={:?}, b={:?}, ab={:?}, c={:?}", a, b, ab, c);
            assert_eq!(ab, c);
        }
    }
    
    

}


use libmpc::mpc_party::MPCParty;
use libmpc::protocols::bitwise_kre::*;
use libmpc::protocols::batch_max_proto::*;
use libmpc::mpc_platform::NetInterface;
use libmpc::offline_data::*;
use fss::{prg::*, RingElm};
use libmpc::offline_data::offline_batch_max::*;
use std::fs::File;
use std::io::Write;
use std::env;

pub const TEST_BITWISE_MAX: bool = false;
pub const TEST_BATCH_MAX: bool = false;
pub const TEST_BITWISE_KRE: bool = true;
pub const TEST_BATCH_KRE: bool = false;
// pub const TEST_SIMULATE_NETWORK: bool = false;
// pub const TEST_REAL_NETWORK: bool = false;

const INPUT_SIZE: usize = 10000usize;
const INPUT_BITS: usize = 32usize;
const BATCH_SIZE: usize = 4usize;

const K_GLOBAL: u32 = 1;

#[tokio::main]
async fn main() {
    let mut is_server=false;

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        // The first command-line argument (index 1) is accessed using args[1]
        let first_argument = args[1].parse::<u8>();

        // Check if the parsing was successful
        match first_argument {
            Ok(value) => {
                match value{
                    0 => is_server = true,
                    1 => {},
                    _ => eprintln!("Error: Party role illegale"),
                }
            }
            Err(_) => {
                eprintln!("Error: Unable to parse the first argument as a u8 value.");
            }
        }
    } else {
        eprintln!("No arguments provided.");
    }

    let seed = if is_server {PrgSeed::zero()} else {PrgSeed::one()};//Guarantee same input bits to ease the debug process
    let mut stream = FixedKeyPrgStream::new();
    stream.set_key(&seed.key);
    let x_share = stream.next_bits(INPUT_BITS*INPUT_SIZE);
    let index =  if is_server {String::from("0")} else {String::from("1")};

    let netlayer = NetInterface::new(is_server, "127.0.0.1:8088").await;

    // let mut offlinedata = BitMaxOffline::new(if is_server{0u8} else {1u8});
    //let mut offlinedata: BitKreOffline = BitKreOffline::new();
    let mut offlinedata = BatchMaxOffline::new();
    offlinedata.loadData(if is_server{&0u8} else {&1u8});

    //let mut p: MPCParty<BitKreOffline> = MPCParty::new(offlinedata, netlayer);
    let mut p: MPCParty<BatchMaxOffline> = MPCParty::<BatchMaxOffline>::new(offlinedata, netlayer);
    p.setup(INPUT_SIZE, INPUT_BITS);

    // let result = bitwise_max(&mut p, &x_share).await;
    //let kValue = RingElm::from(if is_server{0u32} else {2u32});
    //let kValue = RingElm::from(if is_server{0u32} else {K_GLOBAL});
    //let result = bitwise_kre(&mut p, &x_share, &kValue).await;
    let result = batch_max(&mut p, &x_share, BATCH_SIZE).await;
    // for i in 0..INPUT_SIZE{
    //     print!("x_share[{}]=", i);
    //     for j in 0..INPUT_BITS{
    //         if x_share[i*INPUT_BITS+j] == true{
    //             print!("1");
    //         }
    //         else {
    //             print!("0");
    //         }
    //     }
    //     println!("");
    // }
    // print!("cmp_share =");       
    // for i in 0..result.len(){           
    //     if result[i] == true{
    //         print!("1");
    //     }
    //     else {
    //         print!("0");
    //     }
    // }
    // println!(" ");
    let mut f_x = File::create(format!( "../test/x{}.bin", &index)).expect("create failed");
    let mut f_cmp = File::create(format!( "../test/cmp{}.bin", &index)).expect("create failed");
    f_x.write_all(&bincode::serialize(&x_share).expect("Serialize x-bool-share error")).expect("Write x-bool-share error.");
    f_cmp.write_all(&bincode::serialize(&result).expect("Serialize cmp-bool-share error")).expect("Write cmp-bool-share error.");

}

#[cfg(test)]
mod test
{
    use std::fs::File;
    use bincode::deserialize;
    use fss::{ RingElm, Group};
    use std::io::Read;
    use libmpc::offline_data::*;
    use libmpc::offline_data::offline_batch_max::*;
    use libmpc::offline_data::offline_bitwise_kre::*;
    use fss::prg::*;
    use crate::{INPUT_SIZE,INPUT_BITS,K_GLOBAL,BATCH_SIZE};
    use std::slice;
    
    #[tokio::test]
    async fn kre_works(){
        let k_global: usize = K_GLOBAL as usize;

        fn find_k_ranked_max<T: Ord + Clone>(slice: &mut [T], k: usize) -> T {
            let index = k.saturating_sub(1);
            slice.select_nth_unstable_by(index, |a, b| b.cmp(a));
            slice[index].clone()
        }

        let mut x0 = Vec::<bool>::new();
        let mut x1 = Vec::<bool>::new();
        let mut c0 = Vec::<bool>::new();
        let mut c1 = Vec::<bool>::new();
        let mut buf = Vec::<u8>::new();
        
        /*Read x_share[0] */
        let mut f_x0 = File::open("../test/x0.bin").expect("Open file failed");
        f_x0.read_to_end(&mut buf).expect("Read file error!");
        x0 = bincode::deserialize(&buf).expect("Deserialize key-share Error");

        /*Read x_share[1] */
        buf.clear();
        let mut f_x1 = File::open("../test/x1.bin").expect("Open file failed");
        f_x1.read_to_end(&mut buf).expect("Read file error!");
        x1 = bincode::deserialize(&buf).expect("Deserialize key-share Error");

        /*Read cmp[0] */
        buf.clear();
        let mut f_c0 = File::open("../test/cmp0.bin").expect("Open file failed");
        f_c0.read_to_end(&mut buf).expect("Read file error!");
        c0 = bincode::deserialize(&buf).expect("Deserialize key-share Error");

         /*Read cmp[1] */
         buf.clear();
         let mut f_c1 = File::open("../test/cmp1.bin").expect("Open file failed");
         f_c1.read_to_end(&mut buf).expect("Read file error!");
         c1 = bincode::deserialize(&buf).expect("Deserialize key-share Error");

        assert_eq!(x0.len(), x1.len());
        assert_eq!(c0.len(), INPUT_BITS);
        assert_eq!(c1.len(), INPUT_BITS);

        let bv2uint = |b: Vec<bool>|{
            let mut v: u32 = 0;
            
            for e in b.iter()
            {
                v = v << 1;
                if *e {
                    v += 1;
                }
            }
            v
        };
        
        let mut x = x0;

        for i in 0..x.len(){
            x[i] = x[i] ^ x1[i];
        }   //reconstruct the x = x0^x1

        let mut v = Vec::<u32>::new();
        for i in 0..INPUT_SIZE{
            let e = bv2uint(x[i*INPUT_BITS..(i+1)*INPUT_BITS].to_vec());
            v.push(e);
        } // convert x-s to u32-s

        let kre = find_k_ranked_max(&mut v, k_global);

        let mut c = c0;
        for i in 0..c.len(){
            c[i] = c[i] ^ c1[i];
        } 
        let r = bv2uint(c);
        println!("kre={:?}", r);
        assert_eq!(kre, r);
    }

    #[test]
    fn batch_kre_gen_offlinedata(){
        let input_size = INPUT_SIZE;
        let input_bits = INPUT_BITS;
        let batch_size = BATCH_SIZE;
        let every_batch_num = 1 << batch_size;
        let offline = BatchMaxOffline::new();
        offline.genData(&PrgSeed::zero(), input_size, input_bits, batch_size, every_batch_num * every_batch_num);
    }

}

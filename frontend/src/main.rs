use libmpc::mpc_party::MPCParty;
use libmpc::protocols::bitwise_max::*;
use libmpc::protocols::bitwise_kre::*;
use libmpc::protocols::batch_max_proto::*;
use libmpc::protocols::batch_kre_proto::*;
use libmpc::protocols::max_ic_proto::*;
use libmpc::mpc_platform::NetInterface;

use fss::{prg::*, RingElm};
use libmpc::offline_data::offline_bitwise_max::*;
use libmpc::offline_data::offline_bitwise_kre::*;
use libmpc::offline_data::offline_batch_max::*;
use libmpc::offline_data::offline_batch_kre::*;
use libmpc::offline_data::offline_ic_max::*;

use std::fs::File;
use std::io::Write;
use std::env;
use rand::Rng;
use std::time::Instant;
use std::thread::sleep;
use std::time::Duration;


const LAN_ADDRESS: &'static str = "127.0.0.1:8088";
const WAN_ADDRESS: &'static str = "45.63.6.86:8088";

#[derive(PartialEq,PartialOrd)]
pub enum TEST_OPTIONS{
    BITWISE_MAX = 1,
    BATCH_MAX= 2,
    BITWISE_KRE= 3,
    BATCH_KRE= 4,
    TRIVAL_FSS_MAX= 5,
    TRIVAL_FSS_KRE= 6
}

pub const M_TEST_CHOICE: TEST_OPTIONS = TEST_OPTIONS::BITWISE_MAX;
pub const TEST_WAN_NETWORK: bool = true;

//n: input domain length
const INPUT_BITS: usize = 30usize;
const BATCH_SIZE: usize = 3usize;
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

    //m: set pre-defined size
    let INPUT_PARAMETERS:Vec<usize> = vec![100,1000,10000,100000,1000000];
    // let INPUT_PARAMETERS:Vec<usize> = vec![500000];
    for i in 0..INPUT_PARAMETERS.len(){
        let input_size = INPUT_PARAMETERS[i];
        gen_offlinedata(input_size);
        let seed = if is_server {PrgSeed::zero()} else {PrgSeed::one()};//Guarantee same input bits to ease the debug process
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);
        let x_share = stream.next_bits(INPUT_BITS*input_size);
        let index =  if is_server {String::from("0")} else {String::from("1")};
        let index_ID = if is_server{0u8} else {1u8};

        let mut f_x = File::create(format!( "../test/x{}.bin", &index)).expect("create failed");
        f_x.write_all(&bincode::serialize(&x_share).expect("Serialize x-bool-share error")).expect("Write x-bool-share error.");

        let mut result = vec![false;INPUT_BITS];

        let mut netlayer = NetInterface::new(is_server, if TEST_WAN_NETWORK{WAN_ADDRESS}else{LAN_ADDRESS}).await;

        if M_TEST_CHOICE<=TEST_OPTIONS::BATCH_KRE{
            if M_TEST_CHOICE == TEST_OPTIONS::BITWISE_MAX{
                    let mut offlinedata = BitMaxOffline::new();
                    offlinedata.loadData(&index_ID);
                    netlayer.reset_timer().await;
                    let mut p: MPCParty<BitMaxOffline> = MPCParty::new(offlinedata, netlayer);
                    p.setup(input_size, INPUT_BITS);
                    result = bitwise_max(&mut p, &x_share).await;
            }else if M_TEST_CHOICE == TEST_OPTIONS::BATCH_MAX{
                let mut offlinedata = BatchMaxOffline::new();
                    offlinedata.loadData(&index_ID);
                    netlayer.reset_timer().await;
                    let mut p: MPCParty<BatchMaxOffline> = MPCParty::new(offlinedata, netlayer);
                    p.setup(input_size, INPUT_BITS);
                    result = batch_max(&mut p, &x_share, BATCH_SIZE).await;
            }else if M_TEST_CHOICE == TEST_OPTIONS::BITWISE_KRE{
                    let mut offlinedata: BitKreOffline = BitKreOffline::new();
                    offlinedata.loadData(&index_ID);
                    netlayer.reset_timer().await;
                    let mut p: MPCParty<BitKreOffline> = MPCParty::new(offlinedata, netlayer);
                    p.setup(input_size, INPUT_BITS);

                    let kValue = RingElm::from(if is_server{0u32} else {K_GLOBAL});
                    result = bitwise_kre(&mut p, &x_share, &kValue).await;
            }else if M_TEST_CHOICE == TEST_OPTIONS::BATCH_KRE{
                let mut offlinedata = BatchKreOffline::new();
                offlinedata.loadData(&index_ID);
                netlayer.reset_timer().await;
                let mut p: MPCParty<BatchKreOffline> = MPCParty::new(offlinedata, netlayer);
                p.setup(input_size, INPUT_BITS);
                let kValue = RingElm::from(if is_server{0u32} else {K_GLOBAL});
                result = batch_kre(&mut p, &x_share, BATCH_SIZE,&kValue).await;
            }
            let mut f_cmp = File::create(format!( "../test/cmp{}.bin", &index)).expect("create failed");
            f_cmp.write_all(&bincode::serialize(&result).expect("Serialize cmp-bool-share error")).expect("Write cmp-bool-share error.");
        }else{
            let mut rng = rand::thread_rng();
            let mut xx_share = Vec::<RingElm>::new();
            for _i in 0..input_size{
                let r = rng.gen_range(1..50) as u32;
                xx_share.push(RingElm::from(r));
            }
            if M_TEST_CHOICE == TEST_OPTIONS::TRIVAL_FSS_MAX{
                    let mut offlinedata = MaxOffline_IC::new();
                    offlinedata.loadData(&index_ID, false); // if max, false
                    netlayer.reset_timer().await;
                    let mut p = MPCParty::<MaxOffline_IC>::new(offlinedata, netlayer);
                    p.setup(input_size, INPUT_BITS);
                    let _ringele_result = max_ic(&mut p, &xx_share).await;
            }
            else if M_TEST_CHOICE == TEST_OPTIONS::TRIVAL_FSS_KRE{
                let mut offlinedata = MaxOffline_IC::new();
                offlinedata.loadData(&index_ID, true); // if kmax, true
                netlayer.reset_timer().await;
                let mut p = MPCParty::<MaxOffline_IC>::new(offlinedata, netlayer);
                p.setup(input_size, INPUT_BITS);
                let kValue = RingElm::from(if is_server{0u32} else {K_GLOBAL});
                heap_sort(&mut p, &mut xx_share).await;
                let _ringele_result = extract_kmax(&mut p, &xx_share, kValue).await;
            }
        }

        if !is_server{
            if i==3{//the second last one
                sleep(Duration::from_secs(15));
            }else{
                sleep(Duration::from_secs(5));
            }
        }
    }
}

fn gen_offlinedata(input_size:usize){
    let offline_timer = Instant::now();
    match M_TEST_CHOICE{
        TEST_OPTIONS::BITWISE_MAX => {
            let offline = BitMaxOffline::new();
            offline.genData(&PrgSeed::zero(), input_size, INPUT_BITS);
        },
        TEST_OPTIONS::BATCH_MAX => {
            let every_batch_num = 1 << BATCH_SIZE;
            let offline = BatchMaxOffline::new();
            offline.genData(&PrgSeed::zero(), input_size, INPUT_BITS, BATCH_SIZE, every_batch_num * every_batch_num);
        },
        TEST_OPTIONS::BITWISE_KRE => {
            let offline = BitKreOffline::new();
            offline.genData(&PrgSeed::zero(), input_size, INPUT_BITS);
        },
        TEST_OPTIONS::BATCH_KRE => {
            let offline = BatchKreOffline::new();
            offline.genData(&PrgSeed::zero(), input_size, INPUT_BITS, BATCH_SIZE);
        },
        TEST_OPTIONS::TRIVAL_FSS_MAX => {
            let seed = PrgSeed::zero();//Guarantee same input bits to ease the debug process
            let mut stream = FixedKeyPrgStream::new();
            stream.set_key(&seed.key);
            MaxOffline_IC::genData(&mut stream, input_size*(input_size - 1) / 2 , input_size * (input_size-1) / 2 + 2 * input_size, input_size);
        },
        TEST_OPTIONS::TRIVAL_FSS_KRE => {
            let offline = BatchKreOffline::new();
            offline.genData(&PrgSeed::zero(), input_size, INPUT_BITS, BATCH_SIZE);
        },
    }
    println!("Offline key generation time:{:?}",offline_timer.elapsed());
}

#[cfg(test)]
mod test
{
    use std::fs::File;
    
    use fss::{ Group};
    use std::io::Read;
    
    use libmpc::offline_data::offline_bitwise_max::*;
    use libmpc::offline_data::offline_bitwise_kre::*;
    use libmpc::offline_data::offline_batch_max::*;
    use libmpc::offline_data::offline_batch_kre::*;
    use libmpc::offline_data::offline_ic_max::MaxOffline_IC;
    use fss::prg::*;
    use crate::{INPUT_SIZE,INPUT_BITS,K_GLOBAL,BATCH_SIZE,M_TEST_CHOICE};
    
    use crate::TEST_OPTIONS;
    use std::time::Instant;
    

    // #[test]
    // fn gen_offlinedata(){
    //     let offline_timer = Instant::now();
    //     match M_TEST_CHOICE{
    //         TEST_OPTIONS::BITWISE_MAX => {
    //             let offline = BitMaxOffline::new();
    //             offline.genData(&PrgSeed::zero(), INPUT_SIZE, INPUT_BITS);
    //         },
    //         TEST_OPTIONS::BATCH_MAX => {
    //             let every_batch_num = 1 << BATCH_SIZE;
    //             let offline = BatchMaxOffline::new();
    //             offline.genData(&PrgSeed::zero(), INPUT_SIZE, INPUT_BITS, BATCH_SIZE, every_batch_num * every_batch_num);
    //         },
    //         TEST_OPTIONS::BITWISE_KRE => {
    //             let offline = BitKreOffline::new();
    //             offline.genData(&PrgSeed::zero(), INPUT_SIZE, INPUT_BITS);
    //         },
    //         TEST_OPTIONS::BATCH_KRE => {
    //             let offline = BatchKreOffline::new();
    //             offline.genData(&PrgSeed::zero(), INPUT_SIZE, INPUT_BITS, BATCH_SIZE);
    //         },
    //         TEST_OPTIONS::TRIVAL_FSS_MAX => {
    //             let seed = PrgSeed::zero();//Guarantee same input bits to ease the debug process
    //             let mut stream = FixedKeyPrgStream::new();
    //             stream.set_key(&seed.key);
    //             MaxOffline_IC::genData(&mut stream, INPUT_SIZE*(INPUT_SIZE - 1) / 2 , INPUT_SIZE * (INPUT_SIZE-1) / 2 + 2 * INPUT_SIZE, INPUT_SIZE);
    //         },
    //         TEST_OPTIONS::TRIVAL_FSS_KRE => {
    //             let offline = BatchKreOffline::new();
    //             offline.genData(&PrgSeed::zero(), INPUT_SIZE, INPUT_BITS, BATCH_SIZE);
    //         },
    //     }
    //     println!("Offline key generation time:{:?}",offline_timer.elapsed());
    // }

    #[test]
    fn test_results(){
        fn max_works(){
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

            let x_max = v.iter().max().unwrap();

            let mut c = c0;
            for i in 0..c.len(){
                c[i] = c[i] ^ c1[i];
            } 
            let r = bv2uint(c);
            println!("max={:?}", r);
            assert_eq!(*x_max, r);
        }

        fn kre_works(){
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

        match M_TEST_CHOICE{
            TEST_OPTIONS::BITWISE_MAX => {
                max_works();
            },
            TEST_OPTIONS::BATCH_MAX => {
                max_works();
            },
            TEST_OPTIONS::BITWISE_KRE => {
                kre_works();
            },
            TEST_OPTIONS::BATCH_KRE => {
                kre_works();
            },
            TEST_OPTIONS::TRIVAL_FSS_MAX => {
                max_works();
            },
            TEST_OPTIONS::TRIVAL_FSS_KRE => {
                kre_works();
            },
        }
    }
}

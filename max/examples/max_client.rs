use libmpc::mpc_party::{FileConfig, OfflineInfomation, MPCParty, max};
use libmpc::mpc_platform::NetInterface;
use fss::prg::*;
use fss::*;
use std::fs::File;
use std::io::Write;
use bincode;
extern crate tokio;
//use tokio;
//static mut p: MPCParty = MPCParty::new(OfflineInfomation::new(), PartyRole::Active);
//static mut x_share: Vec<bool> = Vec::new();
#[tokio::main]
async fn main(){
    let seed = PrgSeed::one();
    let mut stream = FixedKeyPrgStream::new();
    stream.set_key(&seed.key);

    // let _ = stream.next_bits(INPUT_BITS*INPUT_SIZE);
    // let _ = stream.next_bits(INPUT_BITS*INPUT_SIZE);
    let x_share = stream.next_bits(INPUT_BITS*INPUT_SIZE);
    let config = FileConfig{
        dir_path: "../data",
        a_file: "a1.bin",
        k_file: "k1.bin",
        qa_file: "qa1.bin",
        qb_file: "qb1.bin",
        zc_a_file: "zc_a1.bin",
        zc_k_file: "zc_k1.bin",
        beavers_file: "beaver1.bin"
    };
    let netlayer = NetInterface::new(false, "127.0.0.1:8088").await;
    let offlinedata = OfflineInfomation::new();
    let mut p = MPCParty::new(offlinedata, netlayer);
    p.setup(&config, INPUT_SIZE, INPUT_BITS);
    let result = max(&mut p, &x_share).await;

    for i in 0..INPUT_SIZE{
        print!("x_share[{}]=", i);
        for j in 0..INPUT_BITS{
            if x_share[i*INPUT_BITS+j] == true{
                print!("1");
            }
            else {
                print!("0");
            }
        }
        println!("");
    }
    print!("cmp_share =");       
    for i in 0..result.len(){           
        if result[i] == true{
            print!("1");
        }
        else {
            print!("0");
        }
    }
    println!(" ");
    
    let mut f_x = File::create("../test/x1.bin").expect("create failed");
    let mut f_cmp = File::create("../test/cmp1.bin").expect("create failed");
    f_x.write_all(&bincode::serialize(&x_share).expect("Serialize x-bool-share error")).expect("Write x-bool-share error.");
    f_cmp.write_all(&bincode::serialize(&result).expect("Serialize cmp-bool-share error")).expect("Write cmp-bool-share error.");
}

/*#[tokio::main]
async fn main(){
    let mut s = NetInterface::new(false, "127.0.0.1:8888").await;
    let mut e = Vec::<RingElm>::new();
    e.push(RingElm::from(13));
    e.push(RingElm::from(14));
    e.push(RingElm::from(15));
    e.push(RingElm::from(16));
    println!("x_share={:?}", e);
    let r = s.exchange_ring_vec(e).await;
    println!("{:?}", r);
}*/

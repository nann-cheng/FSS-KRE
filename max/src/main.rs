/* We assume two mpc parties, one playing as a tpc server, while the other plays as the client. */

use libmpc::mpc_party::{FileConfig, OfflineInfomation, MPCParty, max};
use libmpc::mpc_platform::NetInterface;
use idpf::prg::*;
use idpf::*;

//static mut p: MPCParty = MPCParty::new(OfflineInfomation::new(), PartyRole::Active);
//static mut x_share: Vec<bool> = Vec::new();
#[tokio::main]
async fn main(){
    let seed = PrgSeed::zero();
    let mut stream = FixedKeyPrgStream::new();
    stream.set_key(&seed.key);

    let x_share = stream.next_bits(INPUT_BITS*INPUT_SIZE);
    let config = FileConfig{
        dir_path: "../data",
        a_file: "a0.bin",
        k_file: "k0.bin",
        qa_file: "qa0.bin",
        qb_file: "qb0.bin",
        zc_a_file: "zc_a0.bin",
        zc_k_file: "zc_k0.bin",
        beavers_file: "beaver0.bin"
    };
    let netlayer = NetInterface::new(true, "127.0.0.1:8088").await;
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
}




#[cfg(test)]
mod test
{
    use std::fs::File;
    use bincode::deserialize;
    use idpf::{INPUT_BITS, INPUT_SIZE, RingElm, Group};
    use std::io::Read;
    #[tokio::test]
    async fn max_works(){
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
}
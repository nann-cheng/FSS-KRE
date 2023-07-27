/* We assume a mpc party stands at the point of a tpc server is running and the other one stands at the point that a client which intends to connect the server. */
/* It means the server has a mpc party in it and the client has the other mpc part in it. The relationship between server/client and mpc party is association.*/


use libmpc::mpc_platform::MPCServer;


//static mut p: MPCParty = MPCParty::new(OfflineInfomation::new(), PartyRole::Active);
//static mut x_share: Vec<bool> = Vec::new();
#[tokio::main]
async fn main(){
    let mut p = MPCServer::new();
    let _ = p.start().await;
}

#[cfg(test)]
mod test
{
    use std::fs::File;
    use bincode::deserialize;
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
        assert_eq!(c0.len(), 32);
        assert_eq!(c1.len(), 32);

        let r_len = x0.len() / 32;
        let mut r = Vec::<bool>::new();
        for i in 0..32{
            let mut x = false;
            for j in 0..r_len{
                x |= x0[32*j+i] ^ x1[32*j+i];
            }
            r.push(x);
        } // check whether the i-bit of x-s has true;  
        for i in 0..32{
            println!("{}", i);
            assert_eq!(r[i], c0[i] ^ c1[i]);
        }

    }
}
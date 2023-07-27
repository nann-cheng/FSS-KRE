use std::io::Error;

//use std::Arc::Arc;
//use std::io::prelude::*;
//use std::net::*;
//use std::time::Duration;
use super::mpc_party::*;
//use super::channelmessage::*;
//use futures::executor::block_on;
use idpf::*;
use idpf::prg::*;
//use idpf::RingElm;
use tokio::{
    io::{AsyncWriteExt, AsyncReadExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
    //task,
    sync::mpsc
};


pub struct MPCServer{
    s: Option<TcpListener>,
}

impl MPCServer{
    pub fn new() -> Self{
        MPCServer{s: None}
    }

    pub async fn start(&mut self)->Result<(), Error>{
        let seed = PrgSeed::random();
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

        let offlinedata = OfflineInfomation::new();
        let mut p = MPCParty::new(offlinedata, PartyRole::Active);
        p.setup(&config, INPUT_SIZE, INPUT_BITS);
        let listner = TcpListener::bind("127.0.0.1:8888").await.unwrap();
        self.s = Some(listner);
        println!("Listening...");
        let (c, _addr) = self.s.as_ref().unwrap().accept().await.unwrap();
        let (r, w) = c.into_split();

        let result = max(&p, &x_share, r, w).await;
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
        println!("Result = {:?}", result);
        Result::Ok(())
    }
}

pub struct MPCClient{
    //s: Option<TcpStream>
}

impl MPCClient{
    pub fn new() -> Self{
        MPCClient{}
    }

    pub async fn start(&mut self)->Result<(), Error>{
        let seed = PrgSeed::random();
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);
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

        let mut offlinedata = OfflineInfomation::new();
        let mut p = MPCParty::new(offlinedata, PartyRole::Passitive);
        p.setup(&config, INPUT_SIZE, INPUT_BITS);
        let s = TcpStream::connect("127.0.0.1:8888").await?;
        let (r, w) = s.into_split();
       
        let result = max(&p, &x_share, r, w).await;
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
        println!("Result = {:?}", result);
        Result::Ok(())
    }
}

/*async fn read_from_partner(reader: OwnedReadHalf, msg_tx: &mpsc::Sender<Vec<u8>>){
    let mut buf_reader = tokio::io::BufReader::new(reader);
    let mut buf= [0; 1024];
    //let mut buf = Vec::<u8>::new();
    loop {
        match  buf_reader.read(&mut buf[0..10]).await{
            Err(e) => {
                eprintln!("read from client error: {}", e);
                break;
            }
            // 遇到了EOF
            Ok(0) => {
                println!("client closed");
                break;
            }
            Ok(n) => {
                println!("Receive {} bytes from network.", n);
                if msg_tx.send(buf[0..n].to_vec()).await.is_err() {
                    eprintln!("receiver closed");
                    break;
                }
            }
        }     
    }
}

/// 写给客户端
async fn write_to_partner(mut writer: OwnedWriteHalf, msg_rx:&mut mpsc::Receiver<Vec<u8>>) {
    //let mut buf_writer = tokio::io::BufWriter::new(writer);
    
    while let Some(e) = msg_rx.recv().await {
        println!("Write {} to network",e.len());
        if let Err(err) = writer.write_all(e.as_slice()).await {
            eprintln!("write to client failed: {}", err);
            break;
        }
    }
}*/

use tokio::{
    io::{AsyncWriteExt, AsyncReadExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
    //task;
};
use idpf::RingElm;
use idpf::Group;
//use async_trait::async_trait;



pub struct NetInterface{
    //pub listener: TcpListener,
    pub is_server: bool,
    pub reader: OwnedReadHalf,
    pub writer: OwnedWriteHalf
}




impl NetInterface{
    pub async fn new(isserver: bool, addr: &str)->NetInterface{
        if isserver{
            let listner = TcpListener::bind(addr).await.unwrap();
            println!("***Start Listening ......***");
            let (c, caddr) = listner.accept().await.unwrap();
            println!("Accept from {:?}", caddr);
            let (r, w) = c.into_split();

            NetInterface{
                is_server: true,
                reader: r,
                writer: w
            }
        }
        else{
            let s = TcpStream::connect(addr).await.unwrap();
            let (r, w) = s.into_split();
            println!("Connect to {} success.", addr);
            NetInterface{
                is_server: false,
                reader: r,
                writer: w
            }
        }
    }

    pub async fn exchange_a_bool(&mut self, msg: bool)->bool{
        let mut buf: [u8; 1] = [0; 1];
        
        let mut x_msg: Vec<u8> = Vec::new();
        if msg{
            x_msg.push(0x1u8);
        } // convert the bool vec to u8 vec such that the message can be convoyed in the channel
        else{
            x_msg.push(0x0u8);
        }
        let xmsg_len = 1;
        
        if let Err(err) = self.writer.write_all(&x_msg.as_slice()).await{
            eprintln!("Write to partner failed:{}", err);
            std::process::exit(-1);
        }
        else{
            // println!("Write to partner {} bytes.", xmsg_len);
        } // send message to the partner

        match  self.reader.read_exact(&mut buf[0..xmsg_len]).await{
            Err(e) => {
                eprintln!("read from client error: {}", e);
                std::process::exit(-1);
            }
            Ok(0) => {
                println!("client closed.");
                std::process::exit(-1);
            }  
            Ok(n) => {
                assert_eq!(n, xmsg_len);
                // println!("Receive {} bytes from partner.", n);
            }        
        }     
        let mut r = msg; //save the msg
        if buf[0] == 1{
            r = !r;
        }
        r
    }

    pub async fn exchange_bool_vec(&mut self, msg: Vec<bool>)->Vec<bool>{
        let mut buf: [u8; 1024] = [0; 1024];
        
        let x_msg: Vec<u8> = msg.iter().map(|x| if *x == true {1} else {0}).collect(); // convert the bool vec to u8 vec such that the message can be convoyed in the channel
        let xmsg_len = x_msg.len();
        
        if let Err(err) = self.writer.write_all(&x_msg.as_slice()).await{
            eprintln!("Write to partner failed:{}", err);
            std::process::exit(-1);
        }
        else{
            // println!("Write to partner {} bytes.", xmsg_len);
        } // send message to the partner

        match  self.reader.read_exact(&mut buf[0..xmsg_len]).await{
            Err(e) => {
                eprintln!("read from client error: {}", e);
                std::process::exit(-1);
            }
            Ok(0) => {
                println!("client closed.");
                std::process::exit(-1);
            }  
            Ok(n) => {
                assert_eq!(n, xmsg_len);
                // println!("Receive {} bytes from partner.", n);
            }        
        }     
        let mut r = msg; //save the msg
        for i in 0..xmsg_len{
            if buf[i] == 1{
                r[i] = !r[i];
            }
        }
        r
    }

    pub async fn exchange_ring_vec(&mut self, msg: Vec<RingElm>) -> Vec<RingElm>{
        let mut buf: [u8; 1024] = [0; 1024];
        let mut x_msg: Vec<u8> = Vec::<u8>::new();
        for e in &msg{
            x_msg.append(&mut e.to_u32().unwrap().to_be_bytes().to_vec());
        }//convert u32 stream to u8 stream

        let xmsg_len = x_msg.len();
        if let Err(err) = self.writer.write_all(&x_msg.as_slice()).await{
            eprintln!("Write to partner failed:{}", err);
            std::process::exit(-1);
        }
        else{
            // println!("Write to partner {} bytes.", xmsg_len);
        } // send message to the partner

        match  self.reader.read_exact(&mut buf[0..xmsg_len]).await{
            Err(e) => {
                eprintln!("read from client error: {}", e);
                std::process::exit(-1);
            }
            Ok(0) => {
                println!("client closed.");
                std::process::exit(-1);
            }     
            Ok(n) => {
                assert_eq!(n, xmsg_len);
                // println!("Receive {} bytes from partner.", n);
            }        
        }

        let mut r: Vec<RingElm> = msg;
        for i in 0..xmsg_len/4{
            let mut ybuf: [u8; 4]= [0; 4];
            for j in 0..4{
                ybuf[j] = buf[i*4+j];
            }
            let e = RingElm::from(u32::from_be_bytes(ybuf));
            r[i].add(&e);
        }
       
        r     
    }
}

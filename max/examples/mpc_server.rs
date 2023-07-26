use std::{io::Error};
//use std::Arc::Arc;
//use std::io::prelude::*;
//use std::net::*;
//use std::time::Duration;
use std::fmt;
use futures::executor::block_on;
use tokio::{
    io::{AsyncWriteExt, AsyncReadExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
    task,
    sync::mpsc
};
struct MPCServer{
    s: Option<TcpListener>,
}

enum ChannelMessage{
    BoolData(bool),
    RingData(u32)
}
impl fmt::Display for ChannelMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChannelMessage::BoolData(b) => write!(f, "{}", b),
            ChannelMessage::RingData(value) => write!(f, "{}", value)
        }
    }
}
impl ChannelMessage{
    fn to_bytes(&self) -> Vec<u8>{
        match *self{
            ChannelMessage::BoolData(false) => {vec![0]},
            ChannelMessage::BoolData(true) => {vec![1]},
            ChannelMessage::RingData(v) => {v.to_be_bytes().to_vec()}
        }
    }
}

impl MPCServer{
    fn new() -> Self{
        MPCServer{s: None}
    }

    async fn start(&mut self)->Result<(), Error>{
        let listner = TcpListener::bind("127.0.0.1:8888").await.unwrap();
        self.s = Some(listner);
        println!("Listening...");
        let (c, _addr) = self.s.as_ref().unwrap().accept().await.unwrap();
        let (r, w) = c.into_split();
        let channel_x = mpsc::channel::<ChannelMessage>(100); //The channel between main job and the writer
        let channel_y = mpsc::channel::<ChannelMessage>(100); //The channel between main job and the reader
        let msg_tx = channel_x.0;
        let mut msg_rx = channel_x.1;
        let msg_ty = channel_y.0;
        let mut msg_ry = channel_y.1;

        let mut read_task = tokio::spawn(async move {
            read_from_partner(r,  &msg_tx).await;
        });
    
        let mut write_task = tokio::spawn(async move {
            write_to_partner(w, &mut msg_ry).await;
        });
    
        let mut main_task = tokio::spawn(async move {
         f_reveal(&msg_ty, &mut msg_rx).await;
        });

        if tokio::try_join!(&mut read_task, &mut write_task).is_err() {
            eprintln!("read_task/write_task terminated");
            main_task.abort();
            read_task.abort();
            write_task.abort();
        };
        Result::Ok(())
    }
}

async fn f_reveal(msg_ty: &mpsc::Sender<ChannelMessage>, msg_rx: &mut mpsc::Receiver<ChannelMessage>) {
    for i in 0..10{
        let e = msg_rx.recv().await.unwrap();
        let m_share = ChannelMessage::BoolData(true);
        if msg_ty.send(m_share).await.is_err() {
            eprintln!("receiver closed");
        }
        else{
            println!("Job->writer: {}", true);
        }
        
        println!("The {}th bool construction.", i);
    }
    
    
    for i in 0..5{
        let m_share = ChannelMessage::RingData(i);
        if msg_ty.send(m_share).await.is_err() {
            eprintln!("receiver closed");
        }
        else{
            println!("Job->writer: {}", i);
        }
        let e = msg_rx.recv().await.unwrap();
        println!("The {}th u32 construction.", i);
    }
    //tokio::time::sleep(Duration::from_secs(2)).await;
    /*if let Some(e) = msg_rx.recv().await{
        println!("Received from the channel:{}", e);
        let con = m_share + e;
        Ok(con)
    }
    else {
        Err("Construct error!")
    }*/
}
   
async fn read_from_partner(reader: OwnedReadHalf, msg_tx: &mpsc::Sender<ChannelMessage>){
    let mut buf_reader = tokio::io::BufReader::new(reader);
    let mut buf= [0; 4];
    loop {
        match  buf_reader.read(&mut buf).await{
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
                if n == 4{
                    let content = u32::from_be_bytes(buf);
                    println!("read {} bytes from client. content: {}", 4, content);
                    // 将内容发送给writer，让writer响应给客户端，
                    // 如果无法发送给writer，继续从客户端读取内容将没有意义，因此break退出
                    let channel_msg = ChannelMessage::RingData(content);
                    if msg_tx.send(channel_msg).await.is_err() {
                        eprintln!("receiver closed");
                        break;
                    }
                }
                else if n == 1{
                    let content = buf[0] != 0;
                    println!("read {} bytes from client. content: {}", 1, content);
                    // 将内容发送给writer，让writer响应给客户端，
                    // 如果无法发送给writer，继续从客户端读取内容将没有意义，因此break退出
                    let channel_msg = ChannelMessage::BoolData(content);
                    if msg_tx.send(channel_msg).await.is_err() {
                        eprintln!("receiver closed");
                        break;
                    }
                }
            }
        }
    }
}

/// 写给客户端
async fn write_to_partner(mut writer: OwnedWriteHalf, msg_rx:&mut mpsc::Receiver<ChannelMessage>) {
    //let mut buf_writer = tokio::io::BufWriter::new(writer);
    
    while let Some(e) = msg_rx.recv().await {
        println!("Received from the channel:{}",e);
        if let Err(err) = writer.write_all(&e.to_bytes()).await {
            eprintln!("write to client failed: {}", err);
            break;
        }
    }
}

async fn f(msg_ty: &mpsc::Sender<ChannelMessage>, msg_rx: &mut mpsc::Receiver<ChannelMessage>, msg: ChannelMessage)
{
    let f1 = msg_rx.recv();
    let f2 = msg_ty.send(msg);
    //futures::executor::block_on(f1, f2);
}

#[tokio::main]
async fn main(){
    let mut p = MPCServer::new();
    p.start().await;
}



/*impl MPCParty{
    fn new_server() -> MPCServer{
        let mut s = MPCServer{
            s: None,
            c: None
        };
        //s.s = Some(listener);
        s
    }
    fn new_client() -> MPCClient{
        MPCClient { s: None }    
    }

    fn is_server(&self) -> bool {
        match self.net {
            MPCType::Server(_) => true,
            _ => false,
        }
    }

    fn is_client(&self) -> bool {
        match self.net {
            MPCType::Client(_) => true,
            _ => false,
        }
    }

    async fn start(&mut self){
        match self.net {
            MPCType::Server(_) => {
                self.
            },
            MPCType::Client(_) => {

            },
        }
    }
}*/


//use crate::libmpc::MPCServer;
use libmpc::mpc_platform::MPCServer;
#[tokio::main]
async fn main(){
    let mut p = MPCServer::new();
    let _= p.start("127.0.0.1:8888").await;
}
use libmpc::mpc_platform::MPCClient;
#[tokio::main]
async fn main(){
    let mut p = MPCClient::new();
    let _= p.start("127.0.0.1:8888").await;
}
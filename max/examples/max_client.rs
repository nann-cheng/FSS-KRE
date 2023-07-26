use libmpc::mpc_platform::MPCClient;
#[tokio::main]
async fn main(){
    let mut p = MPCClient::new();
    let _= p.start().await;
}
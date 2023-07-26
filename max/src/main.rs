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
    
}
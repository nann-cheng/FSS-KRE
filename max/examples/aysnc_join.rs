use tokio::time::{sleep, Duration};
use tokio::runtime::Runtime;
use futures::executor::block_on;

async fn async_send(msg: usize) {
    sleep(Duration::from_secs(5)).await;
    println!("There, send message: {}", msg);
}

async fn async_recv() -> usize {
    let msg: usize = 5;
    sleep(Duration::from_secs(3)).await;
    println!("There, receive a message: {}", msg);
    msg
}

async fn my_func(msg: usize) -> usize {
    let f1 = async_send(msg);
    let f2 = async_recv();
    let (_, r) = futures::join!(f1, f2);
    r
}
#[tokio::main]
async fn main() {
    let msg1: usize = 12;

    /*let rt = Runtime::new().unwrap();
    let msg2 = rt.block_on(my_func(msg1));*/
    let msg2 = block_on(my_func(msg1));

    let r = msg1 + msg2;
    println!("Result: {}", r);
}
// use tokio::net::TcpListener;
// use tokio::prelude::*;
use tokio::time;
use std::time::{SystemTime, UNIX_EPOCH};

async fn task_that_takes_a_second() {
    // println!("hello {}", num);
    time::delay_for(time::Duration::from_secs(10000)).await
}

#[tokio::main]
async fn main() {
    // let mut listener = TcpListener::bind("127.0.0.1:8080").await?;
    let mut i = 1;
    loop {
        // let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            task_that_takes_a_second().await;
        });
        
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => println!("ts(sec): {:?} thd no: {}", n.as_secs(), i),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
        i += 1;
    }
    // return;
}
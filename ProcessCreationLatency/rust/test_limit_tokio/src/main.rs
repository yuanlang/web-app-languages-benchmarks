// use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::time;

async fn task_that_takes_a_second(num : i32) {
    println!("hello {}", num);
    time::delay_for(time::Duration::from_secs(5)).await
}

#[tokio::main]
async fn main() {
    // let mut listener = TcpListener::bind("127.0.0.1:8080").await?;
    let mut i = 0;
    while i < 100 {
        // let (mut socket, _) = listener.accept().await?;
        let num = i;
        tokio::spawn(async move {
            task_that_takes_a_second(num).await;
        });
        i += 1;
    }
    return;
}
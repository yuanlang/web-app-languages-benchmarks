use std::env;
use tokio::sync::mpsc;
// use std::thread;
use std::time::{Instant};
use tokio::task;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let arg1 = &args[1];
    let num: u32 = arg1.parse().expect("Not a number!");

    let start = Instant::now();
    let mut txs = Vec::new();
    let mut handles = Vec::new();
    for _i in 0..num {
        let (tx, mut rx) = mpsc::channel::<String>(10);
        txs.push(tx);
        let join_handle: task::JoinHandle<_> = tokio::spawn(async move {
            rx.recv().await.expect("Unable to receive from channel");
        });
        handles.push(join_handle);
    }

    let duration = start.elapsed();

    println!("Total time taken: {:?}", duration.as_secs_f64());

    for mut tx in txs {
        let val = String::from("hi");
        tx.send(val).await.unwrap();
    }

    for handle in handles {
         let _ =  tokio::join!(handle);
    }

}

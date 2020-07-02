use std::env;
use std::sync::mpsc;
use std::thread;
use std::time::{Instant};

fn main() {
    let args: Vec<String> = env::args().collect();
    let arg1 = &args[1];
    let num: u32 = arg1.parse().expect("Not a number!");

    let start = Instant::now();
    let mut txs = Vec::new();
    let mut handles = Vec::new();
    for _i in 0..num {
        let (tx, rx) = mpsc::channel::<String>();
        txs.push(tx);
        let handle = thread::spawn(move || {
            let _received = rx.recv().expect("Unable to receive from channel");
            // println!("{} Got: {}", _i, _received);
        });
        handles.push(handle);
    }

    let duration = start.elapsed();

    println!("Total time taken: {:?}", duration);

    for tx in txs {
        let val = String::from("hi");
        tx.send(val).unwrap();
    }

    for handle in handles {
        handle.join().unwrap();
    }

}

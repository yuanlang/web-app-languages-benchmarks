use std::env;
use std::error::Error;
use std::time::{Instant};

use tokio::sync::mpsc;

use rand::random;

const MSG_QUEUE_LEN: usize = 100; //message queue length

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print!("too few parameters\n");
        print!("cargo run repeat_num data_size\n");
        std::process::exit(0);
    }

    let arg1 = &args[1];
    let arg2 = &args[2];
    let repeat_num: u32 = arg1.parse().expect("Not a number!");
    let msg_length: usize = arg2.parse().expect("Not a number!");
    println!("Repeat times: {}, data length: {}", repeat_num, msg_length);

    let mut send_bytes_1 : Vec<u8> = (0..msg_length).map(|_| { random::<u8>() }).collect();
    let mut send_bytes_2 : Vec<u8> = (0..msg_length).map(|_| { random::<u8>() }).collect();

    let (mut tx1, mut rx1) = mpsc::channel::<Vec<u8>>(msg_length * MSG_QUEUE_LEN);
    let (mut tx2, mut rx2) = mpsc::channel::<Vec<u8>>(msg_length * MSG_QUEUE_LEN);

    let start = Instant::now();
    let _p1 = tokio::spawn(async move {
        for _num in 0 .. repeat_num {
            if let Err(_) = tx1.send(send_bytes_1).await {
                println!("the channel 1 receiver dropped");
                break;
            }
            send_bytes_1 = rx2.recv().await.unwrap();
        }
    });

    let _p2 = tokio::spawn(async move {
        for _num in 0 .. repeat_num{
            if let Err(_) = tx2.send(send_bytes_2).await {
                println!("the channel 2 receiver dropped");
                break;
            }
            send_bytes_2 = rx1.recv().await.unwrap();
        }
    });

    let _result1 = _p1.await;
    let _result2 = _p2.await;

    let duration = start.elapsed();

    println!("Total time taken: {:?} seconds", duration.as_secs_f64());

    std::process::exit(0);
}


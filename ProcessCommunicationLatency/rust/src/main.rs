use std::env;
use std::sync::mpsc;
use std::thread;
//use std::sync::Arc;
use std::time::{Instant};
// use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    // first one is the programme name itself
    if args.len() < 3 {
        print!("too few parameters\n");
        print!("cargo run repeat_num data_size\n");
        std::process::exit(0);
    }

    let arg1 = &args[1];
    let arg2 = &args[2];
    let r: u32 = arg1.parse().expect("Not a number!");
    let d: u32 = arg2.parse().expect("Not a number!");
    println!("Repeat times: {}, data length: {}", r, d);

    let mut send_bytes_1 : Vec<u8> = (0..d).map(|_| { rand::random::<u8>() }).collect();
    let mut send_bytes_2 : Vec<u8> = (0..d).map(|_| { rand::random::<u8>() }).collect();

    let (tx1, rx1) = mpsc::channel::<Vec<u8>>();
    let (tx2, rx2) = mpsc::channel::<Vec<u8>>();

    let start = Instant::now();
    let p1 = thread::spawn(move || {
        for _num in 0 .. r {
            tx1.send(send_bytes_1).unwrap();
            send_bytes_1 = rx2.recv().unwrap();
        }
    });

    let p2 = thread::spawn(move || {
        for _num in 0 .. r {
            tx2.send(send_bytes_2).unwrap();
            send_bytes_2 = rx1.recv().unwrap();
        }
    });

    p1.join().unwrap();  
    p2.join().unwrap();

    let duration = start.elapsed();

    println!("Total time taken: {:?} seconds", duration.as_secs_f64());

    std::process::exit(0);
}


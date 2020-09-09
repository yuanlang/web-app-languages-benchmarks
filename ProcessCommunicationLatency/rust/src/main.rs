use std::env;
use std::sync::mpsc;
use std::thread;
//use std::sync::Arc;
use std::time::{Instant};
// use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print!("too few parameters\n");
        print!("cargo run repeat_num data_size\n");
        std::process::exit(0);
    }

    let arg1 = &args[1];
    let arg2 = &args[2];
    let r: u32 = arg1.parse().expect("Not a number!");
    let d: u32 = arg2.parse().expect("Not a number!");
    println!("Repeat times: {}, data length: {}", r, d);

    let send_bytes_1 : Vec<u8> = (0..d).map(|_| { rand::random::<u8>() }).collect();
    let send_bytes_2 : Vec<u8> = (0..d).map(|_| { rand::random::<u8>() }).collect();

    let (tx1, rx1) = mpsc::channel::<Vec<u8>>();
    let (tx2, rx2) = mpsc::channel::<Vec<u8>>();

    let start = Instant::now();
    let p1 = thread::spawn(move || {
        let mut num = 0;
        tx1.send(send_bytes_1).unwrap();
        num += 1;
        while num < r {
            // this is different from other language
            // other langauge no need to clone it every time
            // the overhead all become bigger whent the message size is bigger
            // in Go, it only sends a pointer to the buffer
            let msg = rx2.recv().unwrap();
            tx1.send(msg).unwrap();
            num += 1;
        }
        // rx2.recv().unwrap();
    });

    let p2 = thread::spawn(move || {
        let mut num = 0;
        tx2.send(send_bytes_2).unwrap();
        num += 1;
        while num < r {
            // this is different from other language
            // other langauge no need to clone it every time
            // the overhead all become bigger whent the message size is bigger
            let msg = rx1.recv().unwrap();
            num += 1;
            tx2.send(msg).unwrap();
        }
        // rx1.recv().unwrap();
    });

    // let p3 = thread::spawn(move || {
    //     let d = Duration::from_millis(10);
    //     let mut num = 0;
    //     while num < r {
    //         //println!("p3 recv");
    //         let _r = rx1.recv_timeout(d);
    //         num += 1;
    //     }
    // });
    
    // let p4 = thread::spawn(move || {
    //     let d = Duration::from_millis(10);
    //     let mut num = 0;
    //     while num < r {
    //         //println!("p4 recv");
    //         let _r = rx2.recv_timeout(d);
    //         num += 1;
    //     }
    // });

    p1.join().unwrap();  
    p2.join().unwrap();
    // p3.join().unwrap();  
    // p4.join().unwrap();

    let duration = start.elapsed();

    println!("Total time taken: {:?} seconds", duration.as_secs_f64());

    std::process::exit(0);
}


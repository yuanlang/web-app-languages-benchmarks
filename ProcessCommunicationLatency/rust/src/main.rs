use std::env;
use std::sync::mpsc;
use std::thread;
//use std::sync::Arc;
use std::time::{Instant};
use std::time::Duration;

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
    //println!("{:?}", send_bytes_1);
    let send_bytes_2 : Vec<u8> = (0..d).map(|_| { rand::random::<u8>() }).collect();
    //println!("{:?}", send_bytes_2);

    let (tx1, _rx1) = mpsc::channel::<Vec<u8>>();
    let (tx2, _rx2) = mpsc::channel::<Vec<u8>>();

    let start = Instant::now();
    let p1 = thread::spawn(move || {
        // let val = String::from("hi");
        let mut num = 0;
        while num < r {
            let sending = send_bytes_1.clone();
            tx1.send(sending.to_vec()).unwrap();
            num += 1;
        }
        
        // let received = _rx2.recv().unwrap();
        // println!("Got: {:?}", received.len());
    });

    let p2 = thread::spawn(move || {
        // let val = String::from("hi");
        let mut num = 0;
        while num < r {
            let sending = send_bytes_2.clone();
            tx2.send(sending.to_vec()).unwrap();
            num += 1;
        }
        // let received = _rx1.recv().unwrap();
        // println!("Got: {:?}", received.len());
    });

    let p3 = thread::spawn(move || {
        let d = Duration::from_millis(10);
        let mut num = 0;
        while num < r {
            //println!("p3 recv");
            let _r = _rx1.recv_timeout(d);
            num += 1;
        }
    });
    
    let p4 = thread::spawn(move || {
        let d = Duration::from_millis(10);
        let mut num = 0;
        while num < r {
            //println!("p4 recv");
            let _r = _rx2.recv_timeout(d);
            num += 1;
        }
    });

    p1.join().unwrap();  
    p2.join().unwrap();
    p3.join().unwrap();  
    p4.join().unwrap();

    let duration = start.elapsed();

    println!("Total time taken: {:?} seconds", duration.as_secs_f64());

    std::process::exit(0);
}

/*
/// Copy data in `from` into `to`, until the shortest
/// of the two slices.
///
/// Return the number of bytes written.
fn byte_copy(from: &[u8], mut to: &mut [u8]) -> usize {
    to.write(from).unwrap()
}
*/

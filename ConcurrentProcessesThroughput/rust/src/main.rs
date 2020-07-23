use std::env;
use std::sync::mpsc;
use std::thread;
use std::time::{Instant};
use std::time::Duration;
use std::sync::{Arc, Mutex};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        print!("invalid number of goroutines\n");
        print!("cargo run thread_num\n");
        std::process::exit(0);
    }

    let arg1 = &args[1];
    let n: u32 = arg1.parse().expect("Not a number!");
    println!("Thread number: {}", n);

    let d = 500; //data length
    let send_bytes : Vec<u8> = (0..d).map(|_| { rand::random::<u8>() }).collect();
    //println!("{:?}", send_bytes_1);

    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    let start = Instant::now();
    let mut thread_holder = vec![];
    for _i in 0 .. n {
        let tx1 = mpsc::Sender::clone(&tx);
        let send_bytes_1 = send_bytes.clone();
        thread_holder.push(thread::spawn(move || {
            loop {
                let sending = send_bytes_1.clone();
                tx1.send(sending.to_vec()).unwrap();
            }
        }));
    }

    // for thread_elememt in thread_holder {
    //     thread_elememt.join().unwrap();
    // }

    let counter = Arc::new(Mutex::new(0));
    let counter1 = Arc::clone(&counter);
    let _aggregator = thread::spawn(move || {
        let d = Duration::from_millis(10);        
        loop {
            let mut num = counter1.lock().unwrap();
            //println!("aggregator recv");
            let _r = rx.recv_timeout(d);
            *num += 1;
        }
    });

    // aggregator.join().unwrap();

    thread::sleep(Duration::from_secs(60));

    let duration = start.elapsed();

    println!("Total time taken seconds: {:?} ", duration.as_secs_f64());
    println!("Total Messages: {} ", *counter.lock().unwrap());

    std::process::exit(0);
}

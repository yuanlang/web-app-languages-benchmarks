use std::env;
use std::sync::mpsc;
use std::thread;
use std::time::{Instant};
use std::time::Duration;
use std::sync::{Arc, Mutex};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
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

    // Create thread pool according to the parameter
    let tx_counter = Arc::new(Mutex::new(0));
    let start = Instant::now();
    let mut thread_holder = vec![];
    for _i in 0 .. n {
        let tx1 = mpsc::Sender::clone(&tx);
        let counter1 = Arc::clone(&tx_counter);
        let send_bytes_copy = send_bytes.clone();
        thread_holder.push(thread::spawn(move || {
            loop {
                let mut num = counter1.lock().unwrap();
                let sending = send_bytes_copy.clone();
                tx1.send(sending.to_vec()).unwrap();
                *num += 1;
            }
        }));
    }

    let counter = Arc::new(Mutex::new(0));
    let receiver = Arc::new(Mutex::new(rx));
    
    let mut aggregator_holder = vec![];
    for _i in 0 .. n {
        let counter1 = Arc::clone(&counter);
        let thread_receiver = Arc::clone(&receiver);
        aggregator_holder.push(thread::spawn(move || loop {
            let d = Duration::from_millis(10);
            let mut num = counter1.lock().unwrap();
            //println!("aggregator recv");
            let _r = thread_receiver.lock().unwrap().recv_timeout(d);
            *num += 1;
        }));
    }

    thread::sleep(Duration::from_secs(60));

    let duration = start.elapsed();

    println!("Total time taken seconds: {:?} ", duration.as_secs_f64());
    println!("Total send messages: {} ", *tx_counter.lock().unwrap());
    println!("Total recv messages: {} ", *counter.lock().unwrap());

    std::process::exit(0);
}

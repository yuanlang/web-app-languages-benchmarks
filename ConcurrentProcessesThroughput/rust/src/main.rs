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

    // let (tx_to_aggregator, rx_by_aggregator) = mpsc::channel::<u32>();

    // Create thread pool according to the parameter
    let tx_counter = Arc::new(Mutex::new(0));
    let start = Instant::now();
    let mut server_thread_holder = vec![];
    let mut client_thread_holder = vec![];
    for _i in 0 .. n {
        let (tx_to_client, rx_by_server) = mpsc::channel::<Vec<u8>>();
        let (tx_to_server, rx_by_client) = mpsc::channel::<Vec<u8>>();
        // let tx_to_client = mpsc::Sender::clone(&tx);
        let counter1 = Arc::clone(&tx_counter);
        let send_bytes_copy = send_bytes.clone();
        
        //spawn server threads
        server_thread_holder.push(thread::spawn(move || {
            let sending = send_bytes_copy.clone();
            tx_to_client.send(sending.to_vec()).unwrap();
            while let Ok(msg) = rx_by_server.recv() {
                // received message means one round finish, an increase of 1
                let mut num = counter1.lock().unwrap();
                *num += 1;
                tx_to_client.send(msg).unwrap();
            }
        }));

        //spawn client threads
        client_thread_holder.push(thread::spawn(move || {
            while let Ok(msg) = rx_by_client.recv() {
                tx_to_server.send(msg).unwrap();
            }
        }));
    }

    thread::sleep(Duration::from_secs(60));

    let duration = start.elapsed();

    println!("Total time taken seconds: {:?} ", duration.as_secs_f64());
    println!("Total send messages: {} ", *tx_counter.lock().unwrap());

    std::process::exit(0);
}

use std::env;
use std::sync::mpsc;
use std::thread;
use std::time::{Instant};
use std::time::Duration;
use std::sync::{Arc, Mutex};

const MSG_LENGTH: usize = 500; //data length
const TEST_DURATION: u64 = 60; //the length of test, in seconds
const FREQUENT: usize = 1000; //report per time

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

    let tx_counter = Arc::new(Mutex::new(0));

    let (tx_to_aggregator, rx_by_aggregator) = mpsc::channel::<usize>();

    //spawn aggregator to calculate the count
    let counter1 = Arc::clone(&tx_counter);
    thread::spawn(move || {
        while let Ok(count) = rx_by_aggregator.recv() {
            // received message and add it to the counter
            let mut num = counter1.lock().unwrap();
            *num += count;
        }
    });

    // Create thread pool according to the parameter
    let send_bytes : Vec<u8> = (0..MSG_LENGTH).map(|_| { rand::random::<u8>() }).collect();
    let start = Instant::now();
    let mut server_thread_holder = vec![];
    let mut client_thread_holder = vec![];
    for _i in 0 .. n {
        let (tx_to_client, rx_by_server) = mpsc::channel::<Vec<u8>>();
        let (tx_to_server, rx_by_client) = mpsc::channel::<Vec<u8>>();
        let send_bytes_copy = send_bytes.clone();
        let tx_to_aggregator_copy = tx_to_aggregator.clone();
        
        //spawn server threads
        server_thread_holder.push(thread::spawn(move || {
            let sending = send_bytes_copy.clone();
            tx_to_client.send(sending.to_vec()).unwrap();
            let mut counter: usize = 0;
            while let Ok(msg) = rx_by_server.recv() {
                // received message means one round finish, an increase of 1
                counter += 1;
                if counter == FREQUENT {
                    // send message to aggregator when the counter is 100 and reset it to 0
                    tx_to_aggregator_copy.send(FREQUENT).unwrap();
                    counter = 0;
                }
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

    thread::sleep(Duration::from_secs(TEST_DURATION));

    let duration = start.elapsed();

    println!("Total time taken seconds: {:?} ", duration.as_secs_f64());
    println!("Total send messages: {} ", *tx_counter.lock().unwrap());

    std::process::exit(0);
}

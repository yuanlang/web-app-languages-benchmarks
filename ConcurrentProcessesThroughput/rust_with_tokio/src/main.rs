use std::env;
use std::error::Error;
use std::thread;
use std::time::{Instant};
use std::time::Duration;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;

const MSG_LENGTH: usize = 500; //data length
const MSG_QUEUE_LENGTH: usize = 100;
const TEST_DURATION: u64 = 60; //the length of test, in seconds
const FREQUENT: usize = 100;  //report messages per time

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print!("invalid number of goroutines\n");
        print!("cargo run thread_num\n");
        std::process::exit(0);
    }

    let arg1 = &args[1];
    let n: u32 = arg1.parse().expect("Not a number!");
    println!("Thread number: {}", n);

    let send_bytes : Vec<u8> = (0..MSG_LENGTH).map(|_| { rand::random::<u8>() }).collect();
    //println!("{:?}", send_bytes_1);

    let (tx_to_aggregator, mut rx_by_aggregator) = mpsc::channel::<usize>(MSG_QUEUE_LENGTH);
    
    let tx_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

    // spawn aggregator to recv counter report
    let counter1 = Arc::clone(&tx_counter);
    tokio::spawn(async move {
        while let Some(count) = rx_by_aggregator.recv().await {
            let mut num = counter1.lock().unwrap();
            *num += count;
        }
    });

    // Create thread pool according to the parameter
    let start = Instant::now();
    let mut server_thread_holder = vec![];
    let mut client_thread_holder = vec![];
    for _i in 0 .. n {
        let (mut tx_to_client, mut rx_by_server) = mpsc::channel::<Vec<u8>>(MSG_LENGTH * MSG_QUEUE_LENGTH);
        let (mut tx_to_server, mut rx_by_client) = mpsc::channel::<Vec<u8>>(MSG_LENGTH * MSG_QUEUE_LENGTH);
        let mut tx_to_aggregator_copy = mpsc::Sender::clone(&tx_to_aggregator);
        // let counter1 = Arc::clone(&tx_counter);
        let send_bytes_copy = send_bytes.clone();
        
        //spawn server threads
        server_thread_holder.push(tokio::spawn(async move {
            let sending = send_bytes_copy.clone();
            let mut counter = 0;
            tx_to_client.send(sending.to_vec()).await.unwrap();
            while let Some(msg) = rx_by_server.recv().await {
                // received message means one round finish, an increase of 1
                // let mut num = counter1.lock().unwrap();
                // *num += 1;
                counter += 1;
                if counter == FREQUENT {
                    //send message to aggregator when the counter is 100 and reset it to 0
                    tx_to_aggregator_copy.send(FREQUENT).await.unwrap();
                    counter = 0;
                }
                tx_to_client.send(msg).await.unwrap();
            }
        }));

        //spawn client threads
        client_thread_holder.push(tokio::spawn(async move {
            while let Some(msg) = rx_by_client.recv().await {
                tx_to_server.send(msg).await.unwrap();
            }
        }));
    }

    thread::sleep(Duration::from_secs(TEST_DURATION));

    let duration = start.elapsed();

    println!("Total time taken seconds: {:?} ", duration.as_secs_f64());
    println!("Total send messages: {} ", *tx_counter.lock().unwrap());

    std::process::exit(0);
}

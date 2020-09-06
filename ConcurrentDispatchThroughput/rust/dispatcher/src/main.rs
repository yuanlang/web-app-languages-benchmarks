//! A new benchmark to test the dispatch performance of Rust 
//!
//! This benchmark will create a group of generator and one dispatcher
//! and a group of receiver. The number of generator and the number of 
//! receiver could be specified by the command line parameter.
//!

#![warn(rust_2018_idioms)]

// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::net::TcpListener;
use tokio::sync::mpsc;
// use tokio::net::{TcpStream};
use std::collections::VecDeque;

use std::env;
use std::error::Error;
use std::collections::HashMap;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant};
use std::time::Duration;

const MSG_LEN: usize = 500; //data length

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("invalid number of cli parameter");
        println!("cargo run generator_num receiver_num total_msg_num");
        std::process::exit(0);
    }

    let arg1 = &args[1];
    let generator_num: u32 = arg1.parse().expect("Not a number!");
    println!("Generator number: {}", generator_num);

    let arg2 = &args[2];
    let receiver_num: u32 = arg2.parse().expect("Not a number!");
    println!("Receiver number: {}", receiver_num);

    let arg3 = &args[3];
    let total_msg_num: u32 = arg3.parse().expect("Not a number!");
    println!("Total message number: {}", total_msg_num);

    // Create a channel for generators to dispatcher, 
    // one sender could share with multi generator
    let (chan_tx_to_disp, chan_rx_by_disp) = 
                    mpsc::unbounded_channel::<[u8; MSG_LEN]>();

    // Create a group of channels for dispatcher to receivers
    // dispatcher hold all the sender, and dispatch message by the first byte of a message
    // hashmap: key = receiver_id from 1 : number of receiver, 0 reserve for futher use
    let mut channels_tx_map : HashMap<u32, mpsc::UnboundedSender<[u8; MSG_LEN]>>  = HashMap::new();
    let mut channels_rx_vec: VecDeque<mpsc::UnboundedReceiver<[u8; MSG_LEN]>> = VecDeque::new();
    for curr in 1 .. receiver_num + 1 {
        let (sender, receiver) = mpsc::unbounded_channel::<[u8; MSG_LEN]>();
        channels_tx_map.insert(curr, sender);
        channels_rx_vec.push_back(receiver);
    }

    // Setup a couple of task as Receivers
    // the Receiver only receive message from the channel with the dispatcher
    let mut curr : usize = 1;
    let recv_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    while let Some(rx1) = channels_rx_vec.pop_front() {
        let recv_mutex = Arc::clone(&recv_counter);
        println!("Successfully create receiver {}", curr);

        // get the channel receiver first
        tokio::spawn(async move {
            do_receive(curr, rx1, recv_mutex).await;
        });

        curr = curr + 1;
    }

    // Setup the dispatcher
    println!("Successfully create dispatcher");
    let disp_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let disp_mutex = Arc::clone(&disp_counter);
    tokio::spawn(async move {
        do_dispatch(chan_rx_by_disp, channels_tx_map, disp_mutex).await;
    });

    // Setup the generators
    let gene_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    for curr in 1 .. generator_num + 1 {
        println!("Successfully create generator {}", curr);
        let gene_mutex = Arc::clone(&gene_counter);
        // get a copy of channel sender from hashmap, to allow the ownership move to 
        // the tokio thread and doesn't influence others
        let chan_tx_to_disp_copy = chan_tx_to_disp.clone();

        // all clients to make progress concurrently, rather than
        // blocking one on completion of another. To achieve this we use the
        // `tokio::spawn` function to execute the work in the background.
        tokio::spawn(async move {
            do_generate(chan_tx_to_disp_copy, receiver_num as u8, 
                total_msg_num / generator_num, gene_mutex).await;
        });
    }

    let start = Instant::now();

    thread::sleep(Duration::from_secs(6));

    // get lock, copy the counters
    let gene_num = *gene_counter.lock().unwrap();
    let disp_num = *disp_counter.lock().unwrap();
    let recv_num = *recv_counter.lock().unwrap();

    let duration = start.elapsed();

    println!("Total time taken seconds: {:?} ", duration.as_secs_f64());
    println!("Total generated messages: {} ", gene_num);
    println!("Total dispatched messages: {} ", disp_num);
    println!("Total received messages: {} ", recv_num);

    Ok(())
}

/// Receive message sent by dispatch
///
/// Receive messages from channel and dispatch them to the Receiver through 
/// the TCP connnections
async fn do_receive(_id: usize, 
        mut rx: mpsc::UnboundedReceiver<[u8; MSG_LEN]>, 
        counter: Arc<Mutex<usize>>) {
    loop {
        // Read message from the channel and wait replay
        match rx.recv().await {
            Some(_msg) => {
                // let _n = _msg[0];
                // println!("{} got msg type: {}", _id, n);

                //increase the counter
                let mut num = counter.lock().unwrap();
                *num += 1;
            },
            None => {
                // do nothing
            }
        }
    }    
}

/// Dispatch message to Receivers
///
/// Receive messages from channel and dispatch them to the Receiver through 
/// the TCP connnections
/// 
/// Parameter:
/// 
async fn do_dispatch(
        mut chan_rx: mpsc::UnboundedReceiver<[u8; MSG_LEN]>, 
        mut chan_tx_map: std::collections::HashMap<u32, mpsc::UnboundedSender<[u8; MSG_LEN]>>,
        counter: Arc<Mutex<usize>>) {
    loop {
        // Read message from the channel and wait replay
        match chan_rx.recv().await {
            Some(msg) => {
                let receiver_id = msg[0] as u32;
                // println!("got receiver id: {}", receiver_id);

                //dispatch msg to a Receiver
                match chan_tx_map.get_mut(&receiver_id) {
                    Some(tx_copy) => {
                        if let Err(_) = tx_copy.send(msg) {
                            println!("receiver thread dropped");
                            return;
                        }
                    },
                    None => {
                        println!("Get the wrong dispatch number {}", receiver_id);
                    }
                }
                // println!("Sent msg to No.{} Receiver.", receiver_id);
                
                //increase the counter
                let mut num = counter.lock().unwrap();
                *num += 1;

            },
            None => {
                // do nothing
            }
        }
    }    
}

/// Send received msg to message channel
/// 
/// Receive message from the Generator and send the message to the corresponed receiver
/// by using the first byte of the message
async fn do_generate(chan_tx_to_disp: mpsc::UnboundedSender<[u8; MSG_LEN]>, 
        receiver_num: u8, msg_num: u32,
        counter: Arc<Mutex<usize>>) {
    let send_bytes : Vec<u8> = (0..MSG_LEN).map(|_| { rand::random::<u8>() }).collect();
    let mut send_array = [0u8; MSG_LEN];
    send_array.copy_from_slice(&send_bytes);

    // In a loop, read data from the socket and write the data back.
    for _i in 0 .. msg_num{
        // gen receiver id
        let n = rand::random::<u8>() % receiver_num + 1;
        send_array[0] = n;

        // send message to dispatcher
        if let Err(_) = chan_tx_to_disp.send(send_array) {
            println!("receiver thread dropped");
            return;
        }

        //increase the counter
        let mut num = counter.lock().unwrap();
        *num += 1;
    }    
}

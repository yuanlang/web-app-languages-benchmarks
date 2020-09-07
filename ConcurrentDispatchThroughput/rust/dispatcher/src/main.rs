//! A new benchmark to test the dispatch performance of Rust 
//!
//! This benchmark will create a group of generator and one dispatcher
//! and a group of receiver. The number of generator and the number of 
//! receiver could be specified by the command line parameter.
//!

#![warn(rust_2018_idioms)]

use tokio::sync::mpsc;
use tokio::runtime::Runtime;
use std::collections::VecDeque;

use std::env;
use std::error::Error;
use std::collections::HashMap;

use std::sync::{Arc, Mutex};
use std::time::{Instant};

// const MSG_LEN: usize = 500; //data length
const MSG_QUEUE_LEN: usize = 100; //message queue length

enum Command {
    Data(),
    Done(),
}

type Message = (Command, Vec<u8>);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        println!("invalid number of cli parameter");
        println!("cargo run generator_num receiver_num total_msg_num message_length");
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

    let arg4 = &args[4];
    let msg_length: usize = arg4.parse().expect("Not a number!");
    println!("message length: {}", msg_length);

    let runtime = Runtime::new().unwrap();

    // Create a channel for all others to master, 
    // one sender could share with multi users
    let (chan_tx_to_master, mut chan_rx_by_master) = 
                    mpsc::channel::<Message>(MSG_QUEUE_LEN * msg_length);

    // Create a channel for generators to dispatcher, 
    // one sender could share with multi generator
    let (chan_tx_to_disp, chan_rx_by_disp) = 
                    mpsc::channel::<Message>(MSG_QUEUE_LEN * msg_length);

    // Create a group of channels for dispatcher to receivers
    // dispatcher hold all the sender, and dispatch message by the first byte of a message
    // hashmap: key = receiver_id from 1 : number of receiver, 0 reserve for futher use
    let mut channels_tx_map : HashMap<u32, mpsc::Sender<Message>> = HashMap::new();
    let mut channels_rx_vec: VecDeque<mpsc::Receiver<Message>> = VecDeque::new();
    for curr in 1 .. receiver_num + 1 {
        let (sender, receiver) = mpsc::channel::<Message>(MSG_QUEUE_LEN * msg_length);
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
        let chan_tx_to_master_copy = chan_tx_to_master.clone();
        // get the channel receiver first
        runtime.spawn(async move {
            do_receive(curr, rx1, chan_tx_to_master_copy, recv_mutex).await;
        });

        curr = curr + 1;
    }

    // Setup the dispatcher
    println!("Successfully create dispatcher");
    let disp_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let disp_mutex = Arc::clone(&disp_counter);
    runtime.spawn(async move {
        do_dispatch(chan_rx_by_disp, channels_tx_map, disp_mutex).await;
    });

    let start = Instant::now();

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
        runtime.spawn(async move {
            do_generate(curr as u8, chan_tx_to_disp_copy, receiver_num as u8, 
                total_msg_num / receiver_num, msg_length, gene_mutex).await;
        });
    }

    //wait for all receiver sent done
    let mut recv_done_cnt = 0;
    loop {
        match chan_rx_by_master.recv().await {
            Some((cmd, _msg)) => {
                match cmd {
                    Command::Data() => {
                        //do nothing
                    }
                    Command::Done() => {
                        println!("receive done message from receiver");
                        recv_done_cnt += 1;

                        if recv_done_cnt == receiver_num {
                            break;
                        }
                    },
                }
            },
            None => {
                //do nothing
            }
        }
    }
    // thread::sleep(Duration::from_secs(6));

    // get lock, copy the counters
    let gene_num = *gene_counter.lock().unwrap();
    let disp_num = *disp_counter.lock().unwrap();
    let recv_num = *recv_counter.lock().unwrap();

    let duration = start.elapsed();

    println!("Total time taken seconds:\t {:?} ", duration.as_secs_f64());
    println!("Total generated messages:\t {} ", gene_num);
    println!("Total dispatched messages:\t {} ", disp_num);
    println!("Total received messages:\t {} ", recv_num);

    runtime.shutdown_background();

    Ok(())
}

/// Receive message sent by dispatch
///
/// Receive messages from channel and dispatch them to the Receiver through 
/// the TCP connnections
async fn do_receive(_id: usize, 
        mut rx: mpsc::Receiver<Message>, 
        mut tx_to_master: mpsc::Sender<Message>, 
        counter: Arc<Mutex<usize>>) {
    loop {
        // Read message from the channel and wait replay
        match rx.recv().await {
            Some((cmd, msg)) => {
                match cmd {
                    Command::Data() => {
                        // let n = msg[0];
                        // println!("{} got msg type: {}", _id, n);

                        //increase the counter
                        let mut num = counter.lock().unwrap();
                        *num += 1;
                    },
                    Command::Done() => {
                        println!("{} receive done message from generator", _id);
                        //send done back to master
                        if let Err(_) = tx_to_master.send((cmd, msg)).await {
                            println!("receiver thread dropped");
                            return;
                        }
                    },
                }
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
        mut chan_rx: mpsc::Receiver<Message>, 
        mut chan_tx_map: std::collections::HashMap<u32, mpsc::Sender<Message>>,
        counter: Arc<Mutex<usize>>) {
    loop {
        // Read message from the channel and wait replay
        match chan_rx.recv().await {
            Some((cmd, msg)) => {
                let receiver_id = msg[0] as u32;
                // println!("got receiver id: {}", receiver_id);

                //dispatch msg to a Receiver
                match chan_tx_map.get_mut(&receiver_id) {
                    Some(tx_copy) => {
                        if let Err(_) = tx_copy.send((cmd, msg)).await {
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
async fn do_generate(
        _id: u8,
        mut chan_tx_to_disp: mpsc::Sender<Message>, 
        receiver_num: u8,
        msg_num: u32,
        msg_len: usize,
        counter: Arc<Mutex<usize>>) {
    let mut send_bytes : Vec<u8> = (0..msg_len).map(|_| { rand::random::<u8>() }).collect();
    // let mut send_array = [0u8; MAX_MSG_LEN];
    // send_array.copy_from_slice(&send_bytes);

    // In a loop, read data from the socket and write the data back.
    for _i in 0 .. msg_num{
        let mut send_bytes_copy = send_bytes.clone();
        // gen receiver id
        let n = rand::random::<u8>() % receiver_num + 1;
        send_bytes_copy[0] = n;

        // send message to dispatcher
        if let Err(_) = chan_tx_to_disp.send((Command::Data(), send_bytes_copy)).await {
            println!("receiver thread dropped");
            return;
        }

        //increase the counter
        let mut num = counter.lock().unwrap();
        *num += 1;
    }

    // send a done message to receiver
    send_bytes[0] = _id;
    if let Err(_) = chan_tx_to_disp.send((Command::Done(), send_bytes)).await {
        println!("receiver thread dropped");
        return;
    }

}

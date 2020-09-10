//! The server of the dispatcher model
//!
//! This server will create a TCP listener, accept connections in a loop, and
//! dispatch everything to connected receivers.
//!

#![warn(rust_2018_idioms)]

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::net::{TcpStream};
use tokio::task::JoinHandle;
use std::collections::VecDeque;

use std::net::Shutdown;

use std::env;
use std::error::Error;
use std::collections::HashMap;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant};
use std::time::Duration;

use log::{debug};

use dispatcher::{Command, MSG_LEN, DEFAULT_SERVER_ADDR};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("invalid number of connections");
        println!("cargo run connect_num");
        std::process::exit(0);
    }

    let arg1 = &args[1];
    let connect_num: u32 = arg1.parse().expect("Not a number!");
    println!("Connection number: {}", connect_num);

    let mut settings = config::Config::default();
    settings
        // Add in `./Settings.toml`
        .merge(config::File::with_name("Settings")).unwrap()
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .merge(config::Environment::with_prefix("APP")).unwrap();

    // Print out our settings (as a HashMap)
    let hash_setting = settings.try_into::<HashMap<String, String>>().unwrap();
    println!("{:?}", hash_setting);

    // Create a couple of Channels for connections, one channel for one connnection
    // key is the server id, from 1 to receiver_num
    let receiver_num : u32 = 10;
    let mut channels_tx_map : HashMap<u32, mpsc::Sender<[u8; MSG_LEN]>>  = HashMap::new();
    let mut channels_rx_vec: VecDeque<mpsc::Receiver<[u8; MSG_LEN]>> = VecDeque::new();

    for curr in 1 .. receiver_num + 1 {
        let (sender, receiver) = mpsc::channel::<[u8; MSG_LEN]>(1000 * MSG_LEN);
        channels_tx_map.insert(curr, sender);
        channels_rx_vec.push_back(receiver);
    }

    // Setup a couple of connection with the Receivers
    // the address of the receivers will be read from a config file
    let mut curr : usize = 1;
    let mut disp_thread_holder: Vec<JoinHandle<()>> = Vec::new();
    let disp_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    while let Some(rx1) = channels_rx_vec.pop_front() {
        let server_name = format!("{}{}", "server" , curr);
        let server_addr = hash_setting[&server_name].clone();
        let disp_mutex = Arc::clone(&disp_counter);
        match TcpStream::connect(&server_addr).await {
            Ok(stream) => {
                println!("{} Successfully connected to server {}", curr, &server_addr);

                // get the channel receiver first
                disp_thread_holder.push(tokio::spawn(async move {
                    do_dispatch(curr, rx1, stream, disp_mutex).await;
                }));
            },
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
        curr = curr + 1;
    }

    // Allow passing an address to listen on as the first argument of this
    // program, but otherwise we'll just set up our TCP listener on
    // 127.0.0.1:8888 for connections.
    let addr = env::args()
        .nth(2)
        .unwrap_or_else(|| DEFAULT_SERVER_ADDR.to_string());

    // Create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop.
    let mut listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    // Process incomming connections
    let mut recv_thread_holder: Vec<JoinHandle<()>> = Vec::new();
    let recv_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let mut connection_id = 1;
    let mut tx_by_master_vec: Vec<tokio::sync::oneshot::Sender<dispatcher::Command>> = Vec::new();
    while connection_id < connect_num + 1 {
        let recv_mutex = Arc::clone(&recv_counter);
        // Asynchronously wait for an inbound socket.
        let (socket, _) = listener.accept().await?;

        println!("recv connnection, id {}", connection_id);

        // get a copy of senders hashmap, to allow the ownership move to 
        // the tokio thread and doesn't influence others
        let channels_tx_map_copy = channels_tx_map.clone();
        let (sender, receiver) = oneshot::channel::<Command>();
        tx_by_master_vec.push(sender);

        // all clients to make progress concurrently, rather than
        // blocking one on completion of another. To achieve this we use the
        // `tokio::spawn` function to execute the work in the background.
        recv_thread_holder.push(tokio::spawn(async move {
            push_msg_to_channel(connection_id, socket, receiver, channels_tx_map_copy, recv_mutex).await;
        }));
        connection_id += 1;
    }

    let start = Instant::now();

    //everything is ok, send start message to connect
    for tx in tx_by_master_vec {
        if let Err(_) = tx.send(Command::Start) {
            println!("the receiver dropped");
        }
    }

    for h in recv_thread_holder {
        let _r1 = h.await;
    }
    println!("all recv thread exit");

    // // wait all connector and dispather threads quit
    // for h in disp_thread_holder {
    //     let _r1 = h.await;
    // }
    // println!("all disp thread exit");

    // get lock, copy the counters
    let recv_result = *recv_counter.lock().unwrap();
    let disp_result = *disp_counter.lock().unwrap();

    let duration = start.elapsed();

    println!("Total time taken seconds: {:?} ", duration.as_secs_f64());
    println!("Total received messages: {} ", recv_result);
    println!("Total dispatched messages: {} ", disp_result);

    thread::sleep(Duration::from_secs(5));

    Ok(())
}

/// Dispatch message to Receivers
///
/// Receive messages from channel and dispatch them to the Receiver through 
/// the TCP connnections
async fn do_dispatch(id: usize, 
        mut rx: mpsc::Receiver<[u8; MSG_LEN]>, 
        mut stream: TcpStream,
        counter: Arc<Mutex<usize>>) {
    loop {
        // Read message from the channel and wait replay
        match rx.recv().await {
            Some(msg) => {
                // parse the message
                let t = msg[0];
                let cmd: Command = t.into();
                debug!("got command type: {}", t);

                match cmd {
                    Command::Start => {
                        //do nothing
                    },
                    Command::Data  => {
                        match stream.write(&msg).await {
                            Ok(_) => {
                                debug!("Sent msg to No.{} Receiver.", id);
                            },
                            Err(e) => {
                                println!("Failed to write data through socket: {}", e);
                            }
                        }
                        stream.flush().await.unwrap();
                        
                        //increase the counter
                        let mut num = counter.lock().unwrap();
                        *num += 1;
                    },
                    Command::Done  => {
                        // if this is the last Done message, quit current thread
                        println!("dispatcher receive done message");
                        stream.shutdown(Shutdown::Both).expect("shutdown call failed");
                        break;
                    },
                    Command::Unknown => {
                        // do nothing
                    },
                }
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
async fn push_msg_to_channel(connection_id: u32, mut stream: tokio::net::TcpStream, 
        rx_from_master: tokio::sync::oneshot::Receiver<Command>,
        mut channels_tx_map: std::collections::HashMap<u32, mpsc::Sender<[u8; MSG_LEN]>>,
        counter: Arc<Mutex<usize>>) {
    // Wait master send Start message
    match rx_from_master.await {
        Ok(cmd) => {
            match cmd {
                Command::Start => {
                    println!("{} recv start msg from master", connection_id);
                    // send tcp message to generator
                    let mut send_bytes : Vec<u8> = (0..MSG_LEN).map(|_| { rand::random::<u8>() }).collect();
                    send_bytes[0] = Command::Start as u8;
                    stream.write(&send_bytes).await.unwrap();
                    stream.flush().await.unwrap();
                },
                _ => {
                    //do nothing
                },
            }
        },
        Err(_err) => {
            // do nothing
        },
    }

    // In a loop, read data from the socket and write the data back.
    loop {
        let mut buf = [0u8; MSG_LEN];
        let n = stream
            .read(&mut buf)
            .await
            .expect("failed to read data from socket connected to the Generator");

        if n == 0 {
            println!("receive zero length message {}", connection_id);
            return;
        }

        // parse the message
        let cmd: Command = buf[0].into();
        match cmd {
            // Command::Start => {
            //     // if the message is a start message, it will record as start time
            //     println!("{} receive start message", connection_id)
            // },
            Command::Data => {
                // Dispatch the message to receiver thread by the first byte
                let dispath_num = buf[1] as u32;
                debug!("{} Get dispath num {}", connection_id, dispath_num);
                match channels_tx_map.get_mut(&dispath_num) {
                    Some(tx_copy) => {
                        if let Err(_) = tx_copy.send(buf).await {
                            println!("receiver thread dropped");
                            return;
                        }
                    },
                    None => {
                        println!("Get the wrong dispatch number {}", dispath_num);
                    }
                }

                //increase the counter
                let mut num = counter.lock().unwrap();
                *num += 1;
            },
            Command::Done => {
                println!("{} receive done message", connection_id);
                // stream.shutdown(Shutdown::Both).expect("shutdown write failed");
                break;
            },
            _ => {
                // do nothing
            },
        }

    }    
}

//! A "hello world" echo server with Tokio
//!
//! This server will create a TCP listener, accept connections in a loop, and
//! write back everything that's read off of each TCP connection.
//!
//! Because the Tokio runtime uses a thread pool, each TCP connection is
//! processed concurrently with all other TCP connections across multiple
//! threads.
//!
//! To see this server in action, you can run this in one terminal:
//!
//!     cargo run --example echo
//!
//! and in another terminal you can run:
//!
//!     cargo run --example connect 127.0.0.1:8080
//!
//! Each line you type in to the `connect` terminal should be echo'd back to
//! you! If you open up multiple terminals running the `connect` example you
//! should be able to see them all make progress simultaneously.

#![warn(rust_2018_idioms)]

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::net::{TcpStream};
// use std::str::from_utf8;
use std::collections::VecDeque;

use std::env;
use std::error::Error;
use std::collections::HashMap;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant};
use std::time::Duration;

const MSG_LEN: usize = 500; //data length

#[repr(u8)]
enum Command {
    Start = 1,
    Data  = 2,
    Done  = 3,
    Unknown = 4,
}

impl From<u8> for Command {
    fn from(orig: u8) -> Self {
        match orig {
            0x1 => return Command::Start,
            0x2 => return Command::Data,
            0x3 => return Command::Done,
            _   => return Command::Unknown,
        };
    }
}

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
    let disp_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    while let Some(rx1) = channels_rx_vec.pop_front() {
        let server_name = format!("{}{}", "server" , curr);
        let server_addr = hash_setting[&server_name].clone();
        let disp_mutex = Arc::clone(&disp_counter);
        match TcpStream::connect(&server_addr).await {
            Ok(stream) => {
                println!("{} Successfully connected to server {}", curr, &server_addr);

                // get the channel receiver first
                tokio::spawn(async move {
                    do_dispatch(curr, rx1, stream, disp_mutex).await;
                });
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
    const DEFAULT_ADDR: &str = "127.0.0.1:8888";
    let addr = env::args()
        .nth(2)
        .unwrap_or_else(|| DEFAULT_ADDR.to_string());

    // Create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop.
    let mut listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    // To limit the number of connections
    let mut connection_id = 1;
    while connection_id < connect_num + 1 {
        // Asynchronously wait for an inbound socket.
        let (socket, _) = listener.accept().await?;

        println!("recv connnection, id {}", connection_id);

        // get a copy of senders hashmap, to allow the ownership move to 
        // the tokio thread and doesn't influence others
        let channels_tx_map_copy = channels_tx_map.clone();

        // all clients to make progress concurrently, rather than
        // blocking one on completion of another. To achieve this we use the
        // `tokio::spawn` function to execute the work in the background.
        tokio::spawn(async move {
            push_msg_to_channel(connection_id, socket, channels_tx_map_copy).await;
        });
        connection_id += 1;
    }

    let start = Instant::now();

    thread::sleep(Duration::from_secs(6));

    // get lock, copy the counters
    let result = *disp_counter.lock().unwrap();

    let duration = start.elapsed();

    println!("Total time taken seconds: {:?} ", duration.as_secs_f64());
    println!("Total dispatch messages: {} ", result);

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
                let n = msg[0];
                println!("got msg type: {}", n);

                // parse the message

                // if this is the last Done message, record as end time

                match stream.write(&msg).await {
                    Ok(_) => {
                        println!("Sent msg to No.{} Receiver.", id);
                    },
                    Err(e) => {
                        println!("Failed to write data through socket: {}", e);
                    }
                }
                
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
async fn push_msg_to_channel(connection_id: u32, mut socket: tokio::net::TcpStream, 
    mut channels_tx_map_copy: std::collections::HashMap<u32, mpsc::Sender<[u8; MSG_LEN]>>) {

    // In a loop, read data from the socket and write the data back.
    loop {
        let mut buf = [0u8; MSG_LEN];
        let n = socket
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
            Command::Start => {
                // if the message is a start message, it will record as start time
                println!("{} receive start message", connection_id)
            },
            Command::Data => {
                // Dispatch the message to receiver thread by the first byte
                let dispath_num = buf[1] as u32;
                println!("Get dispath num {}", dispath_num);
                match channels_tx_map_copy.get_mut(&dispath_num) {
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
            },
            Command::Done => {
                println!("{} receive done message", connection_id)
            },
            Command::Unknown => {
                // do nothing
            },
        }

    }    
}

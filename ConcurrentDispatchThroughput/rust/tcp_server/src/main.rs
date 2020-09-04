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
use std::str::from_utf8;
use std::collections::VecDeque;

use std::env;
use std::error::Error;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

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
    let mut channels_tx_map : HashMap<u32, mpsc::Sender<[u8; 1024]>>  = HashMap::new();
    let mut channels_rx_vec: VecDeque<mpsc::Receiver<[u8; 1024]>> = VecDeque::new();

    for curr in 1 .. receiver_num + 1 {
        let (sender, receiver) = mpsc::channel::<[u8; 1024]>(1000 * 1024);
        channels_tx_map.insert(curr, sender);
        channels_rx_vec.push_back(receiver);
    }

    // Setup a couple of connection with the Receivers
    // the address of the receivers will be read from a config file
    let mut curr : usize = 1;
    while let Some(rx1) = channels_rx_vec.pop_front() {
        let server_name = format!("{}{}", "server" , curr);
        let server_addr = hash_setting[&server_name].clone();
        match TcpStream::connect(&server_addr).await {
            Ok(stream) => {
                println!("{} Successfully connected to server {}", curr, &server_addr);

                // get the channel receiver first
                tokio::spawn(async move {
                    do_dispatch(curr, rx1, stream).await;
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
    // 127.0.0.1:8080 for connections.
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Next up we create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop.
    let mut listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        // Asynchronously wait for an inbound socket.
        let (socket, _) = listener.accept().await?;

        // And this is where much of the magic of this server happens. We
        // crucially want all clients to make progress concurrently, rather than
        // blocking one on completion of another. To achieve this we use the
        // `tokio::spawn` function to execute the work in the background.
        //
        // Essentially here we're executing a new task to run concurrently,
        // which will allow all of our clients to be processed concurrently.
        let channels_tx_map_copy = channels_tx_map.clone();

        tokio::spawn(async move {
            push_msg_to_channel(socket, channels_tx_map_copy).await;
        });
    }
}

/// Dispatch message to Receivers
///
/// Receive messages from channel and dispatch them to the Receiver through the TCP connnections
async fn do_dispatch(id: usize, mut rx: mpsc::Receiver<[u8; 1024]>, mut stream: TcpStream) {
    loop {
        // Read message from the channel and wait replay
        match rx.recv().await {
            Some(msg) => {
                let n = msg[0];
                println!("got msg type: {}", n);

                match stream.write(&msg).await {
                    Ok(_) => {
                        println!("{} Sent msg to Receiver, awaiting reply...", id);
                    },
                    Err(e) => {
                        println!("Failed to write data through socket: {}", e);
                    }
                }


                let mut data = [0 as u8; 10]; // using 6 byte buffer
                match stream.read_exact(&mut data).await {
                    Ok(_) => {
                        if data[0] == n {
                            println!("{} Reply is ok!", id);
                        } else {
                            let text = from_utf8(&data).unwrap();
                            println!("{:1} Unexpected reply: {:2}", id, text);
                        }
                    },
                    Err(e) => {
                        println!("Failed to receive data: {}", e);
                    }
                }
            },
            None => {
                // do nothing
            }
        }
    }    
}

async fn push_msg_to_channel(mut socket: tokio::net::TcpStream, 
    mut channels_tx_map_copy: std::collections::HashMap<u32, mpsc::Sender<[u8; 1024]>>) {
    let mut buf = [0; 1024];

    // In a loop, read data from the socket and write the data back.
    loop {
        let n = socket
            .read(&mut buf)
            .await
            .expect("failed to read data from socket to the Generator");

        if n == 0 {
            return;
        }

        // Dispatch the message to receiver thread by the first byte
        let dispath_num = buf[0] as u32;
        println!("get dispath num {}", dispath_num);
        let tx_copy = channels_tx_map_copy.get_mut(&dispath_num).unwrap();
        if let Err(_) = tx_copy.send(buf).await {
            println!("receiver thread dropped");
            return;
        }

        // Send msg back to Generator
        let fix_len : usize = 10;
        socket
            .write_all(&buf[0..fix_len])
            .await
            .expect("failed to write data to socket to the Generator");
    }    
}

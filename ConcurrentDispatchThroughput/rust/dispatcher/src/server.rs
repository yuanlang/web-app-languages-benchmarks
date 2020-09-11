//! The server of the dispatcher model
//!
//! This server will create a TCP listener, accept connections in a loop, and
//! dispatch everything to connected receivers.
//!

#![warn(rust_2018_idioms)]

#[macro_use] extern crate log;

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

#[derive(Debug)]
struct Connector {
    id: u32, 
    stream: tokio::net::TcpStream, 
    rx_from_master: tokio::sync::oneshot::Receiver<Command>,
    channels_tx_map: std::collections::HashMap<u32, mpsc::Sender<[u8; MSG_LEN]>>,
    counter: Arc<Mutex<usize>>
}

impl Connector {
    /// Send received msg to message channel
    /// 
    /// Receive message from the Generator and send the message to the corresponed receiver
    /// by using the first byte of the message
    async fn run(mut self) {
        // Wait master send Start message
        match self.rx_from_master.await {
            Ok(cmd) => {
                match cmd {
                    Command::Start => {
                        info!("{} recv start msg from master", self.id);
                        // send tcp message to generator
                        let mut send_bytes : Vec<u8> = (0..MSG_LEN).map(|_| { rand::random::<u8>() }).collect();
                        send_bytes[0] = Command::Start as u8;
                        self.stream.write(&send_bytes).await.unwrap();
                        self.stream.flush().await.unwrap();
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
            let n = self.stream
                .read(&mut buf)
                .await
                .expect("failed to read data from socket connected to the Generator");

            if n == 0 {
                info!("receive zero length message {}", self.id);
                return;
            }

            // parse the message
            let cmd: Command = buf[0].into();
            match cmd {
                // Command::Start => {
                //     // if the message is a start message, it will record as start time
                //     info!("{} receive start message", connection_id)
                // },
                Command::Data => {
                    // Dispatch the message to receiver thread by the first byte
                    let dispath_num = buf[1] as u32;
                    debug!("{} Get dispath num {}", self.id, dispath_num);
                    match self.channels_tx_map.get_mut(&dispath_num) {
                        Some(tx_copy) => {
                            if let Err(_) = tx_copy.send(buf).await {
                                info!("receiver thread dropped");
                                return;
                            }
                        },
                        None => {
                            error!("Get the wrong dispatch number {}", dispath_num);
                        }
                    }

                    //increase the counter
                    let mut num = self.counter.lock().unwrap();
                    *num += 1;
                },
                Command::Done => {
                    info!("{} receive done message", self.id);
                    // stream.shutdown(Shutdown::Both).expect("shutdown write failed");
                    break;
                },
                _ => {
                    // do nothing
                },
            }

        }    
    }
}

#[derive(Debug)]
struct Dispatcher {
    id: usize, 
    rx: mpsc::Receiver<[u8; MSG_LEN]>, 
    stream: TcpStream,
    counter: Arc<Mutex<usize>>
}

impl Dispatcher {
    /// Dispatch message to Receivers
    ///
    /// Receive messages from channel and dispatch them to the Receiver through 
    /// the TCP connnections
    async fn run(&mut self) {
        loop {
            // Read message from the channel and wait replay
            match self.rx.recv().await {
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
                            match self.stream.write(&msg).await {
                                Ok(_) => {
                                    debug!("Sent msg to No.{} Receiver.", self.id);
                                },
                                Err(e) => {
                                    error!("Failed to write data through socket: {}", e);
                                }
                            }
                            self.stream.flush().await.unwrap();
                            
                            //increase the counter
                            let mut num = self.counter.lock().unwrap();
                            *num += 1;
                        },
                        Command::Done  => {
                            // if this is the last Done message, quit current thread
                            info!("dispatcher receive done message");
                            self.stream.shutdown(Shutdown::Both).expect("shutdown call failed");
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .format_timestamp_micros()
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        error!("invalid number of connections");
        error!("cargo run connect_num");
        std::process::exit(0);
    }

    let arg1 = &args[1];
    let connect_num: u32 = arg1.parse().expect("Not a number!");
    info!("Connection number: {}", connect_num);

    let mut settings = config::Config::default();
    settings
        // Add in `./Settings.toml`
        .merge(config::File::with_name("Settings")).unwrap()
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .merge(config::Environment::with_prefix("APP")).unwrap();

    // Print out our settings (as a HashMap)
    let hash_setting = settings.try_into::<HashMap<String, String>>().unwrap();
    info!("{:?}", hash_setting);

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
                info!("{} Successfully connected to server {}", curr, &server_addr);
                let mut dispatcher = Dispatcher {
                    id: curr,
                    rx: rx1, 
                    stream: stream,
                    counter: disp_mutex
                };
                // do dispatch work
                disp_thread_holder.push(tokio::spawn(async move {
                    dispatcher.run().await;
                }));
            },
            Err(e) => {
                error!("Failed to connect: {}", e);
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
    info!("Listening on: {}", addr);

    // Process incomming connections
    let mut recv_thread_holder: Vec<JoinHandle<()>> = Vec::new();
    let recv_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let mut connection_id = 1;
    let mut tx_by_master_vec: Vec<tokio::sync::oneshot::Sender<dispatcher::Command>> = Vec::new();
    while connection_id < connect_num + 1 {
        let recv_mutex = Arc::clone(&recv_counter);
        // Asynchronously wait for an inbound socket.
        let (socket, _) = listener.accept().await?;

        info!("recv connnection, id {}", connection_id);

        // get a copy of senders hashmap, to allow the ownership move to 
        // the tokio thread and doesn't influence others
        let channels_tx_map_copy = channels_tx_map.clone();
        let (sender, receiver) = oneshot::channel::<Command>();
        tx_by_master_vec.push(sender);

        let connector = Connector {
            id: connection_id, 
            stream: socket, 
            rx_from_master: receiver,
            channels_tx_map: channels_tx_map_copy,
            counter: recv_mutex
        };

        // all clients to make progress concurrently, rather than
        // blocking one on completion of another. To achieve this we use the
        // `tokio::spawn` function to execute the work in the background.
        recv_thread_holder.push(tokio::spawn(async move {
            connector.run().await;
        }));
        connection_id += 1;
    }

    let start = Instant::now();

    //everything is ok, send start message to connect
    for tx in tx_by_master_vec {
        if let Err(_) = tx.send(Command::Start) {
            error!("the receiver dropped");
        }
    }

    for h in recv_thread_holder {
        let _r1 = h.await;
    }
    info!("all recv thread exit");

    // // wait all connector and dispather threads quit
    // for h in disp_thread_holder {
    //     let _r1 = h.await;
    // }
    // info!("all disp thread exit");

    // get lock, copy the counters
    let recv_result = *recv_counter.lock().unwrap();
    let disp_result = *disp_counter.lock().unwrap();

    let duration = start.elapsed();

    info!("Total time taken seconds: {:?} ", duration.as_secs_f64());
    info!("Total received messages: {} ", recv_result);
    info!("Total dispatched messages: {} ", disp_result);

    thread::sleep(Duration::from_secs(5));

    Ok(())
}

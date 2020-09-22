//! The server of the dispatcher model
//!
//! This server will create a TCP listener, accept connections in a loop, and
//! dispatch everything to connected receivers.
//!

#![warn(rust_2018_idioms)]

use tokio::net::{TcpListener};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::net::{TcpStream};
use tokio::task::JoinHandle;
use std::collections::VecDeque;

use std::env;
use std::error::Error;
use std::collections::HashMap;

use std::sync::{Arc, Mutex};
use std::time::{Instant};

use log::{debug, info, error};

use dispatcher::{Command, MSG_LEN, DEFAULT_SERVER_ADDR, DEFAULT_RECV_ADDR};
use dispatcher::Dispatcher;
use dispatcher::Connector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .format_timestamp_micros()
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        error!("invalid number of connectors or dispatchers");
        error!("cargo run connector_num dispatcher_num");
        std::process::exit(0);
    }

    let arg1 = &args[1];
    let connect_num: u32 = arg1.parse().expect("Not a number!");
    info!("Connection number: {}", connect_num);

    let arg2 = &args[2];
    let receiver_num: u32 = arg2.parse().expect("Not a number!");
    info!("Receiver number: {}", receiver_num);

    // Allow passing an address to listen on as the first argument of this
    // program, but otherwise we'll just set up our TCP listener on
    // 127.0.0.1:8888 for connections.
    let listen_addr = env::args()
        .nth(3)
        .unwrap_or_else(|| DEFAULT_SERVER_ADDR.to_string());

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
    let (done_sender, mut done_receiver) = mpsc::channel::<Command>(1);
    while let Some(rx1) = channels_rx_vec.pop_front() {
        // let server_name = format!("{}{}", "server" , curr);
        // let server_addr = hash_setting[&server_name].clone();
        let disp_mutex = Arc::clone(&disp_counter);
        let tx1 = done_sender.clone();

        match TcpStream::connect(&DEFAULT_RECV_ADDR).await {
            Ok(stream) => {
                info!("{} Successfully connected to receiver {}", curr, &DEFAULT_RECV_ADDR);
                let mut dispatcher = Dispatcher::new(
                    curr,
                    rx1,
                    tx1,
                    stream,
                    disp_mutex
                );
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

    // Create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop.
    let mut listener = TcpListener::bind(&listen_addr).await?;
    info!("Listening on: {}", listen_addr);

    // Process incomming connections
    let mut recv_thread_holder: Vec<JoinHandle<()>> = Vec::new();
    let recv_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let mut connection_id: u32 = 1;
    let mut tx_by_master_vec: Vec<tokio::sync::oneshot::Sender<dispatcher::Command>> = Vec::new();
    while connection_id < connect_num + 1 {
        let recv_mutex = Arc::clone(&recv_counter);
        // Asynchronously wait for an inbound socket.
        let (socket, _) = listener.accept().await?;

        debug!("recv connnection, id {}", connection_id);

        // get a copy of senders hashmap, to allow the ownership move to 
        // the tokio thread and doesn't influence others
        let channels_tx_map_copy = channels_tx_map.clone();
        let (sender, receiver) = oneshot::channel::<Command>();
        tx_by_master_vec.push(sender);

        let connector = Connector::new(
            connection_id, 
            socket, 
            receiver,
            channels_tx_map_copy,
            recv_mutex
        );

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

    // recv message from dispatcher
    loop {
        // Read message from the channel and wait replay
        match done_receiver.recv().await {
            Some(cmd) => {
                match cmd {
                    Command::Done  => {
                        info!("receive done message from dispatcher");
                        // send done to all dispatchers
                        for (_, tx1) in channels_tx_map.iter_mut() {
                            let mut buf = [0u8; MSG_LEN];
                            buf[0] = Command::Done as u8;
                            if let Err(_) = tx1.send(buf).await {
                                error!("cannot send message");
                            }
                        }
                        break;
                    },
                    _ => {
                        // do nothing
                        info!("receive other message");
                    },
                }
            },
            None => {
                info!("receive none message");
            }
        }
    }

    drop(channels_tx_map);
    
    // get lock, copy the counters
    let recv_result = *recv_counter.lock().unwrap();
    let disp_result = *disp_counter.lock().unwrap();

    let duration = start.elapsed();

    info!("Total time taken seconds: {:?} ", duration.as_secs_f64());
    info!("Total received messages: {} ", recv_result);
    info!("Total dispatched messages: {} ", disp_result);

    // wait all dispather threads quit
    for h in disp_thread_holder {
        let _r1 = h.await;
    }
    info!("all dispatch thread exit");


    // // wait all connector threads quit
    // for h in recv_thread_holder {
    //     let _r1 = h.await;
    // }
    // info!("all recv thread exit");

    Ok(())
}

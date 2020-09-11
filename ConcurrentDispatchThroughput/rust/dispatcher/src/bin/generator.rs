//! The generators of the dispatcher model
//!
//! This generators will create a group of TCP connection with server, 
//! and send message as quickly as they can
//!
#[macro_use] extern crate log;

use std::env;
use std::net::TcpStream;
use std::error::Error;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant};

use dispatcher::{DEFAULT_SERVER_ADDR};
use dispatcher::Generator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .format_timestamp_micros()
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        error!("invalid number of connections");
        error!("cargo run connect_num receiver_num repeat_num");
        std::process::exit(0);
    }

    let arg1 = &args[1];
    let connect_num: u8 = arg1.parse().expect("Not a number!");
    info!("Connection number: {}", connect_num);

    let arg2 = &args[2];
    let receiver_num: u8 = arg2.parse().expect("Not a number!");
    info!("Receiver number: {}", receiver_num);

    let arg3 = &args[3];
    let repeat_num: u32 = arg3.parse().expect("Not a number!");
    info!("Repeat number: {}", repeat_num);

    let disp_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let start = Instant::now();
    let mut gene_thread_holder = Vec::new();
    for curr in 1 .. connect_num + 1 {
        match TcpStream::connect(&DEFAULT_SERVER_ADDR) {
            Ok(stream) => {
                info!("No.{} Successfully connected to server in port 8888", curr);
                let disp_mutex = Arc::clone(&disp_counter);
                let send_num = repeat_num / connect_num as u32;
                let mut generator = Generator::new(
                    curr, 
                    send_num, 
                    stream, 
                    disp_mutex,
                    receiver_num
                );
                gene_thread_holder.push(thread::spawn(move || {
                    generator.run().unwrap();
                }));
            },
            Err(e) => {
                error!("Failed to connect: {}", e);
            }
        }
    }

    // thread::sleep(Duration::from_secs(6));
    for h in gene_thread_holder {
        h.join().unwrap();
    }
    info!("all generator threads exit");

    let result = *disp_counter.lock().unwrap();
    let duration = start.elapsed();

    info!("Total time taken in generator (seconds): {:?} ", duration.as_secs_f64());
    info!("Total sent messages: {} ", result);

    Ok(())
}

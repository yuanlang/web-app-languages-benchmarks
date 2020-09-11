//! The generators of the dispatcher model
//!
//! This generators will create a group of TCP connection with server, 
//! and send message as quickly as they can
//!
#[macro_use] extern crate log;

use std::env;
use std::net::{Shutdown};
use std::net::TcpStream;
use std::io::{Read, Write};
// use tokio::sync::broadcast;
// use std::io::{Write};
// use rand::{thread_rng, Rng};
use std::error::Error;
// use tokio::task::JoinHandle;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant};
// use std::time::Duration;

use dispatcher::{Command, MSG_LEN, DEFAULT_SERVER_ADDR};

#[derive(Debug)]
struct Generator {
    id: u8, 
    repeat_num: u32, 
    stream: TcpStream, 
    counter: Arc<Mutex<usize>>,
    receiver_num: u8,
}

impl Generator {
    /// Repeat send Data message to the Server
    fn repeat_send(&mut self) -> Result<(), Box<dyn Error>>  {
        // start the send work
        for _i in 0 .. self.repeat_num {
            let mut send_bytes : Vec<u8> = (0..MSG_LEN).map(|_| { rand::random::<u8>() }).collect();

            // gen the msg type
            // let mut rng = thread_rng();
            let n: u8 = rand::random::<u8>() % self.receiver_num + 1;

            send_bytes[0] = Command::Data as u8;
            send_bytes[1] = n;
            self.stream.write(&send_bytes).unwrap();
            self.stream.flush().unwrap();
            debug!("{} Sent msg to No.{} receiver ", self.id, n);

            //increase the counter
            let mut num = self.counter.lock().unwrap();
            *num += 1;
        }

        Ok(())
    }

    /// Send the Done message to the Server after all generator work done
    fn send_done(&mut self) -> Result<(), Box<dyn Error>>  {
        let mut send_done : Vec<u8> = Vec::new();
        send_done.push(Command::Done as u8);
        self.stream.write(&send_done).unwrap();
        self.stream.flush().unwrap();

        self.stream.shutdown(Shutdown::Write).expect("shutdown write failed");
        Ok(())
    }    

    /// the main work of generator
    fn run(&mut self) -> Result<(), Box<dyn Error>>  {
        // wait recv start message from server
        let mut buf = [0u8; MSG_LEN];
        let n = self.stream.read(&mut buf).unwrap();

        if n == 0 {
            info!("receive zero length message {}", self.id);
        }
            
        info!("{} receive start message from server", self.id);

        // start the send work
        self.repeat_send().unwrap();

        // send the Done command to server
        self.send_done().unwrap();

        // wait for 10 seconds
        // thread::sleep(Duration::from_secs(5));

        Ok(())
    }
}

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
                let mut generator = Generator {
                    id: curr, 
                    repeat_num: send_num, 
                    stream: stream, 
                    counter: disp_mutex,
                    receiver_num: receiver_num,
                };
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

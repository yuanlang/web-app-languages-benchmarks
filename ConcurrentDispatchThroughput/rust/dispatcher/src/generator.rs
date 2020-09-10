//! The generators of the dispatcher model
//!
//! This generators will create a group of TCP connection with server, 
//! and send message as quickly as they can
//!

use std::env;
use std::net::{Shutdown};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use std::io::{Write};
// use rand::{thread_rng, Rng};
use std::error::Error;
use tokio::task::JoinHandle;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant};
use std::time::Duration;
use log::{debug};

use dispatcher::{Command, MSG_LEN, DEFAULT_SERVER_ADDR};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!("invalid number of connections");
        println!("cargo run connect_num receiver_num repeat_num");
        std::process::exit(0);
    }

    let arg1 = &args[1];
    let connect_num: u8 = arg1.parse().expect("Not a number!");
    println!("Connection number: {}", connect_num);

    let arg2 = &args[2];
    let receiver_num: u8 = arg2.parse().expect("Not a number!");
    println!("Receiver number: {}", receiver_num);

    let arg3 = &args[3];
    let repeat_num: u32 = arg3.parse().expect("Not a number!");
    println!("Repeat number: {}", repeat_num);

    let disp_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let start = Instant::now();
    let mut gene_thread_holder: Vec<JoinHandle<()>> = Vec::new();
    for curr in 0 .. connect_num {
        match TcpStream::connect(&DEFAULT_SERVER_ADDR).await {
            Ok(stream) => {
                println!("No.{} Successfully connected to server in port 8888", curr);
                let disp_mutex = Arc::clone(&disp_counter);
                gene_thread_holder.push(tokio::spawn(async move {
                    do_send_and_close(curr+1, repeat_num / connect_num as u32, stream, disp_mutex, receiver_num).await;
                }));
            },
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
    }

    // thread::sleep(Duration::from_secs(6));
    for h in gene_thread_holder {
        let _r1 = h.await;
    }
    println!("all generator threads exit");

    let result = *disp_counter.lock().unwrap();
    let duration = start.elapsed();

    println!("Total time taken seconds: {:?} ", duration.as_secs_f64());
    println!("Total sent messages: {} ", result);

    Ok(())
}

async fn do_send_and_close(generator_id: u8, 
        repeat_num: u32, 
        mut stream: TcpStream, 
        counter: Arc<Mutex<usize>>,
        receiver_num: u8) {
    // wait recv start message from server
    let mut buf = [0u8; MSG_LEN];
    let n = stream
            .read(&mut buf)
            .await
            .expect("failed to read data from socket connected to the Generator");


    if n == 0 {
        println!("receive zero length message {}", generator_id);
        return;
    }

    println!("{} receive start message from server", generator_id);

    // let mut send_start : Vec<u8> = Vec::new();
    // send_start.push(Command::Start as u8);
    // stream.write(&send_start).await.unwrap();
    // stream.flush().await.unwrap();

    // start the send work
    for _i in 0 .. repeat_num {
        let mut send_bytes : Vec<u8> = (0..MSG_LEN).map(|_| { rand::random::<u8>() }).collect();

        // gen the msg type
        // let mut rng = thread_rng();
        let n: u8 = rand::random::<u8>() % receiver_num + 1;

        send_bytes[0] = Command::Data as u8;
        send_bytes[1] = n;
        stream.write(&send_bytes).await.unwrap();
        stream.flush().await.unwrap();
        debug!("{} Sent msg to No.{} receiver ", generator_id, n);

        //increase the counter
        let mut num = counter.lock().unwrap();
        *num += 1;
    }

    // send the Done command to server
    let mut send_done : Vec<u8> = Vec::new();
    send_done.push(Command::Done as u8);
    stream.write(&send_done).await.unwrap();
    stream.flush().await.unwrap();

    stream.shutdown(Shutdown::Write).expect("shutdown write failed");

    // wait for 10 seconds
    thread::sleep(Duration::from_secs(5));

}
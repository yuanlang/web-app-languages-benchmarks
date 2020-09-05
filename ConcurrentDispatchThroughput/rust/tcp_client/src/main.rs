use std::env;
use std::net::{Shutdown, TcpStream};
use std::io::{Write};
use rand::{thread_rng, Rng};
use std::error::Error;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant};
use std::time::Duration;

const MSG_LEN: usize = 500; //data length

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("invalid number of connections");
        println!("cargo run connect_num");
        std::process::exit(0);
    }

    let arg1 = &args[1];
    let connect_num: u32 = arg1.parse().expect("Not a number!");
    println!("Connection number: {}", connect_num);

    let arg2 = &args[2];
    let repeat_num: u32 = arg2.parse().expect("Not a number!");
    println!("Repeat number: {}", repeat_num);

    let disp_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let start = Instant::now();

    for curr in 0 .. connect_num {
        match TcpStream::connect("localhost:8888") {
            Ok(stream) => {
                println!("No.{} Successfully connected to server in port 8888", curr);
                let disp_mutex = Arc::clone(&disp_counter);
                tokio::spawn(async move {
                    do_send_and_close(repeat_num / connect_num, stream, disp_mutex).await;
                });
            },
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
    }
    thread::sleep(Duration::from_secs(6));

    let result = *disp_counter.lock().unwrap();
    let duration = start.elapsed();

    println!("Total time taken seconds: {:?} ", duration.as_secs_f64());
    println!("Total sent messages: {} ", result);

    Ok(())
}

async fn do_send_and_close(repeat_num: u32, 
        mut stream: TcpStream, 
        counter: Arc<Mutex<usize>>) {
    for _i in 0 .. repeat_num {
        let mut send_bytes : Vec<u8> = (0..MSG_LEN).map(|_| { rand::random::<u8>() }).collect();

        // gen the msg type
        let mut rng = thread_rng();
        let n: u8 = rng.gen_range(1, 11);

        send_bytes[0] = n;
        stream.write(&send_bytes).unwrap();
        println!("Sent msg to No.{} receiver ", n);

        //increase the counter
        let mut num = counter.lock().unwrap();
        *num += 1;
    }

    stream.shutdown(Shutdown::Both).expect("shutdown call failed");
}
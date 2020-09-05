use std::env;
use std::net::{Shutdown, TcpStream};
use std::io::{Write};
use rand::{thread_rng, Rng};
use std::error::Error;

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

    for curr in 0 .. connect_num {
        match TcpStream::connect("localhost:8888") {
            Ok(stream) => {
                println!("No.{} Successfully connected to server in port 8888", curr);
                tokio::spawn(async move {
                    do_send_and_close(repeat_num / connect_num, stream).await;
                });
            },
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
    }
    println!("Terminated.");
    Ok(())
}

async fn do_send_and_close(repeat_num: u32, mut stream: TcpStream) {
    for _i in 0 .. repeat_num {
        let msg_len = 500; //data length
        let mut send_bytes : Vec<u8> = (0..msg_len).map(|_| { rand::random::<u8>() }).collect();

        // gen the msg type
        let mut rng = thread_rng();
        let n: u8 = rng.gen_range(1, 11);

        send_bytes[0] = n;
        stream.write(&send_bytes).unwrap();
        println!("Sent msg to No.{} receiver ", n);
    }

    stream.shutdown(Shutdown::Both).expect("shutdown call failed");
}
use std::env;
use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;
use rand::{thread_rng, Rng};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print!("invalid number of connections\n");
        print!("cargo run connnect_num\n");
        std::process::exit(0);
    }

    let arg1 = &args[1];
    let n: u32 = arg1.parse().expect("Not a number!");
    println!("Connection number: {}", n);

    let msg_len = 500; //data length
    let mut send_bytes : Vec<u8> = (0..msg_len).map(|_| { rand::random::<u8>() }).collect();

    match TcpStream::connect("localhost:8080") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 8080");

            // gen the msg type
            let mut rng = thread_rng();
            let n: u8 = rng.gen_range(0, 10);

            send_bytes[0] = n;
            stream.write(&send_bytes).unwrap();
            println!("Sent msg, awaiting reply...");

            let mut data = [0 as u8; 6]; // using 6 byte buffer
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if data[0] == n {
                        println!("Reply is ok!");
                    } else {
                        let text = from_utf8(&data).unwrap();
                        println!("Unexpected reply: {}", text);
                    }
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}

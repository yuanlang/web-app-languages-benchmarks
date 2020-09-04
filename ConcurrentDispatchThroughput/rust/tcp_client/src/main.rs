use std::env;
use std::net::{Shutdown, TcpStream};
use std::io::{Write};
use rand::{thread_rng, Rng};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print!("invalid number of connections\n");
        print!("cargo run connect_num\n");
        std::process::exit(0);
    }

    let arg1 = &args[1];
    let connect_num: u32 = arg1.parse().expect("Not a number!");
    println!("Connection number: {}", connect_num);

    let arg2 = &args[2];
    let repeat_num: u32 = arg2.parse().expect("Not a number!");
    println!("Repeat number: {}", repeat_num);

    let msg_len = 500; //data length
    let mut send_bytes : Vec<u8> = (0..msg_len).map(|_| { rand::random::<u8>() }).collect();

    for curr in 0 .. connect_num {
        match TcpStream::connect("localhost:8888") {
            Ok(mut stream) => {
                println!("{} Successfully connected to server in port 8080", curr);
                
                for _i in 0 .. repeat_num {
                    // gen the msg type
                    let mut rng = thread_rng();
                    let n: u8 = rng.gen_range(1, 11);

                    send_bytes[0] = n;
                    stream.write(&send_bytes).unwrap();
                    println!("Sent msg to No.{} receiver ", curr);
                }

                stream.shutdown(Shutdown::Both).expect("shutdown call failed");
            },
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
    }
    println!("Terminated.");
}

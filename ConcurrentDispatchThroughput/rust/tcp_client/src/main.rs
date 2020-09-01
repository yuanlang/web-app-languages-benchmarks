use std::env;
use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;
use rand::{thread_rng, Rng};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print!("invalid number of connections\n");
        print!("cargo run connect_num\n");
        std::process::exit(0);
    }

    let arg1 = &args[1];
    let connect_num: u32 = arg1.parse().expect("Not a number!");
    println!("Connection number: {}", connect_num);

    let msg_len = 500; //data length
    let mut send_bytes : Vec<u8> = (0..msg_len).map(|_| { rand::random::<u8>() }).collect();

    for i in 0 .. connect_num {
        let curr = i;
        match TcpStream::connect("localhost:8080") {
            Ok(mut stream) => {
                println!("{} Successfully connected to server in port 8080", curr);

                // gen the msg type
                let mut rng = thread_rng();
                let n: u8 = rng.gen_range(0, 10);

                send_bytes[0] = n;
                stream.write(&send_bytes).unwrap();
                println!("{} Sent msg, awaiting reply...", curr);

                let mut data = [0 as u8; 6]; // using 6 byte buffer
                match stream.read_exact(&mut data) {
                    Ok(_) => {
                        if data[0] == n {
                            println!("{} Reply is ok!", curr);
                        } else {
                            let text = from_utf8(&data).unwrap();
                            println!("{:1} Unexpected reply: {:2}", curr, text);
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
    }
    println!("Terminated.");
}

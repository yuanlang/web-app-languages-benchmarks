//! The receiver of the dispatcher model
//!
//! This receiver will create a TCP listener, accept connections in a loop, 
//! and do nothing
//!

#![warn(rust_2018_idioms)]

use std::io::{Read};
use std::net::TcpListener;
use std::thread;

use std::env;
use log::{debug, info, error};

use dispatcher::{DEFAULT_RECV_ADDR};

fn main() {
    env_logger::builder()
        .format_timestamp_micros()
        .init();

    // Allow passing an address to listen on as the first argument of this
    // program, but otherwise we'll just set up our TCP listener on
    // 127.0.0.1:14001 for connections.
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_RECV_ADDR.to_string());

    // Next up we create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop.
    let listener = TcpListener::bind(&addr).unwrap();
    info!("Listening on: {}", addr);

    // Asynchronously wait for an inbound socket.
    for socket in listener.incoming() {
        match socket {
            Ok(mut stream) => {
                debug!("new connection");
                thread::spawn(move || {
                    let mut buf = [0; 1024];

                    // In a loop, read data from the socket and write the data back.
                    loop {
                        let n = stream
                            .read(&mut buf)
                            .expect("failed to read data from socket");

                        if n == 0 {
                            info!("socket closed!");
                            return;
                        }

                    }
                });
            }
            Err(e) => { 
                // connection failed
                error!("connection failed {}", e);
            }
        }
    }

}
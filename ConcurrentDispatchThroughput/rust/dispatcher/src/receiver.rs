//! The receiver of the dispatcher model
//!
//! This receiver will create a TCP listener, accept connections in a loop, 
//! and do nothing
//!

#![warn(rust_2018_idioms)]

use tokio::io::{AsyncReadExt};
use tokio::net::TcpListener;

use std::env;
use std::error::Error;

use dispatcher::{DEFAULT_RECV_ADDR};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Allow passing an address to listen on as the first argument of this
    // program, but otherwise we'll just set up our TCP listener on
    // 127.0.0.1:14001 for connections.
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_RECV_ADDR.to_string());

    // Next up we create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop.
    let mut listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?;

        // And this is where much of the magic of this server happens. We
        // crucially want all clients to make progress concurrently, rather than
        // blocking one on completion of another. To achieve this we use the
        // `tokio::spawn` function to execute the work in the background.
        //
        // Essentially here we're executing a new task to run concurrently,
        // which will allow all of our clients to be processed concurrently.

        tokio::spawn(async move {
            let mut buf = [0; 1024];
            // let mut vec : Vec<u8> = Vec::new();

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    println!("msg length error!");
                    return;
                }

            }
        });
    }
}
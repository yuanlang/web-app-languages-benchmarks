use crate::{Command, MSG_LEN};
use tokio::sync::mpsc;
// use tokio::sync::oneshot;
use tokio::net::{TcpStream};
use std::sync::{Arc, Mutex};
use log::{debug, error, info};
use tokio::io::{AsyncWriteExt};
use std::net::Shutdown;

#[derive(Debug)]
pub struct Dispatcher {
    id: usize, 
    rx: mpsc::Receiver<[u8; MSG_LEN]>, 
    stream: TcpStream,
    counter: Arc<Mutex<usize>>
}

impl Dispatcher {
    /// Create a new dispatcher
    pub fn new(id: usize, 
        rx: mpsc::Receiver<[u8; MSG_LEN]>, 
        stream: TcpStream,
        counter: Arc<Mutex<usize>>) -> Dispatcher {
            Dispatcher {
                id : id,
                rx: rx,
                stream: stream,
                counter: counter
            }
        }

    /// Dispatch message to Receivers
    ///
    /// Receive messages from channel and dispatch them to the Receiver through 
    /// the TCP connnections
    pub async fn run(&mut self) {
        loop {
            // Read message from the channel and wait replay
            match self.rx.recv().await {
                Some(msg) => {
                    // parse the message
                    let t = msg[0];
                    let cmd: Command = t.into();
                    debug!("got command type: {}", t);

                    match cmd {
                        Command::Start => {
                            //do nothing
                        },
                        Command::Data  => {
                            match self.stream.write(&msg).await {
                                Ok(_) => {
                                    debug!("Sent msg to No.{} Receiver.", self.id);
                                },
                                Err(e) => {
                                    error!("Failed to write data through socket: {}", e);
                                }
                            }
                            self.stream.flush().await.unwrap();
                            
                            //increase the counter
                            let mut num = self.counter.lock().unwrap();
                            *num += 1;
                        },
                        Command::Done  => {
                            // ! cannnot guarantee every dispatcher could receive the Done message 
                            // if this is the last Done message, quit current thread
                            info!("dispatcher receive done message");
                            // self.stream.shutdown(Shutdown::Both).expect("shutdown call failed");
                            // break;
                        },
                        Command::Unknown => {
                            // do nothing
                            info!("receive an unknow message");
                        },
                    }
                },
                None => {
                    info!("receive none message");
                    // dispatcher will receive this message when all sender droped
                    self.stream.shutdown(Shutdown::Both).expect("shutdown call failed");
                    break;
                }
            }
        }    
    }
}
use crate::{Command, MSG_LEN, TIMESTAMP_LEN, parse_timestamp, TIMEOUT_THRESHOLD};
use tokio::sync::mpsc;
// use tokio::sync::oneshot;
use tokio::net::{TcpStream};
use std::sync::{Arc, Mutex};
use log::{debug, error, info, warn};
use tokio::io::{AsyncWriteExt};
use std::net::Shutdown;
use std::str;
use std::time::SystemTime;

#[derive(Debug)]
pub struct Dispatcher {
    id: usize, 
    rx: mpsc::Receiver<[u8; MSG_LEN]>, 
    tx: tokio::sync::mpsc::Sender<Command>,
    stream: TcpStream,
    counter: Arc<Mutex<usize>>
}

impl Dispatcher {
    /// Create a new dispatcher
    pub fn new(id: usize, 
        rx: mpsc::Receiver<[u8; MSG_LEN]>, 
        tx: tokio::sync::mpsc::Sender<Command>,
        stream: TcpStream,
        counter: Arc<Mutex<usize>>) -> Dispatcher {
            Dispatcher {
                id : id,
                rx: rx,
                tx: tx,
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
                    let id = msg[1];
                    debug!("{} got command type: {}", id, t);

                    match cmd {
                        Command::Start => {
                            //do nothing
                        },
                        Command::Data  => {
                            // get msg timestamp
                            let mut buf = [0u8; TIMESTAMP_LEN];
                            buf.copy_from_slice(&msg[2.. 2+TIMESTAMP_LEN]);
                            let ts = str::from_utf8(&buf).unwrap();
                            debug!("{} msg time stemp {}", id, ts);

                            // got the time difference
                            let systime = parse_timestamp(ts);
                            match SystemTime::now().duration_since(systime) {
                                Ok(n)  => {
                                    let diff = n.as_micros() as f32 / 1000000.0;
                                    info!("msg dispatched, id: {} spend: {} seconds from: {}!", id, diff, ts);
                                    if diff > TIMEOUT_THRESHOLD {
                                        warn!("threshold {} seconds reached!", TIMEOUT_THRESHOLD);
                                        if let Err(_) = self.tx.send(Command::Done).await {
                                            error!("cannot send message");
                                        }
                                        // it doesn't work, terminate it
                                        // return;
                                    }
                                },
                                Err(_) => {
                                    error!("SystemTime before start time!")
                                },
                            }

                            // write msg to socket
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
                            info!("{} dispatcher receive done message", self.id);
                            self.stream.shutdown(Shutdown::Both).expect("shutdown call failed");
                            break;
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

use crate::{Command, MSG_LEN};
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};
use log::{debug, error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug)]
pub struct Connector {
    id: u32, 
    stream: tokio::net::TcpStream, 
    rx_from_master: tokio::sync::oneshot::Receiver<Command>,
    channels_tx_map: std::collections::HashMap<u32, mpsc::Sender<[u8; MSG_LEN]>>,
    counter: Arc<Mutex<usize>>
}

impl Connector {
    /// Create a new `Connector`
    pub fn new(id: u32, 
            stream: tokio::net::TcpStream, 
            rx_from_master: tokio::sync::oneshot::Receiver<Command>,
            channels_tx_map: std::collections::HashMap<u32, mpsc::Sender<[u8; MSG_LEN]>>,
            counter: Arc<Mutex<usize>>) -> Connector {
        Connector {
            id: id,
            stream: stream,
            rx_from_master: rx_from_master,
            channels_tx_map: channels_tx_map,
            counter: counter
        }
    }

    /// Send received msg to message channel
    /// 
    /// Receive message from the Generator and send the message to the corresponed receiver
    /// by using the first byte of the message
    pub async fn run(mut self) {
        // Wait master send Start message
        match self.rx_from_master.await {
            Ok(cmd) => {
                match cmd {
                    Command::Start => {
                        info!("{} recv start msg from master", self.id);
                        // send tcp message to generator
                        let mut send_bytes : Vec<u8> = (0..MSG_LEN).map(|_| { rand::random::<u8>() }).collect();
                        send_bytes[0] = Command::Start as u8;
                        self.stream.write(&send_bytes).await.unwrap();
                        self.stream.flush().await.unwrap();
                    },
                    _ => {
                        //do nothing
                    },
                }
            },
            Err(_err) => {
                // do nothing
            },
        }

        // In a loop, read data from the socket and write the data back.
        loop {
            let mut buf = [0u8; MSG_LEN];
            let n = self.stream
                .read(&mut buf)
                .await
                .expect("failed to read data from socket connected to the Generator");

            if n == 0 {
                info!("receive zero length message {}", self.id);
                return;
            }

            // parse the message
            let cmd: Command = buf[0].into();
            match cmd {
                // Command::Start => {
                //     // if the message is a start message, it will record as start time
                //     info!("{} receive start message", connection_id)
                // },
                Command::Data => {
                    // Dispatch the message to receiver thread by the first byte
                    let dispath_num = buf[1] as u32;
                    debug!("{} Get dispath num {}", self.id, dispath_num);
                    match self.channels_tx_map.get_mut(&dispath_num) {
                        Some(tx_copy) => {
                            if let Err(_) = tx_copy.send(buf).await {
                                info!("receiver thread dropped");
                                return;
                            }
                        },
                        None => {
                            error!("Get the wrong dispatch number {}", dispath_num);
                        }
                    }

                    //increase the counter
                    let mut num = self.counter.lock().unwrap();
                    *num += 1;
                },
                Command::Done => {
                    info!("{} receive done message", self.id);
                    // stream.shutdown(Shutdown::Both).expect("shutdown write failed");
                    // drop the channels_tx_map to notify the dispatcher I'm finished
                    drop(self.channels_tx_map);
                    break;
                },
                _ => {
                    // do nothing
                },
            }

        }    
    }
}
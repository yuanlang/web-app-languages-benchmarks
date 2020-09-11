use std::net::{Shutdown};
use std::net::TcpStream;
use std::io::{Read, Write};
use log::{debug, info, error};
use crate::{Command, MSG_LEN};
use std::sync::{Arc, Mutex};
use std::error::Error;


#[derive(Debug)]
pub struct Generator {
    id: u8, 
    repeat_num: u32, 
    stream: TcpStream, 
    counter: Arc<Mutex<usize>>,
    receiver_num: u8,
}

impl Generator {
    /// Create a new `Connector`
    pub fn new(id: u8, repeat_num: u32, 
    stream: TcpStream, 
    counter: Arc<Mutex<usize>>,
    receiver_num: u8) -> Generator {
        Generator {
            id: id,
            repeat_num: repeat_num,
            stream: stream,
            counter: counter,
            receiver_num: receiver_num
        }
    }

    /// Repeat send Data message to the Server
    fn repeat_send(&mut self) -> Result<(), Box<dyn Error>>  {
        // start the send work
        for _i in 0 .. self.repeat_num {
            let mut send_bytes : Vec<u8> = (0..MSG_LEN).map(|_| { rand::random::<u8>() }).collect();

            // gen the msg type
            // let mut rng = thread_rng();
            let n: u8 = rand::random::<u8>() % self.receiver_num + 1;

            send_bytes[0] = Command::Data as u8;
            send_bytes[1] = n;
            self.stream.write(&send_bytes).unwrap();
            self.stream.flush().unwrap();
            debug!("{} Sent msg to No.{} receiver ", self.id, n);

            //increase the counter
            let mut num = self.counter.lock().unwrap();
            *num += 1;
        }

        Ok(())
    }

    /// Send the Done message to the Server after all generator work done
    fn send_done(&mut self) -> Result<(), Box<dyn Error>>  {
        let mut send_done : Vec<u8> = Vec::new();
        send_done.push(Command::Done as u8);
        self.stream.write(&send_done).unwrap();
        self.stream.flush().unwrap();

        self.stream.shutdown(Shutdown::Write).expect("shutdown write failed");
        Ok(())
    }    

    /// the main work of generator
    pub fn run(&mut self) -> Result<(), Box<dyn Error>>  {
        // wait recv start message from server
        let mut buf = [0u8; MSG_LEN];
        let n = self.stream.read(&mut buf).unwrap();

        if n == 0 {
            error!("receive zero length message {}", self.id);
        }
            
        info!("{} receive start message from server", self.id);

        // start the send work
        self.repeat_send().unwrap();

        // send the Done command to server
        self.send_done().unwrap();

        // wait for 10 seconds
        // thread::sleep(Duration::from_secs(5));

        Ok(())
    }
}

// use tokio::net::TcpListener;
// use tokio::prelude::*;
use tokio::time;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::File;
use std::io::prelude::*;

fn is_print(n: i32) -> bool {
    if n < 192000000 {
         if n % 1000 == 1 {
             true
         } else {
             false
         }
    } else {
        true
    }
}

async fn task_sleep_long_time() {
    // sleep a long time
    time::delay_for(time::Duration::from_secs(10000)).await
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut file = File::create("test_tokio_limit.txt").unwrap();
    let mut i = 1;
    loop {
        tokio::spawn(async move {
            task_sleep_long_time().await;
        });
        
        if is_print(i) {
            let ts = SystemTime::now().duration_since(UNIX_EPOCH)
                              .expect("Clock may have gone backwards");
            writeln!(file, "ts(sec): {:?} thd no: {}", ts.as_secs(), i)?;
        }
        i += 1;
    }
    // return;
}

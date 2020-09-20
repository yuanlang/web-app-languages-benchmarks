//! A lib for dispatcher model benchmark 
//!
use std::fmt;
use std::time::SystemTime;
use humantime::{format_rfc3339_micros, parse_rfc3339};

pub mod dispatcher;
pub use dispatcher::Dispatcher;

pub mod connector;
pub use connector::Connector;

pub mod generator;
pub use generator::Generator;

/// Default port that the server listens on.
///
/// Used if no port is specified.
pub const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:8888";

/// Default receiver listening port
pub const DEFAULT_RECV_ADDR: &str = "127.0.0.1:14001";

/// default message length
pub const MSG_LEN: usize = 500;

/// Error returned by most functions.
///
/// When writing a real application, one might want to consider a specialized
/// error handling crate or defining an error type as an `enum` of causes.
/// However, for our example, using a boxed `std::error::Error` is sufficient.
///
/// For performance reasons, boxing is avoided in any hot path. For example, in
/// `parse`, a custom error `enum` is defined. This is because the error is hit
/// and handled during normal execution when a partial frame is received on a
/// socket. `std::error::Error` is implemented for `parse::Error` which allows
/// it to be converted to `Box<dyn std::error::Error>`.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A specialized `Result` type for mini-redis operations.
///
/// This is defined as a convenience.
pub type Result<T> = std::result::Result<T, Error>;

#[repr(u8)]
pub enum Command {
    Start = 1,
    Data  = 2,
    Done  = 3,
    Unknown = 4,
}

impl From<u8> for Command {
    fn from(orig: u8) -> Self {
        match orig {
            0x1 => return Command::Start,
            0x2 => return Command::Data,
            0x3 => return Command::Done,
            _   => return Command::Unknown,
        };
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Point")
         .field("Command: ", &self)
         .finish()
    }
}

/// get currrent time, and transfer to string
/// return type: 
///     String
pub fn get_timestamp() -> String {
    let now = std::time::SystemTime::now();
    let ts = format_rfc3339_micros(now);
    format!("{}", ts)
}

/// parse timestamp from string
/// return type:
///     SystemTime
pub fn parse_timestamp(s: &str) -> SystemTime {
    parse_rfc3339(s).unwrap()
}

/// length of time stamp
/// eg.
///   2020-09-20T03:31:02.361565Z
pub const TIMESTAMP_LEN: usize = 27;

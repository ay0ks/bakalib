mod server;
mod socket;

pub use server::*;
pub use socket::*;

use std::fmt;

/// Error struct used by socket library
pub struct Error {
    message: String,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "baka::socket::Error: {}", self.message)
    }
}

impl std::convert::From<Error> for std::io::Error {
    fn from(err: Error) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, err.message)
    }
}

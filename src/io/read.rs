use crate::socket::Error;

pub trait Read {
    fn read_stream(&mut self) -> Result<(Vec<u8>, usize), Error>;
    fn read(&mut self) -> Result<String, Error>;
}

pub trait Send {
    fn send(&mut self, data: &str);
    fn send_bytes(&mut self, data: Vec<u8>);
    fn send_string(&mut self, data: String);
}

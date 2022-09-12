use crate::io;
use crate::protoutils;
use crate::socket::Error;

use bakaproto::proto::*;

use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};

type BoxEvent = Box<dyn Fn(&mut Socket, Result<message::Message, Error>) + Send + 'static>;

pub struct Socket {
    stream: TcpStream,
    pub address: SocketAddr,
}

pub struct Events {
    pub on_connect: BoxEvent,
    pub on_disconnect: BoxEvent,
    pub on_error: BoxEvent,
    pub on_message: BoxEvent,
}

pub struct SocketBuilder {
    socket: Socket,
    events: Events,
}

impl Socket {
    pub fn new(address: &str) -> Self {
        let stream = TcpStream::connect(address).unwrap();

        Socket {
            stream: stream,
            address: address.parse::<SocketAddr>().unwrap(),
        }
    }

    pub fn shutdown(&mut self) {
        self.stream.shutdown(Shutdown::Both);
    }

    pub fn local_address(&mut self) -> String {
        self.stream.local_addr().unwrap().to_string()
    }
    pub fn peer_address(&mut self) -> String {
        self.stream.peer_addr().unwrap().to_string()
    }
}

impl Clone for Socket {
    fn clone(&self) -> Self {
        Socket {
            stream: self.stream.try_clone().unwrap(),
            address: self.address,
        }
    }
}

impl std::convert::From<TcpStream> for Socket {
    fn from(stream: TcpStream) -> Self {
        let address = stream.peer_addr();

        Socket {
            stream: stream,
            address: address.unwrap(),
        }
    }
}

impl std::convert::From<String> for Socket {
    fn from(address: String) -> Self {
        Socket::new(address.as_str())
    }
}

impl std::convert::From<SocketAddr> for Socket {
    fn from(address: SocketAddr) -> Self {
        Socket::new(address.to_string().as_str())
    }
}

impl io::Send for Socket {
    fn send(&mut self, data: &str) {
        self.send_bytes(data.as_bytes().to_vec());
    }

    fn send_bytes(&mut self, data: Vec<u8>) {
        self.stream.write_all(data.as_slice()).unwrap();
    }

    fn send_string(&mut self, data: String) {
        self.send(data.as_str());
    }
}

impl io::Read for Socket {
    fn read_stream(&mut self) -> Result<(Vec<u8>, usize), Error> {
        let buffer_size = 4096;

        let mut request_buffer = vec![];
        let mut request_len = 0usize;

        loop {
            let mut buffer = vec![0; buffer_size];

            match self.stream.read(&mut buffer) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    } else {
                        request_len += n;

                        if n < buffer_size {
                            request_buffer.append(&mut buffer[..n].to_vec());
                            break;
                        } else {
                            request_buffer.append(&mut buffer);
                        }
                    }
                }
                Err(e) => {
                    return Err(Error {
                        message: format!("Unexcepted error while reading stream data: {:?}", e),
                    });
                }
            }
        }

        Ok((request_buffer, request_len))
    }

    fn read(&mut self) -> Result<String, Error> {
        {
            match self.read_stream() {
                Ok(_data) => {
                    let temp = std::str::from_utf8(&_data.0[..]).unwrap();
                    Ok(temp.to_string())
                }
                Err(e) => Err(Error {
                    message: format!("{:#?}", e),
                }),
            }
        }
    }
}

impl SocketBuilder {
    pub fn new(address: &str, events: Events) -> Self {
        let socket = Socket::new(address);

        SocketBuilder {
            socket: socket,
            events: events,
        }
    }

    pub fn startup(&mut self) {
        use crate::io::Read;

        let address = self.socket.address.to_string();

        (self.events.on_connect)(
            &mut self.socket,
            Ok(protoutils::BakaMessage {
                author: address.clone(),
                content: "Succefully connected".to_string(),
            }
            .build()),
        );

        loop {
            if let Ok(buffer) = self.socket.read() {
                if buffer.len() < 1 {
                    continue;
                }

                (self.events.on_message)(
                    &mut self.socket,
                    Ok(protoutils::BakaMessage::from(buffer).build()),
                );
            } else {
                (self.events.on_disconnect)(
                    &mut self.socket,
                    Ok(protoutils::BakaMessage {
                        author: address.clone(),
                        content: "Disconnected".to_string(),
                    }
                    .build()),
                );
                break;
            }
        }
    }
}

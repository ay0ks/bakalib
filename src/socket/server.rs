use crate::extensions::string::StringExtension;
use crate::io::Send;
use crate::protoutils;
use crate::socket::{Error, Socket};

use bakaproto::proto::*;
use protobuf::Message;

use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

type BoxEvent = Box<
    dyn Fn(Arc<Mutex<&mut Server>>, Arc<Mutex<&mut Client>>, Result<message::Message, Error>)
        + core::marker::Send
        + 'static,
>;

pub struct Client {
    pub socket: Socket,
    pub flags: HashMap<String, String>,
}

impl Clone for Client {
    fn clone(&self) -> Self {
        let mut flags = HashMap::new();

        for (name, flag) in &self.flags {
            flags.insert(name.clone(), flag.clone());
        }

        Client {
            socket: self.socket.clone(),
            flags: flags,
        }
    }
}

impl Client {
    pub fn has_flag(&mut self, flag: &str) -> bool {
        self.flags.contains_key(&flag.to_string())
    }

    pub fn add_flag(&mut self, flag: &str, value: &str) {
        self.flags.insert(flag.to_string(), value.to_string());
    }

    pub fn remove_flag(&mut self, flag: &str) -> bool {
        self.flags.remove(&flag.to_string())
    }
}

/// ## Sterver
///
/// Properties:
///
/// * `listener`: This is the TCP listener that will listen for incoming connections.
/// * `address`: The address of the server.
/// * `clients`: A HashMap that stores the client's username as the key and the client's socket as the
/// value.
pub struct Server {
    pub listener: Arc<Mutex<TcpListener>>,
    pub address: SocketAddr,
    pub clients: Arc<Mutex<HashMap<String, Client>>>,
    pub channels: Arc<Mutex<HashMap<String, Socket>>>,
}

impl Clone for Server {
    fn clone(&self) -> Self {
        Server {
            listener: self.listener.clone(),
            address: self.address.clone(),
            clients: self.clients.clone(),
            channels: self.channels.clone(),
        }
    }
}


/// ## ServerBuilder
///
/// Properties:
///
/// * `server`: The server object that will be used to listen for connections.
/// * `events`: A HashMap of String keys and BoxEvent values.
pub struct ServerBuilder {
    server: Server,
    events: Arc<Mutex<HashMap<String, BoxEvent>>>,
}

impl Server {
    /// Initialize new instance of the `Server`
    ///
    /// Example:
    /// ```rs
    /// let socket = Socket::new(/* address */);
    /// ```
    ///
    /// Arguments:
    ///
    /// * `address`: The address to bind the server to.
    ///
    pub fn new(address: &str) -> Self {
        let listener = TcpListener::bind(address).unwrap();

        Server {
            listener: Arc::new(Mutex::new(listener)),
            address: address.parse::<SocketAddr>().unwrap(),
            clients: Arc::new(Mutex::new(HashMap::new())),
            channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Broadcast data to the clients
    ///
    /// Arguments:
    ///
    /// * `data`: &str - The data to broadcast to all clients.
    pub fn broadcast(&mut self, data: &str) {
        use crate::io::Send;

        let clients = self.clients.clone();

        {
            let mut clients = clients.lock().unwrap();

            for (_name, value) in &mut *clients {
                value.socket.send_bytes(
                    protoutils::BakaMessage {
                        author: self.address.to_string(),
                        content: data.to_string(),
                    }
                    .build()
                    .write_to_bytes()
                    .unwrap(),
                );
            }
        }
    }

    pub fn send(&mut self, name: &str, data: &str) {
        let clients = self.clients.clone();

        {
            let mut clients = clients.lock().unwrap();
            let socket = &mut clients.get_mut(name).unwrap().socket;

            socket.send_bytes(
                protoutils::BakaMessage {
                    author: self.address.to_string(),
                    content: data.to_string(),
                }
                .build()
                .write_to_bytes()
                .unwrap(),
            );
        }
    }
}

impl ServerBuilder {
    /// Initialize new instance of the `ServerBuilder`
    ///
    /// Example:
    /// ```rs
    /// let server = ServerBuilder::new("127.0.0.1:65432");
    /// ```
    ///
    /// Arguments:
    ///
    /// * `address`: The address to bind the server to.
    ///
    pub fn new(address: &str) -> Self {
        let events: HashMap<String, BoxEvent> = HashMap::new();

        let s = ServerBuilder {
            server: Server::new(address),
            events: Arc::new(Mutex::new(events)),
        };

        return s;
    }

    /// Add delegate function as server event
    ///
    /// Example:
    /// ```rs
    /// server.event("on_connect", Box::new(|s: &Socket, data: Result<Message, baka::socket::Error>| {
    ///     println!("Succefully connected!");
    /// }));
    /// ```
    ///
    /// Arguments:
    ///
    /// * `name`: The name of the event.
    /// * `delegate`: The function that will be called when the event is triggered.
    pub fn event(&mut self, name: &str, delegate: BoxEvent) {
        let events = self.events.clone();
        let name = String::from(name).clone();

        thread::spawn(move || {
            let mut events = events.lock().unwrap();

            events.entry(name).or_insert(delegate);
        });
    }

    /// Start the event loop thread
    pub fn startup(&mut self) {
        use crate::io::Read;

        let server = self.server.clone();

        for stream in server.listener.lock().unwrap().incoming() {
            let events = self.events.clone();
            let mut server = self.server.clone();
            let clients = server.clients.clone();

            thread::spawn(move || {
                let stream = stream.unwrap();
                let mut socket = Socket::from(stream);

                let address = socket.address.to_string();

                {
                    let events = events.lock().unwrap();
                    let mut clients = clients.lock().unwrap();

                    if !clients.contains_key(&address.clone()) {
                        let mut client = Client {
                            socket: socket.clone(),
                            flags: HashMap::new(),
                        };

                        clients.insert(address.clone(), client.clone());
                    }

                    let mut client = clients.get_mut(&address.clone()).unwrap();

                    events[&"on_client_connect".to_string()](
                        Arc::new(Mutex::new(&mut server)),
                        Arc::new(Mutex::new(client)),
                        Ok(protoutils::BakaMessage {
                            author: address.clone(),
                            content: "Succefully connected".to_string(),
                        }
                        .build()),
                    );
                }

                loop {
                    thread::sleep(time::Duration::from_millis(100));

                    if let Ok(buffer) = socket.read() {
                        {
                            let events = events.lock().unwrap();
                            let mut clients = clients.lock().unwrap();
                            let mut client = clients.get_mut(&address.clone()).unwrap();

                            if buffer.len() < 1 {
                                {
                                    events[&"on_client_disconnect".to_string()](
                                        Arc::new(Mutex::new(&mut server)),
                                        Arc::new(Mutex::new(client)),
                                        Ok(protoutils::BakaMessage {
                                            author: address.clone(),
                                            content: "Disconnected".to_string(),
                                        }
                                        .build()),
                                    );
                                    clients.remove(&address);
                                }
                                break;
                            }

                            events[&"on_message".to_string()](
                                Arc::new(Mutex::new(&mut server)),
                                Arc::new(Mutex::new(client)),
                                Ok(protoutils::BakaMessage::from(buffer).build()),
                            );
                        }
                    } else {
                        {
                            let events = events.lock().unwrap();
                            let mut clients = clients.lock().unwrap();
                            let mut client = clients.get_mut(&address.clone()).unwrap();

                            events[&"on_client_disconnect".to_string()](
                                Arc::new(Mutex::new(&mut server)),
                                Arc::new(Mutex::new(client)),
                                Ok(protoutils::BakaMessage {
                                    author: address.clone(),
                                    content: "Disconnected".to_string(),
                                }
                                .build()),
                            );
                            clients.remove(&address);
                        }
                        break;
                    }
                }
            });
        }
    }

    /// Start the polling loop (block the all threads)
    pub fn polling(&mut self) {
        loop {}
    }
}

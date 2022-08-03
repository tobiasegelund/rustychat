use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::sync::mpsc;
// use std::collections::HashMap;


struct Message {}

struct Client {
    id: u8,
    name: String,
    // sender: mpsc::Sender<Message>,
}

impl Client {
    fn new() -> Self {
        Client { id: 1, name: String::from("test")}
    }
}

// #[derive(Debug)]
pub struct Hub {
    // Registered clients
    // clients: Vec<Client>, // change this

    // // Inbound messages from clients
    // broadcast: f32,

    // // Register requests from the clients
    // register: f32,

    // // Unregister requests from clients
    // unregister: f32,
}

enum Action {
    Sender,
    Writer,
}


impl Hub {
    pub fn new() -> Self {
        return Hub{}
    }

    pub fn run(self) { }
}

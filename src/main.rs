use std::fs;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::sync::mpsc::channel;



fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Connection established");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
    }

}

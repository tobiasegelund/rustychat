// use rustychat::Hub;
use std::io::{Read, Write};
use std::error::Error;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::env;
use std::fmt;
// use std::thread;
// use std::sync::mpsc;

const MAX_BUF_LEN: usize = 1024 * 4;

enum UserAction {
    // Start Hub
    Start,
    // Connect to Hub
    Connect,
}

#[derive(Debug)]
struct ActionError;

impl Error for ActionError { }

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Couldn't find action")
    }
}

#[derive(Clone)]
struct Conn {
    stream: Arc<Mutex<TcpStream>>,
    connections: Connections,
}

impl Conn {
    fn read(&self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.lock().unwrap().read(&mut buf)
    }

    fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        match self.stream.try_lock() {
          Ok(mut lock) => {lock.write(buf)},
          Err(_e) => {Ok(0)},
        }
  }
}

#[derive(Clone)]
struct Connections {
    counter: Arc<Mutex<u32>>,
    connections: Arc<Mutex<HashMap<u32, Conn>>>,
}

impl Connections {
    fn new() -> Self {
        Connections {
            counter: Arc::new(Mutex::new(0)),
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn store(&self, conn: Conn) -> u32 {
        *self.counter.lock().unwrap() += 1;
        let id = *self.counter.lock().unwrap();
        self.connections.lock().unwrap().insert(id, conn);
        return id
    }

    fn broadcast(&self, buf: &[u8]) {
        for (id, conn) in self.connections.lock().unwrap().iter() {
            match conn.write(&buf) {
                Ok(size) => {
                    println!("[{}] Wrote {} to connection", id, size);
                }
                Err(_) => {
                    eprintln!("Error trying to broadcast");
                }
            }
        }
    }
}



struct Client {
    name: String,
}

impl Client {
    fn from(name: String) -> Self {
       Client { name }
    }
}


fn handle_connection(conn: Conn) {
    let id = conn.connections.store(conn.clone());
    println!("{} has connected", id);

    loop {
        let mut buf = vec![0; MAX_BUF_LEN];
        match conn.read(&mut buf) {
            Ok(read) if read > 0 => {
                let msg = String::from_utf8_lossy(&buf);
                let msg = format!("[{}] {}", id, msg);
                println!("{}", msg);
                conn.connections.broadcast(msg.as_bytes());
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error with {}", e);
            }
        }
    }
}

fn collect_action(args: &Vec<String>) -> Result<UserAction, ActionError> {
    let flag = &args[1][..];
    match flag {
        "start" => {
            return Ok(UserAction::Start)
        }
        "connect" => {
            return Ok(UserAction::Connect)
        }
        _ => {
           return Err(ActionError)
        }
    }
}

fn collect_name(args: &Vec<String>) -> String {
    if args.len() < 3 {
        return String::from("Default")
    }
    let name = &args[2];

    name.to_string()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = collect_action(&args);
    let name = collect_name(&args);

    let connections = Connections::new();

    match action {
        Ok(UserAction::Start) => {
            let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
            println!("Connection established");

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let conn = Conn {
                           stream: Arc::new(Mutex::new(stream)),
                           connections: connections.clone(), // Implement clone trait
                        };
                        thread::spawn(move || handle_connection(conn));
                    }
                    Err(e) => {
                        eprintln!("Connection failed with error: {}", e);
                    }
                }
            }
        }

        Ok(UserAction::Connect) => {
            // Add name as CLI option
            let client = Client::from(name);
            loop {
                let mut msg = String::new();
                std::io::stdin().read_line(&mut msg).unwrap();
                if msg.len() > 0 {
                    let msg = format!("{}: {}", client.name, msg);
                    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:7878") {
                        stream.set_read_timeout(None).expect("set_read_timeout call failed");
                        stream.write(msg.as_bytes()).unwrap();
                    }
                    else {
                        eprintln!("Couldn't connect to the hub");
                    }
                }
            }
        }

        Err(_) => {
           eprintln!("Terminated: Action was not possible");
        }
    }
}

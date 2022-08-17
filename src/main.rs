// use rustychat::Hub;
use std::io::{Read, Write};
use std::error::Error;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::env;
use std::fmt;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;

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
          Err(_) => {Ok(0)},
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
                Ok(size) => { println!("[{}] Wrote {} to connection...", id, size); },
                Err(e) => { println!("[{}] Error writing to connection {}", id, e); },
            }
        }
    }
}

fn sleep(millis: u64) {
    let duration = std::time::Duration::from_millis(millis);
    thread::sleep(duration);
}



struct Client {
    name: String,
}

impl Client {
    fn from(name: String) -> Self {
       Client { name }
    }
}

fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        tx.send(buffer).unwrap();
    });
    rx
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
                eprintln!("{}", e);
            }
        }
    }
}

fn handle_response(stream: Arc<Mutex<TcpStream>>) {
    let mut buf = [0; MAX_BUF_LEN];
    stream.lock().unwrap().read(&mut buf).unwrap();
    let msg = String::from_utf8_lossy(&buf);
    println!("{}", msg);

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
                        stream.set_nonblocking(true).expect("Unable to set to nonblocking");
                        let conn = Conn {
                           stream: Arc::new(Mutex::new(stream)),
                           connections: connections.clone(),
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
            let client = Client::from(name);
            if let Ok(stream) = TcpStream::connect("127.0.0.1:7878") {
                println!("Connected to the Hub");
                let stdin_channel = spawn_stdin_channel();
                loop {
                    let stream_read = Arc::new(Mutex::new(stream.try_clone().unwrap()));
                    let stream_write = Arc::new(Mutex::new(stream.try_clone().unwrap()));
                    thread::spawn(move || handle_response(stream_read));

                    match stdin_channel.try_recv() {
                        Ok(msg) => {
                            let msg = format!("{}: {}", client.name, msg);
                            thread::spawn(move || {
                                stream_write.lock().unwrap().write(msg.as_bytes()).unwrap();
                            });
                        }
                        Err(_) => {}
                    }

                    // Add delay to hinder maximum open files error from stream cloning
                    sleep(200);
                }
            }
            else {
                eprintln!("Couldn't connect to the hub");
            }
        }

        Err(_) => {
           eprintln!("Terminated: Action was not possible");
        }
    }
}

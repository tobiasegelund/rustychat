// use rustychat::Hub;
use std::io::prelude::*;
use std::error::Error;
use std::net::TcpListener;
use std::net::TcpStream;
use std::env;
use std::fmt;
// use std::thread;
// use std::sync::mpsc;

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

struct Client {
    name: String,
}

impl Client {
    fn from(name: String) -> Self {
       Client { name }
    }

    fn write_msg(self, mut stream: &TcpStream) -> Self {
        let mut msg = String::new();
        std::io::stdin().read_line(&mut msg).unwrap();
        if msg.len() > 0 {
            let msg = format!("{}: {}", self.name, msg);
            stream.write(msg.as_bytes()).unwrap();
        }
        self
    }
}


fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0 as u8; 1024];

    stream.read(&mut buffer).unwrap();
    let msg = String::from_utf8_lossy(&mut buffer);
    println!("{}", msg);

    stream.write(msg.as_bytes()).unwrap();
    // stream.flush().unwrap();
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

    match action {
        Ok(UserAction::Start) => {
            let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
            println!("Connection established");

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        handle_connection(stream);
                    }
                    Err(e) => {
                        eprintln!("Connection failed with error: {}", e);
                    }
                }
                // client.write_msg(&stream);
                // write stream here
            }
        }

        Ok(UserAction::Connect) => {
            // Add name as CLI option
            let mut client = Client::from(name);
            while let Ok(stream) = TcpStream::connect("127.0.0.1:7878") {
                client = client.write_msg(&stream);
                handle_connection(stream);
            }
            eprintln!("Couldn't connect to the hub");
        }

        Err(_) => {
           eprintln!("Terminated: Action was not possible");
        }
    }
}

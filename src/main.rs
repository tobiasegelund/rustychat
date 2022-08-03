// use rustychat::Hub;
use std::io::prelude::*;
use std::error::Error;
use std::net::TcpListener;
use std::net::TcpStream;
use std::env;
use std::process::exit;
use std::fmt;
use std::str::from_utf8;
// use std::thread;
// use std::sync::mpsc;

enum UserAction {
    // Start Hub
    Start,
    // Connect to Hub
    Connect,
    // Terminate Hub
    Terminate,
}

#[derive(Debug)]
struct ActionError;

impl Error for ActionError { }

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Couldn't find action")
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0 as u8; 1024];

    stream.read(&mut buffer).unwrap();
    let msg = from_utf8(&mut buffer).unwrap();
    println!("{}", msg);

    // let response = "Message back";
    // stream.write(response.as_bytes()).unwrap();
    // stream.flush().unwrap();
}

fn collect_action() -> Result<UserAction, ActionError> {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2); // the program itself and action must be the only specified values
    let flag = &args[1][..];
    match flag {
        "start" => {
            return Ok(UserAction::Start)
        }
        "connect" => {
            return Ok(UserAction::Connect)
        }
        _ => {
           return Err(ActionError);
        }
    }
}

fn main() {
    let action = match collect_action() {
        Ok(action) => action,
        Err(e) => {
            println!("{}", e);
            exit(2);
        },
    };

    match action {
        UserAction::Start => {
            let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
            println!("Connection established");

            loop {
                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => {
                            handle_connection(stream);
                        }
                        Err(e) => {
                            eprintln!("Connection failed with error: {}", e);
                        }
                    }
                }
            }
        }

        UserAction::Connect => {
            while let Ok(mut stream) = TcpStream::connect("127.0.0.1:7878") {
                // match input stream here whether something is entered or something needs to be
                // read
                let msg = "hello";
                stream.write(msg.as_bytes()).unwrap();
                // do stuff
            }
            eprintln!("Couldn't connect to the hub");
        }
        UserAction::Terminate => {
            panic!("Terminate Hub");
        }
    }
}

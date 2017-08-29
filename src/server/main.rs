use std::net::{TcpListener, TcpStream};

fn handle_client(stream: TcpStream) {
    // ...
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8001").unwrap();
    listener.set_nonblocking(true).expect("Cannot set non-blocking");

    let mut conn_streams:Vec<TcpStream> = Vec::new();

    loop {
        match listener.accept() {
            Ok((_socket, addr)) => {
                println!("new client: {:?}", addr);
                conn_streams.push(_socket);
            },
            Err(e) => {
                
            },
        }
    }
}
use std::net::{TcpListener, TcpStream};

extern crate audio_analysis;
use audio_analysis::analysis;
use audio_analysis::server::messages;

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

                let mut devices = analysis::pa_interface::get_devices();
                let mut device_msg = messages::MsgDevicesList::new();
                device_msg.devices = devices.unwrap();
            },
            Err(e) => {
                
            },
        }
    }
}
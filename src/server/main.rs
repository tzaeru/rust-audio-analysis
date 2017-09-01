use std::net::{TcpListener, TcpStream};

use std::io::Write;
use std::thread;

extern crate audio_analysis;
use audio_analysis::analysis;
use audio_analysis::server::messages;

use audio_analysis::server::messages::Serializable;


fn handle_client(stream: TcpStream) {
    // ...
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8001").unwrap();
    listener.set_nonblocking(true).expect("Cannot set non-blocking");

    analysis::pa_interface::get_rms(0,
        |rms:f32| {});

    loop {
        match listener.accept() {
            Ok((mut _socket, addr)) => {
                thread::spawn(move || {
                
                    println!("new client: {:?}", addr);
                    
                    let mut devices = analysis::pa_interface::get_devices();
                    let mut device_msg = messages::MsgDevicesList::new();
                    device_msg.devices = devices.unwrap();

                    let mut serialized = device_msg.serialize();
                    let _ = _socket.write(serialized.as_mut_slice());

                });
            },
            Err(e) => {
                
            },
        }
    }
}

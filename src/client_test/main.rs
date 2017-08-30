use std::io::prelude::*;
use std::net::TcpStream;

extern crate audio_analysis;
use audio_analysis::server::messages;

fn main() {
	if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8001") {
	    println!("Connected to the server!");

        loop {
        	let mut data = [0u8; 256];
            let result = stream.read(&mut data); // ignore this too

            let length: i32 = data[3] as i32 | ((data[2] as i32) << 8) | ((data[1] as i32)  << 16) | ((data[0] as i32) << 24);
            println!("Result length: {:?}", result.unwrap());
            println!("Length: {}", length);

            // Skip message length and type (WIP)
            let _ = messages::MsgDevicesList::deserialized(data[8..].to_vec());
        }
	} else {
	    println!("Couldn't connect to server...");
	}
}
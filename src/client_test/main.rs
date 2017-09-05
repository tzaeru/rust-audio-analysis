use std::io::prelude::*;
use std::net::TcpStream;

extern crate audio_analysis;
use audio_analysis::server::messages;
use audio_analysis::server::messages::Serializable;

fn request_rms(mut stream: &TcpStream) -> Result<usize, std::io::Error>
{
    let mut rms_msg = messages::MsgStartStreamRMS::new();
    rms_msg.device = 0;

    let mut serialized = rms_msg.serialize();
    stream.write(serialized.as_mut_slice())
}

fn main() {
	if let Ok(mut stream) = TcpStream::connect("127.0.0.1:50000") {
	    println!("Connected to the server!");
        let _ = request_rms(&stream);
        loop {
        	let mut data = [0u8; 2048];
            match stream.read(&mut data)
            {
                Ok(result) => {
                    if result <= 0
                    {
                        continue;
                    }

                    println!("Got msg.");

                    let length: i32 = data[0] as i32 | ((data[1] as i32) << 8) | ((data[2] as i32)  << 16) | ((data[3] as i32) << 24);
                    println!("Result length: {:?}", result);
                    println!("Length: {}", length);

                    // Skip message length and type (WIP)
                    //let _ = messages::MsgDevicesList::deserialized(data[8..].to_vec());

                    
                },
                Err(e) => {
                    println!("Error terror, {}", e);
                }
            }
        }
	} else {
	    println!("Couldn't connect to server...");
	}
}
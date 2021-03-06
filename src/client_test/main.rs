use std::io::prelude::*;
use std::net::TcpStream;

extern crate raa;
use raa::server::messages;
use raa::server::messages::Serializable;

fn request_rms(mut stream: &TcpStream) -> Result<usize, std::io::Error>
{
    let mut rms_msg = messages::MsgStartStreamRMS::new();
    rms_msg.device_id = "{0.0.1.00000000}.{625fb92c-394b-482e-82ed-30b560d85d8f}".to_string();
    rms_msg.channels = vec![0, 1];

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

                    let length: i32 = data[0] as i32 | ((data[1] as i32) << 8) | ((data[2] as i32)  << 16) | ((data[3] as i32) << 24);

                    let msg_type: i32 = data[4] as i32 | ((data[5] as i32) << 8) | ((data[6] as i32)  << 16) | ((data[7] as i32) << 24);

                    if msg_type == messages::MsgType::MSG_RMS_PACKET as i32
                    {
                        let rms_msg = messages::MsgRMSPacket::deserialized(data[8..].to_vec());

                        println!("RMS: {:?}", rms_msg.value);
                    }
                    else if msg_type == messages::MsgType::MSG_DEVICES_LIST as i32
                    {
                        let _ = messages::MsgDevicesList::deserialized(data[8..].to_vec());
                    }
                    else if msg_type == messages::MsgType::MSG_ERROR as i32
                    {
                        let message = messages::MsgError::deserialized(data[8..].to_vec()).message;
                        println!("Error terror: {:?}", message);
                    }
                    else {
                        println!("Unknown message type.");
                    }
                    //Skip message length and type (WIP)
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
use std::net::{TcpListener, TcpStream};

use std::io::prelude::*;
use std::thread;

extern crate audio_analysis;
use audio_analysis::analysis;
use audio_analysis::server::messages;
use audio_analysis::analysis::pa_interface::Chainable;

use audio_analysis::server::messages::Serializable;
use audio_analysis::server::messages::MsgType;

use std::cell::RefCell;
use std::rc::Rc;

use std::sync::Arc;
use std::sync::RwLock;


fn handle_client(stream: TcpStream) {
    // ...
}

fn send_devices(mut stream: &TcpStream) -> Result<usize, std::io::Error>
{
    let mut devices = analysis::pa_interface::get_devices();
    let mut device_msg = messages::MsgDevicesList::new();
    device_msg.devices = devices.unwrap();

    let mut serialized = device_msg.serialize();
    stream.write(serialized.as_mut_slice())
}

fn send_rms_msg(mut stream: &TcpStream, rms: f32) -> Result<usize, std::io::Error>
{
    let mut rms_msg = messages::MsgRMSPacket::new();
    rms_msg.value = rms;

    println!("Sending RMS {}", rms);

    let mut serialized = rms_msg.serialize();
    stream.write(serialized.as_mut_slice())
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:50000").unwrap();
    listener.set_nonblocking(true).expect("Cannot set non-blocking");

    loop {
        match listener.accept() {
            Ok((mut stream, addr)) => {
                let _ = stream.set_nodelay(true);


                thread::spawn(move || {
                
                    println!("new client: {:?}", addr);
                    
                    let _ = send_devices(&stream);

                    // Ready an analysis chain to be used later on after a proper message has been received.
                    let mut arena = analysis::pa_interface::AArena::new();
                    let mut arena_rc = Arc::new(RwLock::new(arena));
                    let mut chain = analysis::pa_interface::AChain::new(arena_rc.clone());
                    let mut chain_ref = Rc::new(RefCell::new(chain));

                    let rms = Arc::new(RwLock::new(analysis::pa_interface::RMS::new()));
                    let rms_id = arena_rc.write().unwrap().add_chainable(rms);

                    let mut send_rms = false;

                    // Buffer for whole message.
                    // Each message is prefixed by message length.
                    // As TCP is a streaming protocol, message size may vary.
                    // As such, only handle (and remove) a message when whole message is read.
                    let mut msg_buffer: Vec<u8> = Vec::new();

                    // Read stuff until error.
                    loop {
                            let mut data = [0u8; 2048];
                            match stream.read(&mut data)
                            {
                                Ok(read_bytes) => {
                                    if read_bytes <= 0
                                    {
                                        continue;
                                    }

                                    for i in 0..read_bytes
                                    {
                                        msg_buffer.push(data[i]);
                                    }

                                    if msg_buffer.len() < 4
                                    {
                                        continue;
                                    }

                                    // Length that the message should have
                                    let msg_length: i32 = msg_buffer[0] as i32 | ((msg_buffer[1] as i32) << 8) | ((msg_buffer[2] as i32)  << 16) | ((msg_buffer[3] as i32) << 24);
                                    
                                    println!("Message length: {}", msg_length);
                                    println!("Buffer length: {}", msg_buffer.len());

                                    // We have a full length message if buffer has msg_length + 4 or more bytes
                                    if msg_buffer.len() >= msg_length as usize
                                    {
                                        // Remove the message's worth of bytes from the buffer.
                                        let message_bytes: Vec<u8> = msg_buffer.drain(0..msg_length as usize + 4).collect();
                                        let msg_type = message_bytes[4] as i32 | ((message_bytes[5] as i32) << 8) | ((message_bytes[6] as i32)  << 16) | ((message_bytes[7] as i32) << 24);
                                        println!("Message type: {}", msg_type);

                                        if msg_type == MsgType::MSG_GET_RMS as i32
                                        {
                                            chain_ref.borrow_mut().stop();

                                            let rms_msg = messages::MsgStartStreamRMS::deserialized(data[8..].to_vec());
                                            println!("Device: {}", rms_msg.device);
                                            println!("Channels: {:?}", rms_msg.channels);

                                            let source = Arc::new(RefCell::new(analysis::pa_interface::PASource::new(0, vec![0])));
                                            let source_id = arena_rc.write().unwrap().add_sourcable(source);

                                            chain = analysis::pa_interface::AChain::new(arena_rc.clone());
                                            chain.set_source(source_id);
                                            chain.add_node(rms_id);

                                            chain_ref = Rc::new(RefCell::new(chain));
                                            chain_ref.borrow_mut().start(chain_ref.clone());

                                            send_rms = true;
                                        }
                                    }
                                },
                                Err(e) => {
                                    //println!("{:?}", e.kind());
                                    //println!("{:?}", e.raw_os_error());

                                    match e.kind() {
                                        std::io::ErrorKind::WouldBlock => {},
                                        _ => {
                                            println!("Breaking.");
                                            break;
                                        },
                                    }
                                }
                            }
 
                            if send_rms == true
                            {
                                println!("Trying");
                                let arena_borrow = arena_rc.read().unwrap();

                                if arena_borrow.chainables[&rms_id].read().unwrap().output().len() > 0
                                {
                                    send_rms_msg(&stream, arena_borrow.chainables[&rms_id].read().unwrap().output()[0]);
                                }
                                println!("Got");
                            }
                    }

                    chain_ref.borrow_mut().stop();
                });
            },
            Err(e) => {
                
            },
        }
    }
}

use std::net::{TcpListener, TcpStream};

use std::io::prelude::*;
use std::thread;

use std::time::{Duration, Instant};

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
                    let mut chain_ref = Arc::new(RwLock::new(chain));

                    let rms = Arc::new(RwLock::new(analysis::pa_interface::RMS::new()));
                    let rms_id = arena_rc.write().unwrap().add_chainable(rms);

                    let mut source_id = 0u64;

                    let mut send_rms = false;

                    // Cap to 20 outgoing messages per second
                    let mut sent_msg_instant = Instant::now();
                    let send_cap = 20;

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
                                    let mut msg_length: i32 = msg_buffer[0] as i32 | ((msg_buffer[1] as i32) << 8) | ((msg_buffer[2] as i32)  << 16) | ((msg_buffer[3] as i32) << 24);
                                    
                                    println!("\nMessage length: {}", msg_length);
                                    println!("Buffer length: {}\n", msg_buffer.len());

                                    // We have a full length message if buffer has msg_length bytes
                                    while msg_buffer.len() >= msg_length as usize
                                    {
                                        // Remove the message's worth of bytes from the buffer.
                                        println!("Buffer before: {}", msg_buffer.len());
                                        let mut message_bytes: Vec<u8> = msg_buffer.drain(0..msg_length as usize).collect();
                                        println!("Buffer after: {}", msg_buffer.len());
                                        let msg_type = message_bytes[4] as i32 | ((message_bytes[5] as i32) << 8) | ((message_bytes[6] as i32)  << 16) | ((message_bytes[7] as i32) << 24);
                                        println!("Message type: {}", msg_type);

                                        if msg_type == MsgType::MSG_GET_RMS as i32
                                        {
                                            chain_ref.write().unwrap().stop();

                                            // Ignore length & type when passing message_bytes
                                            let msg_length = message_bytes.len();
                                            let messages_bytes_without_type = message_bytes.drain(8..msg_length).collect();
                                            let rms_msg = messages::MsgStartStreamRMS::deserialized(messages_bytes_without_type);
                                            println!("Device: {}", rms_msg.device);
                                            println!("Channels: {:?}", rms_msg.channels);

                                            let source = Arc::new(RwLock::new(analysis::pa_interface::PASource::new(rms_msg.device as u32, rms_msg.channels)));
                                            source_id = arena_rc.write().unwrap().add_sourcable(source);

                                            chain = analysis::pa_interface::AChain::new(arena_rc.clone());
                                            chain.set_source(source_id);
                                            chain.add_node(rms_id);

                                            chain_ref = Arc::new(RwLock::new(chain));
                                            chain_ref.write().unwrap().start(chain_ref.clone());

                                            send_rms = true;
                                        }

                                        if msg_buffer.len() >= 4
                                        {
                                            msg_length = msg_buffer[0] as i32 | ((msg_buffer[1] as i32) << 8) | ((msg_buffer[2] as i32)  << 16) | ((msg_buffer[3] as i32) << 24);
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
                            
                            let elapsed_as_mills = sent_msg_instant.elapsed().as_secs() * 1000
                                            + sent_msg_instant.elapsed().subsec_nanos() as u64 / 1000000;
                            if send_rms == true && elapsed_as_mills > 1000/20
                            {
                                sent_msg_instant = Instant::now();

                                let arena_borrow = arena_rc.read().unwrap();

                                if arena_borrow.chainables[&rms_id].read().unwrap().output().len() > 0
                                {
                                    send_rms_msg(&stream, arena_borrow.chainables[&rms_id].read().unwrap().output()[0]);
                                }
                            }

                            if send_rms == true
                            {
                                let arena_borrow = arena_rc.read().unwrap();

                                /*if arena_borrow.sourcables[&source_id].read().unwrap().is_active() == true
                                {
                                    println!("Is active at {}", elapsed_as_mills);
                                }
                                else {
                                    println!("Is not active");
                                }*/
                            }
                    }

                    chain_ref.write().unwrap().stop();
                });
            },
            Err(e) => {
                
            },
        }
    }
}

use std::net::{TcpListener, TcpStream};

use std::io::prelude::*;
use std::{thread, time};

use std::time::{Duration, Instant};

mod messages;

use messages::Serializable;
use messages::MsgType;

extern crate raa;
use raa::analysis;
use raa::analysis::traits::Sourcable;

use std::sync::Arc;
use std::sync::RwLock;


fn handle_client(stream: TcpStream) {
    // ...
}

fn send_devices(mut stream: &TcpStream) -> Result<usize, std::io::Error>
{
    let mut devices = analysis::soundio_source::SoundioSource::get_devices();
    let mut device_msg = messages::MsgDevicesList::new();
    device_msg.devices = devices.unwrap();

    let mut serialized = device_msg.serialize();
    stream.write(serialized.as_mut_slice())
}

fn send_test_error(mut stream: &TcpStream) -> Result<usize, std::io::Error>
{
    let mut error_msg = messages::MsgError::new();
    error_msg.message = "horror".to_string();

    let mut serialized = error_msg.serialize();
    stream.write(serialized.as_mut_slice())
}

fn send_error(mut stream: &TcpStream, error: String) -> Result<usize, std::io::Error>
{
    let mut error_msg = messages::MsgError::new();
    error_msg.message = error;

    let mut serialized = error_msg.serialize();
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
                    //let _ = send_test_error(&stream);

                    // Ready an analysis chain to be used later on after a proper message has been received.
                    let arena = analysis::analysis::Arena::new();
                    let arena_rc = Arc::new(RwLock::new(arena));
                    let mut chain = analysis::analysis::Chain::new(arena_rc.clone());
                    let mut chain_ref = Arc::new(RwLock::new(chain));

                    let rms = Arc::new(RwLock::new(analysis::rms::RMS::new()));
                    let rms_id = arena_rc.write().unwrap().add_chainable(rms);

                    let mut source_id = None;

                    let mut send_rms = false;

                    // Cap to 20 outgoing messages per second
                    let mut sent_msg_instant = Instant::now();

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
                                    println!("Buffer length: {}", msg_buffer.len());

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
                                            println!("Stopping RMS chain..");
                                            chain_ref.write().unwrap().stop();

                                            println!("Stopped RMS chain!");
                                            match source_id
                                            {
                                                Some(id) => {
                                                    println!("Removing sourcable..");
                                                    match arena_rc.write() {
                                                        Ok(mut rc) => (rc.remove_sourcable(id)),
                                                        Err(e) => (println!("Could not write: {:?}", e))
                                                    }
                                                    println!("Removed sourcable!");
                                                }
                                                None => ()
                                            };

                                            // Ignore length & type when passing message_bytes
                                            println!("1");
                                            let msg_length = message_bytes.len();
                                            println!("2");
                                            let messages_bytes_without_type = message_bytes.drain(8..msg_length).collect();
                                            println!("Creating RMS msg..");
                                            let rms_msg = messages::MsgStartStreamRMS::deserialized(messages_bytes_without_type);
                                            println!("Device: {}", rms_msg.device_id);
                                            println!("Channels: {:?}", rms_msg.channels);

                                            let source = Arc::new(RwLock::new(analysis::soundio_source::SoundioSource::new(rms_msg.device_id, rms_msg.channels)));
                                            source_id = Some(arena_rc.write().unwrap().add_sourcable(source));

                                            chain = analysis::analysis::Chain::new(arena_rc.clone());
                                            chain.set_source(source_id.unwrap());
                                            chain.add_node(rms_id);
                                            println!("Created new RMS chain!");

                                            chain_ref = Arc::new(RwLock::new(chain));
                                            chain_ref.write().unwrap().start(chain_ref.clone());
                                            println!("Started RMS chain!\n");

                                            send_rms = true;
                                        }

                                        if msg_buffer.len() >= 4
                                        {
                                            msg_length = msg_buffer[0] as i32 | ((msg_buffer[1] as i32) << 8) | ((msg_buffer[2] as i32)  << 16) | ((msg_buffer[3] as i32) << 24);
                                        }
                                    }
                                },
                                Err(e) => {
                                    match e.kind() {
                                        std::io::ErrorKind::WouldBlock => {},
                                        _ => {
                                            println!("Breaking.");
                                            arena_rc.write().unwrap().remove_sourcable(source_id.unwrap());
                                            arena_rc.write().unwrap().remove_chainable(rms_id);
                                            break;
                                        },
                                    }
                                }
                            }
                            
                            let elapsed_as_mills = sent_msg_instant.elapsed().as_secs() * 1000
                                            + sent_msg_instant.elapsed().subsec_nanos() as u64 / 1000000;
                            let arena_borrow = arena_rc.read().unwrap();
                            match source_id
                            {
                                Some(id) =>
                                {
                                    let sourcable = arena_borrow.sourcables[&id].read().unwrap();
                                    
                                    match sourcable.get_and_clear_error()
                                    {
                                        Some(error) => {
                                            let _ = send_error(&stream, error);
                                        }
                                        None => ()
                                    }
                                },
                                None => ()
                            }

                            if send_rms == true && elapsed_as_mills > 1000/20
                            {
                                sent_msg_instant = Instant::now();

                                if arena_borrow.chainables[&rms_id].read().unwrap().output().len() > 0
                                {
                                    let error = send_rms_msg(&stream, arena_borrow.chainables[&rms_id].read().unwrap().output()[0]);
                                    match error
                                    {
                                        Ok(_) => (),
                                        Err(e) => {
                                            println!("Connection lost: {:?}", e);
                                            arena_rc.write().unwrap().remove_sourcable(source_id.unwrap());
                                            arena_rc.write().unwrap().remove_chainable(rms_id);
                                        }
                                    }
                                }
                            }

                            let ten_millis = time::Duration::from_millis(10);
                            thread::sleep(ten_millis);
                    }
                    println!("Stopped looping for data - client disconnected?");
                    chain_ref.write().unwrap().stop();
                });
            },
            Err(e) => {
                //println!("Couldn't accept client connection, {:?}", e);
            },
        }
        let ten_millis = time::Duration::from_millis(10);
        thread::sleep(ten_millis);
    }
}

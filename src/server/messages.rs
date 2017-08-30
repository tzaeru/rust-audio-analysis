use std::collections::HashMap;
use std::mem::transmute;
use std::str;

#[derive(Clone)]
pub enum MsgType {
	MSG_GET_RMS = 0,
	MSG_RMS_PACKET = 1,
	MSG_GET_DEVICES = 2,
	MSG_SET_FLOAT_PARAM = 3,
	MSG_DEVICES_LIST = 4,
	MSG_DB_PACKET = 5,
	MSG_SET_BOOLEAN_PARAM = 6,
	MSG_CONFIGUREDB = 7,
}

pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
}


pub struct MsgGetDevices
{
	pub msg_type: MsgType,
}

impl MsgGetDevices {
	pub fn new() -> MsgGetDevices {
        MsgGetDevices { msg_type: MsgType::MSG_GET_DEVICES }
    }
}

pub struct MsgDevicesList<'a> {
	pub msg_type: MsgType,
	pub devices: HashMap<i32, (&'a str, i32)>,
}

impl <'a>MsgDevicesList<'a> {
	pub fn new() -> MsgDevicesList<'a> {
        MsgDevicesList { msg_type: MsgType::MSG_DEVICES_LIST, devices: HashMap::new() }
    }

    pub fn deserialized(mut data: Vec<u8>) -> MsgDevicesList<'a> {
    	let mut devices_list_msg = MsgDevicesList { msg_type: MsgType::MSG_DEVICES_LIST, devices: HashMap::new() };

    	let device_amount: i32 = data[3] as i32 | ((data[2] as i32) << 8) | ((data[1] as i32)  << 16) | ((data[0] as i32) << 24);

    	println!("Device amount: {}", device_amount);

    	data = data[4..].to_vec();

    	for _ in 0..device_amount
    	{
    		let device_id: i32 = data[3] as i32 | ((data[2] as i32) << 8) | ((data[1] as i32)  << 16) | ((data[0] as i32) << 24);
    		data = data[4..].to_vec();
    		println!("ID: {}", device_id);

    		let device_name_length: i32 = data[3] as i32 | ((data[2] as i32) << 8) | ((data[1] as i32)  << 16) | ((data[0] as i32) << 24);
    		data = data[4..].to_vec();
    		println!("Name length: {}", device_name_length);

    		let data_clone = data.clone();
    		let device_name = str::from_utf8(&data_clone[..device_name_length as usize]).unwrap();
    		println!("Name: {}", device_name);

    		data = data[device_name_length as usize..].to_vec();

    		let device_channels = data[3] as i32 | ((data[2] as i32) << 8) | ((data[1] as i32)  << 16) | ((data[0] as i32) << 24);
    		data = data[4..].to_vec();
    		println!("Channels: {}", device_channels);
    	}

    	devices_list_msg
    }
}

impl<'a> Serializable for MsgDevicesList<'a>
{
	fn serialize(&self) -> Vec<u8>
	{
		let mut bytes = Vec::new();

		let type_bytes: [u8; 4] = unsafe { transmute((self.msg_type.clone() as i32).to_be()) };
		
		let device_count_bytes: [u8; 4] = unsafe { transmute((self.devices.len() as i32).to_be()) };

		let mut device_bytes = Vec::new();
		for (id, name_and_channels) in &self.devices
		{
			let id_bytes: [u8; 4] = unsafe { transmute(id.to_be()) };
			device_bytes.extend(id_bytes.iter().cloned());

			let name_length: [u8; 4] = unsafe { transmute((name_and_channels.0.as_bytes().len() as i32).to_be() as i32) };
			device_bytes.extend(name_length.iter().cloned());
			device_bytes.extend(name_and_channels.0.as_bytes());
			println!("Serializing device, name: {} Length: {}", name_and_channels.0, name_and_channels.0.as_bytes().len());
			println!("Name bytes: {:?}", name_length);

			let channel_bytes: [u8; 4] = unsafe { transmute(name_and_channels.1.to_be()) };
			device_bytes.extend(channel_bytes.iter().cloned());
		}

		let length_bytes: [u8; 4] = unsafe { transmute(((4 + 4 + device_bytes.len()) as i32).to_be()) };
		bytes.extend(length_bytes.iter().cloned());
		bytes.extend(type_bytes.iter().cloned());
		bytes.extend(device_count_bytes.iter().cloned());
		bytes.extend(device_bytes.iter().cloned());
		bytes
	}
}
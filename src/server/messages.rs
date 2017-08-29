use std::collections::HashMap;
use std::mem::transmute;

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
    fn Serialize(&self) -> Vec<u8>;
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
}

impl<'a> Serializable for MsgDevicesList<'a>
{
	fn Serialize(&self) -> Vec<u8>
	{
		let mut bytes = Vec::new();

		let type_bytes: [u8; 4] = unsafe { transmute((self.msg_type.clone() as i32).to_be()) };
		
		let device_count_bytes: [u8; 4] = unsafe { transmute((self.devices.len() as i32).to_be()) };

		let mut device_bytes = Vec::new();
		for (id, name_and_channels) in &self.devices
		{
			let id_bytes: [u8; 4] = unsafe { transmute(id.to_be()) };
			device_bytes.extend(id_bytes.iter().cloned());

			device_bytes.extend(name_and_channels.0.as_bytes());

			let channel_bytes: [u8; 4] = unsafe { transmute(name_and_channels.1.to_be()) };
			device_bytes.extend(channel_bytes.iter().cloned());
		}

		bytes.extend(type_bytes.iter().cloned());
		bytes.extend(device_bytes.iter().cloned());
		bytes
	}
}
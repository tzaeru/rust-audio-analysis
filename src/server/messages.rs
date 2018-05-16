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
    MSG_ERROR = 8,
}

pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
}


pub struct MsgGetDevices {
    pub msg_type: MsgType,
}

impl MsgGetDevices {
    pub fn new() -> MsgGetDevices {
        MsgGetDevices { msg_type: MsgType::MSG_GET_DEVICES }
    }
}

pub struct MsgDevicesList {
    pub msg_type: MsgType,
    pub devices: HashMap<i32, (String, i32)>,
}

impl MsgDevicesList {
    pub fn new() -> MsgDevicesList {
        MsgDevicesList {
            msg_type: MsgType::MSG_DEVICES_LIST,
            devices: HashMap::new(),
        }
    }

    pub fn deserialized(mut data: Vec<u8>) -> MsgDevicesList {
        let mut devices_list_msg = MsgDevicesList {
            msg_type: MsgType::MSG_DEVICES_LIST,
            devices: HashMap::new(),
        };

        let device_amount: i32 = data[0] as i32 | ((data[1] as i32) << 8) |
                                 ((data[2] as i32) << 16) |
                                 ((data[3] as i32) << 24);

        println!("Device amount: {}", device_amount);

        data = data[4..].to_vec();

        for _ in 0..device_amount {
            let device_id: i32 = data[0] as i32 | ((data[1] as i32) << 8) |
                                 ((data[2] as i32) << 16) |
                                 ((data[3] as i32) << 24);
            data = data[4..].to_vec();
            println!("ID: {}", device_id);

            let device_name_length: u16 = data[0] as u16 | ((data[1] as u16) << 8);
            data = data[2..].to_vec();
            println!("Name length: {}", device_name_length);

            let data_clone = data.clone();
            let device_name = str::from_utf8(&data_clone[..device_name_length as usize]).unwrap();
            println!("Name: {}", device_name);

            data = data[device_name_length as usize..].to_vec();

            let device_channels = data[0] as i32 | ((data[1] as i32) << 8) |
                                  ((data[2] as i32) << 16) |
                                  ((data[3] as i32) << 24);
            data = data[4..].to_vec();
            println!("Channels: {}", device_channels);
        }

        devices_list_msg
    }
}

impl Serializable for MsgDevicesList {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let type_bytes: [u8; 4] = unsafe { transmute((self.msg_type.clone() as i32).to_le()) };

        let device_count_bytes: [u8; 4] = unsafe { transmute((self.devices.len() as i32).to_le()) };

        let mut device_bytes = Vec::new();

        for (id, name_and_channels) in &self.devices {
            let id_bytes: [u8; 4] = unsafe { transmute(id.to_le()) };
            device_bytes.extend(id_bytes.iter().cloned());

            let name_length: [u8; 2] =
                unsafe { transmute((name_and_channels.0.as_bytes().len() as u16).to_le() as u16) };
            device_bytes.extend(name_length.iter().cloned());
            device_bytes.extend(name_and_channels.0.as_bytes());
            println!("Serializing device, name: {} Length: {}",
                     name_and_channels.0,
                     name_and_channels.0.as_bytes().len());
            println!("Name bytes: {:?}", name_length);

            let channel_bytes: [u8; 4] = unsafe { transmute(name_and_channels.1.to_le()) };
            device_bytes.extend(channel_bytes.iter().cloned());
        }

        let length_bytes: [u8; 4] =
            unsafe { transmute(((4 + 4 + 4 + device_bytes.len()) as i32).to_le()) };
        bytes.extend(length_bytes.iter().cloned());
        bytes.extend(type_bytes.iter().cloned());
        bytes.extend(device_count_bytes.iter().cloned());
        bytes.extend(device_bytes.iter().cloned());
        bytes
    }
}

pub struct MsgStartStreamRMS {
    pub msg_type: MsgType,
    pub device: i32,
    pub channels: Vec<i32>,
}

impl MsgStartStreamRMS {
    pub fn new() -> MsgStartStreamRMS {
        MsgStartStreamRMS {
            msg_type: MsgType::MSG_GET_RMS,
            device: 0,
            channels: Vec::new(),
        }
    }

    pub fn deserialized(mut data: Vec<u8>) -> MsgStartStreamRMS {
        let mut start_msg = MsgStartStreamRMS {
            msg_type: MsgType::MSG_GET_RMS,
            device: 0,
            channels: Vec::new(),
        };

        // In old protocol, multiple devices could be given.
        // This isn't currently supported, but may be in the future?
        // We're assuming that there's only single device for now.

        // Read amount of devices
        let _ = data[0] as i32 | ((data[1] as i32) << 8) | ((data[2] as i32) << 16) |
                           ((data[3] as i32) << 24);
        data = data[4..].to_vec();

        // Read devie ID
        start_msg.device = data[0] as i32 | ((data[1] as i32) << 8) | ((data[2] as i32) << 16) |
                           ((data[3] as i32) << 24);
        data = data[4..].to_vec();

        // Amount of channels - if there were multiple devices, we'd have multiple channel counts too.
        let channel_count = data[0] as i32 | ((data[1] as i32) << 8) | ((data[2] as i32) << 16) |
                            ((data[3] as i32) << 24);
        data = data[4..].to_vec();

        println!("Channel count: {}", channel_count);

        // Read channels
        for _ in 0..channel_count {
            let channel_id: i32 = data[0] as i32 | ((data[1] as i32) << 8) |
                                  ((data[2] as i32) << 16) |
                                  ((data[3] as i32) << 24);
            data = data[4..].to_vec();

            start_msg.channels.push(channel_id);
        }

        start_msg
    }
}

impl Serializable for MsgStartStreamRMS {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let type_bytes: [u8; 4] = unsafe { transmute((self.msg_type.clone() as i32).to_le()) };
        let device_bytes: [u8; 4] = unsafe { transmute((self.device.clone() as i32).to_le()) };


        // Device amount - make it possible to define multiple devices. For now, just 1 device supported.

        let mut channels_bytes = Vec::new();
        for i in 0..self.channels.len() {
            let channel_bytes: [u8; 4] = unsafe { transmute(self.channels[i].to_le()) };
            channels_bytes.extend(channel_bytes.iter().cloned());
        }

        let length_bytes: [u8; 4] =
            unsafe { transmute((4 + 4 + 4 + 4 + 4 + channels_bytes.len() as i32).to_le()) };
        bytes.extend(length_bytes.iter().cloned());
        bytes.extend(type_bytes.iter().cloned());
        let device_count: [u8; 4] = unsafe { transmute((1 as i32).to_le()) };
        bytes.extend(device_count.iter().cloned());
        bytes.extend(device_bytes.iter().cloned());
        let channel_count: [u8; 4] = unsafe { transmute((self.channels.len() as i32).to_le()) };
        bytes.extend(channel_count.iter().cloned());
        bytes.extend(channels_bytes.iter().cloned());

        bytes
    }
}

pub struct MsgRMSPacket {
    pub msg_type: MsgType,
    pub value: f32
}

impl MsgRMSPacket {
    pub fn new() -> MsgRMSPacket {
        MsgRMSPacket {
            msg_type: MsgType::MSG_RMS_PACKET,
            value: 0f32,
        }
    }

    pub fn deserialized(mut data: Vec<u8>) -> MsgRMSPacket {
        let mut start_msg = MsgRMSPacket {
            msg_type: MsgType::MSG_RMS_PACKET,
            value: 0f32,
        };

        let mut data_array = [0u8; 4];
        data_array[0] = data[0];
        data_array[1] = data[1];
        data_array[2] = data[2];
        data_array[3] = data[3];
        start_msg.value = unsafe { transmute::<[u8; 4], f32>(data_array) };

        start_msg
    }
}

impl Serializable for MsgRMSPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let type_bytes: [u8; 4] = unsafe { transmute((self.msg_type.clone() as i32).to_le()) };
        let value_bytes: [u8; 4] = unsafe { transmute(self.value as f32) };
        let length_bytes: [u8; 4] = unsafe { transmute((4 + 4 + 4 as i32).to_le()) };

        bytes.extend(length_bytes.iter().cloned());
        bytes.extend(type_bytes.iter().cloned());
        bytes.extend(value_bytes.iter().cloned());

        bytes
    }
}

pub struct MsgError {
    pub msg_type: MsgType,
    pub message: String,
}

impl MsgError {
    pub fn new() -> MsgError {
        MsgError {
            msg_type: MsgType::MSG_ERROR,
            message: "".to_string(),
        }
    }

    pub fn deserialized(mut data: Vec<u8>) -> MsgError {
        let mut start_msg = MsgError {
            msg_type: MsgType::MSG_ERROR,
            message: "".to_string(),
        };

        let message_length: u16 = data[0] as u16 | ((data[1] as u16) << 8);
        data = data[2..].to_vec();
        
        let data_clone = data.clone();
        let message = str::from_utf8(&data_clone[..message_length as usize]).unwrap();

        start_msg.message = message.to_string();

        start_msg
    }
}

impl Serializable for MsgError {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let type_bytes: [u8; 4] = unsafe { transmute((self.msg_type.clone() as i32).to_le()) };
        let message_length: [u8; 2] = unsafe { transmute ((self.message.as_bytes().len() as u16).to_le() as u16) };
        // Length: 4 from the length information (u32) itself, 4 from the length information of the error message, + error message length
        let length_bytes: [u8; 4] = unsafe { transmute((4 + 4 + 4 + self.message.as_bytes().len() as i32).to_le()) };

        bytes.extend(length_bytes.iter().cloned());
        bytes.extend(type_bytes.iter().cloned());
        bytes.extend(message_length.iter().cloned());
        bytes.extend(self.message.as_bytes());

        bytes
    }
}
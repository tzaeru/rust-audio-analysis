
extern crate portaudio as pa;

use analysis::traits::Sourcable;
use analysis::analysis::Chain;
use std::sync::Arc;
use std::sync::RwLock;
use std::collections::HashMap;

const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES: u32 = 256;
const INTERLEAVED: bool = true;

lazy_static! {
    static ref PORTAUDIO: pa::PortAudio = {
        let pa = pa::PortAudio::new();
        match pa
        {
            Result::Ok(val) => val,
            Result::Err(err) =>
              panic!("called `Result::unwrap()` on an `Err` value: {:?}", err),
        }
    };
}

pub struct PASource {
    device: u32,
    channels: Vec<i32>,

    stream: Option<pa::Stream<pa::NonBlocking, pa::Input<f32>>>,

    error: String
}

impl PASource {
    pub fn new(device: String, channels: Vec<i32>) -> PASource {
        PASource {
            device: device.parse::<u32>().unwrap(),
            channels: channels,

            stream: Option::None,

            error: "".to_string()
        }
    }
}

impl Sourcable for PASource {
    fn start(&mut self, chain: Arc<RwLock<Chain>>) -> () {
        let device_info = PORTAUDIO.device_info(pa::DeviceIndex { 0: self.device }).unwrap();

        let input_params = pa::StreamParameters::<f32>::new(pa::DeviceIndex { 0: self.device },
                                                            device_info.max_input_channels,
                                                            INTERLEAVED,
                                                            0.0f64);

        let channels = self.channels.to_vec();
        let audio_callback = move |pa::InputStreamCallbackArgs { buffer, frames, .. }| {
            // Unleave
            let mut unleaved_buffer:Vec<Vec<f32>> = Vec::new();

            // Initialize with empty arrays for each channel
            for _ in 0..channels.len()
            {
                unleaved_buffer.push(Vec::new());
            }
            // Iterate through the whole interleaved buffer, moving it to unleaved buffer.
            let mut i = 0i32;
            while i < buffer.len() as i32
            {
                // Iterate through all the channels we want.
                for j in 0..channels.len()
                {
                    // Since 'i' points to 1st channel, we'll take element i + channel index (starting from 0) from the interleaved buffer.
                    unleaved_buffer[j].push(buffer[i as usize + channels[j] as usize]);
                }

                // Increase index to next set of channels. I.e. index points to 1st interleaved channel for each sample frame.
                i += device_info.max_input_channels;
            }

            chain.write().unwrap().source_cb(unleaved_buffer, frames);

            if chain.write().unwrap().running == true
            {
                pa::Continue
            }
            else {
                pa::Complete
            }
        };


        let settings = pa::InputStreamSettings::new(input_params, SAMPLE_RATE, FRAMES);
        let mut stream = PORTAUDIO.open_non_blocking_stream(settings, audio_callback).unwrap();
        println!("Starting stream for realz..");

        let _ = stream.start();

        self.stream = Option::Some(stream);
    }

    fn stop(&mut self) -> () {}

    fn is_active(&self) -> bool
    {
        match self.stream
        {
            Some(ref stream) => return stream.is_active().unwrap(),
            None => return false
        }
    }

    fn get_devices() -> Result<HashMap<String, (String, i32)>, ()> where Self: Sized
    {
        let mut devices = HashMap::new();

        let default_host = PORTAUDIO.default_host_api().unwrap();

        for i in 0..PORTAUDIO.host_api_info(default_host).unwrap().device_count {
            let device_index =
                PORTAUDIO.api_device_index_to_device_index(default_host, i as i32).unwrap();
            let input_info = PORTAUDIO.device_info(device_index).unwrap();

            if input_info.max_input_channels <= 0
            {
                continue;
            }

            devices.insert(device_index.0.to_string(),
                           (input_info.name.to_string(), input_info.max_input_channels));
        }

        return Ok(devices);
    }

    fn get_and_clear_error(&self) -> Option<String>
    {
        return None;
    }
}

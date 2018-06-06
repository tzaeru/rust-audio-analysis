extern crate soundio;

use std::collections::HashMap;
use analysis::traits::Sourcable;
use analysis::traits::Chainable;
use analysis::analysis::Chain;

use std;

use std::sync::Arc;
use std::sync::RwLock;

lazy_static! {
    static ref SOUNDIO_CTX: soundio::Context<'static> = {
        let mut ctx = soundio::Context::new();
        ctx.set_app_name("RAA!");
        ctx.connect().unwrap();
        ctx.flush_events();

        println!("Soundio version: {}", soundio::version_string());
        println!("Current backend: {:?}", ctx.current_backend());
        for dev in ctx.input_devices().unwrap() {
            println!("Device {} ", dev.name());
            println!("Is raw: {}", dev.is_raw());
        }
        return ctx;
    };
}

pub struct SoundioSource<'a> {
    device: String,
    channels: Vec<i32>,

    stream: Option<soundio::InStream<'a>>,

    error: Arc<RwLock<String>>
}

impl<'a> SoundioSource<'a> {
    pub fn new (device: String, channels: Vec<i32>) -> SoundioSource<'a> {
        SoundioSource {
            device: device,
            channels: channels,

            stream: Option::None,

            error: Arc::new(RwLock::new("".to_string()))
        }
    }
}

impl<'a> Sourcable for SoundioSource<'a> {
    fn start(&mut self, chain: Arc<RwLock<Chain>>) -> () {

        let channels = self.channels.to_vec();
        let audio_callback = move |stream: &mut soundio::InStreamReader| {
            // Unleave
            let mut unleaved_buffer:Vec<Vec<f32>> = Vec::new();

            // Initialize with empty arrays for each channel
            for _ in 0..channels.len()
            {
                unleaved_buffer.push(Vec::new());
            }
            // Iterate through the whole interleaved buffer, moving it to unleaved buffer.
            let mut frames_left = stream.frame_count_max();
            loop {
                if let Err(e) = stream.begin_read(frames_left) {
                    println!("Error reading from stream: {}", e);
                    return;
                }

                for f in 0..stream.frame_count() {
                    for ch_i in 0..channels.len() {
                        if f%5000 == 0
                        {
                            //println!("{}", stream.sample::<i32>(channels[ch_i] as usize, f));
                        }

                        let mut value = (stream.sample::<i32>(channels[ch_i] as usize, f) as f32)/std::i32::MAX as f32;
                        if !value.is_normal()
                        {
                            value = 0.0f32;
                        }
                        unleaved_buffer[ch_i].push(value);
                    }
                }

                frames_left -= stream.frame_count();
                if frames_left <= 0 {
                    break;
                }

                stream.end_read();
            }
            match chain.try_write() {
                Ok(lock) => lock.source_cb(unleaved_buffer, stream.frame_count()),
                Err(e) => println!("Error writing to chain: {:?}", e)
            }

            return ();
        };

        let error_capture =  self.error.clone();
        let error_callback = move |error: soundio::Error| {
            println!("Error: {:?}", error);
            let mut write_lock = error_capture.write().unwrap();
            *write_lock = error.to_string();
        };

        let soundio_format = soundio::Format::S16LE;
        println!("Going to get default input device..");
        // pa::input<f32>, pa::NonBlocking
        let mut devices = SOUNDIO_CTX.input_devices().unwrap();
        let mut input_dev = &mut SOUNDIO_CTX.default_input_device().map_err(|_| "Error getting default input device".to_string()).unwrap();
        for device in devices.iter_mut()
        {
            if device.id() == self.device
            {
                println!("Found a match: {}", device.id());
                input_dev = device;
                break;
            }
        }

        let sample_rate = input_dev.nearest_sample_rate(44100);

        input_dev.sort_channel_layouts();
        let layout = input_dev.layouts()[0].clone();
        println!("Got default input device: {:?}", input_dev.name());
        println!("Aim: {:?}", input_dev.aim());
        println!("Formats: {:?}", input_dev.formats());
        println!("Sample rates: {:?}", input_dev.sample_rates());
        println!("Layouts: {:?}", input_dev.layouts());
        let mut stream = input_dev.open_instream(
            sample_rate,
            soundio_format,
            layout,
            (1.0/20.0 * sample_rate as f64) / sample_rate as f64,
            audio_callback,
            None::<fn()>,
            Some(error_callback),
        ).unwrap();
        println!("Starting soundio stream..");

        stream.start().unwrap();

        self.stream = Option::Some(stream);
    }

    fn stop(&mut self) -> () {
        println!("Stopping SoundIO source.");
        self.stream.take().unwrap().pause(true).unwrap();
        println!("Stopped SoundIO Source!");
    }

    fn is_active(&self) -> bool
    {
        match self.stream
        {
            Some(ref stream) => true,
            None => return false
        }
    }

    fn get_devices() -> Result<HashMap<String, (String, i32)>, ()> where Self: Sized
    {
        let mut devices = HashMap::new();

        for mut dev in SOUNDIO_CTX.input_devices().unwrap() {
            dev.sort_channel_layouts();
            devices.insert(dev.id(), (dev.name(), dev.layouts()[0].channels.len() as i32));
        }

        return Ok(devices);
    }

    fn get_and_clear_error(&self) -> Option<String>
    {
        let error_clone = self.error.read().unwrap().clone();
        if error_clone.len() <= 0
        {
            return None;
        }
        *self.error.write().unwrap() = "".to_string();

        return Some(error_clone);
    }
}

impl<'a> Drop for SoundioSource<'a> {
    fn drop(&mut self) {
        println!("Dropping SoundioSource!");
    }
}
//! A demonstration of constructing and using a non-blocking stream.
//!
//! Audio from the default input device is passed directly to the default output device in a duplex
//! stream, so beware of feedback!

extern crate portaudio as pa;
extern crate soundio;

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std;

use std::sync::Arc;
use std::sync::RwLock;

const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES: u32 = 256;
const CHANNELS: i32 = 2;
const INTERLEAVED: bool = true;

lazy_static! {
    static ref port_audio: pa::PortAudio = {
        let pa = pa::PortAudio::new();
        match pa
        {
            Result::Ok(val) => val,
            Result::Err(err) =>
              panic!("called `Result::unwrap()` on an `Err` value: {:?}", err),
        }
    };
}

lazy_static! {
    static ref soundio_ctx: soundio::Context<'static> = {
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

pub struct AArena {
    pub sourcables: HashMap<u64, Arc<RwLock<Sourcable>>>,
    pub chainables: HashMap<u64, Arc<RwLock<Chainable>>>,

    created_nodes: u64,
}

impl AArena {
    pub fn new() -> AArena {
        AArena {
            sourcables: HashMap::new(),
            chainables: HashMap::new(),
            created_nodes: 0,
        }
    }

    pub fn add_sourcable(&mut self, sourcable: Arc<RwLock<Sourcable>>) -> u64 {
        let id = self.created_nodes;

        self.sourcables.clear();
        self.sourcables.insert(id, sourcable);
        self.created_nodes += 1;

        return id;
    }

    pub fn add_chainable(&mut self, chainable: Arc<RwLock<Chainable>>) -> u64 {
        let id = self.created_nodes;

        self.chainables.insert(id, chainable);
        self.created_nodes += 1;

        return id;
    }
}

pub trait Sourcable {
    fn start(&mut self, chain: Arc<RwLock<AChain>>);
    fn stop(&self);
    fn get_devices() -> Result<HashMap<i32, (String, i32)>, ()> where Self: Sized;
    fn is_active(&self) -> bool;
}

pub trait Chainable {
    fn update(&mut self, buffer: &Vec<Vec<f32>>);
    fn output(&self) -> &Vec<f32>;
}

pub struct AChain {
    arena: Arc<RwLock<AArena>>,

    source: Option<u64>,
    nodes: Vec<u64>,

    pub running: bool,
}

impl AChain {
    pub fn new(arena: Arc<RwLock<AArena>>) -> AChain {
        AChain {
            arena: arena,

            source: Option::None,
            nodes: Vec::new(),

            running: false,
        }
    }


    pub fn start(&mut self, self_ref: Arc<RwLock<AChain>>) {
        match self.source {
            Some(source) =>
            {
                let arena_borrow = self.arena.read().unwrap();
                arena_borrow.sourcables[&source].write().unwrap().start(self_ref);
                self.running = true;
            },
            None => println!("No sourcable set."),
        }
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn source_cb(&self, buffer: Vec<Vec<f32>>, frames: usize) {

        for i in 0..self.nodes.len() {
            let node = &self.arena.read().unwrap().chainables[&self.nodes[i]];
            node.write().unwrap().update(&buffer);
        }
    }

    pub fn set_source(&mut self, source: u64) {
        self.source = Option::Some(source);
    }

    pub fn add_node(&mut self, node: u64) {
        self.nodes.push(node);
    }
}

pub struct PASource {
    device: u32,
    channels: Vec<i32>,

    stream: Option<pa::Stream<pa::NonBlocking, pa::Input<f32>>>,

    error: String
}

impl PASource {
    pub fn new(device: u32, channels: Vec<i32>) -> PASource {
        PASource {
            device: device,
            channels: channels,

            stream: Option::None,

            error: "".to_string()
        }
    }
}

impl Sourcable for PASource {
    fn start(&mut self, chain: Arc<RwLock<AChain>>) -> () {
        let device_info = port_audio.device_info(pa::DeviceIndex { 0: self.device }).unwrap();

        let input_params = pa::StreamParameters::<f32>::new(pa::DeviceIndex { 0: self.device },
                                                            device_info.max_input_channels,
                                                            INTERLEAVED,
                                                            0.0f64);

        let channels = self.channels.to_vec();
        let audio_callback = move |pa::InputStreamCallbackArgs { mut buffer, frames, time, .. }| {
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


        // pa::input<f32>, pa::NonBlocking
        let settings = pa::InputStreamSettings::new(input_params, SAMPLE_RATE, FRAMES);
        let mut stream = port_audio.open_non_blocking_stream(settings, audio_callback).unwrap();
        println!("Starting stream for realz..");

        stream.start();

        self.stream = Option::Some(stream);
    }

    fn stop(&self) -> () {}

    fn is_active(&self) -> bool
    {
        match self.stream
        {
            Some(ref stream) => return stream.is_active().unwrap(),
            None => return false
        }
    }

    fn get_devices() -> Result<HashMap<i32, (String, i32)>, ()> where Self: Sized
    {
        let mut devices = HashMap::new();

        let default_host = port_audio.default_host_api().unwrap();

        for i in 0..port_audio.host_api_info(default_host).unwrap().device_count {
            let device_index =
                port_audio.api_device_index_to_device_index(default_host, i as i32).unwrap();
            let input_info = port_audio.device_info(device_index).unwrap();

            if input_info.max_input_channels <= 0
            {
                continue;
            }

            devices.insert(device_index.0 as i32,
                           (input_info.name.to_string(), input_info.max_input_channels));
        }

        return Ok(devices);
    }

}

pub struct SoundioSource<'a> {
    device: u32,
    channels: Vec<i32>,

    stream: Option<soundio::InStream<'a>>,

    error: String
}

impl<'a> SoundioSource<'a> {
    pub fn new (device: u32, channels: Vec<i32>) -> SoundioSource<'a> {
        SoundioSource {
            device: device,
            channels: channels,

            stream: Option::None,

            error: "".to_string()
        }
    }
}

impl<'a> Sourcable for SoundioSource<'a> {
    fn start(&mut self, chain: Arc<RwLock<AChain>>) -> () {

        let audio_callback = move |stream: &mut soundio::InStreamReader| {
            // Unleave
            let mut unleaved_buffer:Vec<Vec<f32>> = Vec::new();
            println!("{:?}", stream.get_latency());

            // Initialize with empty arrays for each channel
            for _ in 0..stream.channel_count()
            {
                unleaved_buffer.push(Vec::new());
            }
            // Iterate through the whole interleaved buffer, moving it to unleaved buffer.

            let frame_count_max = stream.frame_count_max();
            if let Err(e) = stream.begin_read(frame_count_max) {
                println!("Error reading from stream: {}", e);
                return;
            }

            for f in 0..stream.frame_count() {
                for c in 0..stream.channel_count() {
                    if f%2000 == 0
                    {
                        //println!("{}", stream.sample::<i32>(c, f));
                    }
                    // In reality you shouldn't write to disk in the callback, but have some buffer instead.
                    let mut value = (stream.sample::<i32>(c, f) as f32)/std::i32::MAX as f32;
                    if !value.is_normal()
                    {
                        value = 0.0f32;
                    }
                    unleaved_buffer[c].push(value);
                }
            }

            chain.write().unwrap().source_cb(unleaved_buffer, stream.frame_count());

            return ();
        };

        let channels = 2;
        let sample_rate = 48000;
        let soundio_format = soundio::Format::S16LE;
        println!("Going to get default input device..");
        // pa::input<f32>, pa::NonBlocking
        let input_dev = &soundio_ctx.input_devices().map_err(|_| "Error getting default input device".to_string()).unwrap()[1];
        let default_layout = soundio::ChannelLayout::get_default(channels as _);
        println!("Got default input device: {:?}", input_dev.name());
        println!("Aim: {:?}", input_dev.aim());
        println!("Formats: {:?}", input_dev.formats());
        println!("Sample rates: {:?}", input_dev.sample_rates());
        println!("Layouts: {:?}", input_dev.layouts());
        let mut stream = input_dev.open_instream(
            sample_rate,
            soundio_format,
            default_layout,
            (1.0/20.0 * sample_rate as f64) / sample_rate as f64,
            audio_callback,
            None::<fn()>,
            None::<fn(soundio::Error)>,
        ).unwrap();
        println!("Starting soundio stream..");

        stream.start().unwrap();

        self.stream = Option::Some(stream);
    }

    fn stop(&self) -> () {}

    fn is_active(&self) -> bool
    {
        match self.stream
        {
            Some(ref stream) => true,
            None => return false
        }
    }

    fn get_devices() -> Result<HashMap<i32, (String, i32)>, ()> where Self: Sized
    {
        let mut devices = HashMap::new();

        let mut i = 0;
        for dev in soundio_ctx.input_devices().unwrap() {
            devices.insert(i as i32, (dev.name(), dev.current_layout().channels.len() as i32));
            i += 1;
        }

        return Ok(devices);
    }
}

pub struct RMS {
    buffer: Vec<f32>,
}

impl RMS {
    pub fn new() -> RMS {
        RMS { buffer: Vec::new() }
    }
}

impl Chainable for RMS {
    fn update(&mut self, buffer: &Vec<Vec<f32>>) {

        let mut rms = 0f32;
        for i in 0..buffer.len()
        {
            let mut square_sum = 0.0f32;
            for x in 0..buffer[i].len() {
                square_sum += buffer[i][x] * buffer[i][x];
            }

            let square_mean = square_sum * 1.0f32 / buffer.len() as f32;

            rms += f32::sqrt(square_mean);
        }

        rms /= buffer.len() as f32;

        self.buffer = Vec::new();
        self.buffer.push(rms);
    }

    fn output(&self) -> &Vec<f32> {
        &self.buffer
    }
}

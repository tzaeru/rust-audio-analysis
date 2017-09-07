//! A demonstration of constructing and using a non-blocking stream.
//!
//! Audio from the default input device is passed directly to the default output device in a duplex
//! stream, so beware of feedback!

extern crate portaudio as pa;

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

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

pub struct AArena {
    pub sourcables: HashMap<u64, Arc<RefCell<Sourcable>>>,
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

    pub fn add_sourcable(&mut self, sourcable: Arc<RefCell<Sourcable>>) -> u64 {
        let id = self.created_nodes;

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
                arena_borrow.sourcables[&source].borrow_mut().start(self_ref);
                self.running = true;
            },
            None => println!("No sourcable set."),
        }
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn source_cb(&self, buffer: Vec<Vec<f32>>, frames: usize) {
        //println!("Got callback?");

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
}

impl PASource {
    pub fn new(device: u32, channels: Vec<i32>) -> PASource {
        PASource {
            device: device,
            channels: channels,

            stream: Option::None,
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

            let mut i = 0i32;
            while i < buffer.len() as i32
            {
                for j in 0..channels.len()
                {
                    unleaved_buffer[j].push(buffer[i as usize + channels[j] as usize]);
                }

                // Increase index to next set of channels. I.e. index points to 1st interleaved channel for each sample.
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
        self.buffer = Vec::new();

        let mut square_sum = 0.0f32;
        for x in 0..buffer.len() {
            square_sum += buffer[0][x] * buffer[0][x];
        }

        let square_mean = square_sum * 1.0f32 / buffer.len() as f32;

        let rms = f32::sqrt(square_mean);

        self.buffer.push(rms);
    }

    fn output(&self) -> &Vec<f32> {
        &self.buffer
    }
}

pub fn init() {}

pub fn get_devices<'a>() -> Result<HashMap<i32, (&'a str, i32)>, pa::Error> {
    let mut devices = HashMap::new();

    let default_host = try!(port_audio.default_host_api());

    for i in 0..port_audio.host_api_info(default_host).unwrap().device_count {
        let device_index =
            try!(port_audio.api_device_index_to_device_index(default_host, i as i32));
        let input_info = try!(port_audio.device_info(device_index));

        if input_info.max_input_channels <= 0
        {
            continue;
        }

        devices.insert(device_index.0 as i32,
                       (input_info.name, input_info.max_input_channels));
    }

    return Ok(devices);
}

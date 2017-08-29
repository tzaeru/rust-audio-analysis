//! A demonstration of constructing and using a non-blocking stream.
//!
//! Audio from the default input device is passed directly to the default output device in a duplex
//! stream, so beware of feedback!

extern crate portaudio as pa;

use std::collections::HashMap;

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

pub fn init()
{

}

pub fn get_devices<'a>() -> Result<HashMap<&'a str, i32>, pa::Error> {
    let mut devices = HashMap::new();

    let default_host = try!(port_audio.default_host_api());

    for i in 0..port_audio.host_api_info(default_host).unwrap().device_count
    {
        let device_index = try!(port_audio.api_device_index_to_device_index(default_host, i as i32));
        let device = try!(port_audio.default_input_device());
        let input_info = try!(port_audio.device_info(device));

        devices.insert(input_info.name, 
            input_info.max_input_channels);
    }

    return Ok(devices);
}

pub fn run() -> Result<(), pa::Error> {

    println!("PortAudio:");
    println!("version: {}", port_audio.version());
    println!("version text: {:?}", port_audio.version_text());
    println!("host count: {}", try!(port_audio.host_api_count()));

    let default_host = try!(port_audio.default_host_api());
    println!("default host: {:#?}", port_audio.host_api_info(default_host));

    let def_input = try!(port_audio.default_input_device());
    let input_info = try!(port_audio.device_info(def_input));
    println!("Default input device info: {:#?}", &input_info);

    // Construct the input stream parameters.
    let latency = input_info.default_low_input_latency;
    let input_params = pa::StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

    let def_output = try!(port_audio.default_output_device());
    let output_info = try!(port_audio.device_info(def_output));
    println!("Default output device info: {:#?}", &output_info);

    // Construct the output stream parameters.
    let latency = output_info.default_low_output_latency;
    let output_params = pa::StreamParameters::new(def_output, CHANNELS, INTERLEAVED, latency);

    // Check that the stream format is supported.
    try!(port_audio.is_duplex_format_supported(input_params, output_params, SAMPLE_RATE));

    // Construct the settings with which we'll open our duplex stream.
    let settings = pa::DuplexStreamSettings::new(input_params, output_params, SAMPLE_RATE, FRAMES);

    // Once the countdown reaches 0 we'll close the stream.
    let mut count_down = 1.0;

    // Keep track of the last `current_time` so we can calculate the delta time.
    let mut maybe_last_time = None;

    // We'll use this channel to send the count_down to the main thread for fun.
    let (sender, receiver) = ::std::sync::mpsc::channel();

    // A callback to pass to the non-blocking stream.
    let callback = move |pa::DuplexStreamCallbackArgs { in_buffer, out_buffer, frames, time, .. }| {
        let current_time = time.current;
        let prev_time = maybe_last_time.unwrap_or(current_time);
        let dt = current_time - prev_time;
        count_down -= dt;
        maybe_last_time = Some(current_time);

        assert!(frames == FRAMES as usize);
        sender.send(count_down).ok();

        // Pass the input straight to the output - BEWARE OF FEEDBACK!
        for (output_sample, input_sample) in out_buffer.iter_mut().zip(in_buffer.iter()) {
            *output_sample = *input_sample;
        }

        if count_down > 0.0 { pa::Continue } else { pa::Complete }
    };

    // Construct a stream with input and output sample types of f32.
    let mut stream = try!(port_audio.open_non_blocking_stream(settings, callback));

    try!(stream.start());

    // Loop while the non-blocking stream is active.
    while let true = try!(stream.is_active()) {

        // Do some stuff!
        while let Ok(count_down) = receiver.try_recv() {
            println!("count_down: {:?}", count_down);
        }

    }

    try!(stream.stop());


    Ok(())
}
#[macro_use]
extern crate lazy_static;

extern crate audio_analysis;
use audio_analysis::analysis;

fn main() {
    // The statements here will be executed when the compiled binary is called

    // Print text to the console
    println!("Hello World3!");

    analysis::pa_interface::run().unwrap()
}
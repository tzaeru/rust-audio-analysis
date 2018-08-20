extern crate raa;
use raa::analysis;

use std::sync::Arc;
use std::sync::RwLock;

use std::{thread, time};

fn main() {
    let mut arena = analysis::analysis::Arena::new();

    let source = Arc::new(RwLock::new(analysis::soundio_source::SoundioSource::new("{0.0.1.00000000}.{625fb92c-394b-482e-82ed-30b560d85d8f}".to_string(), vec![0])));
    let source_id = arena.add_sourcable(source);

    let rms = Arc::new(RwLock::new(analysis::rms::RMS::new()));
    let rms_id = arena.add_chainable(rms);

    let arena_rc = Arc::new(RwLock::new(arena));

    let mut chain = analysis::analysis::Chain::new(arena_rc.clone());
    chain.set_source(source_id);
    chain.add_node(rms_id);

    let chain_rc = Arc::new(RwLock::new(chain));

    chain_rc.write().unwrap().start(chain_rc.clone());

    let millis = time::Duration::from_millis(20);

    loop {
        let arena_borrow = arena_rc.read().unwrap();
        if arena_borrow.chainables[&rms_id].read().unwrap().output().len() > 0
        {
            println!("RMS: {}", arena_borrow.chainables[&rms_id].read().unwrap().output()[0]);
        }
        thread::sleep(millis);
    }
}
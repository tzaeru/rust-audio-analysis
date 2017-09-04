extern crate audio_analysis;
use audio_analysis::analysis;
use std::cell::RefCell;
use std::rc::Rc;

use std::sync::Arc;
use std::sync::RwLock;

fn main() {
    let mut arena = analysis::pa_interface::AArena::new();

    let source = Arc::new(RefCell::new(analysis::pa_interface::PASource::new(0, vec![0])));
    let source_id = arena.add_sourcable(source);

    let rms = Arc::new(RwLock::new(analysis::pa_interface::RMS::new()));
    let rms_id = arena.add_chainable(rms);

    let arena_rc = Rc::new(arena);

    let mut chain = analysis::pa_interface::AChain::new(arena_rc.clone());
    chain.set_source(source_id);
    chain.add_node(rms_id);

    let chain_rc = Rc::new(chain);

    chain_rc.start(chain_rc.clone());

    loop {
        if arena_rc.chainables[&rms_id].read().unwrap().output().len() > 0
        {
            println!("RMS: {}", arena_rc.chainables[&rms_id].read().unwrap().output()[0]);
        }
    }
}
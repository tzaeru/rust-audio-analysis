//! A demonstration of constructing and using a non-blocking stream.
//!
//! Audio from the default input device is passed directly to the default output device in a duplex
//! stream, so beware of feedback!

extern crate soundio;

use std::collections::HashMap;
use analysis::traits::Sourcable;
use analysis::traits::Chainable;

use std::sync::Arc;
use std::sync::RwLock;

pub struct Arena {
    pub sourcables: HashMap<u64, Arc<RwLock<Sourcable>>>,
    pub chainables: HashMap<u64, Arc<RwLock<Chainable>>>,

    created_nodes: u64,
}

impl Arena {
    pub fn new() -> Arena {
        Arena {
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

    pub fn remove_sourcable(&mut self, id: u64) {

        self.sourcables.remove(&id);
    }

    pub fn remove_chainable(&mut self, id: u64) {
        self.chainables.remove(&id);
    }
}

pub struct Chain {
    arena: Arc<RwLock<Arena>>,

    source: Option<u64>,
    nodes: Vec<u64>,

    pub running: bool,
}

impl Chain {
    pub fn new(arena: Arc<RwLock<Arena>>) -> Chain {
        Chain {
            arena: arena,

            source: Option::None,
            nodes: Vec::new(),

            running: false,
        }
    }


    pub fn start(&mut self, self_ref: Arc<RwLock<Chain>>) {
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

    pub fn source_cb(&self, buffer: Vec<Vec<f32>>, _frames: usize, samplerate: u32) {
        for i in 0..self.nodes.len() {
            let node = &self.arena.read().unwrap().chainables[&self.nodes[i]];
            node.write().unwrap().update(&buffer, samplerate);
        }
    }

    pub fn set_source(&mut self, source: u64) {
        self.source = Option::Some(source);
    }

    pub fn add_node(&mut self, node: u64) {
        self.nodes.push(node);
    }
}

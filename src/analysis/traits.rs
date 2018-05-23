use std::cell::RefCell;
use std;

use std::sync::Arc;
use std::sync::RwLock;
use std::rc::Rc;
use std::collections::HashMap;
use analysis::analysis::Chain;


pub trait Sourcable {
    fn start(&mut self, chain: Arc<RwLock<Chain>>);
    fn stop(&self);
    fn get_devices() -> Result<HashMap<String, (String, i32)>, ()> where Self: Sized;
    fn is_active(&self) -> bool;
    fn get_and_clear_error(&self) -> Option<String>;
}

pub trait Chainable {
    fn update(&mut self, buffer: &Vec<Vec<f32>>);
    fn output(&self) -> &Vec<f32>;
}

#![feature(box_patterns)]
#![feature(box_syntax)]

extern crate console_error_panic_hook;
#[cfg(feature = "release")]
extern crate wee_alloc;
use std::panic;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
use log::Level;
use wasm_bindgen::prelude::*;



pub mod discovery;


#[wasm_bindgen(start)]
pub fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Debug).expect("error initializing log");
}

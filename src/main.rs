#![feature(exclusive_range_pattern)]

use std::env;
mod unit;
mod utils;
mod game;
mod bridge;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match args[1].as_ref() {
            "selftest" => { println!("Selftest"); },
            "bridge" | _ => { bridge::start(false, "COM20", "COM5"); }
        }
    } else {
        bridge::start(true, "COM18", "COM6");
    }
}
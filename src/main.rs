#![feature(exclusive_range_pattern)]

use std::io::{self, Write};
use std::time::Duration;

use wacca::TouchLink;

pub mod wacca;

fn main() {
    let mut gleft = serialport::new("COM5", 115_200)
    .timeout(Duration::from_millis(10))
    .open()
    .expect("Failed to open serial port");

    let mut gright = serialport::new("COM6", 115_200)
    .timeout(Duration::from_millis(10))
    .open()
    .expect("Failed to open serial port");
    let mut serial_buf: Vec<u8> = vec![0; 1000];

    let mut left = TouchLink {
        scan_active: false,
        port: &mut gleft,
        sync_board_version: "190523",
        buffer: vec![0; 1000],
        syncboardparams: wacca::SyncBoardParams::get(),
        side: true,
        touchbuffer: vec![0; 36],
    };

    let mut right = TouchLink {
        scan_active: false,
        port: &mut gright,
        sync_board_version: "190523",
        buffer: vec![0; 1000],
        syncboardparams: wacca::SyncBoardParams::get(),
        side: false,
        touchbuffer: vec![0; 36],
    };

    loop {
        left.poll();
        right.poll();
    }
}

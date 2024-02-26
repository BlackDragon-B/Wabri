#![feature(exclusive_range_pattern)]

use core::time;
use std::io::{self, Write};
use std::time::Duration;

use wacca::TouchLink;

pub mod wacca;

fn main() {
    let mut gleft = serialport::new("/dev/tnt0", 115_200)
    .timeout(Duration::from_millis(10))
    .open()
    .expect("Failed to open serial port");

    let mut gright = serialport::new("/dev/tnt2", 115_200)
    .timeout(Duration::from_millis(10))
    .open()
    .expect("Failed to open serial port");

    let mut wleft = serialport::new("/dev/ttyUSB1", 921_600)
    .timeout(Duration::from_millis(10))
    .open()
    .expect("Failed to open serial port");

    let mut wright = serialport::new("/dev/ttyUSB0", 921_600)
    .timeout(Duration::from_millis(10))
    .open()
    .expect("Failed to open serial port");

    let mut left = TouchLink {
        scan_active: false,
        port: &mut gleft,
        sync_board_version: "190523",
        buffer: vec![0; 1000],
        buffer2: vec![0; 1000],
        syncboardparams: wacca::SyncBoardParams::get(),
        side: true,
        touchbuffer: vec![0; 36],
        wport: &mut wleft,
    };

    let mut right = TouchLink {
        scan_active: false,
        port: &mut gright,
        sync_board_version: "190523",
        buffer: vec![0; 1000],
        buffer2: vec![0; 1000],
        syncboardparams: wacca::SyncBoardParams::get(),
        side: false,
        touchbuffer: vec![0; 36],
        wport: &mut wright,
    };

    loop {
        left.poll();
        right.poll();
    }
}

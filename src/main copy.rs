#![feature(exclusive_range_pattern)]

use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use wacca::TouchLink;

pub mod wacca;

#[tokio::main]
async fn main() {
    let mut gleft = serialport::new("/dev/tnt0", 115_200)
    .timeout(Duration::from_millis(10))
    .open()
    .expect("Failed to open serial port");

    let mut gright = serialport::new("/dev/tnt2", 115_200)
    .timeout(Duration::from_millis(10))
    .open()
    .expect("Failed to open serial port");

    let mut wleft = serialport::new("/dev/ttyUSB0", 921_600)
    .timeout(Duration::from_millis(10))
    .open()
    .expect("Failed to open serial port");

    let mut wright = serialport::new("/dev/ttyUSB1", 921_600)
    .timeout(Duration::from_millis(10))
    .open()
    .expect("Failed to open serial port");

    let mut left: TouchLink<'_> = TouchLink {
        scan_active: false,
        port: Arc::new(Mutex::new(gleft)),
        sync_board_version: "190523",
        buffer: vec![0; 1000],
        buffer2: vec![0; 1000],
        syncboardparams: wacca::SyncBoardParams::get(),
        side: false,
        touchbuffer: Arc::new(Mutex::new(vec![0; 36])),
        wport: Arc::new(Mutex::new(wleft)),
        time: Instant::now(),
        pollrate: Duration::from_millis(1),
    };

    let mut right = TouchLink {
        scan_active: false,
        port: Arc::new(Mutex::new(gright)),
        sync_board_version: "190523",
        buffer: vec![0; 1000],
        buffer2: vec![0; 1000],
        syncboardparams: wacca::SyncBoardParams::get(),
        side: true,
        touchbuffer: Arc::new(Mutex::new(vec![0; 36])),
        wport: Arc::new(Mutex::new(wright)),
        time: Instant::now(),
        pollrate: Duration::from_millis(1),
    };

    let mut lefttiming2: Instant = Instant::now();
    let mut righttiming: Instant = Instant::now();
    let mut righttiming2: Instant = Instant::now();
    let ltouch = &left.touchbuffer;
    let ltouch2 = &left.touchbuffer;
    let lwport = &left.wport;
    let lport = &left.port;
    let rtouch = &right.touchbuffer;
    let rtouch2 = &right.touchbuffer;
    let rwport = &right.wport;
    let rport = &right.port;

    tokio::spawn(async move {
        let mut timing: Instant = Instant::now();
        let ltoucha = ltouch.clone();
        loop {
            if timing.elapsed() >= Duration::from_millis(8) {
                timing = Instant::now();
                if left.scan_active {
                    //wacca::touch_recv(lwport.clone(), ltouch.clone(), false)
                }
            }
        }
    });

    tokio::spawn(async move {
        let mut timing: Instant = Instant::now();
        loop {
            if timing.elapsed() >= Duration::from_millis(8) {
                timing = Instant::now();
                if left.scan_active {
                    //wacca::touch_recv(lwport.clone(), ltouch.clone(), false)
                }
            }
        }
    });



    tokio::spawn(async move {
        loop {
            if righttiming.elapsed() >= Duration::from_millis(8) {
                if left.scan_active {
                    
                }
            }
        }
    });

    tokio::spawn(async move {
        loop {
            if righttiming2.elapsed() >= Duration::from_millis(8) {
                if right.scan_active {
                    
                }
            }
        }
    });
    
/*     if left.scan_active {
        if lefttiming.elapsed() >= Duration::from_millis(8) {
            lefttiming = Instant::now();
            tokio::spawn(async {wacca::touch_recv(lwport, ltouch, false)});
            tokio::spawn(async {wacca::touch_send(lport, ltouch2)});    
        }
    } else {
        left.poll();
    }


    if right.scan_active {
        if righttiming.elapsed() >= Duration::from_millis(8) {
            righttiming = Instant::now();
            tokio::spawn(async {wacca::touch_recv(rwport, rtouch, true)});
            tokio::spawn(async {wacca::touch_send(rport, rtouch2)});
        }
    } else {
        right.poll();
 */    //}

}

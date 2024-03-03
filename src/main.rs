#![feature(exclusive_range_pattern)]

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use serialport::SerialPort;
use wacca::{handle_data, SyncBoardParams, TouchLink};

pub mod wacca;

#[tokio::main]
async fn main() {

    struct touch<'a> {
        wedge: Arc<Mutex<Box<dyn SerialPort>>>,
        game: Arc<Mutex<Box<dyn SerialPort>>>,
        sync_board_params: SyncBoardParams<'a>,
    }
    let mut gleft = Arc::new(Mutex::new(serialport::new("/dev/tnt0", 115_200)
    .timeout(Duration::from_millis(10))
    .open()
    .expect("Failed to open serial port")));

    let mut gright = Arc::new(Mutex::new(serialport::new("/dev/tnt2", 115_200)
    .timeout(Duration::from_millis(10))
    .open()
    .expect("Failed to open serial port")));

    let mut wleft = Arc::new(Mutex::new(serialport::new("/dev/ttyUSB0", 921_600)
    .timeout(Duration::from_millis(10))
    .open()
    .expect("Failed to open serial port")));

    let mut wright = Arc::new(Mutex::new(serialport::new("/dev/ttyUSB1", 921_600)
    .timeout(Duration::from_millis(10))
    .open()
    .expect("Failed to open serial port")));

    let touchpairs = vec![touch {wedge: wleft, game: gleft, sync_board_params: SyncBoardParams::get(false)}, touch {wedge: wright.clone(), game: gright, sync_board_params: SyncBoardParams::get(true)}];
    // let ltouch = &left.touchbuffer;
    // let ltouch2 = &left.touchbuffer;
    // let lwport = &left.wport;
    // let lport = &left.port;
    for i in touchpairs.into_iter() {

        let gleft = i.game;
        let wleft = i.wedge;
        let leftparams: SyncBoardParams<'_> = i.sync_board_params;
        let lefttouchbuffer = Arc::new(Mutex::new(vec![0; 36]));
        let lefttouchactive = Arc::new(AtomicBool::new(false));
        let leftactive1 = lefttouchactive.clone();
        let leftactive2 = lefttouchactive.clone();
        let leftactive3 = lefttouchactive.clone();
        // let rtouch = &right.touchbuffer;
        // let rtouch2 = &right.touchbuffer;
        // let rwport = &right.wport;
        // let rport = &right.port;
        let righttouchbuffer = Arc::new(Mutex::new(vec![0; 36]));
        let righttouchactive: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

        let lefttouchbuffer1: Arc<Mutex<Vec<u8>>> = lefttouchbuffer.clone();
        let lport1: Arc<Mutex<Box<dyn SerialPort>>> = gleft.clone();
        let lport2: Arc<Mutex<Box<dyn SerialPort>>> = wleft.clone();

        let (tx, mut rx) = mpsc::channel();

        tokio::spawn(async move {
            let ltouch = &lefttouchbuffer;
            let lport = wleft;
            let mut timing: Instant = Instant::now();
            loop {
                if timing.elapsed() >= Duration::from_millis(8) {
                    if leftactive2.load(Ordering::Relaxed) {
                        let touchbuffer: Arc<Mutex<Vec<u8>>> = ltouch.clone();
                        let port: Arc<Mutex<Box<dyn SerialPort>>> = lport.clone();
                        timing = Instant::now();
                        //println!("bib {:?}",);
                        let _ = tx.send(wacca::touch_recv(port, leftparams.side));    
                    }

                } else {thread::sleep(Duration::from_millis(1))}
            }
        });


        tokio::spawn(async move {
            let ltouch = &lefttouchbuffer1;
            let lport = gleft;
            let mut touchbuffer: Vec<u8> = vec![0; 36];
            let mut timing: Instant = Instant::now();
            loop {
                if timing.elapsed() >= Duration::from_millis(8) {
                    if leftactive1.load(Ordering::Relaxed) {
                        let port: Arc<Mutex<Box<dyn SerialPort>>> = lport.clone();    
                        timing = Instant::now();
                        match rx.try_recv() {
                            Ok(d) => {
                                for b in 0..d.len() {
                                    touchbuffer[b+1] = d[b];
                                }
                            },
                            Err(err) => {},
                        };
                        wacca::touch_send(port, &mut touchbuffer);    
                    }
                } else {thread::sleep(Duration::from_millis(1))}
            }
        });

        tokio::spawn(async move {
            let port1 = &lport1;
            let port2 = &lport2;
            let leftactive3 = &lefttouchactive;
            let mut serialbuffer = vec![0; 1000];
            let mut timing: Instant = Instant::now();

            loop {
                if timing.elapsed() >= Duration::from_millis(100) {
                    if !leftactive3.load(Ordering::Relaxed) {
                        timing = Instant::now();
                        let binding1 = port1.clone();
                        let mut port = binding1.lock().unwrap();
                        let binding2 = port2.clone();
                        let mut port2 = binding2.lock().unwrap();
                        let mut leftactive3 = leftactive3.clone();
        
                        match port.read(serialbuffer.as_mut_slice()) {
                            Ok(t) => handle_data(&serialbuffer[..t].to_vec(), port, port2, &leftparams, leftactive3),
                            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                } else {thread::sleep(Duration::from_millis(100))};
            }


        });
    }

    loop {
        let mut timing: Instant = Instant::now();
        if timing.elapsed() >= Duration::from_millis(8) {
            timing = Instant::now();
            // let mut serialbuffer = vec![0; 1000];
            // let mut wright= loop {
            //     match rport1.lock() {
            //         Ok(g) => {break g},
            //         Err(e) => ()
            //     }
            // };
            // let mut gright = rport2.lock().expect("hi");
            // match gright.read(serialbuffer.as_mut_slice()) {
            //     Ok(t) => { println!("hi"); handle_data(&serialbuffer[..t].to_vec(), gright, wright, &rightparams, righttouchactive2.lock().unwrap()) },
            //     Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            //     Err(e) => eprintln!("{:?}", e),
            // }
        } else {thread::sleep(Duration::from_millis(8))}
    }    
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

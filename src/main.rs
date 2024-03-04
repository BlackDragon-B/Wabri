#![feature(exclusive_range_pattern)]
use std::{io, sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}, Arc}, thread, time::Duration};

use serial2::SerialPort;
use wacca::{calc_checksum, SyncBoardParams, TouchBinding};

mod wacca;

fn main() {
    let game_left = Arc::new(SerialPort::open("/dev/tnt0", 115200).expect("Unable to open serialport"));

    let game_right = Arc::new(SerialPort::open("/dev/tnt2", 115200).expect("Unable to open serialport"));

    let bindings = vec![
        TouchBinding {
            game_serial: game_left,
            side: false,
        },
        TouchBinding {
            game_serial: game_right,
            side: true,
        }
    ];

    for binding in bindings.into_iter() {
        println!("loading thread");
        //let we need some params
        let params = SyncBoardParams::get(&binding.side);
        //Sync board -> Game writer thread logic
        let (writer_tx, writer_rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();
        let command_tx = writer_tx.clone();

        let game_serial_1 = binding.game_serial.clone();
        thread::spawn(move || {
            loop {
                match writer_rx.recv() {
                    Ok(data) => {
                        let _ = SerialPort::write(&game_serial_1,&data);
                    },
                    Err(e) => {println!("{:?}",e)}
                };
            }
        });
        //AtomicBool to check if touch should be active
        let touch_active = Arc::new(AtomicBool::new(false));

        //command handler for handshake between game and sync board
        let touch_active_2 = touch_active.clone();
        let game_serial_2 = binding.game_serial.clone();

        thread::spawn(move || {
            let mut serialbuffer: Vec<u8> = vec![0; 1000];
            loop {
                let data = loop {
                    match SerialPort::read(&game_serial_2, serialbuffer.as_mut_slice()) {
                        Ok(t) => {break serialbuffer[..t].to_vec()},
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(e) => eprintln!("{:?}", e),
                    }
                };
                match wacca::handle_data(&data, &params, touch_active_2.clone()) {
                    Some(d) => command_tx.send(d).expect("Something has gone wrong!"),
                    None => (),
                };
            };
        });

        //Send touch packet
        let touch_active_1 = touch_active.clone();
        thread::spawn(move || {
            loop {
                for i in 0..127 {
                    if touch_active_1.load(Ordering::Relaxed) {
                        let mut touchbuffer: Vec<u8> = vec![0; 36];
                        touchbuffer[0] = 129;
                        touchbuffer[34] = i;
                        touchbuffer[35] = 128;
                        touchbuffer[35] = calc_checksum(&touchbuffer);
                        if touchbuffer[34] == 127 {touchbuffer[34] = 0};
                        let _ = writer_tx.send(touchbuffer);
                    }
                }
                thread::sleep(Duration::from_millis(8));
            }
        });
    }
    loop {
        thread::sleep(Duration::from_secs(1));
    }

}

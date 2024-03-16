#![feature(exclusive_range_pattern)]
use std::{io, sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}, Arc, Mutex}, thread, time::{Duration, Instant}};

use serial2::SerialPort;
use wacca::{calc_checksum, SyncBoardParams, TouchBinding, CommandPacket};

use crate::wacca::fix_touch;

mod wacca;

fn main() {
    let game_left = Arc::new(SerialPort::open("/dev/tnt0", 115200).expect("Unable to open serialport"));

    let game_right = Arc::new(SerialPort::open("/dev/tnt2", 115200).expect("Unable to open serialport"));

    let wedge_left = Arc::new(Mutex::new(SerialPort::open("/dev/ttyUSB0", 921600).expect("Unable to open serialport")));

    let wedge_right = Arc::new(Mutex::new(SerialPort::open("/dev/ttyUSB1", 921600).expect("Unable to open serialport")));

    let bindings = vec![
        TouchBinding {
            game_serial: game_left,
            wedge_serial: wedge_left,
            side: false,
        },
        TouchBinding {
            game_serial: game_right,
            wedge_serial: wedge_right,
            side: true,
        }
    ];

    for binding in bindings.into_iter() {
        println!("loading thread");

        //let we need some params
        let params = Arc::new(SyncBoardParams::get(&binding.side));

        //Sync board -> Game writer thread logic
        let (writer_tx, writer_rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();
        let command_tx = writer_tx.clone();

        let game_serial_1 = binding.game_serial.clone();
        let wedge_serial_1 = binding.wedge_serial.clone();
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
        let touch_active_1 = touch_active.clone();
        let touch_active_2 = touch_active.clone();
        let game_serial_2 = binding.game_serial.clone();
        let wedge_serial_1 = &binding.wedge_serial.clone();
        let wedge_serial_2 = &binding.wedge_serial.clone();
        let params_1 = params.clone();

        thread::spawn(move || {
            let wedge = &binding.wedge_serial.clone();
            let mut serialbuffer: Vec<u8> = vec![0; 1000];
            loop {
                let data = loop {
                    match SerialPort::read(&game_serial_2, serialbuffer.as_mut_slice()) {
                        Ok(t) => {break serialbuffer[..t].to_vec()},
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(e) => eprintln!("{:?}", e),
                    }
                };
                let mut wedge = loop {
                    match wedge.lock() {
                    Ok(g) => {break g},
                    Err(e) => ()
                    }
                };
                match wacca::handle_data(&data, &params, touch_active_2.clone(), wedge) {
                    Some(d) => command_tx.send(d).expect("Something has gone wrong!"),
                    None => (),
                };
            };
        });

        //Send touch packet
        let wedge: Arc<Mutex<SerialPort>> = wedge_serial_2.clone();

        thread::spawn(move || {
            loop {
                for i in 0..127 {
                    if touch_active_1.load(Ordering::Relaxed) {
                        let now = Instant::now();
                        let mut wedge = loop {
                            match wedge.lock() {
                            Ok(g) => {break g},
                            Err(e) => ()
                            }
                        };
                        let mut touchbuffer: Vec<u8> = vec![0; 36];
                        for x in 1..7 {
                            //bottleneck under here
                            let r: usize = 7-x;
                            let touch = match wacca::issue_command(&wedge, &CommandPacket { out: true, wedge_id: x as u8, command_id: 0xA1, data: Vec::new() }) {
                                Ok(data) => data,
                                Err(_) => continue,
                            };
                            if touch.data.len() >= 4 {
                                touchbuffer[r as usize] = touchbuffer[r as usize] | fix_touch(touch.data[0], params_1.side);
                                touchbuffer[(r+6) as usize] = touchbuffer[(r+6) as usize] | fix_touch(touch.data[1], params_1.side);
                                touchbuffer[(r+12) as usize] = touchbuffer[(r+12) as usize] | fix_touch(touch.data[2], params_1.side);
                                touchbuffer[(r+18) as usize] = touchbuffer[(r+18) as usize] | fix_touch(touch.data[3], params_1.side);
                            }
                        }
                        touchbuffer[0] = 129;
                        touchbuffer[34] = i;
                        touchbuffer[35] = 128;
                        touchbuffer[35] = calc_checksum(&touchbuffer);
                        if touchbuffer[34] == 127 {touchbuffer[34] = 0};
                        let _ = writer_tx.send(touchbuffer);
                        io::Write::flush(&mut io::stdout()).unwrap();                
                        thread::sleep(Duration::from_millis(4));
                    } else {
                        thread::sleep(Duration::from_millis(4));
                    }
                }
            }
        });

        //wedge stuff
        //let (wedge_tx, wedge_rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel(); //channel pair for the wedge commands (unit board -> sync board)
        //let (touch_tx, touch_rx): (Sender<wacca::CommandPacket>, Receiver<wacca::CommandPacket>) = mpsc::channel(); //channel pair for the touch commands (unit boards -> sync board)

/*         //thread for submitting commands to the unit boards
        let wedge_serial_1 = binding.wedge_serial.clone();
        thread::spawn(move || {
            loop {
                match wedge_cmd_rx.recv() {
                    Ok(data) => {
                        let _ = SerialPort::write(&wedge_serial_1,&data.serialize());
                    },
                    Err(e) => {println!("{:?}",e)}
                };
            }
        });

        //thread for receiving data from the unit boards
        let wedge_serial_2 = binding.wedge_serial.clone();
        thread::spawn(move || {
            let mut serialbuffer: Vec<u8> = vec![0; 1000];
            loop {
                let data = loop {
                    match SerialPort::read(&wedge_serial_2, serialbuffer.as_mut_slice()) {
                        Ok(t) => {break serialbuffer[..t].to_vec()},
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(e) => eprintln!("{:?}", e),
                    }
                };
                let packet = match CommandPacket::new(data) {
                    Ok(d) => Some(d),
                    Err(e) => {println!("{:?}", e); None}
                };
                match packet {
                    Some(d) => {
                        match d.command_id {
                            0xA0 | 0xA2 | 0x94 | 0xC9 => {
                                println!("hfddfi");
                                let _ = &wedge_cmd_recv_tx.send(d);
                            }
                            _ => {}
                        }
                    },
                    None => {thread::sleep(Duration::from_millis(10))}
                }
                // match wacca::handle_data(&data, &params, touch_active_2.clone()) {
                //     Some(d) => command_tx.send(d).expect("Something has gone wrong!"),
                //     None => (),
                // };
            };
        }); */

        thread::spawn(move || {
            println!("hi");
            thread::sleep(Duration::from_millis(100));
        });
    }
    
    loop {
        thread::sleep(Duration::from_secs(1));
    }

}

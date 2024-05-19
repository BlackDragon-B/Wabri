use std::{env, io::{self, BufReader, Read}, str, sync::{ mpsc::{self, Receiver, Sender}, Arc}, thread, time::{Duration, Instant}};
use serialport;
use crate::{game::{self, fix_touch}, utils::{self, calc_checksum}, unit};

pub fn start(side: bool, wedge_port: &str, game_port: &str) {
    let args: Vec<String> = env::args().collect();
    let mut left = unit::WedgePort {hardware_port: serialport::new(wedge_port, 921600).timeout(Duration::from_micros(1000)).open().expect("Unable to open serialport")};
    let mut game = serialport::new(game_port, 115200).timeout(Duration::from_millis(1)).open().expect("Unable to open serialport");
    // for i in 1..100 {
    //     let mut x: Vec<(u8, String)> = left.get_version();
    //     //println!("{:?}",x);
    // }
    left.set_thresholds(0x11, 0x0C);
    left.init();
    thread::sleep(Duration::from_millis(200));

    let (gametx, gamerx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();

    let mut c = game.try_clone().unwrap();
    thread::spawn(move || {
        let mut serialbuffer: Vec<u8> = vec![0; 1000];
        loop {
            match c.read(serialbuffer.as_mut_slice()) {
                Ok(t) => { println!("{:?}", serialbuffer[..t].to_vec()); let _ = &gametx.send(serialbuffer[..t].to_vec()); },
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => eprintln!("{:?}", e),
            }
        }
    });

    let mut active: bool = false;
    let mut touchbuffer: Vec<u8> = vec![0; 36];
    let params = game::SyncBoardParams::get();
    loop {
        let x = Instant::now();

        //game logic
        match gamerx.try_recv() {
            Ok(m) => {
                println!(" a{:?}",m);
                let res: Option<Vec<u8>> = match m[0] {
                    0xa0 => {
                        active = false;
                        Some([vec![m[0]], "190523".as_bytes().to_vec(), vec![44]].concat().to_vec())
                    },
                    0x77 => {
                        None
                    },
                    0x20 => {
                        active = false;
                        None
                    },
                    0xa2 => {
                        active = false;
                        //Some(vec![ 162, 63, 29, 0, 0, 0, 0 ])
                        Some(vec![ 162, 63, 29 ])
                    },
                    0x94 => {
                        active = false;
                        //Some(vec![ 148, 0, 20, 0, 0, 0, 0 ])
                        Some(vec![ 148, 0, 20 ])
                    },
                    0xc9 => {
                        active = true;
                        //Some(vec![ 201, 0, 73, 0, 0, 0, 0 ])
                        Some(vec![ 201, 0, 73 ])
                    },
                    0xa8 => {
                        let mut versions = vec!["190523", "190523", "190523", "190523", "190523", "190523"];
                        Some(game::UnitBoardVersionPacket {
                            sync_board_version: "190523",
                            unit_board_version: versions,
                            side: side,
                        }.serialize())
                    },
                    0x72 => {
                        active = false;
                        let param: &str = match m[3] {
                            0x30 => {params.param0000}
                            0x31 => {params.param0016}
                            0x33 => {params.param0032}
                            _ => {""}
                        };
                        Some([param.as_bytes(), &vec![utils::calc_checksum(&param.as_bytes().to_vec())]].concat())
                    },
                    0x9a => {
                        active = false;
                        None
                    },
                    _ => {
                        None
                    },
                };
                match res {
                    Some(x) => { let _ = game.write(&x); },
                    None => ()
                }
            }
            Err(_) => ()

        }
        if active {
            let e = left.get_touch();
            utils::copy_into(&mut touchbuffer, 1, &[0; 24]);
            for l in e {
                let r = 7-l.0;
                touchbuffer[r as usize] = touchbuffer[r as usize] | fix_touch(l.1[0], side);
                touchbuffer[(r+6) as usize] = touchbuffer[(r+6) as usize] | fix_touch(l.1[1], side);
                touchbuffer[(r+12) as usize] = touchbuffer[(r+12) as usize] | fix_touch(l.1[2], side);
                touchbuffer[(r+18) as usize] = touchbuffer[(r+18) as usize] | fix_touch(l.1[3], side);
            }
            touchbuffer[0] = 129;
            touchbuffer[34] = touchbuffer[34]+1;
            touchbuffer[35] = 128;
            touchbuffer[35] = calc_checksum(&touchbuffer);
            if touchbuffer[34] == 127 {touchbuffer[34] = 0};
            println!("write");
            let _ = game.write(&touchbuffer);
        } else {
            thread::sleep(Duration::from_millis(1));
        }
        //reader.read(serial_buf.as_mut_slice());
        println!("{:?}",x.elapsed());
    }

}

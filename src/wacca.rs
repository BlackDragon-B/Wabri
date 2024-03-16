use std::{io::{self, Read, Write}, str, sync::{atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender}, Arc, Mutex}, thread, time::{Duration, Instant}};

use serial2::SerialPort;

pub struct SyncBoardParams<'a> {
    pub param0000: &'a str,
    pub param0016: &'a str,
    pub param0032: &'a str,
    pub sync_board_version: &'a str,
    pub side: bool,
}

impl SyncBoardParams<'static> {
    pub fn get(side: &bool) -> SyncBoardParams<'static> {
        SyncBoardParams {
            param0000: "    0    0    1    2    3    4    5   15   15   15   15   15   15   11   11   11",
            param0016: "   11   11   11  128  103  103  115  138  127  103  105  111  126  113   95  100",
            param0032: "  101  115   98   86   76   67   68   48  117    0   82  154    0    6   35    4",
            sync_board_version: "190523",
            side: *side,
        }
    }
}

pub struct TouchBinding {
    pub game_serial: Arc<SerialPort>,
    pub wedge_serial: Arc<Mutex<SerialPort>>,
    pub side: bool,
}

#[derive(Debug)]
pub struct CommandPacket {
    pub out: bool,
    pub wedge_id: u8,
    pub command_id: u8,
    pub data: Vec<u8>
}

pub struct UnitBoardVersionPacket {
    pub sync_board_version: String,
    pub unit_board_version: Vec<String>,
    pub side: bool,
}

impl UnitBoardVersionPacket {
    pub fn serialize(&self) -> Vec<u8> {
        let mut s: Vec<u8> = vec![0xA8];
        s.append(&mut self.sync_board_version.as_bytes().to_vec());
        if self.side {s.push(0x4C)} else {s.push(0x52)};
        for v in &self.unit_board_version {
            s.append(&mut v.as_bytes().to_vec());
        }
        s.push(calc_checksum(&[s.as_slice(),vec![0x80].as_slice()].concat()));
        //if self.side {s.push(104)} else {s.push(118)};
        s
    }
}

impl CommandPacket {
    pub fn serialize(&self) -> Vec<u8> {
        let dir: u8 = if self.out { 0xE0 } else { 0xD0 };
        let mut packet: Vec<u8> = vec![self.wedge_id+dir, self.command_id];
        packet.extend(&self.data);
        let tail: Vec<u8> = vec![calc_checksum(&packet),240];
        return [packet, tail].concat();
    }

    pub fn new(mut data: Vec<u8>) -> Result<CommandPacket, &'static str> { //TODO: Rewrite this parser to be more lenient with incoming data (trailing zero's etc)
        if data.len() < 4 {
            return Err("Invalid Size")
        }
        if data[data.len()-1] == 0 {
            data = data[..data.len()-1].to_vec();
        } 
        //let c: u8 = if data[1] == 0xA1 {
        //    calc_checksum(&[data[0..data.len()-2].to_vec(), vec![0x80]].concat())
        //} else {
        let c = calc_checksum(&data[0..data.len()-2].to_vec());

        let checksum: u8 = data[data.len()-2];
        if c != checksum {
            if calc_checksum(&[data[0..data.len()-2].to_vec(), vec![0x80]].concat()) != checksum {
                println!("check");
                println!("{:X?}",data);
                return Err("Invalid Checksum");    
            }
        };
        let a: (bool, u8) = match data[0] {
            209..215 => (false, data[0]-208),
            225..231 => (true, data[0]-224),
            _ => (false,0)
        };
        if a.1 == 0 {return Err("Wedge ID out of bounds")}

        let datalen: usize = if data.len() > 4 {
            data.len()-3
        } else {
            2
        };
        return Ok(CommandPacket {
            out: a.0,
            wedge_id: a.1,
            command_id: data[1],
            data: data[2..datalen].to_vec()
        })
    }
}

pub fn calc_checksum(data: &Vec<u8>) -> u8 {
    let mut checksum: u8 = 0;
    for byte in data.iter() {
        checksum ^= byte;
    }
    checksum
}

pub fn handle_data(buffer: &Vec<u8>, params: &SyncBoardParams, mut scan_active: Arc<AtomicBool>, mut wedge: std::sync::MutexGuard<'_,  SerialPort>) -> Option<Vec<u8>> {
    println!("aaaa {:?}",buffer[0]);
    match buffer[0] {
        0xa0 => {
            scan_active.store(false, Ordering::Relaxed);
            return Some([vec![buffer[0]], params.sync_board_version.as_bytes().to_vec(), vec![44]].concat().to_vec());
        },
        0x77 => {
            return None;
        },
        0x20 => {
            scan_active.store(false, Ordering::Relaxed);
            return None;
        },
        0xa2 => {
            scan_active.store(false, Ordering::Relaxed);
            for i in 1..7 {
                let packet = issue_command(&wedge,&CommandPacket { out: true, wedge_id: i, command_id: 0xA0, data: Vec::new()});
            }
            return Some(vec![ 162, 63, 29, 0, 0, 0, 0 ]);
        },
        0x94 => {
            scan_active.store(false, Ordering::Relaxed);
            for i in 1..7 {
                let packet = issue_command(&wedge,&CommandPacket { out: true, wedge_id: i, command_id: 0x94, data: Vec::new() });
            }
            return Some(vec![ 148, 0, 20, 0, 0, 0, 0 ]);
        },
        0xc9 => {
            scan_active.store(true, Ordering::Relaxed);
            for i in 1..7 {
                let packet = issue_command(&wedge,&CommandPacket { out: true, wedge_id: i, command_id: 0x90, data: vec![0x14, 0x07, 0x7F, 0x3F,]});
            }
            return Some(vec![ 201, 0, 73, 0, 0, 0, 0 ]);
        },
        0xa8 => {
            let mut versions: Vec<String> = Vec::new();
            for i in 1..7 {
                let packet = issue_command(&wedge,&CommandPacket { out: true, wedge_id: i, command_id: 0xA0, data: Vec::new() });
                //thread::sleep(Duration::from_secs(1));
                let data = packet.expect("veri bad");
                let version = str::from_utf8(&data.data).expect("Error").to_string();
                versions.push(version[..6].to_string());
                println!("{:?}", data)

            }
            //let mut versions = vec!["190523".to_string(), "190523".to_string(), "190523".to_string(), "190523".to_string(), "190523".to_string(), "190523".to_string()];
            return Some(UnitBoardVersionPacket {
                sync_board_version: params.sync_board_version.to_string(),
                unit_board_version: versions,
                side: params.side,
            }.serialize());
        },
        0x72 => {
            scan_active.store(false, Ordering::Relaxed);
            let param: &str = match buffer[3] {
                0x30 => {params.param0000}
                0x31 => {params.param0016}
                0x33 => {params.param0032}
                _ => {""}
            };
            return Some([param.as_bytes(), &vec![calc_checksum(&param.as_bytes().to_vec())]].concat())
        },
        0x9a => {
            scan_active.store(false, Ordering::Relaxed);
            return None;
        },
        _ => {
            return None;
        },
    }
}

pub(crate) fn issue_command(port: &SerialPort, data: &CommandPacket) -> Result<CommandPacket, &'static str> {
    let now = Instant::now();
    let _ = port.write(&data.serialize());
    //thread::sleep(Duration::from_secs(1));
    let mut serialbuffer: Vec<u8> = vec![0; 32];
    println!("afw {:?}",now.elapsed());
    let data = loop {
        //println!("hi");
        match port.read(serialbuffer.as_mut_slice()) { //slow poopy function
            Ok(t) => {break serialbuffer[..t].to_vec()},
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        };
    };
    println!("x {:?}",now.elapsed());
    CommandPacket::new(data)
}

pub fn fix_touch(byte: u8, side: bool) -> u8 {
    let side = side.clone();
    if side {
        byte.reverse_bits() >> 3
    } else {
        byte & 0x7f
    }
}
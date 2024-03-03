use std::{io::{self, Write}, marker::StructuralEq, process::Command, str, sync::{Arc, Mutex, MutexGuard}, time::{Duration, Instant}};

use serialport::SerialPort;

fn fix_touch(byte: u8, side: bool) -> u8 {
    if side {
        println!(byte);
        println!(byte.reverse_bits() >> 2);
        byte.reverse_bits() >> 2
    } else {
        byte & 0x7f
    }
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
#[derive(Debug)]
pub struct CommandPacket {
    pub out: bool,
    pub wedge_id: u8,
    pub command_id: u8,
    pub data: Vec<u8>
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
        let c: u8 = if data[1] == 0xA1 {
            calc_checksum(&[data[0..data.len()-2].to_vec(), vec![0x80]].concat())
        } else {
            calc_checksum(&data[0..data.len()-2].to_vec())
        };
        let checksum: u8 = data[data.len()-2];
        if c != checksum {
            println!("aaa");
            return Err("Invalid Checksum");
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

pub struct SyncBoardParams<'a> {
    pub param0000: &'a str,
    pub param0016: &'a str,
    pub param0032: &'a str,
    pub sync_board_version: &'a str,
}

impl SyncBoardParams<'static> {
    pub fn get() -> SyncBoardParams<'static> {
        SyncBoardParams {
            param0000: "    0    0    1    2    3    4    5   15   15   15   15   15   15   11   11   11",
            param0016: "   11   11   11  128  103  103  115  138  127  103  105  111  126  113   95  100",
            param0032: "  101  115   98   86   76   67   68   48  117    0   82  154    0    6   35    4",
            sync_board_version: "190523",
        }
    }
}
pub fn calc_checksum(data: &Vec<u8>) -> u8 {
    let mut checksum: u8 = 0;
    for byte in data.iter() {
        checksum ^= byte;
    }
    checksum
}

pub struct TouchLink<'a> {
    pub scan_active: bool,
    pub port: Arc<Mutex<Box<dyn SerialPort>>>,
    pub sync_board_version: &'a str,
    pub buffer: Vec<u8>,
    pub buffer2: Vec<u8>,
    pub syncboardparams: SyncBoardParams<'a>,
    pub side: bool,
    pub touchbuffer: Arc<Mutex<Vec<u8>>>,
    pub wport: Arc<Mutex<Box<dyn SerialPort>>>,
    pub time: Instant,
    pub pollrate: Duration,

}
impl TouchLink<'_> {
/*     pub fn poll(&mut self) {
        let binding = self.port.clone();
        let mut port: std::sync::MutexGuard<'_, Box<dyn SerialPort>> = binding.lock().expect("aaa");
        match port.read(self.buffer.as_mut_slice()) {
            Ok(t) => self.handle_data(&self.buffer[..t].to_vec(), port),
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
    } */
    pub fn handle_data(&mut self, buffer: &Vec<u8>, mut port: MutexGuard<'_, Box<dyn SerialPort>>, mut portw: MutexGuard<'_, Box<dyn SerialPort>>) {
        match buffer[0] {
            0xa0 => {
                self.scan_active = false;
                port.write(&[vec![buffer[0]], self.sync_board_version.as_bytes().to_vec(), vec![44]].concat()).unwrap();
            },
            0x77 => {},
            0x20 => {
                self.scan_active = false;
            },
            0xa2 => {
                self.scan_active = false;
                let _ = port.write(&[ 162, 63, 29, 0, 0, 0, 0 ]);
                for i in 1..7 {
                    let packet: Result<CommandPacket, &str> = issue_command( ,CommandPacket { out: true, wedge_id: i, command_id: 0xA0, data: Vec::new() });
                }
            },
            0x94 => {
                self.scan_active = false;
                for i in 1..7 {
                    let packet: Result<CommandPacket, &str> = issue_command( ,CommandPacket { out: true, wedge_id: i, command_id: 0x94, data: Vec::new() });
                }
                let _ = port.write(&[ 148, 0, 20, 0, 0, 0, 0 ]);
            },
            0xc9 => {
                for i in 1..7 {
                    let packet: Result<CommandPacket, &str> = issue_command( ,CommandPacket { out: true, wedge_id: i, command_id: 0x90, data: vec![0x14, 0x07, 0x7F, 0x3F,] });
                }
                self.scan_active = true;
                let _ = port.write(&[ 201, 0, 73, 0, 0, 0, 0 ]);
            },
            0xa8 => {
                let mut versions: Vec<String> = Vec::new();
                for i in 1..7 {
                    let packet = issue_command( ,CommandPacket { out: true, wedge_id: i, command_id: 0xA8, data: Vec::new() });
                    let data = packet.unwrap().data.to_owned();
                    let version = str::from_utf8(&data).expect("Error").to_string();
                    versions.push(version[..6].to_string());
                }
                let _ = port.write(&UnitBoardVersionPacket {
                    sync_board_version: self.sync_board_version.to_string(),
                    unit_board_version: versions,
                    side: self.side,
                }.serialize());
            },
            0x72 => {
                self.scan_active = false;
                let param: &str = match buffer[3] {
                    0x30 => {self.syncboardparams.param0000}
                    0x31 => {self.syncboardparams.param0016}
                    0x33 => {self.syncboardparams.param0032}
                    _ => {""}
                };
                let _ = port.write(&[param.as_bytes(), &vec![calc_checksum(&param.as_bytes().to_vec())]].concat());
            },
            0x9a => {
                self.scan_active = false;
            },
            _ => {},
        }
    }
    pub fn issue_command(&mut self, command: CommandPacket) -> Result<CommandPacket, &str>{
        let binding = self.wport.clone();
        let mut wport = binding.lock().expect("yes");
        let _ = wport.write(&command.serialize());
        let data = loop {
            match wport.read(self.buffer2.as_mut_slice()) {
                Ok(t) => {break self.buffer2[..t].to_vec()},
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => eprintln!("{:?}", e),
            }
        };
        CommandPacket::new(data)
    }
}

pub fn handle_data(buffer: &Vec<u8>, mut port: std::sync::MutexGuard<'_, Box<dyn SerialPort>>, mut portw: std::sync::MutexGuard<'_, Box<dyn SerialPort>>, params: &SyncBoardParams, scan_active: MutexGuard<'_, bool>) {
    match buffer[0] {
        0xa0 => {
            self.scan_active = false;
            port.write(&[vec![buffer[0]], params.sync_board_version.as_bytes().to_vec(), vec![44]].concat()).unwrap();
        },
        0x77 => {},
        0x20 => {
            self.scan_active = false;
        },
        0xa2 => {
            self.scan_active = false;
            let _ = port.write(&[ 162, 63, 29, 0, 0, 0, 0 ]);
            for i in 1..7 {
                let packet: Result<CommandPacket, &str> = issue_command(portw ,CommandPacket { out: true, wedge_id: i, command_id: 0xA0, data: Vec::new() });
            }
        },
        0x94 => {
            self.scan_active = false;
            for i in 1..7 {
                let packet: Result<CommandPacket, &str> = issue_command(portw ,CommandPacket { out: true, wedge_id: i, command_id: 0x94, data: Vec::new() });
            }
            let _ = port.write(&[ 148, 0, 20, 0, 0, 0, 0 ]);
        },
        0xc9 => {
            for i in 1..7 {
                let packet: Result<CommandPacket, &str> = issue_command(portw ,CommandPacket { out: true, wedge_id: i, command_id: 0x90, data: vec![0x14, 0x07, 0x7F, 0x3F,] });
            }
            self.scan_active = true;
            let _ = port.write(&[ 201, 0, 73, 0, 0, 0, 0 ]);
        },
        0xa8 => {
            let mut versions: Vec<String> = Vec::new();
            for i in 1..7 {
                let packet = issue_command(portw ,CommandPacket { out: true, wedge_id: i, command_id: 0xA8, data: Vec::new() });
                let data = packet.unwrap().data.to_owned();
                let version = str::from_utf8(&data).expect("Error").to_string();
                versions.push(version[..6].to_string());
            }
            let _ = port.write(&UnitBoardVersionPacket {
                sync_board_version: params.sync_board_version.to_string(),
                unit_board_version: versions,
                side: self.side,
            }.serialize());
        },
        0x72 => {
            self.scan_active = false;
            let param: &str = match buffer[3] {
                0x30 => {params.param0000}
                0x31 => {params.param0016}
                0x33 => {params.param0032}
                _ => {""}
            };
            let _ = port.write(&[param.as_bytes(), &vec![calc_checksum(&param.as_bytes().to_vec())]].concat());
        },
        0x9a => {
            self.scan_active = false;
        },
        _ => {},
    }
}

pub fn touch_recv(port: Arc<Mutex<Box<dyn SerialPort>>>, touchbuffer: Arc<Mutex<Vec<u8>>>, side: bool) {
    let mut port = port.lock().expect("yes");
    let mut touchbuffer = touchbuffer.lock().expect("yes");
    let mut serialbuffer = vec![0; 1000];
    for i in 1..7 {
        let r: usize = 7-i;
        let _ = port.write(&CommandPacket { out: true, wedge_id: i as u8, command_id: 0xA1, data: Vec::new() }.serialize());
        let data = loop {
            match port.read(serialbuffer.as_mut_slice()) {
                Ok(t) => {break serialbuffer[..t].to_vec()},
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => eprintln!("{:?}", e),
            }
        };
        if let Ok(ret) = CommandPacket::new(data) {
            touchbuffer[r as usize] = fix_touch(ret.data[0], side);
            touchbuffer[(r+6) as usize] = fix_touch(ret.data[1], side);
            touchbuffer[(r+12) as usize] = fix_touch(ret.data[2], side);
            touchbuffer[(r+18) as usize] = fix_touch(ret.data[3], side);
        }
    }

}

pub fn issue_command(mut wport: std::sync::MutexGuard<'_, Box<dyn SerialPort>>, command: CommandPacket) -> Result<CommandPacket, &'static str>{
    let _ = wport.write(&command.serialize());
    let mut serialbuffer = vec![0; 1000];
    let data = loop {
        match wport.read(serialbuffer.as_mut_slice()) {
            Ok(t) => {break serialbuffer[..t].to_vec()},
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
    };
    CommandPacket::new(data)
}

pub fn touch_send(port: Arc<Mutex<Box<dyn SerialPort>>>, touchbuffer: Arc<Mutex<Vec<u8>>>) {
    let mut port = port.lock().unwrap();
    let mut touchbuffer = touchbuffer.lock().unwrap();
    touchbuffer[0] = 129;
    touchbuffer[34] = touchbuffer[34] + 1;
    touchbuffer[35] = 128;
    touchbuffer[35] = calc_checksum(&touchbuffer);
    if touchbuffer[34] == 127 {touchbuffer[34] = 0};
    let _ = port.write(&touchbuffer);
}
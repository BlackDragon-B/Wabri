use std::io::{self, Write};

use serialport::SerialPort;

pub struct UnitBoardVersionPacket<'a> {
    pub sync_board_version: &'a str,
    pub unit_board_version: Vec<&'a str>,
    pub side: bool,
}

impl UnitBoardVersionPacket<'_> {
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

    pub fn new(data: Vec<u8>) -> Result<CommandPacket, &'static str> {
        let checksum: u8 = data[data.len()-2];
        println!("{:X?}",&data[0..data.len()-2]);
        println!("{:?}", calc_checksum(&data[0..data.len()-3].to_vec()));
        if calc_checksum(&data[0..data.len()-2].to_vec()) != checksum as u8 {
            return Err("Invalid Checksum")
        };
        let a: (bool, u8) = match data[0] {
            209..215 => (false, data[0]-208),
            225..231 => (true, data[0]-224),
            _ => (false,0)
        };
        if a.1 == 0 {return Err("Wedge ID out of bounds")}

        return Ok(CommandPacket {
            out: a.0,
            wedge_id: a.1,
            command_id: data[1],
            data: data[2..data.len()-2].to_vec()
        })
    }
}

pub struct SyncBoardParams<'a> {
    pub param0000: &'a str,
    pub param0016: &'a str,
    pub param0032: &'a str,
}

impl SyncBoardParams<'static> {
    pub fn get() -> SyncBoardParams<'static> {
        SyncBoardParams {
            param0000: "    0    0    1    2    3    4    5   15   15   15   15   15   15   11   11   11",
            param0016: "   11   11   11  128  103  103  115  138  127  103  105  111  126  113   95  100",
            param0032: "  101  115   98   86   76   67   68   48  117    0   82  154    0    6   35    4",
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
    pub port: &'a  mut Box<dyn SerialPort>,
    pub sync_board_version: &'a str,
    pub buffer: Vec<u8>,
    pub syncboardparams: SyncBoardParams<'a>,
    pub side: bool,
    pub touchbuffer: Vec<u8>
}
impl TouchLink<'_> {
    pub fn poll(&mut self) {
        match self.port.read(self.buffer.as_mut_slice()) {
            Ok(t) => self.handle_data(&self.buffer[..t].to_vec()),
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
        if self.scan_active {
            self.touch()
        }
    }
    pub fn handle_data(&mut self, buffer: &Vec<u8>) {
        println!("{:X?}", buffer[0]);
        match buffer[0] {
            0xa0 => {
                self.scan_active = false;
                self.port.write(&[vec![buffer[0]], self.sync_board_version.as_bytes().to_vec(), vec![44]].concat()).unwrap();
            },
            0x77 => {},
            0x20 => {
                self.scan_active = false;
            },
            0xa2 => {
                self.scan_active = false;
                let _ = self.port.write(&[ 162, 63, 29, 0, 0, 0, 0 ]);
            },
            0x94 => {
                self.scan_active = false;
                let _ = self.port.write(&[ 148, 0, 20, 0, 0, 0, 0 ]);
            },
            0xc9 => {
                self.scan_active = true;
                let _ = self.port.write(&[ 201, 0, 73, 0, 0, 0, 0 ]);
            },
            0xa8 => {
                let _ = self.port.write(&UnitBoardVersionPacket {
                    sync_board_version: self.sync_board_version,
                    unit_board_version: vec!["190514", "190514", "190514", "190514", "190514", "190514"],
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
                let _ = self.port.write(&[param.as_bytes(), &vec![calc_checksum(&param.as_bytes().to_vec())]].concat());
            },
            0x9a => {
                self.scan_active = false;
            },
            _ => {},
        }
    }
    pub fn touch(&mut self) {
        self.touchbuffer[0] = 129;
        self.touchbuffer[34] = self.touchbuffer[34] + 1;
        self.touchbuffer[35] = 128;
        self.touchbuffer[35] = calc_checksum(&self.touchbuffer);
        if self.touchbuffer[34] == 127 {self.touchbuffer[34] = 0};
        let _ = self.port.write(&self.touchbuffer);
    }
}

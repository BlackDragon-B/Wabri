use serialport::SerialPort;
use std::{io::{self, Read, Write}, ops::Range, str, thread, time::{Duration, Instant}};
use crate::utils::calc_checksum;

#[derive(Debug)]
struct CommandPacket {
    out: bool,
    wedge_id: u8,
    command_id: u8,
    data: Vec<u8>
}

impl CommandPacket {
    fn serialize(&self) -> Vec<u8> {
        let dir: u8 = if self.out { 0xE0 } else { 0xD0 };
        let mut packet: Vec<u8> = vec![self.wedge_id+dir, self.command_id];
        packet.extend(&self.data);
        let tail: Vec<u8> = vec![calc_checksum(&packet),240];
        return [packet, tail].concat();
    }

    fn bulk(&self, r: Range<u8>) -> Vec<CommandPacket> {
        let mut commands: Vec<CommandPacket> = Vec::new();
        for i in r {
            commands.push(CommandPacket { out: self.out, wedge_id: i, command_id: self.command_id, data: self.data.clone()})
        }
        commands
    }

    fn dissect(data: Vec<u8>) -> Vec<CommandPacket> {
        let mut commands: Vec<CommandPacket> = Vec::new();

        let pieces: Vec<_> = data
            .split(|&e| e == 0xf0)
            .filter(|v| v.len() > 1 )
            .collect();

        for command in pieces.iter() {
            match CommandPacket::new(command.to_vec()) {
                Ok(c) => commands.push(c),
                Err(e) => println!("ERR: {:?}", e)
            };
        }
        commands
    }
    fn new(mut data: Vec<u8>) -> Result<CommandPacket, String> {
        if data.len() < 4 {
            println!("LEN {:X?}",data);
            return Err("Invalid Size".to_string())
        }
        match data[data.len()-1] {
            0 => data = data[..data.len()-1].to_vec(),
            0xF0 => (),
            _ => data = [data, [0xF0].to_vec()].concat()
        }
        match data[0] {
            209..215 => (),
            225..231 => (),
            _ => data = data[1..data.len()].to_vec(),
        }

        let c = calc_checksum(&data[0..data.len()-2].to_vec());
        let checksum: u8 = data[data.len()-2];
        if c != checksum {
            if calc_checksum(&[data[0..data.len()-2].to_vec(), vec![0x80]].concat()) != checksum {
                println!("CHECK {:X?}",data);
                return Err("Invalid Checksum".to_string());    
            }
        };
        let a: (bool, u8) = match data[0] {
            209..215 => (false, data[0]-208),
            225..231 => (true, data[0]-224),
            _ => (false,0)
        };
        if a.1 == 0 {return Err("Wedge ID out of bounds".to_string())}

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

pub struct WedgePort {
    pub hardware_port: Box<dyn SerialPort>,
}
impl WedgePort {
    fn issue_command(&mut self, data: &CommandPacket) -> Result<CommandPacket, String> {
        self.hardware_port.write(&data.serialize()).expect("Write failed!");
        let _ = self.hardware_port.flush();
        let mut serialbuffer: Vec<u8> = vec![0; 32];
        let data = loop {
            match self.hardware_port.read(serialbuffer.as_mut_slice()) {
                Ok(t) => {break serialbuffer[..t].to_vec()},
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {},
                Err(e) => eprintln!("{:?}", e),
            };
        };
        //let x = self.hardware_port.read(serialbuffer.as_mut_slice()).expect("Found no data!");
        //println!("{:?}",x);
        CommandPacket::new(data)
    }
    
    fn issue_commandblk(&mut self, data: Vec<CommandPacket>) -> Vec<CommandPacket> {
        for i in data {
            let _ = self.hardware_port.write(&i.serialize());
            let _ = self.hardware_port.flush();
            thread::sleep(Duration::from_micros(1000));
        }
        //println!("poopy {:?}",start.elapsed());
        let mut serialbuffer: Vec<u8> = vec![0; 256];
        let data = loop {
            match self.hardware_port.read(serialbuffer.as_mut_slice()) {
                Ok(t) => {break serialbuffer[..t].to_vec()},
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {},
                Err(e) => eprintln!("{:?}", e),
            };
        };
        //let x = self.hardware_port.read(serialbuffer.as_mut_slice()).expect("Found no data!");
        //println!("{:?}",x);
        //let pieces: Vec<_> = it.split(|v| v == 0xf0 && it.peek() == 0x00).filter(|v| !v.is_empty()).collect();
        CommandPacket::dissect(data)
    }

    pub fn get_version(&mut self) -> Vec<(u8, String)> {
        let mut versions: Vec<(u8, String)> = Vec::new();
        // for i in 1..7 {
        //     match self.issue_command(&CommandPacket { out: true, wedge_id: i, command_id: 0xA8, data: Vec::new() }) {
        //         Ok(t) => {versions.push((i, str::from_utf8(&t.data).unwrap()[..6].to_string()))}
        //         Err(e) => println!("ERROR: {:?}", e),
        //     };
        // }
        let p = self.issue_commandblk(CommandPacket { out: true, wedge_id: 1, command_id: 0xA8, data: Vec::new() }.bulk(1..7));
        for x in p {
            versions.push((x.wedge_id, str::from_utf8(&x.data).unwrap()[..6].to_string()))
        };
        versions
    }

    pub fn set_thresholds(&mut self, on: u8, off: u8) {
        let p = self.issue_commandblk(CommandPacket { out: true, wedge_id: 1, command_id: 0x94, data: vec![on, off] }.bulk(1..7));
    }

    pub fn init(&mut self) {
        let p: Vec<CommandPacket> = self.issue_commandblk(CommandPacket { out: true, wedge_id: 1, command_id: 0x90, data: vec![0x14, 0x07, 0x7F, 0x3F, 0x44] }.bulk(1..7));
    }

    pub fn get_touch(&mut self) -> Vec<(u8, Vec<u8>)> {
        let mut s: Vec<(u8, Vec<u8>)> = Vec::new();
        let p = self.issue_commandblk(CommandPacket { out: true, wedge_id: 1, command_id: 0xA1, data: Vec::new() }.bulk(1..7));
        for i in p {
            s.push((i.wedge_id, i.data[..4].to_vec()))
        }
        s
    }


}
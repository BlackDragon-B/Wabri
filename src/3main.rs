#![feature(exclusive_range_pattern)]

use crate::wacca::CommandPacket;

mod wacca;

fn main() {
    let packet = CommandPacket {
        out: true,
        wedge_id: 6,
        command_id: 0x90,
        data: vec![0x14, 0x07, 0x7F, 0x3F, 0x44],
    };
    let b = packet.serialize();
    println!("{:X?}",b);
    let test: CommandPacket = wacca::CommandPacket::new(wacca::CommandPacket::new(vec![0xD1, 0xA8, 0x31, 0x39, 0x30, 0x35, 0x31, 0x34, 0x30, 0x37, 0x33, 0x34, 0x30, 0x41, 0xF0]).unwrap().serialize()).unwrap();
    println!("{:?}",test);
}
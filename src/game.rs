use crate::utils::calc_checksum;

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

pub fn fix_touch(byte: u8, side: bool) -> u8 {
    let side = side.clone();
    if side {
        byte.reverse_bits() >> 3
    } else {
        byte & 0x7f
    }
}
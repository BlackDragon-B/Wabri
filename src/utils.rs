pub fn calc_checksum(data: &Vec<u8>) -> u8 {
    let mut checksum: u8 = 0;
    for byte in data.iter() {
        checksum ^= byte;
    }
    checksum
}

pub fn copy_into(v: &mut Vec<u8>, position: usize, data: &[u8]) {
    let buf = &mut v[position..];
    let len = data.len().min(buf.len());
    buf[..len].copy_from_slice(&data[..len]);
}
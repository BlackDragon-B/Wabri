
## Command Packet Structure

| Byte  | 1        | 2          | 3-???         | ???+1    | ???+2    |
| ----- | -------- | ---------- | ------------- | -------- | -------- |
| Value | Address  | Command ID | Variable Data | Checksum | End Byte |
#### Address
For the address the following values are used:
- Unit Boards -> Sync Board: 0xD0 + Unit Board ID
- Sync Board -> Unit Boards: 0xE0 + Unit Board ID
#### Command ID
The following Command ID's are known:
- 0xA8: Get the Unit Board Version
- 0xA0:
- 0x94: 
- 0x90: "Initialize" Touch
- 0xA1: Poll board for touch data
#### Variable Data
This contains data, it has a variable length.
#### Checksum
The checksum is a simple XOR sum. The checksum usually contains the address, command id and variable data. In some instances (usually the touch data the unit boards return) a single 0x80 byte will need to be appended to the sum. A function for calculating the checksum would look something like this.

```
fn calc_checksum(data: &Vec<u8>) -> u8 {
	let mut checksum: u8 = 0;
	for byte in data.iter() {
		checksum ^= byte;
	}
	checksum
}
```
#### End Byte
The end byte indicates the end of an packet, this is 0xF0.
## Command Types
For the examples we will assume that unit board 1 will be used.
#### 0xA8 - Get Unit Board Version
This command requests the unit board version.
The sync board sends:

`0xE1 0xA8 0x49 0xF0`

The unit boards reponds with:

`0xD1 0xA8 0x31 0x39 0x30 0x35 0x31 0x34 0x30 0x37 0x33 0x34 0x30 0x41 0xF0`

The third till sixth byte contains the unit board version in ASCII encoding. Those 6 bytes are suffixed with another five bytes of ASCII. I however am unsure what this is for.
#### 0xA0 - Get Unit Board Version 2?
This command also requests the unit board version like 0xA8 and seems to respond in the same manner as 0xA8. I am unsure what this is for and am clueless about why it is part of the handshake.
#### 0x94 - Set Thresholds
This command sets the thresholds for toggling the state of the controller.
The sync board sends:

`0xE1 0x94 0x11 0x0C 0x68 0xF0`

The third and fourth byte correspond to the OnThreshold and OffThreshold values inside DefaultHardware.ini.
The unit boards respond with:

`0xD1 0x94 0x00 0x45 0xF0`
#### 0x90 - "Initialize" unit board
This command "initializes" the unit board and makes it ready to be polled with the 0xA1 command. 0xA1 will not function properly without first issuing 0x9A. Issuing this command will also result in the unit board led changing from a blinking state to a solid state.
The sync board sends:

`0xE1 0x90 0x14 0x07 0x7F 0x3F 0x44 0x66 0xF0`

The unit boards respond with:

`0xD1 0x90 0x41 0xF0`
#### 0xA1 - Poll touch state
This command retrieves the touch state from the unit boards.
The sync board sends:

`0xE1 0xA1 0x40 0xF0`

The unit boards respond with:

`0xD1 0xA1 0x80 0x00 0x00 0x00 0x04 0x00 0x00 0x00 0x00 0x00 0x00 0x01 0x75 0xF0`





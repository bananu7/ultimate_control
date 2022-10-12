// those packets were recorded as a part of a startup transaction
// between Universal Control and Revelator io24

// So far the structure of every one of them seems to be
//
// All numbers are Little-Endian
//
// |-----------|
// | U         | - 4 byte header
// | C         |
// | \00       |
// | \01       |
// |-----------|
// | size      | - 2 byte size field
// |           |
// |-----------|
// | X         | - 2 byte packet type identifier (see below)
// | Y         |
// |-----------|
// | packet    |
// | data ...  |

/* 

A lot of the packets seem to have two 16-bit fields next to each other.
Those are most often filled with values corresponding to lowercase letters
(perhaps UTF-16 by accident?), such as g,h,j,k etc.

I'm reffering to those occurences calling them "Address Pair",
for the lack of a better name/understanding.

Notably, a response to the PV message swaps those two values around
(and doesn't change anything else).

*/

/*

The 2-letter packet codes all seem to use uppercase letters.
Here are the ones observed:

* JM - most probably stands for "Json Message"
  followed by 32-bit JSON size and a regular JSON document

* KA - some system packet, like heartbeat or something
  followed by AddressPair

* UM - no idea what that is
  followed by 6 bytes of data

* PV - "parameter value"? Used for muting a channel.
  followed by AddressPair
  followed by a string identifier (e.g. "line/ch1/mute") but padded with 00s

* FR - used to query data such as presets
  followed by AddressPair
  followed by a 16-bit value
  followed by a string identifier (e.g. "Listpresets/channel")

*/


pub const HEADER: [u8; 539] = [
// UC01
0x55,0x43,0x00,0x01,
// size
0x08,0x00,
// UM
0x55,0x4d,
// 6 bytes data
0x00,0x00,0x65,0x00,0x5e,0xf1,

// UC01
0x55,0x43,0x00,0x01,
// size
0xc3,0x00,
// JM
0x4a,0x4d,
// j -> e
0x6a,0x00,0x65,0x00,
// json size
0xb9,0x00,0x00,0x00,
// JSON
0x7b,0x22,0x69,0x64,0x22,0x3a
,0x20,0x22,0x53,0x75,0x62,0x73,0x63,0x72,0x69,0x62,0x65,0x22,0x2c,0x22,0x63,0x6c
,0x69,0x65,0x6e,0x74,0x4e,0x61,0x6d,0x65,0x22,0x3a,0x20,0x22,0x22,0x2c,0x22,0x63
,0x6c,0x69,0x65,0x6e,0x74,0x49,0x6e,0x74,0x65,0x72,0x6e,0x61,0x6c,0x4e,0x61,0x6d
,0x65,0x22,0x3a,0x20,0x22,0x22,0x2c,0x22,0x63,0x6c,0x69,0x65,0x6e,0x74,0x54,0x79
,0x70,0x65,0x22,0x3a,0x20,0x22,0x50,0x43,0x22,0x2c,0x22,0x63,0x6c,0x69,0x65,0x6e
,0x74,0x44,0x65,0x73,0x63,0x72,0x69,0x70,0x74,0x69,0x6f,0x6e,0x22,0x3a,0x20,0x22
,0x44,0x45,0x53,0x4b,0x54,0x4f,0x50,0x22,0x2c,0x22,0x63,0x6c,0x69,0x65,0x6e,0x74
,0x49,0x64,0x65,0x6e,0x74,0x69,0x66,0x69,0x65,0x72,0x22,0x3a,0x20,0x22,0x44,0x45
,0x53,0x4b,0x54,0x4f,0x50,0x22,0x2c,0x22,0x63,0x6c,0x69,0x65,0x6e,0x74,0x4f,0x70
,0x74,0x69,0x6f,0x6e,0x73,0x22,0x3a,0x20,0x22,0x22,0x2c,0x22,0x63,0x6c,0x69,0x65
,0x6e,0x74,0x45,0x6e,0x63,0x6f,0x64,0x69,0x6e,0x67,0x22,0x3a,0x20,0x32,0x33,0x31
,0x31,0x37,0x7d,

// UC01
0x55,0x43,0x00,0x01,
// size
0x08,0x00,
// UM
0x55,0x4d,
0x00,0x00,0x67,0x00,0x5e,0xf1,

// UC01
0x55,0x43,0x00,0x01,
// size
0xc3,0x00,
// JM
0x4a,0x4d,
// f->g
0x66,0x00,0x67,0x00,
// json data size
0xb9,0x00,0x00,0x00,
// json data
0x7b,0x22,0x69,0x64,0x22,0x3a,0x20,0x22,0x53,0x75,0x62,0x73,0x63,0x72,0x69
,0x62,0x65,0x22,0x2c,0x22,0x63,0x6c,0x69,0x65,0x6e,0x74,0x4e,0x61,0x6d,0x65,0x22
,0x3a,0x20,0x22,0x22,0x2c,0x22,0x63,0x6c,0x69,0x65,0x6e,0x74,0x49,0x6e,0x74,0x65
,0x72,0x6e,0x61,0x6c,0x4e,0x61,0x6d,0x65,0x22,0x3a,0x20,0x22,0x22,0x2c,0x22,0x63
,0x6c,0x69,0x65,0x6e,0x74,0x54,0x79,0x70,0x65,0x22,0x3a,0x20,0x22,0x50,0x43,0x22
,0x2c,0x22,0x63,0x6c,0x69,0x65,0x6e,0x74,0x44,0x65,0x73,0x63,0x72,0x69,0x70,0x74
,0x69,0x6f,0x6e,0x22,0x3a,0x20,0x22,0x44,0x45,0x53,0x4b,0x54,0x4f,0x50,0x22,0x2c
,0x22,0x63,0x6c,0x69,0x65,0x6e,0x74,0x49,0x64,0x65,0x6e,0x74,0x69,0x66,0x69,0x65
,0x72,0x22,0x3a,0x20,0x22,0x44,0x45,0x53,0x4b,0x54,0x4f,0x50,0x22,0x2c,0x22,0x63
,0x6c,0x69,0x65,0x6e,0x74,0x4f,0x70,0x74,0x69,0x6f,0x6e,0x73,0x22,0x3a,0x20,0x22
,0x22,0x2c,0x22,0x63,0x6c,0x69,0x65,0x6e,0x74,0x45,0x6e,0x63,0x6f,0x64,0x69,0x6e
,0x67,0x22,0x3a,0x20,0x32,0x33,0x31,0x31,0x37,0x7d,

// UC01
0x55,0x43,0x00,0x01,
// size
0x06,0x00,
// KA
0x4b,0x41,
0x66,0x00,0x67,0x00,

// UC01
0x55,0x43,0x00,0x01,
0x4f,0x00,
// JM
0x4a,0x4d,
0x66,0x00,0x67,0x00,
// json size
0x45,0x00,0x00,0x00,
// json data
0x7b,0x22,0x69,0x64,0x22,0x3a,0x20,0x22,0x49,0x6e
,0x76,0x6f,0x6b,0x65,0x4d,0x65,0x74,0x68,0x6f,0x64,0x22,0x2c,0x22,0x75,0x72,0x6c
,0x22,0x3a,0x20,0x22,0x22,0x2c,0x22,0x6d,0x65,0x74,0x68,0x6f,0x64,0x22,0x3a,0x20
,0x22,0x67,0x65,0x74,0x4f,0x70,0x74,0x69,0x6f,0x6e,0x73,0x22,0x2c,0x22,0x63,0x61
,0x6c,0x6c,0x69,0x64,0x22,0x3a,0x20,0x31,0x30,0x30,0x7d,

// UC01
0x55,0x43,0x00,0x01,0x06
,0x00,0x4b,0x41,0x6a,0x00,0x65,0x00];

pub const INVOKE_WDM_SETUP: [u8; 90] = [0x55,0x43,0x00,0x01
,0x54,0x00,0x4a,0x4d,0x68,0x00,0x67,0x00,0x4a,0x00,0x00,0x00,0x7b,0x22,0x69,0x64
,0x22,0x3a,0x20,0x22,0x49,0x6e,0x76,0x6f,0x6b,0x65,0x4d,0x65,0x74,0x68,0x6f,0x64
,0x22,0x2c,0x22,0x75,0x72,0x6c,0x22,0x3a,0x20,0x22,0x22,0x2c,0x22,0x6d,0x65,0x74
,0x68,0x6f,0x64,0x22,0x3a,0x20,0x22,0x67,0x65,0x74,0x57,0x44,0x4d,0x53,0x65,0x74
,0x75,0x70,0x4c,0x69,0x73,0x74,0x22,0x2c,0x22,0x63,0x61,0x6c,0x6c,0x69,0x64,0x22
,0x3a,0x20,0x31,0x30,0x31,0x7d];


pub const BINARY_1: [u8; 24] = [
// UC\00\01 - 4byte header
0x55,0x43,0x00,0x01,
// size - 6 bytes
0x06,0x00,
// KA
0x4b,0x41,
// then 6800 -> 6700 (h->g)
0x68,0x00,0x67,0x00,

// UC\00\01 - 4byte header
0x55,0x43,0x00,0x01,
// size - 6 bytes
0x06,0x00,
// KA
0x4b,0x41,
// 6a00 -> 6500 (j->e)
0x6a,0x00,0x65,0x00
];


// then 4 bytes - 06, 00, 4b(K), 41(A)
// then 6b00 -> 6600 (k->f identifier)
pub const BINARY_2: [u8; 12] = [
// UC\00\01 - 4byte header
0x55,0x43,0x00,0x01,
// size - 6 bytes
0x06,0x00,
// 4b(K), 41(A)
0x4b,0x41,
// 6b00 -> 6600 (k->f)
0x6b,0x00,0x66,0x00];


pub const MUTE: [u8; 32] = [
// UC\00\01 - 4byte header
0x55,0x43,0x00,0x01,
// size - 26 bytes
0x1a,0x00,
// PV
0x50,0x56,
// 6b00 -> 6600 (k->f) // the response sends (f->k)
0x6b,0x00,0x66,0x00,
// string: line/ch1/mute
// with zeros at the end
0x6c,0x69,0x6e,0x65,0x2f,0x63,0x68,0x31,0x2f,0x6d,0x75,0x74,0x65,0x00,0x00,0x00,0x00,0x00,
// this isn't always here - sometimes it's 00 00
0x80,0x3f];


pub const SUBSCRIBE_2: [u8; 282] = [
// UC01
0x55,0x43,0x00,0x01,
// size - 8
0x08,0x00,
// UM + 6 bytes of whatever
0x55,0x4d,0x00,0x00, 0x66,0x00,0x84,0xe8,

// UC01
0x55,0x43,0x00,0x01,
// size - 227
0xe3,0x00,
// JM
0x4a,0x4d,
// k->f
0x6b,0x00,0x66,0x00,
// JSON data size - 217
0xd9,0x00,0x00,0x00,
// JSON starts here
0x7b,0x22,0x69,0x64,0x22,0x3a,0x20,0x22,
0x53,0x75,0x62,0x73,0x63,0x72,0x69,0x62,
0x65,0x22,0x2c,0x22,0x63,0x6c,0x69,0x65,
0x6e,0x74,0x4e,0x61,0x6d,0x65,0x22,0x3a,
0x20,0x22,0x55,0x6e,0x69,0x76,0x65,0x72,
0x73,0x61,0x6c,0x20,0x43,0x6f,0x6e,0x74,0x72,0x6f,0x6c,0x22,0x2c,0x22
,0x63,0x6c,0x69,0x65,0x6e,0x74,0x49,0x6e,0x74,0x65,0x72,0x6e,0x61,0x6c,0x4e,0x61
,0x6d,0x65,0x22,0x3a,0x20,0x22,0x75,0x63,0x61,0x70,0x70,0x22,0x2c,0x22,0x63,0x6c
,0x69,0x65,0x6e,0x74,0x54,0x79,0x70,0x65,0x22,0x3a,0x20,0x22,0x50,0x43,0x22,0x2c
,0x22,0x63,0x6c,0x69,0x65,0x6e,0x74,0x44,0x65,0x73,0x63,0x72,0x69,0x70,0x74,0x69
,0x6f,0x6e,0x22,0x3a,0x20,0x22,0x44,0x45,0x53,0x4b,0x54,0x4f,0x50,0x22,0x2c,0x22
,0x63,0x6c,0x69,0x65,0x6e,0x74,0x49,0x64,0x65,0x6e,0x74,0x69,0x66,0x69,0x65,0x72
,0x22,0x3a,0x20,0x22,0x44,0x45,0x53,0x4b,0x54,0x4f,0x50,0x22,0x2c,0x22,0x63,0x6c
,0x69,0x65,0x6e,0x74,0x4f,0x70,0x74,0x69,0x6f,0x6e,0x73,0x22,0x3a,0x20,0x22,0x70
,0x65,0x72,0x6d,0x20,0x75,0x73,0x65,0x72,0x73,0x22,0x2c,0x22,0x63,0x6c,0x69,0x65
,0x6e,0x74,0x45,0x6e,0x63,0x6f,0x64,0x69,0x6e,0x67,0x22,0x3a,0x20,0x32,0x33,0x31
,0x31,0x37,0x7d,

// UC01
0x55,0x43,0x00,0x01,
// size - 29
0x1d,0x00,
// FR
0x46,0x52,
// k->f
0x6b,0x00,0x66,0x00,
// 1
0x01,0x00,
// string: Listpresets/channel
0x4c,0x69,0x73,0x74,0x70,0x72,0x65,0x73,0x65,0x74,0x73,0x2f,0x63,0x68,0x61,0x6e,0x6e,0x65,0x6c,0x00,0x00];


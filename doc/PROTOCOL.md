# Universal Control communication protocol

## Software stack

## Header

The packets were recorded as a part of a startup transaction
between Universal Control and Revelator io24. See [`types.rs`](./header.rs);
file for raw byte capture examples.

So far the structure of every one of them seems to be:

```
 All numbers are Little-Endian

 |-----------|
 | U         | - 4 byte header
 | C         |
 | \00       |
 | \01       |
 |-----------|
 | size      | - 2 byte size field
 |           |
 |-----------|
 | X         | - 2 byte packet type identifier (see below)
 | Y         |
 |-----------|
 | packet    |
 | data      |
 | ...       |
```

## Packet types

Those packets are identified by two bytes in the packet header. So far
those bytes were always uppercase letters.

To see how they're parsed in code, look at [`types.rs`](../src/types.rs);

> **Note**
>
> A lot of the packets seem to have two 16-bit fields next to each other.
> Those are filled with values corresponding to lowercase letters
> (perhaps UTF-16 by accident?), such as g,h,j,k etc.
>
> I'm reffering to those occurences calling them "Address Pair",
> for the lack of a better name/understanding.
>
> Notably, a response to the PV message swaps those two values around
> (and doesn't change anything else).

### JM

Most probably means "JSON message". This packet stores an AddressPair,
a 32-bit size field and a JSON block in text format.

```
 All numbers are Little-Endian

 |-----------|
 | a         | - 2 byte AddressPair
 | b         |
 |-----------|
 | size      | - 32-bit size of the json payload (LE)
 |           |
 |           |
 |           |
 |-----------|
 | JSON      |

 | ...       |

 |           |
 |-----------|
```

The contents of those messages is rather obvious, and I'll document them in a separate document.

### PV

Most probably means "parameter value". As name implies, it's used to set
(and receive) values of different numeric parameters of the device, such
as mute state, send and fader positions etc.

The packet contains an AddressPair, a string identifier of the field, 3 bytes of padding
(seemingly always 00s) and a 32-bit float (LE).

> **Note**
>
> For boolean values, `0.0` (byte sequence `[0, 0, 0, 0]`) corresponds to `false`,
> and `1.0` (byte sequence `[0, 0, 0x80,0x3f]`) corresponds to `true`.

```
 |-----------|
 | a         | - 2 byte AddressPair
 | b         |
 |-----------|
 | name      | - string name of the parameter

 | ...       |

 |           |
 |-----------|
 | \00       | - 3 bytes of padding
 | \00       |
 | \00       |
 |-----------|
 | value     | - parameter value
 |           |
 |           |
 |           |
 |-----------|
```

### KA

"Keep-Alive". Contains an AddressPair, but the first field
is always 00. That could mean that the first field is the receiver field,
and it's 0 here because it's only meant to upkeep the subscription?
TBD.

### FR

Packet for asking for presets etc.
TBD.

### UM

No idea what UM stands for, but this packet is used together with the subscription packet
to indicate a random open UDP port that the device is supposed to send updates to.

The last 2 bytes of data are the port number.

### ZM

Most likely "zip message". Contains data compressed with `zlib`. In my testing
the magic header indicated `7801` (and `785E`), meaning "No Compression/low" - [source](https://stackoverflow.com/questions/9050260/what-does-a-zlib-header-look-like.)

Bytes 0..3 are unknown but likely decompressed size as u32 (+1 for `\0`?)
Bytes 4&5 are zlib magic number. From byte 6 onwards the payload starts.


### PS

Probably for selecting presets.
TBD.

### PL

Looks like it's used for preset lists for specific fat channel block presets.
(i.e. not full device presets, but e.g. EQ presets)

WRONG, it's for the device presets actually?
TBD.

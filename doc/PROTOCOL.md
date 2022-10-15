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

Probably heartbeat packet. Contains an AddressPair.
TBD.

### FR

Packet for asking for presets etc.
TBD.

### UM

No idea what that is.
TBD.

### ZM

TBD.

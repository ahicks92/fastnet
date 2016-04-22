Fastnet 1.0 Protocol Specification

##Introduction and Goals

Fastnet is a connection-based, channel-based protocol intended for games and realtime applications.
It intensionally forgoes much of the automatic management of TCP and assumes that settings will be provided by the application developer.

Unlike TCP, Fastnet works at the level of a message: a chunk of binary data of any size.
Messages are either fully received or never received at all, and the application need not provide further separation logic.
One Fastnet connection is logically 32768 smaller multiplexed connections called channels.

Messages can be sent reliably or unreliably.
Unreliable messages may or may not arrive at the other end, but are extremely fast and capable of representing frequently updated data such as position.
Reliable messages are intended for chat messages, status updates, or other information that absolutely must arrive.
A reliable message stalls all other messages in the queue behind it until such time as it arrives on the other end.

When considering the advantages of Fastnet, it is helpful and mostly accurate to think of  TCP as a train.
If one train car derails, everything behind it stops.
The primary advantage of Fastnet, then, is that you have the ability to use multiple train tracks at once.
Furthermore, for applications which can tolerate data loss, it is possible to mark some of the metaphorical cars as unimportant;
if these unimportant cars derail, the stream continues regardless.

Other semireliable UDP-based protocols exist, but usually only for one language.
Fastnet aims to be fully documented to the point where it can be recoded from scratch.
The other two features that Fastnet aims to support are the ability to fall back to other transports (TCP, HTTP, WebRTC) and the ability to support UDP hole-punching.
In my experience, it is difficult to find networking libraries that offer all three of these benefits.

##Definition of Terms##

The key words "must", "must not", "required", "shall", "shall not", "should", "should not", "recommended",  "may", and
"optional" in this document are to be interpreted as described in
[RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

A client refers to an entity who requests a connection from a server.
A server refers to an entity capable of accepting multiple connections.
All entities involved in Fastnet are either a client or a server, never both.

A sender is an entity capable of sending messages.
A receiver is an entity capable of receiving messages.
Clients and servers are both senders and receivers.

A packet refers to a chunk of data of any size.

A message refers to a chunk of data specified by the application.  This specification places no semantic meaning on the content of messages provided by applications.

A transport means a method of moving packets from one destination to another.  Transports usually involve the internet.

A transport's implementation refers to the concrete implementation of a transport.

UDP hole-punching refers to a technique for  circumventing network Address Translation and connecting two computers directly without the use of an intermediate server.

##Specifying packet Format##

This specification uses a mostly self-explanatory language to specify packet formats, and this section may likely be skipped.  For clarity:

- Nonterminals are written in the form `<example>`.  Terminals are written in the form <example: i8`.

- The recognized type suffixes are `u`, `i`, `s`, and `p`.  A terminal is followed by a `:` and a type suffix.

- `u` and `i` represent unsigned and signed integers stored in network byte order using twos complement.  Each is followed by a bit count, for example `u8` or `i16`.  The bit count must be a multiple of 8.  All math on integer types must be performed mod `2^n` where n is the number of bits in the integer (put another way, math wraps).  0 is considered positive.

- `b` represents a boolean encoded as a `u8`.  0 represents false.  1 represents true.  No other value is allowed.

- `s` represents null-terminated UTF8 strings.  It may be followed with a length restriction of the form `{min, max}`.  The default length restriction is `{1, INFINITY}`.  Said restrictions are in UTF8 code points.

- `a` is like `s`, but the string must be ascii.

- `p` stands for payload, an arbitrary sequence of bytes.  `p` segments will be described further by the specification.  They are usually the last field of a packet.

- Space means concatenate without padding.  For example, `part1:i8 part2:i8` is two 1-byte signed integers without padding.

- Integer literals are noted as `1:u8`, `-5:i16`, etc.  String literals are `"this is a test."`.  In the case of string literals, the type is not ambiguous and so need not be specified.

- `[]` means optional.  The specified section may or may not be included, as specified in documentation of the packet's use.

- `|` means or.  `|` has a precedence lower than concatenation.

- `()` is for grouping, as usual.

The rest of this specification demonstrates this language, which is mostly self-explanatory in practice.

It should be noted that the strings can naively be used for DDOS attacks: send the encoder a sufficiently large packet and it will grind to a hault.
This is not an issue in practice because of two things.
First, the largest packet Fastnet sends is 1000 bytes.  Larger packets are not permitted, so a conforming implementation can assume that any larger packet is invalid.
Second, extremely large UDP packets are unlikely to arrive at all even assuming they can be sent in the first place.

##Transports##

A transport is a bidirectional method for moving packets from a sender to a receiver.
Fastnet requires that the maximum packet size of the transport be greater than or equal to 1000 bytes and guarantees that it will never send a packet over this size.
In practice, Fastnet almost always sends much smaller packets.

##Basic Packet Format##

Basic packet format:

```
fastnet_packet = checksum: u64 channel:i16 payload:p
```

A fastnet packet consists of:

- A 4-byte CRC32 Castagnoli (CRC32C) checksum.  The polynomial for this checksum is 0x1EDC6F41.

- A 2-byte signed channel identifier.  Negative values are for Fastnet's use.

- The payload of the packet.

Channels 0 to 32767 must be reserved for the application and are referred to as message channels.
All other channels must be reserved for Fastnet's protocol and used as specified here.

An implementation must prevent the user from using negative channel numbers for any purpose.

An implementation must verify the checksum and ignore all packets for which the checksum is not valid.

The checksum used here is only 32 bits.  This specification may strengthen it in future.  When coupled with the optional UDP checksum, the chance of receiving a corrupted packet is quite small.  Roughly 1 in 4 billion corrupted packets can get through.
On a link where enough packets are corrupted to cause a problem, the user will probably not be connected reliably anyway.
Even then, the application will on average have to send multiple gigabytes of data before seeing corruption.

This checksum does make packets larger.  Unfortunately, the UDP checksum is optional.  Without it, it is possible for every corrupted packet to get through.

The rest of this specification assumes that the checksum is present and refrains from mentioning it in packet format specifications for brevity.

##Status Queries##

Format:
```
query = -1:i16 0:u8 (<listening_query> | <version_query> | <extension_query>)
response = -1:i16 1:u8 (<fastnet_response> | <version_response> | <extension_response>)

fastnet_quiery  = 0: u8
fastnet_response = 0: u8 response: b

version_query = 1:u8
version_response = 1: u8 version: a

extension_query = 2: u8 name: a
extension_response = 2: u8 name: a supported: b
```

Channel -1 is the status query and connection handshake channel.
Queries are specifically designed such that they need not arrive in order.
Queries must be responded to in both directions.
All of the following query operations must be implemented.
Unless otherwise noted, the result of a query should never change.

- 0 is a query to determine if Fastnet is listening on a specified port.  Return `false` for no, `true` for yes.  An implementation shall assume that there is no server listening if this query is consistently not responded to.  An implementation shall send this query no more than ten times.  An implementation functioning only as a client should respond with `false` but is permitted not to respond at all.

- 1 is the version query.  The version response must contain a version string in the form `major.minor`.  The current version is "1.0".  Incrementing the minor component indicates a backwards compatible change.  Incrementing the major component indicates a backwards-incompatible change.

- 2 is the extension supported query.  Extension names should be of the form `vendorname_extensionname` and stored in lower case.  This specification reserves the prefix `fastnet_` for use by this specification.

An implementation shall not place limits on the number of times that a query may be sent and must always respond.
An implementation must continue to respond to queries even after a connection is established.

##Connection Establishment##

packets:

```
connect = -1:i16 2:u8
connected = -1:i16 3:u8 connection_identifier:u64
aborted = -1:i16 4:u8 error:s
```

With the exception of UDP hole-punching, connections are established using the following algorithm.  UDP hole-punching is described elsewhere in this specification.

A Fastnet server must allow only one connection from a specific IP and port.

The following must always take place on channel -1 before a connection is considered established.

To begin a connection, a client must:

- Use the `fastnet_query` from the status query section to determine if a fastnet implementation is listening.  An implementation must make no more than 10 attempts before aborting.

- Use the `version_query` to determine that the implementations are compatible.  Again, an implementation must make no more than 10 attempts before aborting.

- Send the connect packet.

- begin waiting for either the connected packet or the aborted packet with a timeout of 5000 MS.  The client must resend the connect packet every 200 MS during this process.

If the client receives the connected packet, it must parse the connection id, notify the application that the connection has been established, and begin processing packets.
The client must disregard all other packets including queries until it manages to receive the connected packet.

If the client receives the aborted packet, it must report the error string to the user of the implementation in an implementation-defined manner.

if the client times out before receiving either the connected or aborted packet, the implementation must report an implementation-defined error.

When the server sees the connect packet, it begins establishing a connection.
To establish a connection, a server must generate an unsigned 64-bit integer ID.
This ID must be unique among all currently-established connections to said server.
It must then encode and send the connected packet and immediately notify the application that a connection was established.
If the server continues to receive the connection request packet, it must continue to respond with the connected packet but do nothing further; it is possible for the client to not yet know that it is connected due to packet loss.

This specification reserves ID 0 for "no ID".  A server must never use an ID of 0 for a connection.  This enables implementations to use ID 0 for internal purposes, though no other meaning is assigned to it by this specification.

To refuse a connection, a server must send the aborted packet with an implementation-defined string as to the reason.
This string must not be empty.
This specification suggests "unspecified error" as a reasonable default for situations in which a string is not available, i.e. because a game does not wish to advertise that a player has been banned.

Servers must ignore any packets not involved in an active connection.

##The Heartbeat Channel and Connection Breaking##

Packet format:

```
heartbeat = -2:i16 counter: u64 sent_packets: u64 received_packets: u64
```

Channel -2 must be the heartbeat channel.

A heartbeat is composed of three pieces of information:

- A 64-bit counter, interpreted as a sequence number.

- A 64-bit integer specifying how many packets this end of the connection has sent.

- A 64-bit integer specifying how many packets this end of the connection has received from the other end of the connection.

Both parties involved in a fastnet connection must send a heartbeat to each other  once a second.

If either end of a fastnet connection does not receive any packets from the other end of the connection for a timeout period  then it must consider the connection broken.  This period must be configurable by the user and should default to 10 seconds.  This period must not go below 2 seconds.

Heartbeat packet counts do not include any packets exchanged before the establishment of a connection.

An implementation must not place any semantinc meaning on anything in the heartbeat beyond using it for rough packet loss estimation.

##The Echo Channel##

Packet format:

```
echo = -3: i16 payload: i16
```

When an implementation receives a packet on the echo channel for a connected Fastnet peer, it must immediately resend (echo) the packet back to the sender without modification.

Clients shall always use negative integers for their payload.  Servers  shall always use positive integers for their payload.

This channnel exists for implementations wishing to attempt provision of round-trip estimation.  A conforming implementation must implement the echo channel but is not required to provide a round-trip estimation algorithm.

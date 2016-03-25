#Fastnet 1.0 Protocol Specification

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
First, the largest packet Fastnet sends is 500 bytes.  Larger packets are not permitted, so a conforming implementation can assume that any larger packet is invalid.
Second, extremely large UDP packets are unlikely to arrive at all even assuming they can be sent in the first place.

##Transports##

A transport is a bidirectional method for moving packets from a sender to a receiver.
Fastnet requires that the maximum packet size of the transport be greater than or equal to 500 bytes and guarantees that it will never send a packet over this size.

##Basic Packet Format##

Basic packet format:

```
fastnet_packet = channel:i16 payload:p
```

A Fastnet packet must consist of a 2-byte channel identifier as a signed 16-bit integer followed by a packet payload of no more than the maximum packet size of the transport minus 2 bytes.
Channels 0 to 32767 must be reserved for the application and are referred to as message channels.
All other channels must be reserved for Fastnet's protocol and used as specified here.

An implementation must prevent the user from using negative channel numbers for any purpose.

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
connected = -1:i16 3:u8 connection_identifier:u32
aborted = -1:i16 4:u8 error:s
```

With the exception of UDP hole-punching, connections are established using the following algorithm.  UDP hole-punching is described elsewhere in this specification.

A Fastnet server must allow only one connection from a specific IP and port.

The following must always take place on channel -1 before a connection is considered established.

To begin a connection, a client must:

- Use the `fastnet_query` from the status query section to determine if a fastnet implementation is listening.

- Use the `version_query` to determine that the implementations are compatible.

- Send the connect packet.

- begin waiting for either the connected packet or the aborted packet with a timeout of 5000 MS.  The client must resend the connect packet every 200 MS during this process.

If the client receives the connected packet, it must parse the connection id, notify the application that the connection has been established, and begin processing packets.
The client must disregard all other packets including queries until it manages to receive the connected packet.

If the client receives the aborted packet, it must report the error string to the user of the implementation in an implementation-defined manner.

if the client times out before receiving either the connected or aborted packet, the implementation must report an implementation-defined error.

When the server sees the connect packet, it begins establishing a connection.
To establish a connection, a server must generate an unsigned 32-bit integer ID.
This ID must be unique among all currently-established connections to said server.
It must then encode and send the connected packet and immediately notify the application that a connection was established.
If the server continues to receive the connection request packet, it must continue to respond with the connected packet but do nothing further; it is possible for the client to not yet know that it is connected due to packet loss.

To refuse a connection, a server must send the aborted packet with an implementation-defined string as to the reason.
This string must not be empty.
This specification suggests "unspecified error" as a reasonable default for situations in which a string is not available, i.e. because a game does not wish to advertise that a player has been banned.

Both clients and servers must expose the Fastnet connection ID for a connection in a manner that can be reached by the application developer; this ID is part of the UDP hole-punching algorithm as well as a crutial piece of information for many extensions.

Servers must ignore any packets not involved in an active connection.

##The Heartbeat Channel, Connection Breaking, and Round-trip Estimation##

Packet format:

```
heartbeat = -2:i16 payload:i16
```

Channel -2 must be the heartbeat channel.

If a client receives a positive integer on the heartbeat channel, it is to immediately echo it back to the server.
If a server receives a negative integer on the heartbeat channel, it is to immediately echo it back to the client.
In all other cases, the client and/or server must do nothing and ignore the packet.

Connections must be considered broken in one of two cases:

- If the transport reports that the connection is broken for any reason, i.e. TCP closed or the operating system lost the network.

- If one end of the connection does not receive a heartbeat within a user-specified timeout whose default value must be 20 seconds and whose minimum must be no less than 1 second.  For this purpose, implementations must consider both echoed heartbeats and sent heartbeats to be equivalent.

Both the client and the server must report broken connections to the application without delay.
Servers must also begin behaving as though the client had not connected in the first place; all packets save connection requests and queries must be ignored.

The implementation must send heartbeats to the other end of the connection with an interval no greater than 500 MS.
The implementation must always respond to heartbeats instantly.

Implementations are required to provide packet round-trip estimation without violating this specification or using extra channels for their protocol implementations.
The default round-trip time must be 200 MS.
The most basic conforming algorithm for packet estimation is to use this default, but it is suggested that implementations take advantage of the heartbeat channel to perform a smarter estimate.

##Determining Payload MTU##

Packets:

```
reset_mtu_count = (-3:i16 | -4:i16) 0:u8
mtu_count_was_reset = (-3:i16 | -4:i16) 1:u8
mtu_estimate = (-3:i16 | -4:i16) payload:p
mtu_response = (-3:i16 | -4:i16) count:u32
```

We refer to the MTU (maximum transmition unit) as the length of the largest packet that is received by the other end of the connection with enough reliability to be useful.
Maximizing the MTU is important to avoid excessive fragmentation.
Regardless of the determined MTU, either end of the connection must be prepared for packets of up to 500 bytes.
The default and minimum MTU is 32.

Channels -3 and -4 are the server MTU and client MTU estimation channels respectively.
We refer to them as the estimator and the responder, and to the channel of the estimator as the channel.
Both the client and the server must be capable of playing both roles simultaneously.
The server is the estimator on channel -3.  The client is the estimator on channel -4.

The responder is simplest.
When the responder sees the reset_mtu_count packet, it must reset an internal counter to 0 and send the mtu_count_was_reset packet.
When it sees the mtu_estimate packet, it must increment the internal counter and respond with the mtu_response packet, encoding the count.

This specification leaves the algorithm of the estimator unspecified for now, as implementation experience is required to properly design it.
The minimum conforming algorithm is to use the default MTU of 32.
This specification mandates the  following only:

- The default and minimum MTU is 32.

- The client and server must perform MTU estimation immediately upon the establishment of the connection.

It is strongly recommended that the MTU estimation algorithms be re-executed periodically.

Note that the payload of the mtu_estimate packet is arbitrary.  It should be set to random bytes and is used to estimate the largest packet that arrives somewhat reliably at the other end of connections.
A basic MTU estimation algorithm is to send `n` packets of some fixed size, wait a while, and see what the largest received count was.
Pseudocode will be added to this section when an algorithm proves itself.

The rest of this specification is pending and cannot be written without an implementation to play with.

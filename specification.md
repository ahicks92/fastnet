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

- Nonterminals are written in all lower case and followed by `:tt`, i.e. `example:i8`  `tt` is a type specifier.

- The recognized type suffixes are `u`, `i`, `s`, and `p`.  

- `u` and `i` represent unsigned and signed integers stored in network byte order using twos complement.  Each is followed by a bit count, for example `u8` or `i16`.  The bit count must be a multiple of 8.  All math on integer types must be performed mod `2^n` where n is the number of bits in the integer (put another way, math wraps).

- `s` represents length-prefixed strings of up to 255 bytes, encoded as UTF8.  The length prefix is 1 byte.  `s` may be followed by a length range in braces, i.e. `s{0, 2}`  The default range is `{1, 255}`.

- `p` stands for payload, an arbitrary sequence of bytes.  `p` segments will be described further by the specification.  They are usually the last field of a packet.

- Space means concatenate without padding.  For example, `part1:i8 part2:i8` is two 1-byte signed integers without padding.

- `=` means definition.  The order of definitions does not matter. Defined sections can be used in other definitions without type suffixes (because this is already defined in the definition being used).

- Integer literals are noted as `1:u8`, `-5:i16`, etc.  String literals are `"this is a test."`.

- `[]` means optional.  The specified section may or may not be included, as specified in documentation of the packet's use.

- `|` means or.  `|` has a precedence lower than concatenation.

- `()` is for grouping, as usual.

The rest of this specification demonstrates this language, which is mostly self-explanatory in practice.

##Transports##

A transport is a method for moving packets from one destination to another.
Transports must have the following characteristics.

- An ordered transport must guarantee that, if packets arrive, they arrive in order.

- An incorruptible transport must guarantee that, if packets arrive, they match the sent packet.

- A reliable transport must guarantee that packets arrive if sent.  Reliable transports must be ordered and incorruptible transports.

- A connection-based transport must handle all logic of forming and keeping a connection open.  This specification refers to transports which are not connection-based as connectionless.

All implementations must provide at least the TCP and UDP transports.
The TCP transport must be reliable and connection-based.
The UDp transport must be unordered, unreliable, and connectionless.
Over IPV4, UDP's built-in checksum is optional.
To this end, UDP must also be corruptible.

Every transport must advertise the maximum packet size it supports in a way that allows the user of the implementation to obtain the value.
Every transport must support a maximum packet size of at least 4096 bytes.
Note that Fastnet will perform packet MTU discovery.

##Basic Packet Format##

Basic packet format:

```
fastnet_packet = channel:i16 payload:p
```

A Fastnet packet must consist of a 2-byte channel identifier as a signed 16-bit integer followed by a packet payload of no more than the maximum packet size of the transport minus 2 bytes.
Channels 0 to 32767 must be reserved for the application and are referred to as message channels.
All other channels must be reserved for Fastnet's protocol and used as specified.

An implementation must prevent the user from using negative channel numbers for any purpose.
This may be done via a type system (use unsigned integers, for example) or by generating an error.

##Status Queries##

Format:
```
query = -1:i16 0:u8 query_identifier:u16 [argument:p]
response = -1:i16 1:u8 query_identifier:u16 response_payload:p
```

Channel -1 is the status query and connection handshake channel.
Queries must be responded to in both directions.
All of the following query operations must be implemented:

- 0 is a query to determine if Fastnet is listening on a specified port.  The response payload takes the form `0:u8` for no and `1:u8` for yes.  The argument is not used.

- 1 is the version query.  The response payload takes the form `version_string:s`, currently "1.0".

- 2 is the extension supported query.  The argument takes the form `name:s`.  Names starting with the prefix "fastnet_" are reserved for this specification.  The response payload takes the form `name:s supported:u8` where `name` matches the sent name and `supported` is 1 if and only if the extension is supported, otherwise 0.

An implementation shall not place limits on the number of times that a query may be sent and must always respond.
An implementation must continue to respond to queries even after a connection is established.

Extension names must be of the form "manufacturer_extensionname."  "manufacturer" should be replaced with a string unique to the person or organization implementing the extension and "extensionname" with a string unique to the extension itself.
Extension and manufacturer names must be ASCII and lower-case.
This specification reserves the manufacturer string "fastnet" for official fastnet extensions.
An implementer must refrain from use of extensions with names of the form "fastnet_xxx".
An implementation must not make use of an extension without first receiving a positive query as to the server's support for said extension.

This specification defines two extensions: "fastnet_p2p" and "fastnet_file".
In order to insure uniqueness, implementers wishing to implement extensions must open a pull request or issue against the GitHub repository containing this specification in order to have a name and description added to the extension index.
Implementers are encouraged in the strongest terms to specify their extensions publicly so that other Fastnet implementations may make use of them.

##Connection Establishment##

packets:

```
connect = -1:i16 2:u8
connected = -1:i16 3:u8 connection_identifier:u32
aborted = -1:i16 4:u8 error:s
```

Connections are established from client to server with the exception of UDP hole punching, described later in this specification and using a different algorithm from that described here.

A Fastnet server must allow only one connection from a specific IP and port.

Before beginning connection establishment, an implementation must use the above query interface to establish that Fastnet is listening in an implementation-defined manner.
This specification suggests that this be done in a similar manner to the following connection handshake algorithm but using the is Fastnet listening query (query number 0).

An implementation must not consider the connection of a connection-based transport to be the establishment of a fastnet connection.

The following must always take place on channel -1 before a connection is considered established.

To begin a connection, a client must:

- Send the connect packet.

- begin waiting for either the connected packet or the aborted packet with a timeout of 5000 MS.  If the transport is unreliable, the client must resend the connect packet every 200 MS during this process; otherwise, it must not.

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
Zero must be considered positive.

Connections must be considered broken in one of two cases:

- If the transport is connection-based and provides a mechanism for determining if the connection is dropped, and this mechanism reports that this is the case.

- If one end of the connection does not receive a heartbeat within a user-specified timeout whose default value must be 20 seconds and whose minimum must be no less than 1 second.  For this purpose, implementations must consider both echoed heartbeats and sent heartbeats to be equivalent.

Both the client and the server must report broken connections to the application without delay.
Servers must also begin behaving as though the client had not connected in the first place; all packets save connection requests and queries must be ignored.

The implementation must send heartbeats to the other end of the connection with an interval no greater than once a second.
The heartbeat interval must be automatically adjusted such that a minimum of 20 heartbeats are sent to the other end of the connection before the connection timeout is reached.

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

We refer to the MTU as the length of the largest packet that is received by the other end of the connection with enough reliability to be useful.
Maximizing the MTU is important to avoid excessive fragmentation.
In this specification, the MTU excludes the initial 2-byte channel count.

Regardless of the determined MTU, either end of the connection must be prepared for packets of up to the maximum packet size as specified by the transport.
For reliable transports, the MTU  must be 2 minus the maximum packet size supported by the transport.
Reliable transports must not perform the following algorithm and should ignore the rest of this section.

The default and minimum MTU for unreliable transports is 32.
This gives a 34-byte packet before fragmentation.

If the transport provides a mechanism for determining the MTU, the transport's algorithm should be delegated to and the rest of this section should be ignored.
In this case, an implementation should determine the transport's MTU and arrive at the payload MTU by subtracting 2.

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

##Message Channel Packet Format##

The following is the complete grammar for a packet on a message channel.

```
message_packet = packet_header [reliability_checksum] [message_header] payload:p
packet_header = channel:i16 packet_flags:u8 [sequence_number:u32 ]
reliability_checksum = checksum:u32
message_header = message_flags:u8 [reliable_message_count:u32]
ack = -5:i16 channel:i16 sequence_number:u32
```

###packet_header###

The packet header consists of the channel on which the packet is being sent, followed by a 1-byte flags byte.
Unused bits must be set to 0.
The following flags are defined:

- 0 is the start of message flag.  This flag is set if and only if this is the first packet in a message.

- 1 is the end of message flag.  It is set if and only if this packet is the last packet in a message.

- 2 is the reliability flag.  It must be set if and only if this packet was sent reliably.

The sequence number must be present if and only if the transport is unreliable.
In all other cases, the sequence number must be inferred by incrementing an internal counter by 1 every time a packet is received.

###reliability_checksum###

The reliability checksum ensures that a reliable packet arrives uncorrupted.
It is present if and only if the reliability flag is set.
To compute it, encode the packet completely, set the checksum portion to zero, and compute the CRC32C checksum over the packet.
This includes the channel count.

A reliable packet is not considered to have arrived until the checksum matches.
Receivers must verify the checksums of incoming reliable packets.

###message_header###

The message header is present for the beginning of all messages. The flags byte currently only uses bit 0, which must be set if the message was sent reliably.
All other bits are currently unused and reserved for future use.

The reliable message count must begin at 0 for all senders and receivers.
It must be incremented every time a reliable message is sent.
It must be present if and only if the transport is unreliable.
it is used later in this specification to coalesce messages.

###payload###

The payload of the packet is a fragment of a message and extends until the maximum MTU.
The content of this payload is defined by the application and by the message encoding algorithm, which specifies how to turn a message into a stream of bytes.

###acks###

Ack stands for acknowledge.
Acks must be sent for all reliable packets which arrive with valid checksums.
They consist of the channel and sequence number for the packet being acknowledged.
The sender's resending algorithm is implementation-defined.

A reliable packet is considered to have arrived in only one of the following two cases:

- The transport is reliable.

- The sender received an ack.

If the transport is reliable, ack sending must not be performed; this floods the network to no gain.

##Encoding Messages##

Converting a message to packets for sending occurs in two phases: payload conversion and packet splitting.

to convert a message payload for sending, do the first of the following that applies:

- If the transport is incorruptible, leave the message's content as-is.

- Otherwise, if the message is less than 16 bytes in length and the first bit of the message is different than the last bit of the message, duplicate the message's content.

- Otherwise, if the message is less than 16 bytes in length, append the bitwise not of the message's contents.

- Otherwise, append the MD5 checksum of the message's contents.

Then prepare a message header as described above and prepend it to the message.

Finally, to send the message, an implementation must split this prepared contents among however many packets must be used to avoid going over the MTU including the headers of said packets, setting flags appropriately and submitting each in turn for sending.
All other concerns are handled by this lower layer.

##Receiving Messages##

There are two variants of message reception, depending on if the transport is reliable or unreliable.
There is one user-defined setting: the message coalescing duration (MCD).
The MCD is ignored for reliable transports

Implementations must implement both of the following algorithms:

###Receiving Messages on a Reliable Transport###

This is by far the simplest.  When a packet with the start of message flag is received, begin saving packets until a packet with the end of message flag is received.   Then, decode the payloads into a message.
Since reliable transports are ordered and incorruptible, no further action is needed; the payload must be submitted directly to the application.

###Receiving Messages on Unreliable Transports###

Packet sequence numbers are related to one another by the comes-before relation.  This relation is defined in [RFC 1982](https://tools.ietf.org/html/rfc1982).
This relation can become undefined, but only if 2^32-1 packets go missing consecutively.

In order to receive messages on unreliable transports, implementations must do three things:

- First, tag all packets with the time in which they were received.

- Second, place all packets in a container ordered by the comes-before erlation.

- Third, execute the algorithm described here every time a packet with the start of message or end of message flag set is received.

The rest of this algorithm is defined in terms of the message coalescing group (MCG).  It is computed by applying the following in order:

- The MCG starts by covering the first packet only.

- If the first packet has both the start of message and end of message flags set, the MCG is computed; stop.  Otherwise, continue.

- If the second packet in the container has a different reliability flag than the first, stop.

- Move the MCG's future end forward one packet.

- Continue moving the MCG's future end forward until it encounters the end of the container, a packet with a reliability flag that doesn't match the reliability flag of the first packet in the MCG,  a packet with the start of message flag, or a packet with the end of message flag.

- If the MCG's future end is on a packet with the start of message flag set, move it back one packet.

If the MCG meets the following conditions, then it is called completed.

- The first packet has the start of message flag set.

- The last packet has the end of message flag set.

- The sequence numbers of all packets are consecutive.

- Decoding the message header in the first packet and examining the reliable message count reveals that no reliable messages are missing between the last message received and this one (see below).

If the MCG is completed, combine the payloads, perform message verification in the straightforward manner (see above about payload preparation), and give the message to the application.

If the MCG is not completed, compute the MCGLMT (MCG last modified time) as the maximum of the recorded reception times for all packets in the MCG.
If the MCGLMT is more than MCD seconds ago and the first packet is not reliable, throw out the packets in the MCG.
otherwise, abort this algorithm and run it again later.

A reliable message is expected if either:

- The next message being received is unreliable and it has a different reliable message count than that of the last message we received.

- The next message being received is reliable and the reliable message count is not the same as that we would receive by incrementing the currently saved reliable message count of the last received reliable message.

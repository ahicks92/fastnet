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

- The recognized type suffixes are `u`, `i`, `s`, `p`, and `id`.  A terminal is followed by a `:` and a type suffix.

- `u` and `i` represent unsigned and signed integers stored in network byte order using twos complement.  Each is followed by a bit count, for example `u8` or `i16`.  The bit count must be a multiple of 8.  All math on integer types must be performed mod `2^n` where n is the number of bits in the integer (put another way, math wraps).  0 is considered positive.

- `b` represents a boolean encoded as a `u8`.  0 represents false.  1 represents true.  No other value is allowed.

- `s` represents null-terminated UTF8 strings.  It may be followed with a length restriction of the form `{min, max}`.  The default length restriction is `{1, INFINITY}`.  Said restrictions are in UTF8 code points.

- `a` is like `s`, but the string must be ascii.

- `p` stands for payload, an arbitrary sequence of bytes.  `p` segments will be described further by the specification.  They are usually the last field of a packet.

- `id` means a 16-byte identifier computed as A UUID.

- * means 0 or more. + means one or more.  These have the highest precedence.


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
Such channels are referred to as private or reserved channels.

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

- 1 is the version query.  The version response must contain a version string in the form `major.minor`.  The current version is "1.0".  Two implementations must be compatible if they have the same version number.  It is anticipated that implementations will be considered compatible if they have the same major version number, but this specification avoids mandating it for now.

- 2 is the extension supported query.  Extension names should be of the form `vendorname_extensionname` and stored in lower case.  This specification reserves the prefix `fastnet_` for use by this specification.

An implementation shall not place limits on the number of times that a query may be sent and must always respond.
An implementation must continue to respond to queries even after a connection is established.

##Connection Establishment##

packets:

```
connect = -1:i16 2:u8 id: id
connected = -1:i16 3:u8 id: id
aborted = -1:i16 4:u8 error:s
```

With the exception of UDP hole-punching, connections are established using the following algorithm.  UDP hole-punching is described elsewhere in this specification.

A Fastnet server must allow only one connection from a specific IP and port.

The following must always take place on channel -1 before a connection is considered established.

To begin a connection, a client must:

1. Generate an id.  This id will be used to identify the connection to the user.

2. Use the `fastnet_query` from the status query section to determine if a fastnet implementation is listening.  An implementation must make no more than 10 attempts before aborting.

3. Use the `version_query` to determine that the implementations are compatible.  Again, an implementation must make no more than 10 attempts before aborting.

4. Send the connect packet, containing the id.

5. Begin waiting for either the connected packet or the aborted packet with a timeout of 5000 MS.  The client must resend the connect packet every 200 MS during this process.  If the connected packet does not contain an ID matching the ID we sent, ignore it.

If the client receives the connected packet, it must notify the application that the connection has been established and begin processing packets.
The client must disregard all other packets including queries until it manages to receive the connected packet.

If the client receives the aborted packet, it must report the error string to the user of the implementation in an implementation-defined manner.

if the client times out before receiving either the connected or aborted packet, the implementation must report an implementation-defined error.

When the server sees the connect packet and wishes to accept a connection, it must send the connected packet containing the id sent by the client.
If the server continues to receive the connect packet, it must continue to respond with the connected packet but do nothing further; it is possible for the client to not yet know that it is connected due to packet loss.
If there is a UUID collision, the server is free to simply ignore the incoming packet.
Given the unlikelihood of having two connections from two different IP/port pairs generating the same UUID, such behavior merely causes an astronomically small percent of connection attempts to time out.

To refuse a connection, a server must send the aborted packet with an implementation-defined string as to the reason.
This string must not be empty.
This specification suggests "unspecified error" as a reasonable default for situations in which a string is not available, i.e. because a game does not wish to advertise that a player has been banned.

Servers must ignore any packets not involved in an active connection.

##Connection Closing and Breaking##

TODO: define packets and intentional closing logic (this is awaiting a coming refactor involving identification of connections by UUID).

If either end of a fastnet connection does not receive any packets from the other end of the connection for a timeout period  then it must consider the connection broken.  This period must be configurable by the user on either an implementation-wide or connection-specific basis and should default to 10 seconds.

##The Heartbeat Channel##

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

Heartbeat packet counts do not include any packets exchanged before the establishment of a connection.

An implementation must not place any semantic meaning on anything in the heartbeat beyond using it for rough packet loss estimation.

##The Echo Channel##

Packet format:

```
echo = -3: i16 endpoint: u128 uuid: u128
```

When an implementation receives a packet on the echo channel for a connected Fastnet peer, it must immediately resend (echo) the packet back to the sender without modification.

This channnel exists for implementations wishing to attempt provision of round-trip estimation.  A conforming implementation must implement the echo channel but is not required to provide a round-trip estimation algorithm.

Endpoint is a value which must be generated from a UUID at connection startup and not changed thereafter.  uuid is a value which must be generated per-echo.  An implementation must not respond to echoes whose endpoint value matches their own, as this means that the other side of the connection sent it in response to a previous echo.

##Frame Channels

A frame is a container whose maximum size is 4GB.
These frames are then sent across the network using the algorithms  described below.
Channels which have frames sent on them are known as frame channels.

As will be seen later in this specification, Fastnet needs the ability to send large chunks of information to clients.
this section lays out how to send or receive a frame on a channel, either reliably or unreliably.

###Frame Packets

grammar:

```
data = 0: u8 sn: u64 flags: u8 payload: p
ack = 1: u8 (sn: u64)+
```

Data packets represent chunks of data of arbetrary length.
The flags byte has the following flags:

- 0 is the start of frame flag.

- 1is the end of frame flag.

- 2 is the reliability flag.

All other bits of the flags byte must be 0.

The sequence number must be set to 0 for the first packet sent on some channel, 1 for the next, etc.
Sending enough data to exhaust the available sequence numbers is nearly impossible.
A 64-bit sequence number is capable of handling at least 18 million terabytes of data, well beyond what can be handled by any consumer and most professional-grade internet.
In practice, few data packets will carry only one byte of data, so this limit is all but meaningless.
To that end, an implementation must drop the connection if the sequence number for any channel is exhausted.

Borrowing from TCP terminology, ack is short for acknowledge.
The ack packet is used to indicate reception of a reliable packet.
It must only be sent for reliable packets in the manner described below.

###Encoding Frames

A frame is an array of bytes of any length, with a header of the following form:

```
last_reliable: u64 length: u32
```

Any limits on the size of outgoing frames are implementation-defined, but are limited to 4GB because of the length field.
Limits on incoming frames are implicitly defined by the packet reception algorithm, defined later in this document.

To send a frame, an implementation must implement the following algorithm or something functionally equivalent:

- Get data from the user and add the header.

- Split the frame into some number of chunks, putting each into a data packet with all fields set to 0.  The entire header must be in the first chunk.

- Allocate sequence numbers to all packets in the obvious manner.

- If the frame is being sent reliably (hereafter a reliable frame), set the reliability flag on all packets.

- Set the start of frame flag on the first packet; set the end of frame flag on the last packet.  If these are the same packet, both flags should be set.

- Send all packets using the algorithms described below.

- If this is a reliable frame, update the stored sequence number used to set `last_reliable` by setting it to the first sequence number assigned to this frame.  `last_reliable` is described below.

Frames must be split into chunks of data which can be encapsulated in a data packet of no more than 1000 bytes in total length.
4 bytes are taken by the checksum, 2 by the channel identifier, 1 by the specifier for data  packet, 8 by the sequence number, and 1 by the  flags byte.
This leaves a total of 984 bytes of payload for each data packet.

In order to encode a frame, an implementation must split the frame into some number of chunks.
the algorithm for doing this is implementation-defined.
For the earliest stages of implementing Fastnet, using a simple fixed size is the easiest option, though this specification suggests that implementations taking this path allow the user to configure it.
There are more detailed and involved approaches that work better, however, such as MTU estimation.

Frame headers currently have 2 fields:

- `last_reliable` must be set to the sequence number of the packet which started the previous reliable frame.
If no reliable frame has been sent yet, `last_reliable` must be set to 0.

- `length` must be set to the length of the frame including the header.

###Sending Unreliable Data Packets

To send an unreliable data packet, an implementation shall encode and broadcast it, as with all other unreliable packets.

###Sending Reliable Data packets

An implementation must send reliable packets as if sending unreliable packets.
The difference is that an implementation must remember and periodically resend unacked reliable packets.
The resending algorithm is implementation-defined but should use exponential backoff if possible.
This section will be updated once experience shows which method is the best.

It is possible to attack a fastnet implementation by never acking a packet.
A fastnet implementation must provide facilities to detect this case and deal with it; at a minimum, it must be possible for the application developer to forceably drop such bad-behaved connections if they begin using too many resources.

###Receiving Frames and Acking

Though it can be used for other purposes, Fastnet is designed primarily for the use case of games.
When given a choice between two alternatives, Fastnet chooses the one which is most likely to be friendly to a client-server architecture where the server sends much more data than the client.
To that end, most of the complexity of sending and receiving frames is placed in the receiver.

This subsection is long and involved, and by far the most complicated part of this specification.
If your goal is to implement the Fastnet specification, read carefully.
To aid in comprehension, a number of subsections follow.

Most of this section is per-channel.
When it is not, the specification will be sure to inform you.

####The Packet Storage Area

For the purposes of public APIs, there must be a per-connection and per-channel packet storage area allocated to each frame channel.
It is used to store data packets which have not yet been assembled into frames.

At a minimum, the packet storage area must be able to store whether or not a packet has been acked.
One good way of doing this is to use separate containers for acked and unacked packets.
As will be seen shortly, acking packets immediately is not always possible.

To protect against a variety of attacks that involve sending incomplete frames on all channels, There must be a connection-specific memory limit on the size of the packet storage areas for all channels.
if this limit is exceeded, the application must be informed in a manner that allows it to drop the connection or raise the limit.
This limit must default to 2 MB and be configurable by the application.
This limit must not be allowed to go below 100KB and must count private channels.
Without this limit, it is possible for an attacker to send multible gigabytes of reliable data packets on each channel, eventually exhausting ram.
the rest of this specification refers to this limit as the per-connection memory limit.


There must be a per-channel limit on the size of the packet storage area, hereafter the per-channel memory limit.
The per-channel memory limit must default to 100KB for all message channels and be configurable by the application developer.
Whether it can be configured on a per-channel basis is implementation-defined behavior.
The channel-specific memory limit will be stated for private channels when it deviates from this default.
how it is used it described below.


Both of the above described limits must be specified in bytes of data payload.

the maximum frame size which can be used is the minimum of the two above limits.
It is recommended that implementations provide helper functions which can be used to determine and configure appropriate settings for these values given a specification of whichb message channels the application will use and the maximum message size on each.

####Ignoring and Dropping Data Packets

Let there be a channel-specific unsigned 64-bit integer called the ignore number.
The initial value of the ignore number is 0.
How the ignore number is updated will be described below when assembling and delivering frames is discussed.
If an incoming and unreliable packet has a sequence number less than the ignore number, it is immediately dropped.
If an incoming and reliable packet has a sequence number less than the ignore nyumber, it is immediately acked and then dropped.

When a receiver receives an unreliable packet that would take a channel over the per-channel memory limit, it must ignore it as though it was never received.

When a receiver receives a reliable packet that would take it over the per-channel memory limit, it must perform the following, stopping  at the first which succeeds:

- First, if there are one or more unreliable packets which have not yet been processed and whose removal would make enough room for the packet, they must be dropped and the new packet accepted.  Implementations should first prefer packets whose sequence number is less than the packet: if the packet storage area is a sorted array, the implementation should iterate from lowest to highest index.

- Next, if there are one or more reliable packets with higher sequence numbers whose absence would make enough room for the reliable packet, they must be dropped in favor of the packet.  Implementations should prefer reliable packets with the highest sequence number first: if the packet storage area is  a sorted array, implementations should iterate from highest to lowest index.

- Next, if performing both of the above cases would result in enough room, perform both.

- Finally, the packet should be dropped.

If performing the first two above cases cannot make room for the packet, the packet should be dropped without modifying the contents of the packet storage area.

####verifying Frame Headers

When an implementation receives a data packet which has the start of frame flag set, it must check the length against the per-connection memory limit and the per-channel memory limit.
If the length of the incoming frame is larger than the smaller of these limits, the connection must be closed.
Applications should be given an opportunity to intercept and prevent this closure.
This procedure must be performed before acking occurs.

Note that raising the limit without taking care is a possible vector of attack.
Suppose the application always increases both limits by 5%.
In this case, an attacker can start by sending a frame of only a few bytes.
To continue the attack, the attacker simply increases the size by 4.5% every time, resending the frame.
Eventually, the application has increased the limit to at least the size of the largest packet the attacker has sent.
In the best case, this was done per-channel and the attacker has to keep sending for a while to exhaust memory.
In the worst case, the application raised the limit for all channels and the attacker need only get to 1 MB or so before sending a 1MB frame to all channels, taking all available memory.
To prevent this, implementations are strongly encouraged to inform users that changing these settings and preventing connection closing should only be used for debugging purposes or in very special cases by expert users.

####Acking Packets

Immediately after an ack is sent, the ignore number must be updated to one more than the sequence number sent in the ack packet.
An ack must be sent if and only if one of the following two cases is met:

- First, there is an unacked reliable packet in the packet storage area whose sequence number equals the ignore number.

- Second, there is an unacked reliable packet in the packet storage area which begins a frame and has a header with `last_reliable` set to the first packet of the most recent delivered reliable frame sent on the channel in question.

It is anticipated that implementations will periodically iterate over the packet storage area, performing the above acking algorithm.
Implementations should attempt to keep the delay between receiving an ackable reliable packet and sending the ack for that packet to under 200 MS.

Implementations must be prepared to receive duplicate reliable packets.

Preemptively sending the first packet of large reliable frames multiple times is a possible performance-improving measure.

####Assembling frames

An unassembled frame is a group of data packets meeting the following conditions:

- Has consecutive sequence numbers.

- Has the same value for the reliability flag in all packets.

- If the packets are reliable, all have been acked.


- The first packet has the start of frame flag set.

- The last packet has the end of frame flag set.

- No other packet has the start of frame or end of frame flags set.

- The total lengths of the payloads of all the packets in the group sum to match the length as specified in the header contained in the first packet in the group.

An unassembled frame may be assembled by concatenating all of the payloads of the unassembled frame into a  buffer.
A deliverable frame is an unassembled frame whose `last_reliable` is set to the sequence number of the first packet of the last reliable frame delivered on this channel.
Delivering a frame refers to assembling a frame, stripping the header, and notifying other parts of Fastnet of the frame's existance as needed.

An implementation must periodically scan the packet storage area to identify possible deliverable frames.
When it finds one, it must immediately deliver it.
How this is done is implementation-defined.

After delivering a frame, the ignore number must be updated to the sequence number of the last packet in the frame; additionally, all packets in the packet storage areaa whose sequence number is now less than the ignore number must be dropped.

##Messages and Message Channels

A message channel is a frame channel with no particular limits.
A message is a frame sent on a message channel.
To receive a message meanns to have a frame be delivered on a message channel.

When an implementation receives a message, it must immediately notify the application of the message in an implementation-defined manner.

An implementation must support sending messages in an implementation-defined manner.

Messages will eventually have headers and additional features.  This section is incomplete, but is enough to code.
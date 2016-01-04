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

##Transports##

A transport is a method for moving packets from one destination to another.
Transports must have the following characteristics.

- An ordered transport must guarantee that, if packets arrive, they arrive in order.

- An incorruptible transport must guarantee that, if packets arrive, they match the sent packet.

- A reliable transport must guarantee that packets arrive if sent.  Reliable transports must be ordered and incorruptible transports.

- A connection-based transport must handle all logic of forming and keeping a connection open.  This specification refers to transports which are not connection-based as connectionless.

- A punch-capable transport must support the UDP hole-punching algorithm specified later in this specification.

All implementations must provide at least the TCP and UDP transports.
The TCP transport must be ordered, reliable, connection-based, and not punch-capable.
The UDp transport must be unordered, unreliable, connectionless, and punch-capable.

Every transport must advertise the maximum packet size it supports in a way that allows the user of the implementation to obtain the value.
Every transport must support a maximum packet size of at least 4096 bytes.

##Basic Packet Format##

A Fastnet packet must consist of a 2-byte channel identifier as a signed 16-bit integer followed by a packet payload of no more than the maximum packet size of the transport minus 2 bytes.
All integers are sent in big-endian.

This specification specifies per-channel packet formats.
Channels 0 to 32767 must be reserved for the application and are referred to as "message channels."
All other channels must be reserved for Fastnet's protocol and used as specified heere.

##Status Queries##

Channel -1 is the status query and connection handshake channel.
Packets on channel -1 are ascii literals with one exception.
The following query operations must be supported:

- If a client sends the string "query:fastnet" as a packet payload on channel -1, the server must respond with "query:fastnet=yes"

- If the client sends "query:version", the server must respond with "query:version=1.0"

An implementation shall not place limits on the number of times that a query may be requested from the server.
An implementation must continue to respond to queries even after a connection is established.

##Connection Establishment##

Before beginning connection establishment, an implementation must use the above query interface to establish that Fastnet is listening in an implementation-defined manner.
An implementation must not consider the connection of a connection-based transport to be the establishment of a fastnet connection.
The following must always take place on channel -1 before a connection is considered established.

NOTE: because reliable transports are ordered, the following algorithm cannot cause data loss on a reliable transport even though the client intensionally drops packets.

To begin a connection, a client must:

- Send the string "connect?"

- begin waiting for the string "connected" followed by a 4-byte integer or "abort" followed by any number of UTF8 bytes with a timeout of 5000 MS.  If the transport is unreliable, the client must send "connect?" every 200 MS during this process; otherwise, it must not.

If the string "connected" was received before the timeout expired, the client must consider itself connected and parse the integer.
This integer is the connection's ID.
Otherwise, the client must report an implementation-defined error.

If the client receives the string "abort", it must abort connection immediately.
The rest of the packet must be a UTF8-encoded string without a trailing NULL character.
This specification places no restriction on the content, but suggests that it be used as an error condition and displayed to the user.

The client must disregard all other packets until it is connected.

When the server sees the string "connect?", it begins establishing a connection.
To establish a connection, a server must generate an integer ID.
This ID must be unique among all currently-established connections.
It must then echo the packet "connected" followed by the integer on channel -1 and immediately notify the application that a connection was established.
If the server continues to receive "connect?" packets from the same client, it must respond with "connected" and the integer ID and do nothing further; it is possible for the client to not yet know that it is connected due to packet loss.

To refuse a connection, a server must echo the packet "abort" followed by an optional UTF8 string (without a NULL character) on channel -1.

Both clients and servers must expose the Fastnet connection ID for a connection in a manner that can be reached by the application developer; this ID is part of the UDP hole-punching algorithm.

Servers must ignore any packets not involved in an active connection.

##The Heartbeat Channel, Connection Breaking, and Round-trip Estimation##

Channel -2 must be the heartbeat channel.
Heartbeats must consist of a signed 16-bit integer.

If a client receives a positive integer on the heartbeat channel, it is to immediately echo it back to the server.
If a server receives a negative integer on the heartbeat channel, it is to immediately echo it back to the client.
In all other cases, the client and server should respond by sending nothing.

Connections must be considered broken in one of two cases:

- If the transport is connection-based and provides a mechanism for determining if the connection is dropped, and this mechanism reports that this is the case.

- If one end of the connection does not receive a heartbeat within a user-specified timeout whose default value is to be 20 seconds.

Both the client and the server must report broken connections to the application as soon as they have determined that the connection is in fact broken.
Servers must also begin rejecting packets from this client.

The implementation must send heartbeats to the other end of the connection with an interval no greater than once a second.
The heartbeat interval must be automatically adjusted such that a minimum of 20 heartbeats are sent to the other end of the connection before the connection timeout is reached.

Implementations are required to provide packet round-trip estimation without violating this specification or using extra channels for their protocol implementations.
The default round-trip time must be 200 MS.
The most basic conforming algorithm for packet estimation is to use this default, but it is suggested that implementations take advantage of the heartbeat channel to perform a smarter estimate.

##Determining Payload MTU##

The Payload MTU refers to the maximum size of a packet's payload and should never be allowed to fall below 32.
Regardless of the determined payload MTU, either end of the connection must be prepared for packets of up to the maximum packet size as specified by the transport.
For reliable transports, the payload MTU  must be 2 minus the maximum packet size supported by the transport.
Reliable transports must not perform the following algorithm and should ignore the rest of this section.

The default payload MTU for unreliable transports is 32.
If the transport provides a mechanism for determining the MTU, the transport's algorithm should be delegated to and the rest of this section should be ignored.
In this case, an implementation should determine the transport's MTU and arrive at the payload MTU by subtracting 2.

Channels -3 and -4 are the server MTU and client MTU estimation channels respectively.
Both the client and server must perform the following algorithm at connection start-up and periodically thereafter.
The period is implementation-defined.
the only difference between the client and server is which channel they are sending on.
We refer to them as the estimator and the responder, and to the channel of the estimator as the channel.
Both the client and the server must be capable of playing both roles simultaneously.

The estimator must implement the following pseudocode, repeating the algorithm every period.

```
interval <- an implementation-defined interval.
mtu <- 32 (the estimated payload MTU)
estimated_mtu <- 32 (the final estimated MTU)
final_timeout <- an implementation-defined timeout, should be on the order of 1 second.
prev_count <- 0
count <- 0
initial_percent <- 0
first <- true
while mtu < transport.maximum_size:
    prev_count <- count
    count <- 0
    send_count <- 100
    while the client has not seen a packet on the channel whose value is 1:
        send a packet on the channel containing the single byte 0.
        Wait for three times the estimated round-trip time.
    for i from 1 to send_count:
        send a packet of random contents whose payload is mtu bytes long.
        Wait for the interval. If a packet from the responder is received and is 4 bytes long:
            count <- max(count, packet.payload.as_i32)
    Wait for the final_timeout.  If a packet is received on the channel with payload of 4 bytes during this time:
        count <- max(count, packet.payload.as_i32)
    percent = count/send_count
    if first:
        initial_percent <- percent
        first <- false
    if percent < initial_percent and initial_percent-percent > 0.1:
        break because we've gotten significantly worse.
    else if prev_count != 0:
        prev_percent = prev_count/sent_count
        if percent < prev_percent and percent-prev_percent > 0.05:
            break because we found a sharp drop-off.
    final_mtu <- mtu
Update the estimated MTU with final_mtu
```

The responder is simpler.
When the responder sees a packet whose payload is the single byte 0, it must immediately set an internal counter to 0 and respond with a packet whose payload is the single byte 1 on the channel.
For all other packets, it must increment the count and respond with a packet whose payload is the count as a 4-byte integer.

##The Message Format##

All implementations must be able to assemble a message, consisting of the following four elements with no padding between them:

- A flags byte.

- A length. See below for the format.

- The message's payload.

- A checksum of the message's payload, specified below.

The flags byte must have  1 flag in bit 0.
The flag in bit 0 is reliability.  It is to be set if the message must be sent reliably.

Bits 7 and 8 control length parsing.
To determine the number of bytes in the length, the implementation must pull bits 7 and 8 out and place them into an integer, such that bit 7 is bit 1 and bit 8 bit is bit 2.
The implementation must then add 1 to this integer, and use that to determine the length of the length field.
lengths are at most four bytes, stored as big-endian unsigned integers; this scheme is chosen so that small messages are not penalized by carrying the entire length when it is unnecessary.
Other bits of the flags byte are reserved for future additions to the protocol.

The length must consist of the length of the payload only. The length of the checksum is inferred as described below.

The payload must consist of the payload as specified by the user, in the obvious and straightforward manner.

Checksum computation depends on the size of the payload.

For payloads of less than 16 bytes, the checksum must be a copy of the payload bit-reversed and notted.
In this context bit-reversed means reversing the string of bits as opposed to the string of bytes: a payload `b1b2b3` would have the checksum `not(b3b2b1)`.

For payloads of 16 bytes or more, the checksum consists of the MD5 digest of the payload.

Implementors should note that the actual length of the payload is twice the length field for messages of less than 16 bytes, and the length field plus 16 for messages of 16 bytes or more.

##Message Channel Wire Format##

Channels 0 through 32767 shall be the message channels, available for any purpose that the application requires.
In order to send messages on these channels, it is necessary for Fastnet to convert them to an ordered transport with optional reliability.

If the transport is reliable (and thus ordered), the packet format is not special: encoding must consist only of packaging the payload in a Fastnet packet.

Otherwise, implementations must use the following format:

- A single flags byte.

- A 3-byte integer in big endian called the packet sequence number.

- A 3-byte integer in big endian called the reliable sequence number.

- The packet's payload.

The flags byte must have two flags.  Bit 0 must be the reliability bit, specifying whether or not this packet is reliable.  Bit 1 must be set if and only if this packet starts a message (the start of message flag).

The sequence number starts at 0.  When the sender wraps their sequence number, it must wrap to -10000.
A packet a comes-before a packet b if either  a's sequence number is is less than b's or if a has a positive sequence number and b has a negative sequence number.

The reliable sequence number is a sequence number incremented only when reliable packets are sent.
it is used to detect the likelihood of more incoming unreliable packets.

##Message to Paccket Conversion##

The message MTU is the payload MTU minus 7 bytes for the channel packet overhead.

To convert a message to packets, an implementation must:

- Allocate enough message MTU-sized packets to hold the entire message string.

- Allocate all packets adjacent sequence numbers.

- Set the start of message flag in the first packet.

##Sending Packets Unreliably##

A sender must be able to send a packet unreliably.
To do so, the sender must submit the packet to the transport and then forget about it.
If the transport is unreliable and the implementation has detected that high packet delivery rates are not possible, the sender is permitted to send the packet twice.
This specification recommends that an implementation delay the second send of the packet if it opts to send the packet twice, sending other packets in the meantime if possible.
Note that resending packets in this manner should be reserved as a last resort and not done for large packets; such behavior makes the implementation unfriendly to the network and may at worst result in penalties imposed by routers and firewalls.

##Sending packets Reliably##

Channel -5 is the ack (acknowledge) channel.
The format of an ack packet from a client must be a n-byte payload of any size, such that n is a multiple of 5.
This packet must consist of one or more channel and sequence number pairs (here called acks) for packets that a receiver wishes to confirm receipt of.
That is `c1s1 c2s2 c3s3` etc. packed without padding.

When a sender sends a packet reliably and if the transport is reliable, the sender simply assumes reception.

When a sender sends a packet reliably and the transport is not reliable, the sender must hold the packet until it receives an ack.
The sender must also resend the packet on an interval.
This interval is computed as `round_trip*fn(n+2)` where round_trip is the currently estimated round-trip time, n is the number of the retry attempt, and fn represents the fibinacci sequence.
This specification defines `fn(0) = 1`, `fn(1) = 1`, `fn(n) = fn(n-1)+fn(n-2)`.

Implementations should not use recursive or iterative Fibinacci functions. Instead, implementations should use a pair of variables or a table.
Recursive and iterative fibinacci functions are expensive and may be called tens of thousands of times a second.

if the transport is unreliable and the implementation has estimated low packet reception, reliable packets may be sent preemptively up to 5 times.
Note that resending packets in this manner should be reserved as a last resort and not done for large packets; such behavior makes the implementation unfriendly to the network and may at worst result in penalties imposed by routers and firewalls.

##Receiving Packets Unreliably##

The receiver has a value called the ignore number.  Any packet which comes before the ignore number and which is being sent unreliably is to be ignored.  The ignore number starts at -1.

When a packet is received by a receiver, the receiver must put the packet in a data structure sorted by the comes-before relation: a comes-before b if a.sequence < b.sequence or (a.sequence > 0 and b.sequence < 0).
The receiver must also record the reception time of the packet to at least a millisecond accuracy and using a monatonically increasing clock if at all possible.
The receiver may receive packets out-of-order and the sender may send very large messages, so arrays are not recommended for this purpose.
The rest of this specification refers to this structure as a queue and treats it as such in algorithmic descriptions.
If the receiver is receiving an unreliable packet and the receiver does not have enough ram, the receiver is permitted to disgard the packet.
The receiver should not disgard packets which are already recorded, save for as described in the message coalescing section.
This specification does not make this final condition binding: there may be no other way to obtain ram.

##Receiving Packets Reliably##

When a reliable packet arrives, it is always placed in the queue of packets, regardless of the ignore number.

If the transport is unreliable, the receiver must send an ack on the ack channel.
Receivers are allowed to group multiple acks into one ack packet.

If there is not enough ram to receive a reliable packet, the implementation should attempt to forget about unreliable packets to make room for it.
Reliable packets get priority over everything.

##Message Coalescing##

The message Coalescing interval is an implementation-defined period.  The suggested default for the message coalescing interval (MCI) is 25 MS.

The current message group (CMG) is a group of packets.
It consists of at least the packet with the lowest sequence number and continues until the next packet with the start of message flag.
less formally, it is all of the packets which may currently make up one message.

The last modified time of the CMG is the maximum of the reception times of the packets in the CMG.

An implementation must do one of three things with the CMG:

- If the first packet of the CMG has a reliable sequennce number  different than that of the last reliable packet that we recieve, abort and do nothing.  More reliable packets are on their way.

- If the first packet of the CMG is a start-of-message, check to see if the total length of the CMG's payloads is large enough to form a message and if the sequence numbers indicate no missing holes.  If this is the case, decode, verify, and deliver the message.

- If the last modified time of the CMG was more than MCI ago and the CMG is unreliable, drop the entire CMG.

If a message was delivered or a CMG was dropped, update the ignore number to be the highest sequence number in the CMG.
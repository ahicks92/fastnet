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

All implementations must provide at least the TCP and UDP transports.
The TCP transport must be reliable and connection-based.
The UDp transport must be unordered, unreliable, and connectionless.
Over IPV4, UDP's built-in checksum is optional.
To this end, UDP must also be corruptible.

Every transport must advertise the maximum packet size it supports in a way that allows the user of the implementation to obtain the value.
Every transport must support a maximum packet size of at least 4096 bytes.
Note that Fastnet will perform packet MTU discovery.

##Basic Packet Format##

A Fastnet packet must consist of a 2-byte channel identifier as a signed 16-bit integer followed by a packet payload of no more than the maximum packet size of the transport minus 2 bytes.
All integers are sent in big-endian.

This specification specifies per-channel packet formats.
Channels 0 to 32767 must be reserved for the application and are referred to as "message channels."
All other channels must be reserved for Fastnet's protocol and used as specified.

An implementation must prevent the user from using negative channel numbers for any purpose.
This may be done via a type system (use unsigned integers, for example) or by generating an error.

##Status Queries##

Channel -1 is the status query and connection handshake channel.
Queries must be responded to in both directions.
A query packet consists of the channel specifier, the byte 0, the number of the query to be performed as a 16-bit integer, and an optional argument.
A response packet consists of the byte 1, the number sent in the query packet, and a response as specified in the following table.
All of the following query operations must be implemented:

number | description | Arguments | response
------ | ----------- | --------- | --------
0 | Is Fastnet listening on this port? | None | one byte, 0 if no, 1 iif yes.
1 | version query | None | The version of the protocol as a string without terminating null. Currently this is "1.0"
2 | extension query. | Name of the extension as an ASCII string without terminating NULL | the byte 0 (not supported) or the byte 1 (supported) followed by the extension's name as sent by the client without terminating NULL.

An implementation shall not place limits on the number of times that a query may be sent and must always respond.
An implementation must continue to respond to queries even after a connection is established.

Extension names must be of the form "manufacturer_extensionname" and no longer than 64 characters.  "manufacturer" should be replaced with a string unique to the person or organization implementing the extension and "extensionname" with a string unique to the extension itself.
Extension and manufacturer names must be ASCII and lower-case.
This specification reserves the manufacturer string "fastnet" for official fastnet extensions.
An implementor must refrain from use of extensions with names of the form "fastnet_xxx".
An implementation must not make use of an extension without first receiving a positive query as to the server's support for said extension.

This specification defines two extensions: "fastnet_p2p" and "fastnet_file".
In order to insure uniqueness, implementors wishing to implement extensions must open a pull request or issue against the GitHub repository containing this specification in order to have a name and description added to the extension index.
Implementors are encouraged in the strongest terms to specify their extensions publicly so that other Fastnet implementations may make use of them.

##Connection Establishment##

Connections are established from client to server with the exception of UDP hole punching, described later in this specification and using a different algorithm from that described here.

A Fastnet server must allow only one connection from a specific IP and port.

Before beginning connection establishment, an implementation must use the above query interface to establish that Fastnet is listening in an implementation-defined manner.
This specification suggests that this be done in a similar manner to the following connection handshake algorithm but using the is Fastnet listening query (query number 0).

An implementation must not consider the connection of a connection-based transport to be the establishment of a fastnet connection.

The following must always take place on channel -1 before a connection is considered established.

NOTE: because reliable transports are ordered, the following algorithm cannot cause data loss on a reliable transport even though the client intensionally drops packets.

There are three packets involved in the connection handshake protocol, all of which are sent on channel -1:

- The connection request packet consists of the single byte 2.

- The connect packet consists of the single byte 3 followed by a 4-byte integer.  This integer is the connection's identifier.

- The abort packet consists of the single byte 4 followed immediately by a UTF8-encodedd error string without a trailing null.

To begin a connection, a client must:

- Send the connect request packet.

- begin waiting for either the connect packet or the abort packet with a timeout of 5000 MS.  If the transport is unreliable, the client must resend the connection request packet every 200 MS during this process; otherwise, it must not.

If the client receives the connect packet, it must parse the connection id, notify the application that the connection has been established, and begin processing packets.
The client must disregard all other packets including queries until it manages to receive the connect packet.

If the client receives the abort packet, it must report the error string.

if the client times out before receiving either the connect or abort packet, the implementation must report an implementation-defined error.

When the server sees the connection request packet, it begins establishing a connection.
To establish a connection, a server must generate an integer ID.
This ID must be unique among all currently-established connections.
It must then encode and send the connect packet and immediately notify the application that a connection was established.
If the server continues to receive the connection request packet, it must continue to respond with the connect packet but do nothing further; it is possible for the client to not yet know that it is connected due to packet loss.

To refuse a connection, a server must send the abort packet with an implementation-defined string as to the reason.
This string must not be empty.
This specification suggests "unspecified error" as a reasonable default for situations in which a string is not available, i.e. because a game does not wish to advertise that a player has been banned.

Both clients and servers must expose the Fastnet connection ID for a connection in a manner that can be reached by the application developer; this ID is part of the UDP hole-punching algorithm.

Servers must ignore any packets not involved in an active connection.

##The Heartbeat Channel, Connection Breaking, and Round-trip Estimation##

Channel -2 must be the heartbeat channel.
Heartbeats must consist of a signed 16-bit integer.

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
        Wait for the interval. If a packet from the responder is received and len(packet.payload) == 4:
            count <- max(count, packet.payload.as_i32)
    Wait for the final_timeout.  If a packet is received on the channel and len(packet.payload) == 4:
        count <- max(count, packet.payload.as_i32)
    percent = count/send_count
    if first:
        initial_percent <- percent
        first <- false
    if percent < initial_percent and initial_percent-percent > 0.1:
        break
    else if prev_count != 0:
        prev_percent = prev_count/sent_count
        if percent < prev_percent and percent-prev_percent > 0.05:
            break
    final_mtu <- mtu
Update the estimated MTU with final_mtu
```

The responder is simpler.
When the responder sees a packet whose payload is the single byte 0, it must immediately set an internal counter to 0 and respond with a packet whose payload is the single byte 1 on the channel.
For all other packets, it must increment the count and respond with a packet whose payload is the count as a 4-byte integer.

##Message Channel Packet Format##

The following subsections describe the packet format for the message channels.
For the duration of this section, understand packet to refer to a packet on a message channel.
Furthermore, understand that the following packets are packed inside the wider context of a Fastnet packet as payloads destined to the specific message channel in question and that all state discussed in this section (most notably the sequence numbers and message optimization number) are channel-specific.

###Common Elements###

All packets begin with the flags byte:

bit | description
--- | -----------
0 | Reliability flag
1 | Start of message flag
2 | End of message flag
5 | First bit of Message Optimization Number
6 | Second bit of message optimization number.
7 | Final bit of message optimization number

All unused bits must be zero.

The reliability flag must be set if this packet is being sent reliably.

The start of message flag must be set if this packet is the first packet in a message.
The end of message flag must be set if this is the last packet in the message.
Both the start of message and end of message flags may be set.
This special case has defined behavior in the section on sending messages.

Bits 6, 7, and 8 are the message optimization number.
These three bits must be set to the same value for the duration of a message, but may be set to a different value for different messages.
When set to different values for different messages, they can be used to optimize the message coalescing algorithm.
An implementation is permitted to set them to zero, but it is recommended that they be treated as a 3-bit sequence number.

All packets have two sequence numbers, though the format depends on the transport's reliability.
In all cases, these must be unsigned 4-byte integers that wrap.

- The packet sequence number is incremented for every packet.  It may start at any value, but implementations are strongly encouraged to start it at 0.

- The reliable packet sequence number is incremented every time a reliable packet is sent.  The reliable packet sequence number must start at zero.

###For Reliable Transports###

For a reliable transport, the packet format must be simply the above flags byte followed by the packet's payload.
The two sequence numbers must be inferred and filled in by the receiver's implementation of the protocol.

###For Unreliable Transports###

There are two variants for unreliable transports:

####If the packet is Sent Unreliably####

In this case, the packet must consist of the flags byte, the packet sequence number, the reliable packet sequence number, and the packet's payload.

####If the Packet is Sent Reliably####

In this case, the packet must consist of the flags byte, the packet sequence number, the reliable packet sequence number, optionally a CRC32C checksum of the entire packet, and the payload.

To compute the checksum of the packet, an implementation must encode the packet, set the checksum field to 0, and compute the CRC32C of the entire packet.
The checksum must be included if and only if the transport is corruptible; if the transport is incorruptible, a receiver must not expect to receive the checksum.

This specification chooses CRC32C because X86 and ARM contain special instructions for its computation.

##Sending and Receiving Packets on Message Channels##

Once again, this section discusses packets in the context of a specific message channel; nothing here is binding on a non-message channel and all packets must be sent inside the wider context of a Fastnet packet.

###Sending packets###

Every message channel has a packet sequence number and reliable packet sequence number.
Furthermore, every packet is either sent reliably or unreliably.

To send a packet unreliably, a sender must:

- Pack the packet according to the above section on packet formats.  The sequence number and reliable sequence number are the sequence numbers from the channel's current state.

- Increment the packet sequence number using wrapping arithmetic.

- Submit the packet to the transport.

- Forget the packet exists.

To send a packet reliably, an implementation must:

- Pack the packet as though it is being sent unreliably.

- Increment both the packet sequence number and reliable packet sequence number using wrapping arithmetic.

- Set the reliability flag.

- Submit the packet to the transport.

If the transport is reliable, the sender must forget about packets being sent reliably because the transport will ensure delivery.
If the transport is not reliable, the sender must hold on to the packet and await an acknowledgement (hereafter ack).
The resending logic shall now be described.

####Acks and Resending####

The following section does not apply to reliable transports and must not be implemented for them.

Channel -5 is the ack channel.
Packets on the ack channel consist of any number of 6-byte acks packed without padding.
An individual ack consists of the 2-byte channel identifier and the 4-byte sequence number of the packet being acknowledged.

When a sender sends a reliable packet, it must hold onto a copy of the packet until such time as the receiver sends an ack on the ack channel which decodes to the packet or the connection is broken.
Furthermore, the sender must periodically resend the packet on an interval (the resend interval) computed as follows: `(n+0.5)*r` where `n` is the number of the retry attempt and `r` is the estimated round-trip time.
If resending fails 10 or more times, the implementation is permitted to consider the connection broken.

###Receiving Packets###

Receivers have a value called the ignore number.
The ignore number's advancement strategy is described in the section on receiving messages.
The purpose of the ignore number is to provide implementations with a method of ignoring packets which are being resent due to lost acks.
It also serves to allow messages to be lost completely without causing slowdown.

To receive a packet unreliably, a receiver must:

- Infer a packet sequence number and reliable packet sequence number, if required due to the transport being reliable.

- If the packet's sequence number comes before the ignore number, ignore the packet.

- Record the time at which the packet was received with at least millisecond precision.  This is used in later portions of this specification.

- Insert the packet into a data structure that orders packets by packet sequence number using the comes-before relationship.

To receive a packet reliably, an implementation must:

- Infer a packet sequence number and reliable packet sequence number, if required.

- If the packet's sequence number comes-before the ignore number, send an ack and otherwise ignore the packet.  We have already received this packet and the ack was lost.

- Verify that the packet's checksum matches the packet's contents.  If it doesn't, ignore the packet.

- Send an ack to the server.

- Insert the packet into the aforementioned data structure.

Updating the reliable packet sequence number is discussed in the message receiving section.

Note that it is technically possible for the reliability flag to be corrupted.
if an implementation receives an uncorrupted reliable packet with the same sequence number as an unreliable packet, it must replace the unreliable packet with the reliable one.
Since unreliable packets are obviously unreliable and since the circumstance in which the packet is corrupted in such a manner as to set the reliability flag and produce a CRC32C checksum that matches the packet is astronomically rare, this specification assumes that the corruption check for reliable packets will weed out such packets.

The comes-before relation is defined in [RFC 1982](https://tools.ietf.org/html/rfc1982).
It is notably undefined if two packets are separated by more than `2^32`.
In this protocol, this equates to at least 4 GB of consecutively lost data per channel.
This specification leaves the behavior of this case undefined.

The data structure for packets should provide quick iteration over the contents, as the message receiving algorithm must group packets by looking ahead.
A tree is acceptible for all defined cases of the comes-before relation, but will crash or produce gibberish (possibly without the ability to recover) in the one undefined case above.
If an implementation fails to extract a message within a specified time, it is suggested that all unreliable packets be cleared from the container: as described below, reliable packets cannot invoke the undefined case of the comes-before relation unless at least a 4 GB message is sent.

##Messages##

A message refers to the minimal amount of content with meaning on a message channel.
Messages must have a payload, a reliability flag, and checksum.
This section describes the encoding and sending of messages.

###Message Encoding and Checksum###

A message's encoding consists of the message's content followed optionally by the checksum.

The checksum must be computed as one of the following cases:

- If the message's content is less than 16 bytes in length and the first and last bit of the messages content are different, , the checksum consists of the message's content repeated twice.

- If the message's content is less than 16 bytes in length and the first and last bit of the message's content are the same, the checksum consists of the message's content with all bits flipped.  Algorithmically, this means notting each byte of the message's content.

- Otherwise, the checksum consists of the md5 of the message's content.

The checksum must not be included if the message is being sent reliably or if the transport is incorruptible.
In all other cases, the checksum must be included.

###Sending a Message Unreliably###

to send a message unreliably, an implementation must split the message into packets which are no longer than the payload MTU.
Specifically, this means `payload_mtu-9` byte segments (1 for flags, 4 for the packet sequence number, and 4 for the reliable packet sequence number).
These packets must then be sent on the specified message channel with consecutive sequence numbers and forgotten.
Implementations should set the 3-bit message optimization number to a unique value for the entire duration of the message; if an implementation opts not to, it must set the 3-bit message optimization number to zero.

###Sending a Message Reliably###

To send a message reliably, the implementation must split the packet into chunks no longer than the payload MTU.
For reliable packets, this is `payload_mtu-13` (1 byte for flags, 4 for the packet sequence number, 4 for the reliable packet sequence number, and 4 for the reliable packet checksum).
These segments must then be sent to the receiver in reliable packets.
As with unreliable messages, an implementation should set the message optimization number to a unique value for the duration of the message; if it opts not to, the message optimization number must be set to zero.

An implementation shall never send more than one reliable message on a channel at a time.
If a second reliable message is to be sent on the same channel, the implementation must wait for the previous reliable message to be received and fully acked.
Failure to do so will cause bugs at best: part of the message receiving algorithm, described below, relies on there being only one reliable message in progress per channel.

###Receiving Messages###

There is an entity called the message coalescing group (MCG).
There is a time called the message coalescing delay (MCD).
The MCD must be settable by the user of the implementation with at least millisecond precision and must default to 25 MS.
Finally, there is an entity called the message coalescing group modification time (MCGMT).

The following algorithm must be executed on a per-message-channel basis every time an uncorrupted packet with the end of message flag is received.

The MCG is computed by looking at the container of messages for the channel in question.
The following description is only conseptual.
An implementation is encouraged to use a more efficient algorithm.

To compute the MCG:

- Start with all packets in the packet container for the channel in question.

- Move the end of the MCG to the first packet with the end of message flag set, if any.

- if the first packet in the MCG is not the first packet in the MCG with the start of message flag set, move the end of the MCG to the packet immediately before the first packet with the start of message flag set.

- Finally, consider the message optimization numbers.  Move the end of the MCG to the first packet with a different message optimization number than that of the packet at the beginning of the MCG.

The MCGMT must be computed as the maximum of the reception times of the packets in the MCG.

At this point, the MCG consists of all packets which may be in one message.
The MCG is now in one of three states: aborted, completed, or expired.
AN implementation must choose the first of the following conditions which applies:

- First, if the first packet of the MCG was sent unreliably and the reliable packet sequence number in the first packet differs from that found in the last reliable packet received on this channle, the MCG is aborted.

- Next, if the first packet in the MCG has the start of message flag set, the last packet in the MCG has the end of message flag set, and the sequence numbers of all packets in the MCG are consecutive, then the MCG is completed.

- Next, if the MCGMT is less than MCD seconds ago or the first packet in the MCG was sent reliably, the MCG is aborted.

- Otherwise, the MCG is expired.

What happens next depends on the state of the MCG.

If the MCG is aborted, this algorithm stops until the next time a packet with the end of message flag set is received.

If the MCG is completed, the message in the MCG is a complete message.
it shall be extracted and checked for corruption if applicable (the checksum is only sent if the message is being sent unreliably on a corruptible transport; reliable messages and incorruptible transports handle this at a lower level).
if the message is not corrupted, it must be delivered  to the application immediately.

If the MCG is expired, all packets in the MCG are dropped immediately.

Finally, if the MCG was completed or expired, the ignore number must be set to one more than the packet sequence number of the last packet in the MCG mod  `2^32`.

The reason for the one reliable message at a time constraint is due to the updating of the ignore number.
We need a way to ignore duplicate packets or packets that may be being resent due to lost acks from the receiver.
To this end and described above, we ignore packets which come-before the ignore number by the relation defined in RFC 1982.
Unfortunately, this means that if we send two reliable messages and the second arrives first, the ignore number will now ignore all packets from the first message.
This would violate the guarantee of reliability, even though acks were properly received.
It would also be possible to fix this by adding an additional 4-byte value to the header of the packet, but the point of this library is to optimize for unreliable message sending and multiple streams; applications that need to use one efficient , reliable stream should opt in to TCP.

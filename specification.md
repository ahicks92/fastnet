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
The TCP transport must be reliable, connection-based, and not punch-capable.
The UDp transport must be unordered, unreliable, connectionless, and punch-capable.
Over IPV4, UDP's built-in checksum is optional.
To this end, UDP must also be corruptible.

Every transport must advertise the maximum packet size it supports in a way that allows the user of the implementation to obtain the value.
Every transport must support a maximum packet size of at least 4096 bytes.

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
Query packets on channel -1 are ascii literals.
The following query operations must be supported:

- If a client sends the string "query:fastnet" as a packet payload on channel -1, the server must respond with "query:fastnet=yes"

- If the client sends "query:version", the server must respond with "query:version=1.0"

An implementation shall not place limits on the number of times that a query may be requested from the server.
An implementation must continue to respond to queries even after a connection is established.

##Connection Establishment##

Connections are established from client to server with the exception of UDP hole punching, described later in this specification and using a different algorithm from that described here.

A Fastnet server must allow only one cobnnection from a specific IP and port.

Before beginning connection establishment, an implementation must use the above query interface to establish that Fastnet is listening in an implementation-defined manner.
This specification suggests that this be done by sending "query:fastnet" in a similar manner to the following connection handshake algorithm and looking for "query:fastnet=yes".

An implementation must not consider the connection of a connection-based transport to be the establishment of a fastnet connection.

The following must always take place on channel -1 before a connection is considered established.

NOTE: because reliable transports are ordered, the following algorithm cannot cause data loss on a reliable transport even though the client intensionally drops packets.

There are three packets involved in the connection handshake protocol, all of which are sent on channel -1:

- The connection request packet consists of the string "connect?"

- The connect packet consists of the string "connected" followed by a 4-byte integer.  This integer is the connection's identifier.

- The abort packet is the string "abort" followed immediately by a UTF8-encodedd error string without a trailing null.

To begin a connection, a client must:

- Send the connect request packet.

- begin waiting for either the connect packet or the abort packet with a timeout of 5000 MS.  If the transport is unreliable, the client must resend the connection request packet every 200 MS during this process; otherwise, it must not.

If the client receives the connect packet, it must parse the connection id, notify the application that the connection has been established, and begin processing packets.
The client must disregard all other packets until it manages to receive the connect packet.

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

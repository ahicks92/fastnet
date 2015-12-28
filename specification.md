#Fastnet 1.0 Protocol Specification

##0. Introduction and Goals

Fastnet is a connection-based, channel-based protocol intended for games and realtime applications.
It intensionally forgoes much of the automatic management of TCP and assumes that settings will be provided by the application developer.

Unlike TCP, Fastnet works at the level of a message: a chunk of binary data of any size.
Messages are either fully received or never received at all, and the application need not provide further separation logic.
Fastnet connections are divided into parallel channels.
One Fastnet connection is logically 32768 smaller multiplexed connections.

Messages can be sent reliably or unreliably.
Unreliable messages may or may not arrive at the other end, but are extremely fast and capable of representing frequently updated data such as position.
Reliable messages are intended for chat messages, status updates, or other information that absolutely must arrive.
A reliable message stalls all other messages in the queue behind it until such time as it arrives on the other end.

You can think of TCP as a train.
If one train car derails, everything behind it stops.
The primary advantage of Fastnet, then, is that you have the ability to use multiple train tracks at once.
Furthermore, for applications which can tolerate data loss, it is possible to mark some of the metaphorical cars as unimportant;
if these unimportant cars derail, the stream continues regardless.

Other semireliable UDP-based protocols exist, but usually only for one language.
Fastnet aims to be fully documented to the point where it can be recoded from scratch.
The other two features that Fastnet aims to support are the ability to fall back to other transports (TCP, HTTP, WebRTC) and the ability to support UDP hole-punching.
In my experience, it is difficult to find networking libraries that offer all three of these benefits.

Each section of this documentation provides a specification and a justification explaining why I made the choices I did.
This protocol is being developed as a learning project, and I expect this specification to change many times before finalization.

##1. Packets and The Transport##

###Specification###

A packet is a chunk of data with any content of no more than 8192 bytes.

A transport is a mechanism for the delivery of packets.
Examples of transports include TCP and UDP.
If the underlying mechanism powering a transport does not work directly with packets, it is the job of the transport to abstract it.
As a concrete example, TCP would require length prefixes.

A transport is either ordered or unordered.
An ordered transport will deliver all packets in order; an unordered transport will not.

A transport is either reliable or unreliable.
A reliable transport will deliver all packets; an unreliable transport will not.
Being reliable implies being ordered.

A transport is either incorruptible or corruptible.
Incorruptible transports guarantee that all data which arrives is not corrupted.
Note that UDP is actually corruptible.

A transport is either connectionless or connection-based.
Connectionless transports require virtual connections, and can send to any destination.
UDP is a connectionless transport.

###Justification###

Transports are tricky because UDP can be blocked, i.e. on universiity campuses.
Allowing for the ability to swap out the underlying transport allows for applications to provide fallback paths.
Unfortunately, this protocol is optimized for corruptible, connectionless, unreliable, and unordered transports.
Allowing the ability to turn off protocol features can increase efficiency, and the actual runtime cost (is this transport reliable? Fine, drop the flag and don't send the ack) is usually only a few if statements.

##Channels##

###Specification###

All packets begin with a 2-byte integer specifying the channel.
Channels 0 to 32767 are for the application.
Channels -32768 to -1 are reserved for Fastnet usage.
This channel specification is then followed by the packet payload of any length.

###Justification###

Not much to say here.
32768 unique channels for any purpose should be sufficient for application developers, and by over-reserving we ensure that we have enough headroom for future growth or extensions to the protocol, i.e. file transfers.

##Fastnet Status Queries##

###Specification###

The following must be supported even for connection-based transports.

A server supporting Fastnet shall support the following query operations on channel -1.
These operations must be supported no matter whether a connection is established or not.
No limit is placed on the number of times each query can be made.

- If a client connects to an sends a packet on channel -1 containing the string "fastnet?" the server shall respond with a packet on channel -1 containing the string "fastnet."
This confirms that a Fastnet server is listening.

- If a client sends the string "version?" on channel -1, the server shall respond with the Fastnet protocol version that it supports in the format "version=1.0"

###Justification###

This makes Fastnet implementations prone to UDP DDOS: simply spam the queries.
Fortunately, UDP DDOS can be prevented with firewalls such as iptables, so this specification makes no effort to deal with it.

##Establishing a Connection##

###Specification###

The following sequences must be implemented even for connection-based transports.

In order to connect, a client:

- If the underlying transport requires it, connects to the server.

- Issues a "fastnet?" query up to 20 times, separating attempts by 200 MS.  If no "fastnet." response is received by 1000 MS after the last attempt, connection fails.  If the transport is reliable, the query is sent only once and the program waits 5000 MS before failing the connection attempt.

- Allocates all structures required to deal with incoming data.

- Issues the packet "connect?" on channel -1 up to 20 times, separating attempts by 200 MS.  The client is looking for the packet "connected." on channel -1.  If this packet is not seen by 1000 MS after the final "connect?" send, connection fails.  As with the initial fastnet query, reliable transports send only once and then wait 5000 MS.  Note that other data may arrive before the "connected." packet.  If this occurs, said data is ignored.

- begins accepting data.

The server is simpler:

- Upon the reception of a "connect?" packet on channel -1 from a client, record the client as connected and send the "connected." response.

- Notify the program and begin sending data.  Unreliable messages will be dropped, but reliable messages will be recovered from by the machinery for sending reliable messages.

- On every subsequent "connect?" request from the client in question, respond with "connected." as usual but do nothing further.

###Justification###

if it takes more than 5000 MS to connect, the connection is likely to be way too slow for any sort of game whatsoever, and we are possibly being blocked by a firewall.
This specification will shortly discuss reliable packets and acking, but hasn't yet; suffice it to say that dropping data at this stage is actually not a loss.
Since the server sends the "connected." packet first and since we specify that reliable transports are ordered transports, a client won't get packets until after it has responded to the "connected." packet and no data will be lost.
As far as I know, there are no unordered but reliable transports of actual interest.

##Heartbeats and round-trip estimation##

###Specification###

Channel -2 is the heartbeat channel.
A heartbeat is a random 4-byte integer.
The client uses positive heartbeat values and the server uses negative heartbeat values.
When the client sees a negative heartbeat value, it must echo said value unmodified; when the server sees a positive heartbeat value, it must likewise echo it unmodified.

For connectionless transports, both ends of the connection must send a packet at least twice a second.
More often is permitted.
If the transport is connectionless and one end of the connection fails to receive a heartbeat for a user-defined timeout, the connection is considered broken and the application is notified.

The initial round-trip time is assumed to be 200 MS.
Round-trip estimation must occur at least once at the formation of the initial connection, but should continue to occur periodically thereafter.
This specification does not mandate how often this algorithm should occur, but suggests once a minute as a default.
The round-trip estimation algorithm proceeds as follows:

- Pick a random heartbeat value and record the starting time.  Send this value every 200 MS until such time as the heartbeat is echoed successfully.

-  Record the time it took to see the heartbeat again.

- Repeat the above two steps 20 times.

- Take an average of the 20 recordings.  This is the new round-trip time.

###Justification###

For UDP, this keeps the NAT from closing on us and lets us know if the connection is going to need to die.
It is anticipated that typical timeout times will be on the order of one or two seconds, but different applications have different needs.

We need the round-trip time to determine when we should resend reliable messages, specified momentarily.
While Fastnet tries to not take on all of TCP, we do need some basic congestion control for reliable messages, and some idea of how long it takes a packet to get lost.
Allowing the round-trip algorithm's periodicity to be configurable serves to save on bandwidth, though it does need to run periodically and can't be turned off for good performance.
it would also be possible for us to allow the heartbeat interval used for round-trip estimation to be configured, but this spec chooses to be conservative for now.

##Message Format##

###Specification###

Channels 0 through 32767 are for sending messages of arbetrary size.  The following is the format of a message and a description of how to split it into packets.

A message consists of the following. The wire format is described momentarily.

- An 8-bit flags field.  See below.

- A 16-bit sequence number as an unsigned integer.

- The content of the message.

- If the transport is corruptible, a checksum of the message's content.  This is MD5 for all messages over 16 bytes.  Otherwise, it is the content of the message reversed and ones complemented.

Messages use the following flags:

- Bit 0 is whether or not the message is reliable.

- Bit 1 should be set when the sender's sequence number has just wrapped and should remain set for the first    1000 increments.

A message is split into packets by splitting the content and the checksum into the packet size.  Determining the packet size is described below, but the default and minimum size is 32.
These packets then have three pieces of information prepended:

- A packet-specific flag field of 1 byte.  At the moment, only bit 0 is used. Bit 0  is set if this is the final packet in the message.

- The message's flags field.

- The message's sequence number.

- The position in the message, an unsigned 32-bit integer and encoded as a [variable-length quantity](https://en.wikipedia.org/wiki/Variable-length_quantity).  We call this the message's position number.

###Justification###

UDP packet size is really important for some people; I have at least one test case where delivery drops from 97% at 4 bytes to around 75% at 300 bytes.
For this reason, we use the odd checksum algorithm and encode the packet's position as a variable-length quantity.
This pushes the size of a 1-byte message down by 19 bytes (165 from MD5, 4 from the sequence number).

There are a variety of ways to determine if the sequence number has wrapped, but we choose to use a flag.
if the client somehow misses a few thousand messages, the client has probably already disconnected.
Should this prove problematic, we can raise it later.

The effective size limit on messages is 32 GB.
This is large enough that we might as well call it unlimited, as most consumer-grade machines will have run out of ram while assembling or decoding it.

We prepend a completeness flag to the message's packets so that large messages don't need to use additional bytes to specify the maximum sequence number.
By looking for the completeness flag, we can infer the maximum sequence number without sending it.

##Sending and Receiving Unreliable Messages##

###Specification###

To send an unreliable message, split it into packets as above and send the packets as fast as possible, one after the next.

For reception, use the following algorithm.

First, if the sequence number on the packet comes-before the sequence number of the last delivered message, ignore the packet.

Otherwise, begin recording packets until either an application configurable period has passed (15 MS by default) or all packets are received.

Finally, decide whether or not to deliver the message.
If the message timed out or the checksum verification fails, the message is not delivered; otherwise it is.

###Justification###

The interesting thing here is the delay, and it exists because messages can be out of order.
We need to put some sort of delay in or nothing will ever be received.
I conceptually think of Fastnet as being implemented in a background thread and communicating via queues, but other implementations are possible.

The one disadvantage of our message sending algorithm is that it can't construct messages in-place.
Fastnet makes the opinionated decision that network is more valuable than ram, however,  and so we don't send the lengths with every packet.

##Sending Reliable Messages##

###Specification###

Channel -3 is the ack channel.

Ack packets consist of:

- The channel for which the ack is acking.

- The message's sequence number.

- The messages' position number.

These are in the same formats as used for messages themselves.

When the server begins sending a reliable message, any further messages on the same channel are to be queued and processed in batch after the reliable message has been sent.
The server is to send the packets as though this were an unreliable message, but to hold on to them until it receives acks on the ack channel.

When the client receives a packet for a reliable message, it is to immediately send exactly one ack on the ack channel.

After the initial message, the server should begin resending any unacked packets.
These resends should occur using the output of the fibinacci sequence times the currently estimated round-trip time: `1r, 1r, 2r, 3r, 5r, 8r, 13r...` where r is round-trip as specified above.

###Justification###

By holding unreliable packets on the server and sending them in bulk later, we make the client simpler and allow for congestion control.
#Fastnet#

Fastnet aims to be a primaerily UDP-based internet protocol optimized for games, media, and interactive applications.
This repository will include a specification and a reference implementation in rust.

Unlike my other projects, Fastnet is primarily aimed at teaching me things (namely rust and the design of reliable protocols over unreliable transports).
When finished, Fastnet will offer a message-based protocol with the ability to send multiple streams of data in parallel on the same connection.
The advantages are intended to be those of [enet](http://enet.bespin.org/) with a  few additions.
To summarize:

- A client connects to a server, in a bidirectional manner.

- Both ends of the connection may send messages of any reasonable size to the other end.

- Messages can be assigned to channels.  All messages on the same channel arrive in order, but ordering between different channels provides no guarantees.

- Some messages may be flagged as reliable.  Reliable messages are always delivered, or the connection drops.

- If messages arrive, they are extremely unlikely to be corrupted.

- If UDP is blocked, falling back to TCP is possible.  While optimized for UDP, Fastnet aims to work over any transport.

- If UDP hole-punching is required, fastnet can take over a socket after establishment.  Connections will be kept alive with heartbeats.

- Fastnet will dynamically determine the best packet size to use for messages.  For some users, packet size drastically effects reliability.

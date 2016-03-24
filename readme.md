#Fastnet#

This is the rust reference implementation of [Fastnet](http://github.com/camlorn/fastnet).
Fastnet aims to be a UDP-based protocol for games, supporting multiplexing, both reliable and unreliable messages, NAT hole-punching, and resource downloading.
Both the protocol specification and this repository are definitively works in progress.

In addition to being the reference implementation, this crate intends to expose a C API for accessing the protocol.  The intent is that this will allow binding from other languages such as Python.

License is [MPL 2](license).
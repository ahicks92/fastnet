#Fastnet#

Fastnet aims to be a multiplexed, message-based protocol over UDP.
It splits a physical connection into 32768 streams, each of which guarantees in-order delivery of messages.
Every message may be sent either reliably or unreliably.
In addition, Fastnet aims to support P2P connections via UDP hole punching, the ability to efficiently distribute large assets and other data to clients, and efficient TCP fallback.
Finally, Fastnet will be fully specified.

This repository includes the specificationn.
When complete, a reference implementation will be implemented in Rust.
For ease of maintenance, the Rust implementation of this protocol will live in an as-of-yet uncreated repository separate from this one.
When the repository exists, it will be linked here.

Unlike my other projects, Fastnet is primarily aimed at teaching me things (namely rust and the design of reliable protocols over unreliable transports).
I anticipate that Fastnet will be useful, but other libraries already implement much of this functionality.
That said and in so far as I know, you have to go to C++ or one of the big game engine/distribution platforms to find everything here with a convenient interface.

The biggest advantage of this protocol, however, is that it is fully and formally specified.
Consequently, others may learn from and maintain it without having to read the code from top to bottom.
A not insignificant part of my motivation for this project is the documentation: finding good resources on the designs of such protocols as offered here is surprisingly difficult, and source code for libraries providing them is often large, cumbersome, and uncommented.
Furthermore, having a specification makes it easier for third parties to identify possible security issues and evaluate Fastnet as compared to other alternatives.
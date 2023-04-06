# Rust Asynchronous Reliable UDP

This library provides a [reliable UDP](https://en.wikipedia.org/wiki/Reliable_User_Datagram_Protocol) implementation, based on the [Rust tokio UdpSocket](https://docs.rs/tokio/latest/tokio/net/struct.UdpSocket.html).  

Within a fixed window size (defined as some consecutively sent UDP packets, and they must all be
acknowledged before sending packets in next window), this library will do:
1. Acknowledgment of received packets
2. Retransmission of lost packets
3. Reorder the received UDP packets
4. Message fragmentation

An *UdpReliable* socket represents a connected, [one-to-one](https://docs.rs/tokio/latest/tokio/net/struct.UdpSocket.html#example-one-to-one-connect) UDP socket, 
this library does not support one-to-many UDP socket, broadcast and multicast, for the purpose of simplicity.

An *UdpReliable* socket has client and server concept, server side will listen on a *UdpSocket* socket, and create a new *UdpReliable* socket for each client.  
An *UdpSocket* will be split as a *Sender* and a *Receiver* object at both client and server side. Within each window, the *Sender* and the *Receiver* should hold a mutex to protect its state.  

As for message fragmentation, when the message exceeds the UDP maximum packet size, it will be fragmented at *Sender* end, and reassembled at *Receiver* end.  
With this capability, *Sender* can call *send()* to send a [stream](https://grpc.io/docs/what-is-grpc/core-concepts/#bidirectional-streaming-rpc) of user defined, variable sized messages, the *Receiver* will receive these message in original order, one *recv()* call to get a message sent from a *send()* call.   
The order of the messages at sender side will be preserved at receiver end.   

The basic flow of using this *UdpReliable* socket at client side is:
1. Create a socket, tell the library what's the window size. The window size can be 1, which means for each outgoing packet, you are
   expecting an acknowledgment before sending next packet, the maximum value is 32, the sender will block if no acknowledgment
   (or implied retransmission request) received.
2. Do connect. This is not a TCP connect, it means exactly the same as the one in [std::net::UdpSocket](https://doc.rust-lang.org/std/net/struct.UdpSocket.html#method.connect). 
3. Transmit a series of UDP packets, with mutex holds on *Sender* and/or *Receiver*:
    1. From *Sender* object:
        1. Send first packet, with start_of_stream flag on.
        2. Send other packets, with the start_of_stream and the end_of_stream flag off.
        3. Send last packet, with end_of_stream flag on.
    2. If client is expecting responses, then from *Receiver* object, call *recv()* on the socket, loop until last *recv()* returned the end_of_stream true.

What's the difference compared to a TCP socket?
1. No connection maintenance on the wire.
2. Maintains message boundaries.
3. No explicit flow control and congestion control: if receiver can not ack for last window, sender will not send next window.

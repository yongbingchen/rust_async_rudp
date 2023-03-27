# Rust Asynchronous Reliable UDP

This library provides a [reliable UDP](https://en.wikipedia.org/wiki/Reliable_User_Datagram_Protocol) implementation, based on the [Rust tokio UdpSocket](https://docs.rs/tokio/latest/tokio/net/struct.UdpSocket.html).  

Within a fixed window size (defined as some consecutively sent UDP packets, and they must all be
acknowledged before sending packets in next window), this library will do:
1. Acknowledgment of received packets
2. Retransmission of lost packets
3. Reorder the received UDP packets (optional)

An *UdpReliable* socket represents a connected, [one-to-one](https://docs.rs/tokio/latest/tokio/net/struct.UdpSocket.html#example-one-to-one-connect) UDP socket, 
this library does not support one-to-many UDP socket, broadcast and multicast, for the purpose of simplicity.

This socket does not do fragmentation, so if an outgoing packet exceeds the maximum size for an UDP datagram, it will still fail: this is for keeping this library design to be compact and atomic, you can add another wrapper layer for that functionality if needed.  

This socket does not really build and maintain a TCP style connection: its connect() method means
exactly the same as the one in [std::net::UdpSocket](https://doc.rust-lang.org/std/net/struct.UdpSocket.html#method.connect).  

But beyond underlying pure datagram protocol, the sender and receiver has an additional protocol for maintaining the active
window for the retransmission mechanism: the sender need to define the window size (can not change
after the socket is created), the start of a stream (defined as a session in which all the
retransmission mechanism applies, they are still datagram packets, each send() and recv() keeps
original message boundary, the total number of packets in a stream can be larger or smaller than the window size), and the end of the stream.  

Like [tokio::net::UdpSocket](https://docs.rs/tokio/latest/tokio/net/struct.UdpSocket.html), you do not need a Mutex to share the UdpSocket — an Arc<UdpSocket> is enough.  

The basic flow of using this UdpReliable socket is:
1. Create a socket, tell the library what's the window size, and if you want the received packets
   to be reordered or not. The window size can be 1, which means for each outgoing packet, you are
   expecting an acknowledgment before sending next packet, the maximum value is 32, the sender will block if no acknowledgment
   (or implied retransmission request) received.
2. Do connect.
3. Transmit a stream of UDP packets:
    1. On sender side:
        1. Send first packet, with start_of_stream flag on.
        2. Send other packets, with the start_of_stream and the end_of_stream flag off.
        3. Send last packet, with end_of_stream flag on.
    2. On receiver side, Call recv() on the socket, loop until last recv() returned the end_of_stream true.

Here's a full example, you can compare it to the [example](https://docs.rs/tokio/latest/tokio/net/struct.UdpSocket.html#example-one-to-one-connect) in *tokio::net::UdpSocket* for the difference:
```
use async_rudp::UdpReliable;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reordered = true;
    const WINDOW_SIZE: usize = 32;
    let sock = UdpReliable::new("0.0.0.0:8080", WINDOW_SIZE, reordered).await?;

    let remote_addr = "127.0.0.1:59611";
    sock.connect(remote_addr).await?;

    let sock = Arc::new(sock);
    let send_socket = Arc::clone(&sock);
    let sender = tokio::spawn(async move {
        let send_socket = send_socket.as_ref();
        let buffers = [[0; 1024]; 128];
        for i in 0..128 {
            let sos = i == 0;
            let eos = i == 127;
            let len = send_socket.send(&buffers[i], sos, eos).await;
            println!("{:?} bytes sent", len);
        }
    });

    let recv_socket = Arc::clone(&sock);
    let receiver = tokio::spawn(async move {
        let recv_socket = recv_socket.as_ref();
        loop {
            let mut buffer = [0; 1024];
            let (len, eos) = recv_socket.recv(&mut buffer).await.unwrap();
            println!("{:?} bytes received from {:?}", len, remote_addr);
            if eos {
                println!("A stream from sender ended");
                break;
            }
        }
    });

    let (_, _) = tokio::join!(sender, receiver);

    Ok(())
}
```

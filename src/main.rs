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

use tokio::net::ToSocketAddrs;
use tokio::net::UdpSocket;

pub struct UdpReliable {
    inner: UdpSocket,
}

impl UdpReliable {
    pub async fn new<A: ToSocketAddrs>(
        addr: A,
        window_size: usize,
        reordered: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let inner = UdpSocket::bind(addr).await?;
        Ok(UdpReliable { inner })
    }

    pub async fn connect<A: ToSocketAddrs>(
        &self,
        addr: A,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn send(
        &self,
        buf: &[u8],
        sos: bool,
        eos: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn recv(&self, buf: &mut [u8]) -> Result<(usize, bool), Box<dyn std::error::Error>> {
        Ok((1024, false))
    }
}

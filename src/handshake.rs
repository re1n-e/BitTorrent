use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::{Result, Context};

#[repr(C)]
pub struct Handshake {
    pub length: u8,
    pub bittorrent: [u8; 19],
    pub reserved: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Self {
            length: 19,
            bittorrent: *b"BitTorrent protocol",
            reserved: [0; 8],
            info_hash,
            peer_id,
        }
    }

    pub async fn handshake(&self, addr: &str) -> Result<String> {
        let mut stream = TcpStream::connect(addr)
            .await
            .with_context(|| format!("Failed to connect to {}", addr))?;

        // Serialize the handshake message and send it
        let buffer = self.serialize();
        stream.write_all(&buffer)
            .await
            .context("Failed to send handshake")?;

        // Prepare a buffer to read the handshake response
        let mut response = vec![0u8; 68];
        stream.read_exact(&mut response)
            .await
            .context("Failed to read handshake response")?;

        // Extract the peer_id from the response and convert it to hex
        let peer_id = &response[48..];
        Ok(hex::encode(peer_id))
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::with_capacity(68);
        buffer.push(self.length);
        buffer.extend(self.bittorrent);
        buffer.extend(self.reserved);
        buffer.extend(self.info_hash);
        buffer.extend(self.peer_id);
        buffer
    }
}

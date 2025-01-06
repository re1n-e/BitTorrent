use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::Result;

#[derive(Clone, Debug)]
pub struct Message {
    pub prefix: i32,
    pub id: MESSAGE,
    pub payload: Vec<u8>,
}

impl Message {
    pub fn new(id: MESSAGE, payload: Vec<u8>) -> Self {
        Self {
            prefix: (payload.len() as i32) + 1,
            id,
            payload,
        }
    }

    pub async fn send(&self, stream: &mut TcpStream) -> Result<()> {
        let data = self.to_bytes();
        stream.write_all(&data).await?;
        Ok(())
    }

    pub async fn recv(stream: &mut TcpStream) -> Result<Option<Self>> {
        let mut prefix_bytes = [0u8; 4];
        if stream.read_exact(&mut prefix_bytes).await.is_err() {
            return Ok(None);
        }
        let prefix = i32::from_be_bytes(prefix_bytes);

        let mut id_byte = [0u8; 1];
        stream.read_exact(&mut id_byte).await?;
        let id = MESSAGE::try_from(id_byte[0]).unwrap();

        let mut payload = vec![0u8; (prefix - 1) as usize];
        stream.read_exact(&mut payload).await?;

        Ok(Some(Self { prefix, id, payload }))
    }

    // Serialize the message into bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        [
            &self.prefix.to_be_bytes()[..],     
            &[self.id.clone() as u8][..],        
            &self.payload[..]                    
        ]
        .concat()  
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 5 {
            return None; 
        }
        let prefix = i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        
        // Ensure that bytes match the specified prefix length
        if bytes.len() != (prefix + 4) as usize {
            return None;
        }
        
        let id = MESSAGE::try_from(bytes[4]).ok()?;
        let payload = bytes[5..].to_vec();
        Some(Self { prefix, id, payload })
    }
    
}

#[derive(PartialEq, Clone, Debug)]
#[repr(u8)]
pub enum MESSAGE {
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,
}

use std::convert::TryFrom;

impl TryFrom<u8> for MESSAGE {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MESSAGE::Choke),
            1 => Ok(MESSAGE::Unchoke),
            2 => Ok(MESSAGE::Interested),
            3 => Ok(MESSAGE::NotInterested),
            4 => Ok(MESSAGE::Have),
            5 => Ok(MESSAGE::Bitfield),
            6 => Ok(MESSAGE::Request),
            7 => Ok(MESSAGE::Piece),
            8 => Ok(MESSAGE::Cancel),
            _ => Err("Invalid message ID"),
        }
    }
}

pub struct Bitfield {
    bits: Vec<u8>,
}

impl Bitfield {
    /// Create a new Bitfield
    pub fn new() -> Self {
        Self { bits: vec![0; 8], }
    }

    /// Check if a particular index is set in the bitfield
    pub fn has_piece(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let offset = index % 8;
        (self.bits[byte_index] >> (7 - offset)) & 1 != 0
    }

    /// Set a particular index in the bitfield
    pub fn set_piece(&mut self, index: usize) {
        let byte_index = index / 8;
        let offset = index % 8;
        self.bits[byte_index] |= 1 << (7 - offset);
    }
}

// Functions for each step
async fn wait_for_bitfield(stream: &mut TcpStream, bitfield: &mut Bitfield) -> Result<()> {
    if let Some(message) = Message::recv(stream).await? {
        if message.id == MESSAGE::Bitfield {
            bitfield.bits = message.payload.clone();
            println!("Received bitfield with available pieces.");
        }
    }
    Ok(())
}

async fn send_interested(stream: &mut TcpStream) -> Result<()> {
    let message = Message::new(MESSAGE::Interested, vec![]);
    message.send(stream).await?;
    println!("Sent Interested message.");
    Ok(())
}

async fn wait_for_unchoke(stream: &mut TcpStream) -> Result<()> {
    while let Some(message) = Message::recv(stream).await? {
        if message.id == MESSAGE::Unchoke {
            println!("Received Unchoke message.");
            break;
        }
    }
    Ok(())
}

async fn request_piece_block(stream: &mut TcpStream, index: u32, begin: u32, length: u32) -> Result<()> {
    let mut payload = vec![];
    payload.extend_from_slice(&index.to_be_bytes());
    payload.extend_from_slice(&begin.to_be_bytes());
    payload.extend_from_slice(&length.to_be_bytes());

    let message = Message::new(MESSAGE::Request, payload);
    message.send(stream).await?;
    println!("Sent Request message for piece {index}, offset {begin}.");
    Ok(())
}

async fn receive_piece_block(stream: &mut TcpStream) -> Result<Option<(u32, u32, Vec<u8>)>> {
    if let Some(message) = Message::recv(stream).await? {
        if message.id == MESSAGE::Piece {
            let index = u32::from_be_bytes([message.payload[0], message.payload[1], message.payload[2], message.payload[3]]);
            let begin = u32::from_be_bytes([message.payload[4], message.payload[5], message.payload[6], message.payload[7]]);
            let block = message.payload[8..].to_vec();
            return Ok(Some((index, begin, block)));
        }
    }
    Ok(None)
}

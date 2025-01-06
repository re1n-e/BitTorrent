use serde::{Deserialize, Serialize};
use serde_bencode;
use serde_bytes::ByteBuf;
use anyhow::{Result, bail};
use sha1::{Sha1, Digest};
use serde_bencode::to_bytes;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Torrent {
    pub announce: String,
    pub info: Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    #[serde(skip)]
    pub pieces: Vec<String>,  // Change pieces to Vec<String> for SHA1 hashes
    #[serde(flatten)]
    pub keys: Keys, 
    #[serde(rename = "pieces")]
    raw_pieces: ByteBuf, // Add this to store raw pieces data for parsing
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Keys {
    SingleFile {
        length: usize,
    },
    MultiFile { files: Vec<File> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub length: usize,
    pub path: Vec<String>,
}

impl Info {
    /// Converts raw `pieces` from ByteBuf to a vector of 20-byte SHA-1 hash strings.
    fn process_pieces(&mut self) -> Result<()> {
        let pieces_data = &self.raw_pieces;
        if pieces_data.len() % 20 != 0 {
            bail!("Invalid pieces length, should be multiple of 20 bytes");
        }

        // Create 20-byte chunks and convert each to hex string
        self.pieces = pieces_data.chunks(20)
            .map(|chunk| hex::encode(chunk))
            .collect();
        Ok(())
    }
}

pub fn parse_torrent_file(dot_torrent: Vec<u8>) -> Result<Torrent> {
    let mut torrent: Torrent = serde_bencode::from_bytes(&dot_torrent)?;
    torrent.info.process_pieces()?;  // Process pieces after parsing
    Ok(torrent)
}

pub fn info_hash(info: &Info) -> Result<String> {
    let mut hasher = Sha1::new();
    let info_encoded = serde_bencode::to_bytes(&info)?;
    hasher.update(info_encoded);
    let info_hash = hasher.finalize();
    Ok(hex::encode(info_hash))
} 

pub fn info_hash_vec(info: &Info) -> Result<[u8; 20]> {
    let mut hasher = Sha1::new();
    
    let info_encoded = to_bytes(info)?;
    hasher.update(info_encoded);
    
    let info_hash = hasher.finalize();
    let info_hash_array: [u8; 20] = info_hash
        .as_slice()
        .try_into()
        .expect("SHA-1 hash should be exactly 20 bytes");
    Ok(info_hash_array)
}

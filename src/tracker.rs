use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use anyhow::{Result, Context, bail};
use reqwest;

#[derive(Debug, Deserialize, Serialize)]
pub struct Tracker {
    peer_id: String,
    port: u16,
    uploaded: usize,
    downloaded: usize,
    left: usize,
    compact: u32,
}

impl Tracker {
    pub fn new(peer_id: String, left: usize) -> Self {
        Tracker {
            peer_id,
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left,
            compact: 1,
        }
    }

    fn serialize(&self) -> Result<String> {
        Ok(serde_urlencoded::to_string(self)?)
    }

    pub fn url(&self, announce: &str, info_hash: &str) -> String {
        let tracker_serialize = self.serialize().unwrap();
        let encoded_hash = optimized_url_encode(info_hash);
        format!("{}?{}&info_hash={}", announce, tracker_serialize, encoded_hash)
    }
    
    pub async fn request(&self, announce: &str, info_hash: &str) -> Result<Vec<String>> {
        let tracker_url = self.url(announce, info_hash);
        let response = reqwest::get(&tracker_url).await.context("query tracker")?;
        let response_bytes = response.bytes().await.context("fetch tracker response")?;
        let mut tracker_response: TrackerResponse =
            serde_bencode::from_bytes(&response_bytes).context("parse tracker response")?;
    
        if !tracker_response.peers.is_empty() {
            tracker_response.process_pieces().unwrap();
            for peer in &tracker_response.peer_vec {
                println!("{peer}");
            }
        } else {
            eprintln!("No peer information found.");
        }
        
        Ok(tracker_response.peer_vec)
    }
}

fn optimized_url_encode(info_hash: &str) -> String {
    let mut encoded = String::new();
    
    for i in (0..info_hash.len()).step_by(2) {
        let byte_hex = &info_hash[i..i + 2];
        let byte = u8::from_str_radix(byte_hex, 16).expect("Invalid hex character");

        if (b'0'..=b'9').contains(&byte) || (b'A'..=b'Z').contains(&byte)
            || (b'a'..=b'z').contains(&byte) || byte == b'-' || byte == b'_'
            || byte == b'.' || byte == b'~'
        {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{:02x}", byte));
        }
    }
    encoded
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackerResponse {
    pub interval: u64,
    pub complete: u64,
    pub incomplete: u64,
    #[serde(skip)]
    pub peer_vec: Vec<String>,
    peers: ByteBuf,
}

impl TrackerResponse {
    fn process_pieces(&mut self) -> Result<()> {
        let pieces_data = &self.peers;
        if pieces_data.len() % 6 != 0 {
            bail!("Invalid peer length, should be multiple of 6 bytes");
        }
        self.peer_vec.clear();
        for arr in pieces_data.chunks(6) {
            if arr.len() != 6 {
                bail!("Invalid chunk length, expected 6 bytes");
            }
            let ip = format!("{}.{}.{}.{}", arr[0], arr[1], arr[2], arr[3]);
            let port = u16::from_be_bytes([arr[4], arr[5]]);
            self.peer_vec.push(format!("{}:{}", ip, port));
        }
        Ok(())
    }
}





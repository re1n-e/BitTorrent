use std::env;
use bittorrent::decode_bencode::decode_bencoded_value;
use bittorrent::torrent::{parse_torrent_file, Keys, info_hash, info_hash_vec};
use bittorrent::tracker::Tracker;
use bittorrent::handshake::Handshake;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("No command provided.");
        return Ok(());
    }

    let command = &args[1];
    match command.as_str() {
        "decode" => {
            if args.len() < 3 {
                eprintln!("No value provided for decoding.");
                return Ok(());
            }
            let encoded_value = &args[2];
            let decoded_value = decode_bencoded_value(encoded_value);
            println!("{}", decoded_value.0);
        }
        "info" => {
            if args.len() < 3 {
                eprintln!("No .torrent file provided.");
                return Ok(());
            }
            let dot_torrent = std::fs::read(&args[2])?;
            let torrent = parse_torrent_file(dot_torrent)?;
            println!("Tracker URL: {}", torrent.announce);
            match torrent.info.keys {
                Keys::SingleFile { length } => {
                    println!("Length: {}", length);
                }
                Keys::MultiFile { ref files } => {
                    println!("Files:");
                    for file in files {
                        println!("{:?}", file);
                    }
                }
            }
            println!("Info Hash: {}", info_hash(&torrent.info)?);
            println!("Piece Length: {}", torrent.info.piece_length);
            for piece in torrent.info.pieces {
                println!("{}", piece);
            }
        }
        "peers" => {
            if args.len() < 3 {
                eprintln!("No .torrent file provided.");
                return Ok(());
            }
            let dot_torrent = std::fs::read(&args[2])?;
            let torrent = parse_torrent_file(dot_torrent)?;
            let url = &torrent.announce;
            let infohash = info_hash(&torrent.info)?;
            let tracker = Tracker::new("00112233445566778899".to_string(), 10);
            tracker.request(url, &infohash).await?;
        }
        "handshake" => {
            if args.len() < 3 {
                eprintln!("No .torrent file provided.");
                return Ok(());
            }
            if args.len() < 4 {
                eprintln!("No Ip provided.");
                return Ok(());
            }
            let dot_torrent = std::fs::read(&args[2])?;
            let torrent = parse_torrent_file(dot_torrent)?;
            let infohash = info_hash_vec(&torrent.info).unwrap();
            let addr = &args[3];
            let handshake = Handshake::new(infohash, *b"00112233445566778899");
            println!("Peer ID: {}", handshake.handshake(&addr).await.unwrap());
        }
        "download_piece" => {
            if args.len() < 3 {
                eprintln!("No .torrent file provided.");
                return Ok(());
            }
            let dot_torrent = std::fs::read(&args[2])?;
            let torrent = parse_torrent_file(dot_torrent)?;
            let infohash = info_hash(&torrent.info).unwrap();
            let tracker = Tracker::new("00112233445566778899".to_string(), 10);
            let infohash = info_hash_vec(&torrent.info).unwrap();
            let handshake = Handshake::new(infohash, *b"00112233445566778899");
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
    Ok(())
}

use serde::{Deserialize, Deserializer, Serialize};
use serde_xml_rs::from_str;
use base64::{Engine, prelude::BASE64_STANDARD};
use crate::metadata::PlaybackStatus::{Paused, Playing, Unknown};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Item {
    #[serde(rename = "type", deserialize_with = "hex_to_u32")]
    r#type: u32,
    #[serde(deserialize_with = "hex_to_u32")]
    code: u32,
    length: u32,
    data: Option<String>,
}

#[derive(PartialEq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Unknown,
}

pub struct SongData {
    pub title: String,
    pub album: String,
    pub artist: String,
    pub genre: String,
    pub album_art: String,
    pub track_length_ms: u32,
    pub progress: String,
    pub playback_status: PlaybackStatus,
}

fn hex_to_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let hex_str: String = Deserialize::deserialize(deserializer)?;
    u32::from_str_radix(&hex_str, 16).map_err(serde::de::Error::custom)
}

fn base64_decode(base64_string: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let decoded_data = BASE64_STANDARD.decode(base64_string)?;
    Ok(decoded_data)
}

pub fn process_xml(xml: &str) -> SongData {
    let items: Vec<Item> = match from_str(xml) {
        Ok(items) => items,
        Err(e) => {
            eprintln!("Failed to parse XML: {:?}", e);
            return SongData {
                title: "".to_string(),
                album: "".to_string(),
                artist: "".to_string(),
                genre: "".to_string(),
                album_art: "".to_string(),
                track_length_ms: 0,
                progress: "".to_string(),
                playback_status: Unknown,

            };
        }
    };

    let mut data = SongData {
        title: "".to_string(),
        album: "".to_string(),
        artist: "".to_string(),
        genre: "".to_string(),
        album_art: "".to_string(),
        track_length_ms: 0,
        progress: "".to_string(),
        playback_status: Unknown,
    };

    for item in items {
        let core: u32 = 0x636F7265; // "core"
        if item.r#type == core {
            match item.code {
                0x6D696E6D => {
                    if let Some(base64_data) = &item.data {
                        data.title = String::from_utf8(base64_decode(base64_data).unwrap()).unwrap();
                    }
                },
                0x6173616C => {
                    if let Some(base64_data) = &item.data {
                        data.album = String::from_utf8(base64_decode(base64_data).unwrap()).unwrap();
                    }
                },
                0x61736172 => {
                    if let Some(base64_data) = &item.data {
                        data.artist = String::from_utf8(base64_decode(base64_data).unwrap()).unwrap();
                    }
                },
                0x6173676E => {
                    if let Some(base64_data) = &item.data {
                        data.genre = String::from_utf8(base64_decode(base64_data).unwrap()).unwrap();
                    }
                },
                0x6173746D => {
                    if let Some(base64_data) = &item.data {
                        let decoded_bytes = base64_decode(base64_data).unwrap();
                        data.track_length_ms = u32::from_be_bytes([decoded_bytes[0], decoded_bytes[1], decoded_bytes[2], decoded_bytes[3]]);
                    }
                },
                _ => (),
            }
        } else if item.r#type == 0x73736E63 {
            if item.code == 0x50494354 {
                if let Some(base64_data) = &item.data {
                    data.album_art.clone_from(base64_data);
                }
            }
            if item.code == 1886545778 {
                data.progress = String::from_utf8(base64_decode(&item.data.unwrap()).unwrap()).unwrap();
            }

            if item.code == 1885435251 || item.code == 1885695588 {
                println!("paused");
                data.playback_status = Paused;
            } else if item.code == 1886545267 || item.code == 1885496679 || item.code == 1886548845 {
                println!("playing");
                data.playback_status = Playing;
            }
        }
    }

    data
}
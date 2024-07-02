use serde::{Deserialize, Deserializer, Serialize};
use serde_xml_rs::from_str;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Item {
    #[serde(rename = "type", deserialize_with = "hex_to_u32")]
    r#type: u32,
    #[serde(deserialize_with = "hex_to_u32")]
    code: u32,
    length: i32,
    data: Option<String>,
}


pub struct SongData {
    pub title: String,
    pub album: String,
    pub artist: String,
    pub genre: String,
    pub album_art: String,
}

fn hex_to_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let hex_str: String = Deserialize::deserialize(deserializer)?;
    u32::from_str_radix(&hex_str, 16).map_err(serde::de::Error::custom)
}

fn base64_decode(base64_string: String) -> Result<String, Box<dyn std::error::Error>> {
    match BASE64_STANDARD.decode(&base64_string) {
        Ok(decoded_data) => match String::from_utf8(decoded_data) {
            Ok(decoded_str) => Ok(decoded_str),
            Err(e) => Err(Box::new(e)),
        },
        Err(e) => Err(Box::new(e)),
    }
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
            };
        }
    };

    let mut data = SongData {
        title: "".to_string(),
        album: "".to_string(),
        artist: "".to_string(),
        genre: "".to_string(),
        album_art: "".to_string(),
    };

    for item in items {
        let core: u32 = 1668248165;
        if item.r#type == core {
            match item.code {
                1634956652 => println!("URL: {:?}", base64_decode(item.data.unwrap_or_else(|| "".to_string()))), // asul
                1634951532 => data.album = base64_decode(item.data.unwrap_or_else(|| "".to_string())).unwrap(), // asal
                1634951538 => data.artist =  base64_decode(item.data.unwrap_or_else(|| "".to_string())).unwrap(), // asar
                1634952045 => println!("Comment: {:?}", base64_decode(item.data.unwrap_or_else(|| "".to_string()))), // ascm
                1634953070 => data.genre = base64_decode(item.data.unwrap_or_else(|| "".to_string())).unwrap(), // asgn
                1835626093 => data.title = base64_decode(item.data.unwrap_or_else(|| "".to_string())).unwrap(), // minm
                1634952048 => println!("Composer: {:?}", base64_decode(item.data.unwrap_or_else(|| "".to_string()))), // ascp
                1634952299 => println!("Song Data Kind: {:?}", item.data.unwrap_or_else(|| "".to_string())), // asdk
                1634956142 => println!("Sort as: {:?}", base64_decode(item.data.unwrap_or_else(|| "".to_string()))), // assn
                _ => ()
            }
        } else if item.r#type == 0x73736e63 {
            if item.code == 0x50494354 {
                data.album_art = item.data.unwrap_or_else(|| "".to_string());
            }
        }
    }

    data
}
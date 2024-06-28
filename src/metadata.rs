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

pub fn process_xml(xml: &str) {
    let items: Vec<Item> = match from_str(xml) {
        Ok(items) => items,
        Err(e) => {
            eprintln!("Failed to parse XML: {:?}", e);
            return;
        }
    };

    for item in items {
        let core: u32 = 1668248165;
        if item.r#type == core {
            println!("Type matches 'core'");
            match item.code {
                1634956652 => println!("URL: {:?}", base64_decode(item.data.unwrap_or_else(|| "".to_string()))), // asul
                1634951532 => println!("Album Name: {:?}", base64_decode(item.data.unwrap_or_else(|| "".to_string()))), // asal
                1634951538 => println!("Artist: {:?}", base64_decode(item.data.unwrap_or_else(|| "".to_string()))), // asar
                1634952045 => println!("Comment: {:?}", base64_decode(item.data.unwrap_or_else(|| "".to_string()))), // ascm
                1634953070 => println!("Genre: {:?}", base64_decode(item.data.unwrap_or_else(|| "".to_string()))), // asgn
                1835626093 => println!("Title: {:?}", base64_decode(item.data.unwrap_or_else(|| "".to_string()))), // minm
                1634952048 => println!("Composer: {:?}", base64_decode(item.data.unwrap_or_else(|| "".to_string()))), // ascp
                1634952299 => println!("Song Data Kind: {:?}", item.data.unwrap_or_else(|| "".to_string())), // asdk
                1634956142 => println!("Sort as: {:?}", base64_decode(item.data.unwrap_or_else(|| "".to_string()))), // assn
                _ => ()
            }
        }
    }
}
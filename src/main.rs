mod metadata;
extern crate sdl2;

use sdl2::ttf;
use sdl2::pixels::Color;
use std::io::{self, BufRead};
use sdl2::render::TextureQuery;
use base64::{engine::general_purpose, Engine as _};
use image::load_from_memory;
use sdl2::rect::Rect;
use crate::DataTypes::Album;


enum DataTypes {
    Artist,
    Album,
    Title,
    Genre,
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let ttf_context = ttf::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("display", 1024, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let stdin = io::stdin();
    let handle = stdin.lock();
    let mut buffer = String::new();

    let mut data = metadata::SongData {
        title: "".to_string(),
        artist: "".to_string(),
        album: "".to_string(),
        album_art: "".to_string(),
        genre: "".to_string(),
    };

    for line in handle.lines() {
        match line {
            Ok(line) => {
                buffer.push_str(&line);
                if line.ends_with("</item>") {
                    let new_data = metadata::process_xml(&buffer);
                    if !new_data.genre.is_empty() {
                        data.genre = new_data.genre;
                    } else if !new_data.title.is_empty() {
                        data.title = new_data.title;
                    } else if !new_data.album.is_empty() {
                        data.album = new_data.album;
                    } else if !new_data.artist.is_empty() {
                        data.artist = new_data.artist;
                    } else if !new_data.album_art.is_empty() {
                        data.album_art = new_data.album_art;
                    }

                    canvas.clear();
                    draw_album_art(&data, &mut canvas);
                    draw_data(&data, &mut canvas, &ttf_context, 1024, 600, DataTypes::Artist);
                    draw_data(&data, &mut canvas, &ttf_context, 1024, 600, DataTypes::Title);
                    draw_data(&data, &mut canvas, &ttf_context, 1024, 600, Album);
                    canvas.present();
                    buffer.clear();
                }
            }
            Err(e) => eprintln!("Error reading from stdin: {:?}", e),
        }
    }
}

fn draw_album_art(data: &metadata::SongData, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    let base64_data = &data.album_art; // Assume `data.album_art` contains the base64-encoded album art

    if !base64_data.is_empty() {
        let decoded_data = general_purpose::STANDARD.decode(base64_data).unwrap();
        let img = load_from_memory(&decoded_data).unwrap();

        let texture_creator = canvas.texture_creator();

        // Convert the image to an SDL2 surface with a mutable buffer
        let mut img_buffer = img.to_rgba8().into_vec();
        let width = img.width();
        let height = img.height();
        let surface = sdl2::surface::Surface::from_data(
            &mut img_buffer,
            width,
            height,
            4 * width,
            sdl2::pixels::PixelFormatEnum::RGBA32
        ).unwrap();

        let texture = texture_creator.create_texture_from_surface(&surface).unwrap();
        let TextureQuery { width, height, .. } = texture.query();

        let position_x = 50; // Left of the screen
        let position_y = 44; // Top margin

        let target = Rect::new(position_x, position_y, width, height);
        if let Err(e) = canvas.copy(&texture, None, Some(target)) {
            eprintln!("Could not copy texture to canvas: {}", e);
        }
    }
}

fn draw_data(
    data: &metadata::SongData,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    ttf_context: &ttf::Sdl2TtfContext,
    window_width: i32,
    window_height: i32,
    data_type: DataTypes
) {
    let base_font_size: u16 = match data_type {
        DataTypes::Title => 64,
        DataTypes::Album => 64,
        DataTypes::Artist => 64,
        _ => 24,
    };


    let text = match data_type {
        DataTypes::Title => &data.title,
        DataTypes::Album => &data.album,
        DataTypes::Artist => &data.artist,
        _ => &data.genre,
    };

    if !text.is_empty() {
        let truncated_text: String = if text.chars().count() > 50 {
            format!("{}...", &text.chars().take(50).collect::<String>())
        } else {
            text.clone()
        };

        let image_width = 512;
        let margin = 50;
        let max_width = (window_width - image_width - 3 * margin) as u32;

        let mut font_size = base_font_size;
        let font_raw = include_bytes!("Roboto-Medium.ttf");

        loop {
            let font = ttf_context.load_font_from_rwops(sdl2::rwops::RWops::from_bytes(font_raw).unwrap(), font_size).unwrap();
            let surface = font.render(&truncated_text).blended(Color::RGB(255, 255, 255)).unwrap();
            let texture_creator = canvas.texture_creator();
            let texture = texture_creator.create_texture_from_surface(&surface).unwrap();
            let TextureQuery { width, .. } = texture.query();

            if width <= max_width {
                break;
            }

            if font_size == 1 {
                break;
            }

            font_size -= 1;
        }

        let font = ttf_context.load_font_from_rwops(sdl2::rwops::RWops::from_bytes(font_raw).unwrap(), font_size).unwrap();
        let surface = font.render(&truncated_text).blended(Color::RGB(255, 255, 255)).unwrap();
        let texture_creator = canvas.texture_creator();
        let texture = texture_creator.create_texture_from_surface(&surface).unwrap();
        let TextureQuery { width, height, .. } = texture.query();

        let position_x = window_width - margin - width as i32;
        let position_y = match data_type {
            DataTypes::Title => window_height / 2 - 100,
            DataTypes::Album => window_height / 2,
            DataTypes::Artist => window_height / 2 + 100,
            _ => 5
        };

        let target = Rect::new(position_x, position_y, width, height);

        if let Err(e) = canvas.copy(&texture, None, Some(target)) {
            eprintln!("Could not copy texture to canvas: {}", e);
        }
    }
}

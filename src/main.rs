mod metadata;

use fltk::{app, image::RgbImage, prelude::*, text::TextDisplay, window::Window, frame::Frame};
use std::io::{self, BufRead};
use base64::{engine::general_purpose, Engine as _};
use image::load_from_memory;
use std::sync::{Arc, Mutex};
use std::thread;
use fltk::enums::{Color, FrameType};

enum DataTypes {
    Artist,
    Album,
    Title,
    Genre,
}

struct SongData {
    title: String,
    artist: String,
    album: String,
    album_art: String,
    old_album_art: String,
    genre: String,
}

fn main() {
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 1024, 600, "Display");
    wind.set_color(Color::from_hex(0));

    let mut album_art_frame = Frame::new(50, 44, 512, 512, "");
    let mut artist_display = TextDisplay::new(600, 300, 400, 50, "");
    let mut title_display = TextDisplay::new(600, 250, 400, 50, "");
    let mut album_display = TextDisplay::new(600, 350, 400, 50, "");
    artist_display.set_color(Color::from_hex(0));
    title_display.set_color(Color::from_hex(0));
    album_display.set_color(Color::from_hex(0));
    artist_display.set_frame(FrameType::NoBox);
    title_display.set_frame(FrameType::NoBox);
    album_display.set_frame(FrameType::NoBox);


    wind.end();
    wind.show();

    let data = Arc::new(Mutex::new(SongData {
        title: "".to_string(),
        artist: "".to_string(),
        album: "".to_string(),
        album_art: "".to_string(),
        old_album_art: "".to_string(),
        genre: "".to_string(),
    }));

    let data_clone = data.clone();
    thread::spawn(move || {
        let stdin = io::stdin();
        let handle = stdin.lock();
        let mut buffer = String::new();

        for line in handle.lines() {
            match line {
                Ok(line) => {
                    buffer.push_str(&line);
                    if line.ends_with("</item>") {
                        let new_data = metadata::process_xml(&buffer);
                        {
                            let mut data = data_clone.lock().unwrap();
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
                        }
                        app::awake();
                        buffer.clear();
                    }
                }
                Err(e) => eprintln!("Error reading from stdin: {:?}", e),
            }
        }
    });

    app::add_idle3(move |_| {
        let mut data = data.lock().unwrap();
        update_display(&data, &mut album_art_frame, &mut artist_display, &mut title_display, &mut album_display);
        data.old_album_art = data.album_art.clone();
    });

    app.run().unwrap();
}

fn update_display(data: &SongData, album_art_frame: &mut Frame, artist_display: &mut TextDisplay, title_display: &mut TextDisplay, album_display: &mut TextDisplay) {
    if data.old_album_art != data.album_art {
        draw_album_art(data, album_art_frame);
    }
    draw_data(data, artist_display, DataTypes::Artist);
    draw_data(data, title_display, DataTypes::Title);
    draw_data(data, album_display, DataTypes::Album);
}

fn draw_album_art(data: &SongData, frame: &mut Frame) {
    let base64_data = &data.album_art;

    if !base64_data.is_empty() {
        match general_purpose::STANDARD.decode(base64_data) {
            Ok(decoded_data) => {
                match load_from_memory(&decoded_data) {
                    Ok(img) => {
                        let img_buffer = img.to_rgba8().into_vec();
                        let width = img.width() as i32;
                        let height = img.height() as i32;

                        match RgbImage::new(&img_buffer, width, height, fltk::enums::ColorDepth::Rgba8) {
                            Ok(mut image) => {
                                if width > 512 || height > 512 {
                                    // Resize the image to fit within the frame if necessary
                                    image.scale(512, 512, true, true);
                                }
                                frame.set_image(Some(image));
                                frame.redraw(); // Explicitly redraw the frame
                                println!("Album art set successfully");
                            }
                            Err(e) => eprintln!("Error creating RgbImage: {:?}", e),
                        }
                    }
                    Err(e) => eprintln!("Error loading image from memory: {:?}", e),
                }
            }
            Err(e) => eprintln!("Error decoding base64 album art: {:?}", e),
        }
    } else {
        println!("No album art to display");
    }
}

fn draw_data(data: &SongData, display: &mut TextDisplay, data_type: DataTypes) {
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

        let mut buffer = fltk::text::TextBuffer::default();
        buffer.set_text(&truncated_text);

        display.set_text_color(Color::rgb_color(255, 255, 255));
        display.set_buffer(Some(buffer));
        display.redraw(); // Explicitly redraw the display
    }
}

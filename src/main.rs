mod metadata;

use std::time::Instant;
use fltk::{app, image::RgbImage, prelude::*, text::TextDisplay, window::Window, frame::Frame};
use std::io::{self, BufRead};
use base64::{engine::general_purpose, Engine as _};
use image::load_from_memory;
use std::sync::{Arc, Mutex};
use std::thread;
use fltk::enums::{Color, Font, FrameType};
use fltk::misc::Progress;

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
    progress_start: u64,
    progress_curr: u64,
    progress_end: u64,
    last_update: Instant,
    is_playing: bool,
}

const RTP_CLOCK_RATE: f64 = 44100.0; // Assuming a 44.1 kHz sample rate

fn main() {
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 1024, 600, "Display");
    wind.set_color(Color::from_hex(0));

    let font = Font::load_font("Roboto-Medium.ttf").unwrap();
    Font::set_font(Font::Helvetica, &font);

    let mut album_art_frame = Frame::new(50, 44, 512, 512, "");
    let mut artist_display = TextDisplay::new(600, 300, 400, 50, "");
    let mut title_display = TextDisplay::new(600, 250, 400, 50, "");
    let mut album_display = TextDisplay::new(600, 350, 400, 50, "");
    let mut position_display = TextDisplay::new(50, 568, 75, 50, "");
    let mut time_left_display = TextDisplay::new(924, 568, 75, 50, "");

    artist_display.set_color(Color::from_hex(0));
    title_display.set_color(Color::from_hex(0));
    album_display.set_color(Color::from_hex(0));
    position_display.set_color(Color::from_hex(0));
    time_left_display.set_color(Color::from_hex(0));

    artist_display.set_frame(FrameType::NoBox);
    title_display.set_frame(FrameType::NoBox);
    album_display.set_frame(FrameType::NoBox);
    position_display.set_frame(FrameType::NoBox);
    time_left_display.set_frame(FrameType::NoBox);


    let mut progress_bar = Progress::new(100, 571, 824, 14, "");
    progress_bar.set_color(Color::from_hex(0));
    progress_bar.set_selection_color(Color::from_hex(0xffffff));
    progress_bar.set_frame(FrameType::FlatBox);

    wind.end();
    wind.show();

    let data = Arc::new(Mutex::new(SongData {
        title: "".to_string(),
        artist: "".to_string(),
        album: "".to_string(),
        album_art: "".to_string(),
        old_album_art: "".to_string(),
        genre: "".to_string(),
        progress_start: 0,
        progress_curr: 0,
        progress_end: 0,
        last_update: Instant::now(),
        is_playing: false,
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
                            if let Some((start, curr, end)) = parse_progress(&new_data.progress) {
                                data.progress_start = start;
                                data.progress_curr = curr;
                                data.progress_end = end;
                                data.last_update = Instant::now();
                                data.is_playing = true;
                            }
                            if new_data.playback_status == metadata::PlaybackStatus::Paused {
                                data.is_playing = false;
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
        update_display(&mut data,
                       &mut album_art_frame,
                       &mut progress_bar,
                       &mut artist_display,
                       &mut title_display,
                       &mut album_display,
                       &mut position_display,
                       &mut time_left_display);
    });

    app.run().unwrap();
}

fn update_display(
    data: &mut SongData,
    album_art_frame: &mut Frame,
    progress_bar: &mut Progress,
    artist_display: &mut TextDisplay,
    title_display: &mut TextDisplay,
    album_display: &mut TextDisplay,
    position_display: &mut TextDisplay,
    time_left_display: &mut TextDisplay,
) {
    let now = Instant::now();
    let elapsed = now.duration_since(data.last_update);
    data.last_update = now;

    if data.is_playing {
        let elapsed_rtp = (elapsed.as_secs_f64() * RTP_CLOCK_RATE) as u64;
        data.progress_curr = data.progress_curr.saturating_add(elapsed_rtp);
        if data.progress_curr > data.progress_end {
            data.progress_curr = data.progress_end;
        }
    }

    if data.old_album_art != data.album_art {
        draw_album_art(data, album_art_frame);
    }
    draw_data(data, artist_display, DataTypes::Artist);
    draw_data(data, title_display, DataTypes::Title);
    draw_data(data, album_display, DataTypes::Album);
    update_progress_bar(progress_bar, data);

    let position = rtptime_to_sec(data.progress_curr.saturating_sub(data.progress_start));
    let duration = rtptime_to_sec(data.progress_end.saturating_sub(data.progress_start));
    let time_left = duration - position;

    let formatted_position = format_time(position as u32);
    let formatted_time_left = format_time(time_left as u32);

    let mut buffer = fltk::text::TextBuffer::default();
    buffer.set_text(&formatted_position);
    position_display.set_text_color(Color::rgb_color(255, 255, 255));
    position_display.set_buffer(Some(buffer));

    let mut buffer = fltk::text::TextBuffer::default();

    buffer.set_text(&format!("-{}", formatted_time_left));
    time_left_display.set_text_color(Color::from_hex(0xffffff));
    time_left_display.set_buffer(Some(buffer));

    time_left_display.redraw();
    position_display.redraw();
}

fn format_time(seconds: u32) -> String {
    let minutes = seconds / 60;
    let remaining_seconds = seconds % 60;
    format!("{:02}:{:02}", minutes, remaining_seconds)
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
                                    image.scale(512, 512, true, true);
                                }
                                frame.set_image(Some(image));
                                frame.redraw();
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
        display.set_text_size(24);
        display.set_buffer(Some(buffer));
        display.redraw();
    }
}

fn update_progress_bar(progress_bar: &mut Progress, data: &SongData) {
    if data.progress_start == data.progress_end {
        progress_bar.set_value(0.0);
    } else {
        let position = data.progress_curr.saturating_sub(data.progress_start) as f64;
        let duration = data.progress_end.saturating_sub(data.progress_start) as f64;

        if duration > 0.0 {
            let progress_percentage = (position / duration) * 100.0;
            progress_bar.set_value(progress_percentage);
        }
    }
    progress_bar.redraw();
}

fn parse_progress(progress: &str) -> Option<(u64, u64, u64)> {
    let parts: Vec<&str> = progress.split('/').collect();
    if parts.len() == 3 {
        let start = parts[0].parse().ok()?;
        let curr = parts[1].parse().ok()?;
        let end = parts[2].parse().ok()?;
        Some((start, curr, end))
    } else {
        None
    }
}

fn rtptime_to_sec(diff: u64) -> f64 {
    diff as f64 / RTP_CLOCK_RATE
}
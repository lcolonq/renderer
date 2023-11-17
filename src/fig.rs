use std::{io::Write, io::BufRead, collections::HashMap, sync::mpsc::Sender};

use colors_transform::{Rgb, Color};
use lexpr::sexp;

use crate::{avatar, context};

pub enum ControlCommand {
    PlayVideo {
        pty: context::PaletteType,
        url: String,
    },
    ReloadPumpkin,
}

pub struct ControlState {
    // pub video_playback: Option<(context::PaletteType, String)>,
    // pub video_is_playing: bool,
    pub properties: HashMap<String, i64>,
    pub palette: avatar::Palette,
}

#[derive(Clone)]
pub struct Control(pub std::sync::Arc<std::sync::Mutex<ControlState>>);

impl ControlState {
    pub fn new() -> Self {
        Self {
            // video_playback: None,
            // video_is_playing: false,
            properties: HashMap::new(),
            palette: avatar::Palette::new(),
        }
    }
}

impl Control {
    pub fn new() -> Self {
        let cs = ControlState::new();
        Self(std::sync::Arc::new(std::sync::Mutex::new(cs)))
    }

    pub fn get(&self, prop: &str) -> i64 {
        if let Some(v) = self.0.lock().unwrap().properties.get(prop) {
            *v
        } else { 0 }
    }

    pub fn is(&self, prop: &str) -> bool {
        if self.get(prop) == 0 { false } else { true }
    }

    pub fn run(self, command_sender: Sender<ControlCommand>) {
        std::thread::spawn(move || {
            // let mut stream = std::net::TcpStream::connect("shiro:32050").unwrap();
            let mut stream = std::net::TcpStream::connect("localhost:32050").unwrap();
            stream.write_all("(sub (avatar toggle))
(sub (avatar palette word))
(sub (avatar palette color))
(sub (avatar palette image))
(sub (avatar palette video))
(sub (avatar reset))
(sub (avatar pumpkinreload))
".as_bytes()).unwrap();
            let reader = std::io::BufReader::new(stream);
            for l in reader.lines() {
                let strl = l.unwrap();
                match lexpr::from_str(&strl) {
                    Err(_) => {},
                    Ok(v) => {
                        if v[0] == sexp!((avatar toggle)) {
                            let tnm = v[1].as_str().unwrap();
                            let old = self.get(tnm);
                            let new = if old == 0 { 1 } else { 0 };
                            self.0.lock().unwrap().properties.insert(tnm.to_owned(), new);
                        } else if v[0] == sexp!((avatar reset)) {
                            *self.0.lock().unwrap() = ControlState::new();
                        } else if v[0] == sexp!((avatar palette word)) {
                            let pty = context::PaletteType::from_string(v[1].as_str().unwrap()).unwrap();
                            let encodedword = v[2].as_str().unwrap().to_owned();
                            let decodedword = base64::decode(encodedword).unwrap();
                            self.0.lock().unwrap().palette.word_mapping.insert(
                                pty,
                                String::from_utf8(decodedword).unwrap(),
                            );
                        } else if v[0] == sexp!((avatar palette color)) {
                            let pty = context::PaletteType::from_string(v[1].as_str().unwrap()).unwrap();
                            let encodedcol = v[2].as_str().unwrap().to_owned();
                            let decodedcol = base64::decode(encodedcol).unwrap();
                            let col = String::from_utf8(decodedcol).unwrap();
                            log::info!("Requested color on {:?}: {}", pty, &col);
                            if let Ok(rgb) = Rgb::from_hex_str(&col) {
                                self.0.lock().unwrap().palette.color_mapping.insert(
                                    pty,
                                    avatar::PaletteEntry::Color(
                                        (rgb.get_red() as _, rgb.get_green() as _, rgb.get_blue() as _),
                                    ),
                            );
                            }
                        } else if v[0] == sexp!((avatar palette image)) {
                            let pty = context::PaletteType::from_string(v[1].as_str().unwrap()).unwrap();
                            let encodedpath = v[2].as_str().unwrap().to_owned();
                            let decodedpath = base64::decode(encodedpath).unwrap();
                            let path = String::from_utf8(decodedpath.clone()).unwrap();
                            log::info!("Requested image on {:?}: {}", pty, &path);
                            if let Some(pal) = avatar::PaletteEntry::from_image(&path) {
                                self.0.lock().unwrap().palette.color_mapping.insert(
                                    pty,
                                    pal,
                                );
                            }
                        } else if v[0] == sexp!((avatar palette video)) {
                            let pty = context::PaletteType::from_string(v[1].as_str().unwrap()).unwrap();
                            let encodedpath = v[2].as_str().unwrap().to_owned();
                            let decodedpath = base64::decode(encodedpath).unwrap();
                            let path = String::from_utf8(decodedpath.clone()).unwrap();
                            log::info!("Requested video on {:?}: {}", pty, &path);
                            self.0.lock().unwrap().palette.color_mapping.insert(
                                pty.clone(),
                                avatar::PaletteEntry::Video,
                            );
                            command_sender.send(ControlCommand::PlayVideo { pty, url: path }).unwrap();
                        } else if v[0] == sexp!((avatar pumpkinreload)) {
                            log::info!("Requested pumpkin reload");
                            command_sender.send(ControlCommand::ReloadPumpkin).unwrap();
                        }
                    },
                }
            }
        });
    }
}

use std::io::Write;
use colored::Colorize;
use byteorder::WriteBytesExt;

fn choose_glyph(custom: bool, g: char, v: f32) -> char {
    if custom { return g; }
    if g == ' ' || g == '/' || g == '-' || g == '_' || g == '\\' { return g; }
    if v < 0.2 {
        '.'
    } else if v < 0.4 {
        ','
    } else if v < 0.6 {
        '`'
    } else if v < 0.8 {
        '*'
    } else {
        '#'
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Foreground {
        custom_glyph: bool,
        glyph0: char,
        glyph1: Option<char>,
        color: (u8, u8, u8),
    },
    Background,
}

impl Cell {
    pub fn serialize<W>(&self, out: &mut W) -> Option<()>
    where W: Write
    {
        match self {
            Self::Foreground { custom_glyph, glyph0, glyph1, color } => {
                out.write_u8(1).ok()?;
                out.write_u8(if *custom_glyph { 1 } else { 0 }).ok()?;
                out.write_u8(color.0).ok()?;
                out.write_u8(color.1).ok()?;
                out.write_u8(color.2).ok()?;
                out.write_u32::<byteorder::BigEndian>(*glyph0 as _).ok()?;
                if let Some(g1) = *glyph1 {
                    out.write_u8(1).ok()?;
                    out.write_u32::<byteorder::BigEndian>(g1 as _).ok()?;
                } else {
                    out.write_u8(0).ok()?;
                }
            },
            Self::Background => {
                out.write_u8(0).ok()?;
            },
        }
        Some(())
    }
}

pub struct Diff {
    pub diff: Vec<(u8, u8, Cell)>,
}

impl Diff {
    pub fn new(old: &Term, new: &Term) -> Self {
        let mut diff = Vec::new();
        for y in 0..64 {
            for x in 0..64 {
                if old.cells[y][x] != new.cells[y][x] {
                    diff.push((x as _, y as _, new.cells[y][x]));
                }
            }
        }
        Self {
            diff,
        }
    }

    pub fn serialize<W>(&self, out: &mut W)
    where W: Write
    {
        out.write_u8(1).unwrap();
        out.write_u32::<byteorder::BigEndian>(self.diff.len() as _).unwrap();
        for (x, y, c) in self.diff.iter() {
            out.write_u8(*x).unwrap();
            out.write_u8(*y).unwrap();
            c.serialize(out).unwrap();
        }
    }

    pub fn serialize_and_compress(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        {
            let mut writer = flate2::write::GzEncoder::new(&mut ret, flate2::Compression::default());
            self.serialize(&mut writer);
        }
        ret
    }
}

pub struct Term {
    pub count: usize, 
    pub cells: [[Cell; 64]; 64],
}

impl Term {
    pub fn new() -> Self {
        Self {
            count: 0,
            cells: [[Cell::Background; 64]; 64],
        }
    }

    pub fn write(&mut self, x: usize, y: usize, custom_glyph: bool, glyph0: char, glyph1: Option<char>, color: (u8, u8, u8)) {
        self.cells[y][x] = Cell::Foreground { custom_glyph, glyph0, glyph1, color, };
    }

    pub fn serialize<W>(&self, out: &mut W)
    where W: Write
    {
        out.write_u8(0).unwrap();
        for row in self.cells.iter() {
            for cell in row.iter() {
                cell.serialize(out).unwrap();
            }
        }
    }

    pub fn serialize_and_compress(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        {
            let mut writer = flate2::write::GzEncoder::new(&mut ret, flate2::Compression::default());
            self.serialize(&mut writer);
        }
        ret
    }

    pub fn render_stream<W>(&self, out: &mut W, bg: (u8, u8, u8))
    where W: Write {
        let bgcolor = colored::customcolors::CustomColor::new(bg.0, bg.1, bg.2);
        let mut output: Vec<u8> = Vec::new();
        write!(output, "\x1b[2J\x1b[1;1H").unwrap();
        for row in self.cells.iter() {
            for cell in row.iter() {
                match cell {
                    Cell::Foreground { custom_glyph: _, glyph0, glyph1, color } => {
                        let s = if let Some(g) = glyph1 {
                            format!("{}{}", glyph0, g)
                        } else {
                            format!("{}", glyph0)
                        };
                        write!(
                            output, "{}",
                            s.truecolor(color.0, color.1, color.2).on_custom_color(bgcolor)
                        ).unwrap();
                    },
                    Cell::Background => {},
                }
            }
            write!(output, "\r\n").unwrap();
        }
        out.write(&output).unwrap();
    }

    pub fn render_stream_nocolor<W>(&self, out: &mut W)
    where W: Write {
        let mut output: Vec<u8> = Vec::new();
        write!(output, "\x1b[2J\x1b[1;1H").unwrap();
        for row in self.cells.iter() {
            for cell in row.iter() {
                match cell {
                    Cell::Foreground { custom_glyph, glyph0, glyph1, color, } => {
                        let r: f32 = (color.0 as f32) / 255.0;
                        let g: f32 = (color.1 as f32) / 255.0;
                        let b: f32 = (color.2 as f32) / 255.0;
                        let mono = (0.2125 * r) + (0.7154 * g) + (0.0721 * b);
                        let s = if let Some(g) = glyph1 {
                            format!(
                                "{}{}",
                                choose_glyph(*custom_glyph, *glyph0, mono),
                                choose_glyph(*custom_glyph, *g, mono)
                            )
                        } else {
                            format!(
                                "{}",
                                choose_glyph(*custom_glyph, *glyph0, mono)
                            )
                        };
                        write!(
                            output, "{}",
                            s
                        ).unwrap();
                    },
                    Cell::Background => {},
                }
            }
            write!(output, "\r\n").unwrap();
        }
        out.write(&output).unwrap();
    }

    pub fn render_stream_nocolor_small<W>(&self, out: &mut W)
    where W: Write {
        let mut output: Vec<u8> = Vec::new();
        write!(output, "\x1b[2J\x1b[1;1H").unwrap();
        for row in self.cells.iter().step_by(2) {
            for cell in row.iter().step_by(2) {
                match cell {
                    Cell::Foreground { custom_glyph, glyph0, glyph1, color, } => {
                        let r: f32 = (color.0 as f32) / 255.0;
                        let g: f32 = (color.1 as f32) / 255.0;
                        let b: f32 = (color.2 as f32) / 255.0;
                        let mono = (0.2125 * r) + (0.7154 * g) + (0.0721 * b);
                        let s = if let Some(g) = glyph1 {
                            format!(
                                "{}{}",
                                choose_glyph(*custom_glyph, *glyph0, mono),
                                choose_glyph(*custom_glyph, *g, mono)
                            )
                        } else {
                            format!(
                                "{}",
                                choose_glyph(*custom_glyph, *glyph0, mono)
                            )
                        };
                        write!(
                            output, "{}",
                            s
                        ).unwrap();
                    },
                    Cell::Background => {},
                }
            }
            write!(output, "\r\n").unwrap();
        }
        out.write(&output).unwrap();
    }
}

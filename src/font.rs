use std::{collections::HashMap, io::BufRead};

use tap::TapOptional;

use crate::{Color, PixBuf};

struct Chunks<'a>(&'a str, usize);

impl<'a> Iterator for Chunks<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.len() >= self.1 {
            let (next, rest) = self.0.split_at(self.1);
            self.0 = rest;
            Some(next)
        } else {
            None
        }
    }
}

fn chunks(s: &str, n: usize) -> impl Iterator<Item = &str> {
    Chunks(s, n)
}

#[derive(Debug)]
pub struct Font {
    pub width: u32,
    pub chars: HashMap<char, CharData>,
    pub ligatures: HashMap<(char, char), CharData>,
}

#[derive(Debug)]
pub struct CharData {
    pub width: u32,
    pub height: u32,
    pub xo: i32,
    pub yo: i32,
    pub data: Vec<u8>,
}

impl CharData {
    fn draw(&self, buf: &mut PixBuf, pos: (i32, i32), color: Color, scale: i32) {
        let mut data = self.data.as_slice();

        let iheight = self.height as i32;

        let data_width = (self.width as usize + 7) >> 3;

        for y in 0..iheight {
            let line = &data[0..data_width];
            data = &data[data_width..];

            for (x8, mut byte) in line.iter().copied().enumerate() {
                for x in (x8 as i32 * 8..x8 as i32 * 8 + 8).rev() {
                    let pixel = byte & 1 == 1;
                    byte = byte >> 1;

                    if pixel {
                        buf.set_scaled_pixel(
                            pos.0 / scale + x + self.xo,
                            pos.1 / scale + y - iheight - self.yo,
                            scale,
                            color,
                        );
                    }
                }
            }
        }
    }
}

impl Font {
    pub fn parse_bdf(bdf: impl BufRead, width: u32) -> Option<Self> {
        let mut lines = bdf.lines().filter_map(|line| line.ok());

        let mut font = Self {
            chars: HashMap::new(),
            ligatures: HashMap::new(),
            width,
        };

        loop {
            // get next character
            let char = loop {
                if let Some(next) = lines.next() {
                    if next.starts_with("ENCODING") {
                        break next;
                    }
                } else {
                    // no more characters
                    return Some(font);
                }
            };
            let char = char.split_whitespace().skip(1).next().unwrap();
            let char = char::from_u32(u32::from_str_radix(char, 10).ok()?)?;

            // get bounding box
            let bbx = loop {
                let next = lines.next()?;
                if next.starts_with("BBX") {
                    break next;
                }
            };
            let mut bbx = bbx.split_whitespace().skip(1);
            let width = u32::from_str_radix(bbx.next()?, 10).ok()?;
            let height = u32::from_str_radix(bbx.next()?, 10).ok()?;
            let xo = i32::from_str_radix(bbx.next()?, 10).ok()?;
            let yo = i32::from_str_radix(bbx.next()?, 10).ok()?;

            // get data
            loop {
                let next = lines.next()?;
                if next.starts_with("BITMAP") {
                    break;
                }
            }

            let mut data = Vec::new();
            for _ in 0..height {
                let line = lines.next()?;
                let bytes = chunks(&line, 2).map(|s| u8::from_str_radix(s, 16));
                for byte in bytes {
                    data.push(byte.ok()?);
                }
            }

            // add character
            font.chars.insert(
                char,
                CharData {
                    width,
                    height,
                    xo,
                    yo,
                    data,
                },
            );
        }
    }

    pub fn draw(&self, buf: &mut PixBuf, s: &str, mut pos: (i32, i32), color: Color, scale: i32) {
        let mut chars = s.chars().peekable();
        loop {
            let Some(n) = chars.next() else {
                break;
            };
            if let Some(char) = chars
                .peek()
                .copied()
                .and_then(|snd| self.ligatures.get(&(n, snd)))
                .tap_some(|_| {
                    chars.next();
                })
                .or_else(|| self.chars.get(&n))
            {
                char.draw(buf, pos, color, scale);
            }
            pos.0 += self.width as i32 * scale;
        }
    }
}

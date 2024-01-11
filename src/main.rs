use std::io::Cursor;
use std::slice::from_raw_parts_mut;

use error_iter::ErrorIter;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use widget::{Canvas, CutDir, Rect};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use crate::font::{CharData, Font};

mod font;
mod widget;

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

#[inline]
pub fn as_chunks_mut<T, const N: usize>(s: &mut [T]) -> (&mut [[T; N]], &mut [T]) {
    assert_ne!(N, 0);
    let len = s.len() / N;
    let (multiple_of_n, remainder) = s.split_at_mut(len * N);
    // SAFETY: We cast a slice of `len * N` elements into
    // a slice of `len` many `N` elements chunks.
    let array_slice: &mut [[T; N]] =
        unsafe { from_raw_parts_mut(multiple_of_n.as_mut_ptr().cast(), len) };
    (array_slice, remainder)
}

type Color = [u8; 4];

struct PixBuf<'a> {
    buf: &'a mut [Color],
    width: i32,
    height: i32,
}

impl<'a> PixBuf<'a> {
    fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && y >= 0 && x < self.width && y < self.height {
            self.buf[(x + y * self.width) as usize] = color;
        }
    }
    fn set_scaled_pixel(&mut self, x: i32, y: i32, scale: i32, color: Color) {
        for y in y * scale..y * scale + scale {
            // TODO: set slice range?
            for x in x * scale..x * scale + scale {
                self.set_pixel(x, y, color);
            }
        }
    }
}

const COZETTE: &'static [u8; 342005] = include_bytes!("../cozette.bdf");

fn main() -> Result<(), Error> {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    let mut font = Font::parse_bdf(Cursor::new(COZETTE), 6, 13).unwrap();

    // double sharp
    font.chars.insert(
        '𝄪',
        CharData {
            width: 5,
            height: 5,
            xo: 1,
            yo: 0,
            data: vec![0b11011000, 0b11011000, 0b00100000, 0b11011000, 0b11011000],
        },
    );
    // double flat
    font.chars.insert(
        '𝄫',
        CharData {
            width: 5,
            height: 7,
            xo: 1,
            yo: 0,
            data: vec![
                0b10100000, 0b10100000, 0b10100000, 0b11111000, 0b10101000, 0b10101000, 0b11110000,
            ],
        },
    );
    // half sharp
    font.chars.insert(
        '𝄲',
        CharData {
            width: 3,
            height: 7,
            xo: 2,
            yo: -1,
            data: vec![
                0b01000000, 0b01100000, 0b11000000, 0b01000000, 0b01100000, 0b11000000, 0b01000000,
            ],
        },
    );
    // half flat
    font.chars.insert(
        '𝄳',
        CharData {
            width: 3,
            height: 7,
            xo: 2,
            yo: 0,
            data: vec![
                0b00100000, 0b00100000, 0b00100000, 0b11100000, 0b10100000, 0b10100000, 0b01100000,
            ],
        },
    );
    // three halves sharp
    font.ligatures.insert(
        ('𝄲', '♯'),
        CharData {
            width: 5,
            height: 9,
            xo: 1,
            yo: -1,
            data: vec![
                0b00001000, 0b00101000, 0b10111000, 0b11101000, 0b10101000, 0b10111000, 0b11101000,
                0b10100000, 0b10000000,
            ],
        },
    );
    // three halves flat
    font.ligatures.insert(
        ('𝄳', '♭'),
        CharData {
            width: 5,
            height: 7,
            xo: 1,
            yo: 0,
            data: vec![
                0b00100000, 0b00100000, 0b00100000, 0b11111000, 0b10101000, 0b10101000, 0b01110000,
            ],
        },
    );

    let mut width = WIDTH as i32;
    let mut height = HEIGHT as i32;

    event_loop
        .run(move |event, target| {
            // Draw current frame
            if let Event::WindowEvent {
                window_id: _,
                event: WindowEvent::RedrawRequested,
            } = event
            {
                let mut canvas = Canvas {
                    pix: PixBuf {
                        buf: as_chunks_mut(pixels.frame_mut()).0,
                        width,
                        height,
                    },
                    font: &font,
                    rect: Rect {
                        x: 0,
                        y: 0,
                        width,
                        height,
                        dir: CutDir::Vertical,
                    },
                };

                canvas.clear();
                canvas.center(
                    38 * canvas.font.width * 2,
                    canvas.font.height * 2 * 3,
                    |canvas| {
                        canvas.text("C  C♯  C♭  C♮  C𝄪  C𝄫  C𝄲  C𝄳  C𝄲♯  C𝄳♭ ");
                        canvas.text("C7 C♯7 C♭7 C♮7 C𝄪7 C𝄫7 C𝄲7 C𝄳7 C𝄲♯7 C𝄳♭7");
                        canvas.text("Cm C♯m C♭m C♮m C𝄪m C𝄫m C𝄲m C𝄳m C𝄲♯m C𝄳♭m");
                    },
                );

                if let Err(err) = pixels.render() {
                    log_error("pixels.render", err);
                    target.exit();
                    return;
                }
            }

            // Handle input events
            if input.update(&event) {
                // Close
                if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                    target.exit();
                    return;
                }

                // Resize the window
                if let Some(size) = input.window_resized() {
                    width = size.width as i32;
                    height = size.height as i32;
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        log_error("pixels.resize_surface", err);
                        target.exit();
                        return;
                    }
                    if let Err(err) = pixels.resize_buffer(size.width, size.height) {
                        log_error("pixels.resize_buffer", err);
                        target.exit();
                        return;
                    }
                }

                // Request redraw
                window.request_redraw();
            }
        })
        .unwrap();

    Ok(())
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

use crate::{font::Font, Color, PixBuf};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CutDir {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub dir: CutDir,
}

pub struct Canvas<'a> {
    pub pix: PixBuf<'a>,
    pub font: &'a Font,
    pub rect: Rect,
}

impl Canvas<'_> {
    pub fn center(&mut self, width: i32, height: i32, f: impl FnOnce(&mut Self)) {
        let tmp = self.rect;
        self.rect = Rect {
            x: self.rect.x + self.rect.width / 2 - width / 2,
            y: self.rect.y + self.rect.height / 2 - height / 2,
            width,
            height,
            dir: self.rect.dir,
        };
        f(self);
        self.rect = tmp;
    }
    pub fn cut(&mut self, width: i32, height: i32, f: impl FnOnce(&mut Self)) {
        let rect = match self.rect.dir {
            CutDir::Horizontal => {
                let r = Rect {
                    x: self.rect.x,
                    y: self.rect.y,
                    width,
                    height,
                    dir: self.rect.dir,
                };
                self.rect.x += width;
                r
            }
            CutDir::Vertical => {
                let r = Rect {
                    x: self.rect.x,
                    y: self.rect.y,
                    width,
                    height,
                    dir: self.rect.dir,
                };
                self.rect.y += height;
                r
            }
        };
        self.rect.width -= width;
        self.rect.height -= height;

        let tmp = self.rect;
        self.rect = rect;
        f(self);
        self.rect = tmp;
    }

    pub fn clear(&mut self) {
        self.pix.buf.fill([0, 0, 0, 0]);
    }
    pub fn text(&mut self, s: &str) {
        Text::new(s).draw(self);
    }
}

pub trait Widget {
    fn draw(&mut self, canvas: &mut Canvas);
}

pub struct Text<'a> {
    pub text: &'a str,
    pub scale: i32,
    pub color: Color,
}

impl<'a> Text<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            scale: 2,
            color: [255, 255, 255, 255],
        }
    }
}

impl Widget for Text<'_> {
    fn draw(&mut self, canvas: &mut Canvas) {
        let len = canvas.font.draw(
            &mut canvas.pix,
            self.text,
            (
                canvas.rect.x,
                canvas.rect.y + canvas.font.height * self.scale,
            ),
            self.color,
            self.scale,
        );
        canvas.cut(
            len * canvas.font.width * self.scale,
            canvas.font.height * self.scale,
            |_| {},
        );
    }
}

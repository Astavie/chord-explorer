use crate::{font::Font, invert, Color, PixBuf};

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
}

impl Rect {
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.width && y < self.y + self.height
    }
}

pub struct Canvas<'a> {
    pub pix: PixBuf<'a>,
    pub rect: Rect,
    pub visuals: Visuals<'a>,
    pub events: Events,
}

pub struct Events {
    pub mouse_left: bool,
    pub mouse_middle: bool,
    pub mouse_right: bool,
    pub cursor: Option<(i32, i32)>,
}

#[derive(Clone)]
pub struct Visuals<'a> {
    pub font: &'a Font,
    pub text_size: i32,
    pub dir: CutDir,
    pub color: Color,
}

pub struct Tabs<'a, T: Tab> {
    pub selected: &'a mut T,
}

impl<'a, T: Tab> Tabs<'a, T> {
    pub fn new(selected: &'a mut T) -> Self {
        Self { selected }
    }
}

impl<T: Tab> Widget for Tabs<'_, T> {
    fn draw(&mut self, canvas: &mut Canvas) {
        let tabs = T::iter().collect::<Vec<_>>();
        let (width, height) = match canvas.visuals.dir {
            CutDir::Horizontal => (canvas.rect.width / tabs.len() as i32, canvas.rect.height),
            CutDir::Vertical => (canvas.rect.width, canvas.rect.height / tabs.len() as i32),
        };

        let selecting = canvas.mouse_left();
        for tab in tabs {
            canvas.cut(width, height, |canvas| {
                if canvas.mouse_left() {
                    *self.selected = tab;
                }

                if tab.eq(self.selected) && (!selecting || canvas.mouse_left()) {
                    canvas.fill(canvas.visuals.color);
                    canvas.visuals.color = invert(canvas.visuals.color);
                    tab.draw(canvas);
                    canvas.visuals.color = invert(canvas.visuals.color);
                } else {
                    tab.draw(canvas);
                }
            });
        }
    }
}

pub trait Tab: Clone + Copy + PartialEq + Eq {
    type Iterator: Iterator<Item = Self>;

    fn iter() -> Self::Iterator;
    fn name(&self) -> &str;

    fn draw(&self, canvas: &mut Canvas) {
        let len = canvas.visuals.font.len(self.name());
        canvas.center(
            len * canvas.visuals.font_width(),
            canvas.visuals.font_height() + canvas.visuals.text_size * 4,
            |canvas| {
                canvas.text(self.name());
            },
        )
    }
}

impl Visuals<'_> {
    pub fn font_height(&self) -> i32 {
        self.font.height * self.text_size
    }
    pub fn font_width(&self) -> i32 {
        self.font.width * self.text_size
    }
}

impl Canvas<'_> {
    pub fn hover(&self) -> bool {
        self.events
            .cursor
            .is_some_and(|(x, y)| self.rect.contains(x, y))
    }
    pub fn mouse_left(&self) -> bool {
        self.hover() && self.events.mouse_left
    }
    pub fn mouse_middle(&self) -> bool {
        self.hover() && self.events.mouse_middle
    }
    pub fn mouse_right(&self) -> bool {
        self.hover() && self.events.mouse_right
    }

    pub fn with_rect(&mut self, rect: Rect, f: impl FnOnce(&mut Self)) {
        let pushed_rect = self.rect;
        let pushed_vis = self.visuals.clone();

        self.rect = rect;
        f(self);

        self.visuals = pushed_vis;
        self.rect = pushed_rect;
    }
    pub fn center(&mut self, width: i32, height: i32, f: impl FnOnce(&mut Self)) {
        self.with_rect(
            Rect {
                x: self.rect.x + self.rect.width / 2 - width / 2,
                y: self.rect.y + self.rect.height / 2 - height / 2,
                width,
                height,
            },
            f,
        );
    }
    pub fn cut_top(&mut self, height: i32, f: impl FnOnce(&mut Self)) {
        let rect = Rect {
            x: self.rect.x,
            y: self.rect.y,
            width: self.rect.width,
            height,
        };
        self.rect.y += height;
        self.rect.height -= height;

        self.with_rect(rect, f);
    }
    pub fn cut(&mut self, width: i32, height: i32, f: impl FnOnce(&mut Self)) {
        let rect = match self.visuals.dir {
            CutDir::Horizontal => {
                let r = Rect {
                    x: self.rect.x,
                    y: self.rect.y,
                    width,
                    height,
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
                };
                self.rect.y += height;
                r
            }
        };
        self.rect.width -= width;
        self.rect.height -= height;

        self.with_rect(rect, f);
    }

    pub fn clear(&mut self) {
        self.pix.buf.fill([0, 0, 0, 0]);
    }
    pub fn fill(&mut self, color: Color) {
        for y in self.rect.y..self.rect.y + self.rect.height {
            let start = y * self.pix.width + self.rect.x;
            let end = start + self.rect.width;
            self.pix.buf[start as usize..end as usize].fill(color);
        }
    }
    pub fn text(&mut self, s: &str) {
        Text::new(s, self.visuals.text_size, self.visuals.color).draw(self);
    }
    pub fn tabs<T: Tab>(&mut self, selected: &mut T) {
        Tabs::new(selected).draw(self);
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
    pub fn new(text: &'a str, scale: i32, color: Color) -> Self {
        Self { text, scale, color }
    }
}

impl Widget for Text<'_> {
    fn draw(&mut self, canvas: &mut Canvas) {
        let len = canvas.visuals.font.draw(
            &mut canvas.pix,
            self.text,
            (
                canvas.rect.x,
                canvas.rect.y + canvas.visuals.font.height * self.scale,
            ),
            self.color,
            self.scale,
        );
        canvas.cut(
            len * canvas.visuals.font.width * self.scale,
            canvas.visuals.font.height * self.scale,
            |_| {},
        );
    }
}

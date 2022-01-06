use super::{
    buffer::{Buffer, Color, ColorCode},
    Position,
};
use alloc::{
    rc::Rc,
    vec::{self, Vec},
};
use bootloader::boot_info::{FrameBuffer, FrameBufferInfo, Optional};
use core::{
    fmt::{self, Write},
    ptr,
};
use font8x8::UnicodeFonts;

pub struct TextWriter<'a> {
    front: FrontBuffer<'a>,
    back: BackBuffer,
}

impl<'a> TextWriter<'a> {
    pub fn new(frame_buffer: &'static mut FrameBuffer) -> Self {
        Self {
            front: FrontBuffer::new(frame_buffer),
            back: BackBuffer::new(),
        }
    }

    pub fn try_new(frame_buffer: &'static mut Optional<FrameBuffer>) -> Result<Self, &'a str> {
        match frame_buffer {
            Optional::Some(frame_buffer) => Ok(Self::new(frame_buffer)),
            Optional::None => Err("Failed to acquire frame buffer handle"),
        }
    }

    pub fn write(&mut self, s: &'a str) {
        s.chars().for_each(|c| self.write_char(c));
    }

    pub fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),

            c => {
                self.front.put_char_current(c);
                self.increment_x();
            }
        }
    }

    pub fn clear(&mut self) {
        self.front.inner.clear();
    }

    fn newline(&mut self) {
        self.increment_y();
    }

    fn carriage_return(&mut self) {
        self.front.position.x = 0;
    }

    fn shift(&mut self) {
        self.back.shift();
        self.write_back_buffer()
    }

    fn write_back_buffer(&mut self) {
        for (y, row) in self.back.as_ref().iter().enumerate() {
            for (x, c) in row.iter().enumerate() {
                self.front.put_char(*c, Position::new(x, y));
            }
        }
    }

    fn increment_x(&mut self) {
        if self.front.position.x == self.front.dimensions.x {
            self.increment_y();
            return;
        }

        self.front.position.x += 1;
    }

    fn decrement_x(&mut self) {
        if self.front.position.x == 0 {
            self.decrement_y();
            return;
        }

        self.front.position.x -= 1;
    }

    fn increment_y(&mut self) {
        if self.front.position.y == self.front.dimensions.y {
            self.shift();
            return;
        }

        self.front.position.y += 1;
        self.front.position.x = 0;
    }

    fn decrement_y(&mut self) {
        if self.front.position.y == 0 {
            return;
        }

        self.front.position.y -= 1;
        self.front.position.x -= self.front.dimensions.x;
    }
}

struct FrontBuffer<'a> {
    inner: Buffer<'a>,
    dimensions: Position,
    position: Position,
}

impl<'a> FrontBuffer<'a> {
    pub fn new(frame_buffer: &'static mut FrameBuffer) -> Self {
        let info = frame_buffer.info();

        Self {
            inner: Buffer::new(frame_buffer),
            dimensions: Position::new(info.horizontal_resolution / 8, info.vertical_resolution / 8),
            position: Position::default(),
        }
    }

    fn put_char_current(&mut self, c: char) {
        self.put_char(c, self.position);
    }

    fn put_char(&mut self, c: char, position: Position) {
        let offset = self.get_offset(position);
        let rendered_char = match self.render_char(c) {
            Ok(rendered_char) => rendered_char,
            Err(err) => panic!("{err}"),
        };

        for (y, byte) in rendered_char.iter().enumerate() {
            for (x, bit) in (0..8).enumerate() {
                let color = if *byte & (1 << bit) == 0 {
                    ColorCode::Black
                } else {
                    ColorCode::White
                };
                self.inner.draw(
                    Position::new(offset.x + x, offset.y + y),
                    Color::from(color),
                );
            }
        }
    }

    fn render_char(&self, c: char) -> Result<[u8; 8], &'a str> {
        match font8x8::BASIC_FONTS.get(c) {
            Some(rendered_char) => Ok(rendered_char),
            None => Err("Invalid keycode"),
        }
    }

    fn get_offset(&self, position: Position) -> Position {
        Position::new(position.x * 8, position.y * 8)
    }
}

struct BackBuffer(Vec<Vec<char>>);

impl BackBuffer {
    fn new() -> Self {
        Self(Vec::from(Vec::new()))
    }

    fn shift(&mut self) {
        let inner = self.as_mut();
        if inner.len() == 0 {
            return;
        }

        inner.rotate_left(1);
        inner.pop();
    }
}

impl AsRef<Vec<Vec<char>>> for BackBuffer {
    fn as_ref(&self) -> &Vec<Vec<char>> {
        &self.0
    }
}

impl AsMut<Vec<Vec<char>>> for BackBuffer {
    fn as_mut(&mut self) -> &mut Vec<Vec<char>> {
        &mut self.0
    }
}

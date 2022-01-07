use super::{
    buffer::{Buffer, Color, ColorCode},
    Position,
};
use alloc::vec::Vec;
use bootloader::boot_info::{FrameBuffer, Optional};
use font8x8::UnicodeFonts;

pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

pub struct TextWriter {
    front: FrontBuffer,
    back: BackBuffer,
}

impl TextWriter {
    pub fn new(frame_buffer: &'static mut FrameBuffer) -> Self {
        let info = frame_buffer.info();
        let dimensions =
            Position::new(info.horizontal_resolution / 8, info.vertical_resolution / 8);
        Self {
            front: FrontBuffer::new(frame_buffer, dimensions),
            back: BackBuffer::new(dimensions),
        }
    }

    pub fn try_new(frame_buffer: &'static mut Optional<FrameBuffer>) -> Result<Self, &str> {
        match frame_buffer {
            Optional::Some(frame_buffer) => Ok(Self::new(frame_buffer)),
            Optional::None => Err("Failed to acquire frame buffer handle"),
        }
    }

    pub fn write_str(&mut self, s: &str) {
        s.chars().for_each(|c| self.write_char(c))
    }

    pub fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            '\t' => (0..4).into_iter().for_each(|_| self.write_char(' ')),
            c => {
                self.front.put_char_current(c);
                self.back.set(self.front.position, c);
                self.increment_x();
            }
        };
    }

    pub fn clear(&mut self) {
        self.front.clear();
        self.back.clear();
    }

    pub fn clear_last(&mut self) {
        self.decrement_x();
        self.front.put_char_current(' ');
        self.back.set(self.front.position, ' ');
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Left => self.decrement_x(),
            Direction::Right => self.increment_x(),
            Direction::Down => self.decrement_y(),
            Direction::Up => self.increment_y(),
        }
    }

    pub fn newline(&mut self) {
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
        let back_buffer = &self.back;
        (0..self.back.dimensions.x).for_each(|x| {
            (0..self.back.dimensions.y).for_each(|y| {
                let position = Position::new(x, y);
                if let Some(c) = back_buffer.get(position) {
                    self.front.put_char(*c, position);
                };
            });
        });
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

struct FrontBuffer {
    inner: Buffer<'static>,
    dimensions: Position,
    position: Position,
}

impl FrontBuffer {
    pub fn new(frame_buffer: &'static mut FrameBuffer, dimensions: Position) -> Self {
        Self {
            inner: Buffer::new(frame_buffer),
            dimensions,
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

        rendered_char.into_iter().enumerate().for_each(|(y, byte)| {
            (0..8).enumerate().for_each(|(x, bit)| {
                self.inner.draw(
                    Position::new(offset.x + x, offset.y + y),
                    Color::from(match byte & (1 << bit) {
                        0 => ColorCode::Black,
                        _ => ColorCode::White,
                    }),
                );
            });
        });
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    fn render_char(&self, c: char) -> Result<[u8; 8], &str> {
        match font8x8::BASIC_FONTS.get(c) {
            Some(rendered_char) => Ok(rendered_char),
            None => Err("Invalid keycode"),
        }
    }

    fn get_offset(&self, position: Position) -> Position {
        Position::new(position.x * 8, position.y * 8)
    }
}

struct BackBuffer {
    inner: Vec<Option<char>>,
    dimensions: Position,
}

impl BackBuffer {
    pub fn new(dimensions: Position) -> Self {
        let mut inner = Vec::with_capacity(dimensions.flat());
        inner.fill(None);
        Self { inner, dimensions }
    }

    pub fn set(&mut self, position: Position, c: char) {
        let index = self.index(position);
        self.as_mut().insert(index, Some(c))
    }

    pub fn get(&self, position: Position) -> &Option<char> {
        // Unwrap Safety: vec is filled
        self.as_ref().get(self.index(position)).unwrap()
    }

    pub fn clear(&mut self) {
        let max = self.dimensions.flat();
        let inner = self.as_mut();
        for i in 0..max {
            inner.insert(i, None);
        }
    }

    pub fn shift(&mut self) {
        let inner = self.as_ref();
        if inner.len() == 0 {
            return;
        }

        let dimensions = self.dimensions;
        let max = self.index(dimensions);
        let inner = self.as_mut();

        // TODO: ensure this is the first index of the last row
        inner.rotate_left(dimensions.x);
        (max - dimensions.x + 1..max).for_each(|i| inner.insert(i, None));
    }

    fn index(&self, position: Position) -> usize {
        (position.y * self.dimensions.x) + position.x
    }
}

impl AsRef<Vec<Option<char>>> for BackBuffer {
    fn as_ref(&self) -> &Vec<Option<char>> {
        &self.inner
    }
}

impl AsMut<Vec<Option<char>>> for BackBuffer {
    fn as_mut(&mut self) -> &mut Vec<Option<char>> {
        &mut self.inner
    }
}

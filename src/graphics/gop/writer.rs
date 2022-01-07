use super::{
    buffer::{Buffer, Color, ColorCode},
    Position,
};
use alloc::{
    boxed::Box,
    fmt::{self, Write},
    vec::Vec,
};
use bootloader::boot_info::{FrameBuffer, Optional};
use font8x8::UnicodeFonts;
use spin::Mutex;

/// A global `TextWriter` instance that can be used for printing text to the GOP buffer.
///
/// Used by the `print!` and `println!` macros.
static TEXT_WRITER: Mutex<Option<TextWriter>> = Mutex::new(None);

pub fn init_console(frame_buffer: &'static mut FrameBuffer) {
     *TEXT_WRITER.lock() = Some(TextWriter::new(frame_buffer));
}

pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

pub struct TextWriter {
    front: FrontBuffer,
    back: BackBuffer,
    position: Position,
    dimensions: Position,
}

impl TextWriter {
    pub fn new(frame_buffer: &'static mut FrameBuffer) -> Self {
        let info = frame_buffer.info();
        let dimensions =
            Position::new(info.horizontal_resolution / 8, info.vertical_resolution / 8);
        Self {
            front: FrontBuffer::new(frame_buffer),
            back: BackBuffer::new(dimensions),
            position: Position::default(),
            dimensions,
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
                self.front.put_char(c, self.position);
                self.back.set(self.position, Some(c));
                self.increment_x();
            }
        }
    }

    pub fn clear(&mut self) {
        self.front.clear();
        self.back.clear();
    }

    pub fn clear_last(&mut self) {
        self.decrement_x();
        self.front.put_char(' ', self.position);
        self.back.set(self.position, Some(' '));
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

    pub fn carriage_return(&mut self) {
        self.position.x = 0;
    }

    pub fn shift(&mut self) {
        self.back.shift();
        self.write_back_buffer()
    }

    fn write_back_buffer(&mut self) {
        let back_buffer = &self.back;
        (0..self.dimensions.x).for_each(|x| {
            (0..self.dimensions.y).for_each(|y| {
                let position = Position::new(x, y);
                if let Some(c) = back_buffer.get(position) {
                    self.front.put_char(c, position);
                };
            });
        });
    }

    fn increment_x(&mut self) {
        match self.position {
            Position { x, y: _ } if x == self.dimensions.x => self.increment_y(),
            Position { x, y } => self.position = Position::new(x + 1, y),
        }
    }

    fn decrement_x(&mut self) {
        match self.position {
            Position { x: 0, y: _ } => self.decrement_y(),
            Position { x, y } => self.position = Position::new(x - 1, y),
        }
    }

    fn increment_y(&mut self) {
        match self.position {
            Position { x: _, y } if y == self.dimensions.y => self.shift(),
            Position { x: _, y } => self.position = Position::new(0, y + 1),
        }
    }

    fn decrement_y(&mut self) {
        match self.position {
            Position { x: _, y: 0 } => return,
            Position { x: _, y } => self.position = Position::new(self.dimensions.x, y - 1),
        }
    }
}

impl AsRef<TextWriter> for TextWriter {
    fn as_ref(&self) -> &TextWriter {
        &self
    }
}

impl Write for TextWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.write_char(c);
        Ok(())
    }
}

struct FrontBuffer(Buffer<'static>);

impl FrontBuffer {
    pub fn new(frame_buffer: &'static mut FrameBuffer) -> Self {
        Self(Buffer::new(frame_buffer))
    }

    fn put_char(&mut self, c: char, position: Position) {
        let offset = self.get_offset(position);
        let rendered_char = match self.render_char(c) {
            Ok(rendered_char) => rendered_char,
            Err(err) => panic!("{err}"),
        };

        rendered_char.into_iter().enumerate().for_each(|(y, byte)| {
            (0..8).enumerate().for_each(|(x, bit)| {
                self.as_mut().draw(
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
        self.as_mut().clear();
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

impl AsRef<Buffer<'static>> for FrontBuffer {
    fn as_ref(&self) -> &Buffer<'static> {
        &self.0
    }
}

impl AsMut<Buffer<'static>> for FrontBuffer {
    fn as_mut(&mut self) -> &mut Buffer<'static> {
        &mut self.0
    }
}

#[derive(Clone)]
struct BackBuffer {
    inner: Vec<Option<char>>,
    capacity: usize,
    width: usize,
}

impl BackBuffer {
    pub fn new(dimensions: Position) -> Self {
        let capacity = dimensions.flat();

        Self {
            inner: vec![None; capacity],
            capacity,
            width: dimensions.x,
        }
    }

    // Unwrap Safety: vec is filled
    pub fn get(&self, position: Position) -> Option<char> {
        *self.as_ref().get(self.index(position)).unwrap()
    }

    pub fn set(&mut self, position: Position, c: Option<char>) {
        self.set_index(self.index(position), c);
    }

    pub fn set_index(&mut self, i: usize, c: Option<char>) {
        self.as_mut()[i] = c;
    }

    pub fn clear(&mut self) {
        (0..self.capacity).for_each(|i| self.set_index(i, None));
    }

    pub fn shift(&mut self) {
        // TODO: ensure this is the first index of the last row
        (0..self.width).for_each(|i| self.set_index(i, None));
        self.rotate_left(self.width);
    }

    fn rotate_left(&mut self, n: usize) {
        self.as_mut().rotate_left(n);
    }

    fn _rotate_right(&mut self, n: usize) {
        self.as_mut().rotate_right(n);
    }

    fn _len(&self) -> usize {
        self.as_ref().len()
    }

    fn index(&self, position: Position) -> usize {
        (position.y * self.width) + position.x
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

#[doc(hidden)]
pub unsafe fn _print(args: fmt::Arguments) {
    TEXT_WRITER.lock().as_mut().unwrap().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) =>  (unsafe { $crate::graphics::gop::writer::_print(format_args!($($arg)*)) });
}

/// Like the `println!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn write_char_succeeds() {}
}
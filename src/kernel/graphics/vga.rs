use crate::io::outb;
use core::{
    fmt::{self, Write},
    ops::{Deref, DerefMut},
};
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

lazy_static! {
    /// A global `Writer` instance that can be used for printing to the VGA text buffer.
    ///
    /// Used by the `print!` and `println!` macros.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// A combination of a foreground and a background color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    /// Create a new `ColorCode` with the given foreground and background colors.
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// A screen character in the VGA text buffer, consisting of an ASCII character and a `ColorCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct Char {
    ascii_char: u8,
    color_code: ColorCode,
}

impl Char {
    pub fn new(ascii_char: u8, color_code: ColorCode) -> Self {
        Self {
            ascii_char,
            color_code,
        }
    }

    pub fn blank(color_code: ColorCode) -> Self {
        Self::new(b' ', color_code)
    }
}

impl Deref for Char {
    type Target = Char;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl DerefMut for Char {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self {
        self
    }
}

/// A structure representing the VGA text buffer.
#[repr(transparent)]
pub struct Buffer {
    video_mem: [[Volatile<Char>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// A writer type that allows writing ASCII bytes and strings to an underlying `Buffer`.
///
/// Wraps lines at `BUFFER_WIDTH`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct Writer {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

#[allow(dead_code)]
impl Writer {
    /// Writes an ASCII byte to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                self.put_byte(byte);
                self.increment_column();
            }
        }
    }

    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character. Does **not**
    /// support strings with non-ASCII characters, since they can't be printed in the VGA text
    /// mode.
    fn write_string(&mut self, s: &str) {
        s.bytes().into_iter().for_each(|byte| match byte {
            // printable ASCII byte or newline
            0x20..=0x7e | b'\n' => self.write_byte(byte),
            // not part of printable ASCII range
            _ => self.write_byte(0xfe),
        });
    }

    fn make_char(&self, ascii_char: u8) -> Char {
        Char::new(ascii_char, self.color_code)
    }

    fn get_char_at(&mut self, row: usize, col: usize) -> Char {
        self.buffer.video_mem[row][col].read()
    }

    fn get_char(&mut self) -> Char {
        self.get_char_at(self.row_position, self.column_position)
    }

    fn put_char_at(&mut self, c: Char, row: usize, col: usize) {
        self.buffer.video_mem[row][col].write(c);
    }

    fn put_char(&mut self, c: Char) {
        self.put_char_at(c, self.row_position, self.column_position);
    }

    fn put_byte(&mut self, byte: u8) {
        let char = self.make_char(byte);
        self.put_char(char);
    }

    fn put_char_color_at(&mut self, c: u8, color_code: ColorCode, row: usize, col: usize) {
        let char = Char::new(c, color_code);
        self.put_char_at(char, row, col);
    }

    fn put_char_color(&mut self, c: u8, color_code: ColorCode) {
        self.put_char_color_at(c, color_code, self.row_position, self.column_position);
    }

    /// Sets the buffer row to the next line, shifting the buffer up if it is at the end.
    fn new_line(&mut self) {
        self.column_position = 0;
        if self.row_position == BUFFER_HEIGHT - 1 {
            self.shift_buffer();
            return;
        }

        self.row_position += 1;
    }

    /// Shifts all lines one line up and clears the last row.
    fn shift_buffer(&mut self) {
        (1..BUFFER_HEIGHT).into_iter().for_each(|row| {
            (0..BUFFER_WIDTH).into_iter().for_each(|col| {
                let char = self.get_char_at(row, col);
                self.put_char_at(char, row - 1, col);
            })
        });

        self.clear_row(BUFFER_HEIGHT - 1);
    }

    /// Shifts the buffer n times.
    fn shift_buffer_n(&mut self, n: usize) {
        (0..n).into_iter().for_each(|_| self.shift_buffer());
    }

    fn increment_row(&mut self) {
        if self.row_position == BUFFER_HEIGHT - 1 {
            return;
        }

        self.row_position += 1;
    }

    fn decrement_row(&mut self) {
        if self.row_position == 0 {
            return;
        }

        self.row_position -= 1;
    }

    fn increment_column(&mut self) {
        if self.column_position == BUFFER_WIDTH - 1 {
            self.new_line();
            return;
        };

        self.column_position += 1;
    }

    fn decrement_column(&mut self) {
        if self.column_position == 0 {
            if self.row_position == 0 {
                return;
            }

            self.column_position = BUFFER_WIDTH - 1;
            self.row_position -= 1;
            return;
        }

        self.column_position -= 1;
    }

    fn update_cursor(&self) {
        let position = self.row_position * BUFFER_WIDTH + self.column_position;
        unsafe {
            outb(0x3d4, 0x0f);
            outb(0x3d5, position as u8);
            outb(0x3d4, 0x0e);
            outb(0x3d5, (position >> 8) as u8);
        }
    }

    /// Fills the buffer with blank characters.
    fn clear_screen(&mut self) {
        (0..BUFFER_HEIGHT).into_iter().for_each(|row| {
            (0..BUFFER_WIDTH).into_iter().for_each(|col| {
                let char = Char::blank(self.color_code);
                self.put_char_at(char, row, col);
            })
        });

        self.column_position = 0;
        self.row_position = 0;
    }

    /// Clears a row by overwriting it with blank characters.
    fn clear_row(&mut self, row: usize) {
        let blank = Char::blank(self.color_code);
        (0..BUFFER_WIDTH)
            .into_iter()
            .for_each(|col| self.put_char_at(blank, row, col));
    }

    /// Clears the character at the previous position in the buffer.
    fn clear_last(&mut self) {
        self.decrement_column();
        self.clear_current();
    }

    /// Sets the current buffer position to a blank Char
    fn clear_current(&mut self) {
        self.put_char(Char::blank(self.color_code));
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Like the `print!` macro in the standard library, but prints to the VGA text buffer.
// #[macro_export]
// macro_rules! print {
//     ($($arg:tt)*) => ($crate::graphics::vga::_print(format_args!($($arg)*)));
// }

// /// Like the `println!` macro in the standard library, but prints to the VGA text buffer.
// #[macro_export]
// macro_rules! println {
//     () => ($crate::print!("\n"));
//     ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
// }

/// Prints the given formatted string to the VGA text buffer through the global `WRITER` instance.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let mut w = WRITER.lock();
    w.write_fmt(args).unwrap();
    w.update_cursor();
}

/// An extraordinarily sinful method, i know
#[doc(hidden)]
pub fn newline() {
    let mut w = WRITER.lock();
    w.new_line();
    w.update_cursor();
}

/// An extraordinarily sinful method, i know
#[doc(hidden)]
pub fn tabline() {
    let mut w = WRITER.lock();
    w.write_fmt(format_args!("  ")).unwrap();
    w.update_cursor();
}

/// Clears the last printed char in the buffer
#[doc(hidden)]
pub fn clear_last() {
    let mut w = WRITER.lock();
    w.clear_last();
    w.update_cursor();
}

pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Handles arrow keys, moving the cursor to the appropriate buffer position
#[doc(hidden)]
pub fn move_cursor(direction: Direction) {
    let mut w = WRITER.lock();
    match direction {
        Direction::Left => w.decrement_column(),
        Direction::Right => w.increment_column(),
        Direction::Up => w.decrement_row(),
        Direction::Down => w.increment_row(),
    }
    w.update_cursor();
}

// #[cfg(test)]
// mod tests {
// use super::{BUFFER_HEIGHT, WRITER};

// #[test_case]
// fn test_println() {
//     println!("test_println output");
// }

// #[test_case]
// fn test_println_many() {
//     for _ in 0..200 {
//         println!("test_println_many output");
//     }
// }

// #[test_case]
// fn test_println_output() {
//     use core::fmt::Write;
//     use x86_64::instructions::interrupts;

//     let s = "Some test string that fits on a single line";
//     interrupts::without_interrupts(|| {
//         let mut writer = WRITER.lock();
//         writeln!(writer, "\n{}", s).expect("writeln failed");
//         for (i, c) in s.chars().enumerate() {
//             let char = writer.get_char_at(BUFFER_HEIGHT - 2, i);
//             assert_eq!(char::from(char.ascii_char), c);
//         }
//     });
// }
// }

use crate::graphics::{Color, Font, GraphicsDevice, Pixel, PixelMap};
use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::{BTreeMap, LinkedList},
    rc::Rc,
    string::String,
    sync::Arc,
    vec::Vec,
};
use core::{
    cell::{RefCell, RefMut},
    fmt::{self, Write},
    mem::MaybeUninit,
};
use lazy_static::lazy_static;
use rusttype::Point;
use spin::Mutex;

const DEFAULT_FONT_SIZE: usize = 28;

lazy_static! {
    static ref TERMINAL: Arc<Mutex<Option<Terminal<'static>>>> = Arc::new(Mutex::new(None));
}

pub fn initialize(graphics_device: Rc<RefCell<dyn GraphicsDevice>>) {
    *TERMINAL.lock() = Some(Terminal::new(graphics_device, DEFAULT_FONT_SIZE));
    clear();
}

#[doc(hidden)]
pub fn print(args: fmt::Arguments) {
    TERMINAL.lock().as_mut().unwrap().write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn clear() {
    TERMINAL.lock().as_ref().unwrap().clear();
}

#[macro_export]
macro_rules! clear {
    () => {
        $crate::terminal::clear()
    };
}

#[macro_export]
macro_rules! print {
    () => ($crate::terminal::print(""));
    ($($arg:tt)*) => ($crate::terminal::print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub struct Terminal<'a> {
    size: Point<usize>,
    cursor: Point<usize>,
    background: Color,
    foreground: Color,
    font_size: usize,
    backend: TerminalBackend<'a>,

    prompt: String,
    command: String,
    back_buffer: TerminalBackBuffer,
}

impl<'a> Terminal<'a> {
    pub fn new(device: Rc<RefCell<dyn GraphicsDevice>>, font_size: usize) -> Self {
        let device_ref = device.borrow();
        let cursor = Point { x: 0, y: 0 };
        let size = Point {
            x: device_ref.width(),
            y: device_ref.height(),
        };

        drop(device_ref);
        let backend = TerminalBackend::new(device, font_size);

        Self {
            size,
            cursor,
            background: Color::Black,
            foreground: Color::White,
            font_size,
            backend,

            prompt: "".to_owned(),
            command: "".to_owned(),
            back_buffer: TerminalBackBuffer::default(),
        }
    }

    pub fn clear(&self) {
        self.backend.clear(self.background);
    }

    fn line_height(&self) -> usize {
        self.backend.font.height()
    }

    fn newline(&mut self) {
        self.cursor.x = 0;
        self.cursor.y += self.line_height();
    }
}

impl<'a> core::fmt::Write for Terminal<'a> {
    fn write_fmt(mut self: &mut Self, args: fmt::Arguments<'_>) -> fmt::Result {
        fmt::write(&mut self, args)
    }

    fn write_str(&mut self, data: &str) -> fmt::Result {
        data.chars().try_for_each(|char| self.write_char(char))
    }

    fn write_char(&mut self, char: char) -> fmt::Result {
        match char {
            '\n' => self.newline(),
            _ => {
                let pixel_map = self.backend.render_character(char);
                let glyph_width = pixel_map.dimensions.x;
                if self.cursor.x + pixel_map.dimensions.x > self.size.x {
                    self.cursor.x = 0;
                    // TODO: scroll if we're at the bottom of the terminal
                    self.cursor.y += self.line_height();
                }

                self.backend
                    .write_character(pixel_map, self.cursor.x as i32, self.cursor.y as i32);
                self.cursor.x += glyph_width;
            }
        }

        Ok(())
    }
}

pub struct TerminalBackend<'a> {
    font: Font<'a>,
    render_cache: Rc<RefCell<BTreeMap<char, PixelMap>>>,
    device: Rc<RefCell<dyn GraphicsDevice>>,
}

impl<'a> TerminalBackend<'a> {
    pub fn new(device: Rc<RefCell<dyn GraphicsDevice>>, font_size: usize) -> Self {
        // let background = Color::Black;
        // let foreground = Color::White;

        // let mut device_ref = device.borrow_mut();
        // let cursor_position = Point { x: 0, y: 0 };
        // let dimensions = Point {
        //     x: device_ref.width(),
        //     y: device_ref.height(),
        // };

        // device_ref.fill(background);
        // drop(device_ref);

        let font = Font::new(
            include_bytes!("../../data/fonts/open-sans/OpenSans-Regular.ttf"),
            // include_bytes!("../../data/fonts/unifont-15.0.01.otf"),
            font_size,
        )
        .unwrap();
        let render_cache = Rc::new(RefCell::new(BTreeMap::new()));

        Self {
            font,
            render_cache,
            device,
        }
    }

    pub fn update_font_size(&mut self, font_size: usize) {
        self.render_cache.borrow_mut().clear();
        self.font.update_height(font_size);
    }

    pub fn width(&self) -> usize {
        self.device.borrow().width()
    }

    pub fn height(&self) -> usize {
        self.device.borrow().width()
    }

    pub fn clear(&self, color: Color) {
        self.device.borrow_mut().fill(color);
    }

    // writes a character, returing the dimensions of the glyph so we can calcualte offsets for the next character
    pub fn write_character(&self, pixel_map: PixelMap, x_offset: i32, y_offset: i32) {
        let mut device_ref = self.device.borrow_mut();
        pixel_map.iter().for_each(|Pixel { position, color }| {
            let x = (position.x + x_offset) as usize;
            let y = (position.y + y_offset) as usize;

            device_ref.set_pixel(x, y, Color::Rgb(*color))
        });
    }

    pub fn render_character(&self, char: char) -> PixelMap {
        let mut cache_ref = self.render_cache.borrow_mut();
        match cache_ref.get(&char) {
            Some(map) => map.clone(),
            None => {
                let pixel_map = self.font.rasterize(char);
                cache_ref.insert(char, pixel_map);

                // UNWRAP: we just inserted the bitmap into the cache if it wasn't there, so we know it exists
                cache_ref.get(&char).unwrap().clone()
            }
        }
    }
}

// SAFETY: Terminals will only exist as static references behind a mutex and will need to be locked for access
unsafe impl<'a> Send for TerminalBackend<'a> {}

pub struct Span<T: PartialEq + PartialOrd> {
    pub start: T,
    pub end: T,
}

impl<T: PartialEq + PartialOrd> Span<T> {
    pub fn new(start: T, end: T) -> Self {
        Self { start, end }
    }
}

struct TerminalBackBuffer {
    history: Capped<LinkedList<Box<str>>>,
    buffer: Capped<Vec<Box<str>>>,
}

impl TerminalBackBuffer {
    pub fn new(history_capacity: usize, buffer_capacity: usize) -> Self {
        Self {
            history: (history_capacity, LinkedList::new()),
            buffer: (buffer_capacity, vec![]),
        }
    }

    pub fn push_command(&mut self, prompt: String, command: String) {
        // Remove a line from the back of the buffer if it's full
        if self.buffer.1.len() > self.buffer.0 {
            self.buffer.1.remove(0);
        }
        self.buffer
            .1
            .push(format!("{prompt}{command}").into_boxed_str());

        // Remove a command from history if it's full
        if self.history.1.len() == self.history.0 {
            self.history.1.pop_front();
        }
        self.history.1.push_back(command.into_boxed_str())
    }
}

impl Default for TerminalBackBuffer {
    fn default() -> Self {
        Self::new(250, 50)
    }
}

type Capped<T> = (usize, T);

use alloc::vec::Vec;
use bootloader::boot_info::{FrameBuffer, FrameBufferInfo, Optional, PixelFormat};
use lazy_static::lazy_static;
// use spin::Mutex;

pub struct Position {
    x: usize,
    y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

pub enum ColorCode {
    RGB(u32),
    BGR(u32),
    Grayscale(u8),
}

impl ColorCode {
    pub fn format(&self) -> [u8; 4] {
        match self {
            ColorCode::RGB(color_code) => [
                (color_code >> 24) as u8,
                (color_code >> 16) as u8,
                (color_code >> 8) as u8,
                0_u8,
            ],
            ColorCode::BGR(color_code) => [
                (color_code >> 8) as u8,
                (color_code >> 16) as u8,
                (color_code >> 24) as u8,
                0_u8,
            ],
            ColorCode::Grayscale(byte) => [*byte, *byte, *byte, 0_u8],
        }
    }
}

pub struct Writer<'a> {
    frame_buffer: &'a mut FrameBuffer,
    info: FrameBufferInfo,
    position: Position,
}

impl<'a> Writer<'a> {
    pub fn new(frame_buffer: &'a mut FrameBuffer) -> Self {
        let info = frame_buffer.info();
        Self {
            frame_buffer,
            info,
            position: Position::default(),
        }
    }

    pub fn try_new(frame_buffer: &'a mut Optional<FrameBuffer>) -> Result<Self, &'a str> {
        match frame_buffer {
            Optional::Some(frame_buffer) => Ok(Self::new(frame_buffer)),
            Optional::None => Err("Failed to acquire frame buffer handle"),
        }
    }

    pub fn draw(&mut self, position: Position, color_code: &ColorCode) {
        let pixel = self.flatten_position(position);
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let buffer = self.as_mut();
        let formatted_color = color_code.format();

        match bytes_per_pixel {
            4 => {
                formatted_color
                    .into_iter()
                    .enumerate()
                    .for_each(|(i, b)| buffer[pixel + i] = b);
            }
            3 => {
                formatted_color[1..=3]
                    .into_iter()
                    .enumerate()
                    .for_each(|(i, b)| buffer[pixel + i] = *b);
            }
            _ => buffer[pixel] = formatted_color[0],
        };
    }

    pub fn fill(&mut self, color_code: &ColorCode) {
        (0..self.info.horizontal_resolution).for_each(|y| {
            (0..self.info.vertical_resolution)
                .for_each(|x| self.draw(Position::new(x, y), color_code))
        });
    }

    pub fn clear(&mut self) {
        let color_code = match self.info.pixel_format {
            PixelFormat::RGB => ColorCode::RGB(0x00000000),
            PixelFormat::BGR => ColorCode::BGR(0x00000000),
            _ => ColorCode::Grayscale(0x00),
        };
        self.fill(&color_code);
    }

    /// Gets the physical inxed of a framebuffer pixel
    ///
    /// multiplies the virtual position by our framebuffer's bytes per pixel
    fn flatten_position(&self, position: Position) -> usize {
        self.info.bytes_per_pixel * (self.info.horizontal_resolution * position.y + position.x)
    }
}

impl<'a> AsRef<[u8]> for Writer<'a> {
    fn as_ref(&self) -> &[u8] {
        self.frame_buffer.buffer()
    }
}

impl<'a> AsMut<[u8]> for Writer<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.frame_buffer.buffer_mut()
    }
}

// turn the screen gray
// if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
//     for byte in framebuffer.buffer_mut() {
//         *byte = 0x90;
//     }
// };

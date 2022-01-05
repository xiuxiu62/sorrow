use bootloader::boot_info::{FrameBuffer, FrameBufferInfo, Optional, PixelFormat};

use crate::io::insb;

// Color scheme in RGB
#[derive(Clone, Copy)]
pub enum ColorCode {
    Black = 0x00_00_00_00,
    Gray = 0x22_22_22_00,
    Blue = 0x00_00_ff_00,
    Green = 0x00_ff_00_00,
    // Cyan = 3,
    Red = 0xff_00_00_00,
    Magenta = 0xff_00_ff_00,
    // Brown = 6,
    // LightGray = 7,
    // DarkGray = 8,
    // LightBlue = 9,
    // LightGreen = 10,
    // LightCyan = 11,
    // LightRed = 12,
    // Pink = 13,
    // Yellow = 14,
    White = 0xff_ff_ff_00,
}

#[derive(Clone, Copy)]
pub struct Color(u32);

impl Color {
    pub fn new(color_code: ColorCode) -> Self {
        Self(color_code as u32)
    }

    pub fn format_as_bgr(&self) -> [u8; 4] {
        let mut buf = self.format();
        buf.swap(0, 2);
        buf
    }

    pub fn format(&self) -> [u8; 4] {
        let color_code = self.0;
        [
            (color_code >> 24) as u8,
            (color_code >> 16) as u8,
            (color_code >> 8) as u8,
            0_u8,
        ]
    }
}

impl From<ColorCode> for Color {
    fn from(c: ColorCode) -> Self {
        Self(c as u32)
    }
}

pub struct Coordinates {
    x: usize,
    y: usize,
}

impl Coordinates {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl Default for Coordinates {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

pub struct Writer<'a> {
    frame_buffer: &'a mut FrameBuffer,
    info: FrameBufferInfo,
}

impl<'a> Writer<'a> {
    pub fn new(frame_buffer: &'a mut FrameBuffer) -> Self {
        let info = frame_buffer.info();
        Self { frame_buffer, info }
    }

    pub fn try_new(frame_buffer: &'a mut Optional<FrameBuffer>) -> Result<Self, &'a str> {
        match frame_buffer {
            Optional::Some(frame_buffer) => Ok(Self::new(frame_buffer)),
            Optional::None => Err("Failed to acquire frame buffer handle"),
        }
    }

    // TODO: ensure dimensions are within the 2d bounds of the buffer
    pub fn draw_rectangle(
        &mut self,
        start_position: Coordinates,
        dimensions: Coordinates,
        color_code: Color,
    ) {
        (start_position.x..(start_position.x + dimensions.x)).for_each(|x| {
            (start_position.y..(start_position.y + dimensions.y))
                .for_each(|y| self.draw(Coordinates::new(x, y), color_code))
        })
    }

    pub fn draw(&mut self, position: Coordinates, color: Color) {
        let offset = self.get_offset(position);
        let bytes_per_pixel = self.info.bytes_per_pixel;

        // Format color code to BGR if supported by system
        let pixel_format = self.info.pixel_format;
        let color_formatted = if pixel_format == PixelFormat::BGR {
            color.format_as_bgr()
        } else {
            color.format()
        };

        // Write bytes to the framebuffer
        self.as_mut()[offset..(offset + bytes_per_pixel)]
            .copy_from_slice(&color_formatted[..bytes_per_pixel]);
    }

    pub fn fill(&mut self, color: Color) {
        for x in 0..self.info.horizontal_resolution {
            for y in 0..(self.info.vertical_resolution) {
                self.draw(Coordinates::new(x, y), color)
            }
        }
    }

    pub fn clear(&mut self) {
        self.fill(Color::from(ColorCode::Gray));
    }

    /// Gets the physical index of a framebuffer pixel
    ///
    /// multiplies the virtual position by our framebuffer's bytes per pixel
    #[inline]
    fn get_offset(&self, position: Coordinates) -> usize {
        (self.info.stride * self.info.bytes_per_pixel * position.y)
            + (self.info.bytes_per_pixel * position.x)
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

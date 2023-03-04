use super::{Color, GraphicsDevice};
use bootloader_api::info::FrameBuffer;

pub struct GopDevice<'a> {
    buffer: &'a mut [u8],
    width: usize,
    height: usize,
    pitch: usize,
    pixel_bytes: usize,
    pixel_format: PixelFormat,
}

impl<'a> GraphicsDevice for GopDevice<'a> {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.width
    }

    fn pitch(&self) -> usize {
        self.pitch
    }

    fn pixel_bytes(&self) -> usize {
        self.pixel_bytes
    }

    fn set_byte(&mut self, i: usize, value: u8) {
        // let offset = self.pixel_start_offset(i % self.width, i / self.width);
        self.buffer[i] = value;
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        let offset = self.pixel_start_offset(x, y);
        let color = self.format_color(color);

        self.buffer[offset] = (color >> 24) as u8;
        self.buffer[offset + 1] = (color >> 16) as u8;
        self.buffer[offset + 2] = (color >> 8) as u8;
        self.buffer[offset + 3] = (color >> 0) as u8;
    }

    fn fill(&mut self, color: Color) {
        (0..self.height).for_each(|y| (0..self.width).for_each(|x| self.set_pixel(x, y, color)));
    }
}

impl<'a> GopDevice<'a> {
    pub fn new(frame_buffer: Option<&'a mut FrameBuffer>) -> Option<Self> {
        frame_buffer.and_then(|framer_buffer| {
            let info = framer_buffer.info();

            Some(Self {
                buffer: framer_buffer.buffer_mut(),
                width: info.width,
                height: info.height,
                pitch: info.stride * 4,
                pixel_bytes: info.bytes_per_pixel,
                pixel_format: PixelFormat::from(info.pixel_format),
            })
        })
    }

    pub fn draw_square(&mut self, x: usize, y: usize, size: usize, color: Color) {
        (x..x + size).for_each(|x| (y..y + size).for_each(|y| self.set_pixel(x, y, color)));
    }

    fn pixel_start_offset(&self, x: usize, y: usize) -> usize {
        y * self.pitch + x * self.pixel_bytes
    }

    fn format_color(&self, color: Color) -> u32 {
        match self.pixel_format {
            PixelFormat::Rgb => color.as_rgb(),
            PixelFormat::Bgr => color.as_bgr(),
        }
    }
}

pub enum PixelFormat {
    Rgb,
    Bgr,
}

impl From<bootloader_api::info::PixelFormat> for PixelFormat {
    fn from(value: bootloader_api::info::PixelFormat) -> Self {
        match value {
            bootloader_api::info::PixelFormat::Rgb => Self::Rgb,
            bootloader_api::info::PixelFormat::Bgr => Self::Bgr,
            _ => unimplemented!("Other color formats are not implemented"),
        }
    }
}

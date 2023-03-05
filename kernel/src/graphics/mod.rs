mod color;
mod font;
mod gop;

pub use color::Color;
pub use font::{Font, Pixel, PixelMap};
pub use gop::GopDevice;

pub trait GraphicsDevice {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn pitch(&self) -> usize;
    fn pixel_bytes(&self) -> usize;

    fn set_byte(&mut self, offset: usize, value: u8);
    // fn set_byte(&mut self, x: usize, y: usize, value: u8);
    fn set_pixel(&mut self, x: usize, y: usize, color: Color);

    fn fill(&mut self, color: Color);
}

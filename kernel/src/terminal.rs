use crate::graphics::{Color, Font, GraphicsDevice};
use alloc::rc::Rc;
use core::cell::RefCell;

pub struct Terminal<'a> {
    graphic_device: Rc<RefCell<dyn GraphicsDevice>>,
    font: Font<'a>,
    background: Color,
    foreground: Color,
}

impl<'a> Terminal<'a> {
    pub fn new(graphic_device: Rc<RefCell<dyn GraphicsDevice>>) -> Self {
        let font = Font::new(
            include_bytes!("../../data/fonts/open-sans/OpenSans-Regular.ttf"),
            28,
        )
        .unwrap();
        let background = Color::Black;
        let foreground = Color::White;
        graphic_device.borrow_mut().fill(background);

        Self {
            graphic_device,
            font,
            background,
            foreground,
        }
    }

    pub fn width(&self) -> usize {
        self.graphic_device.borrow().width()
    }

    pub fn height(&self) -> usize {
        self.graphic_device.borrow().width()
    }

    pub fn clear(&self) {
        self.graphic_device.borrow_mut().fill(self.background);
    }

    // writes a character, returing the width and height of the glyph
    pub fn write_char(&self, x_offset: usize, y_offset: usize, char: char) -> (usize, usize) {
        let mut graphics_device_ref = self.graphic_device.borrow_mut();
        let pixel_map = self.font.rasterize(char);
        pixel_map
            .as_ref()
            .chunks(pixel_map.width)
            .enumerate()
            .for_each(|(y, row)| {
                row.iter()
                    .enumerate()
                    .filter(|(_, alpha)| **alpha > 0)
                    .for_each(|(x, _)| {
                        graphics_device_ref.set_pixel(x + x_offset, y + y_offset, self.foreground)
                    });
            });

        (pixel_map.width, pixel_map.height)
    }
}

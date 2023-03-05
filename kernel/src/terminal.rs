use crate::graphics::{Color, Font, GraphicsDevice, Pixel, PixelMap};
use alloc::{collections::BTreeMap, rc::Rc};
use core::cell::RefCell;
use rusttype::Point;

pub struct Terminal<'a> {
    graphic_device: Rc<RefCell<dyn GraphicsDevice>>,
    font: Font<'a>,
    background: Color,
    foreground: Color,
    font_size: usize,
    render_cache: BTreeMap<char, PixelMap>,
}

impl<'a> Terminal<'a> {
    pub fn new(graphic_device: Rc<RefCell<dyn GraphicsDevice>>, font_size: usize) -> Self {
        let font = Font::new(
            include_bytes!("../../data/fonts/open-sans/OpenSans-Regular.ttf"),
            font_size,
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
            font_size,
            render_cache: BTreeMap::new(),
        }
    }

    pub fn update_font_size(&mut self, font_size: usize) {
        self.render_cache.clear();
        self.font.update_height(font_size);
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

    // writes a character, returing the dimensions of the glyph so we can calcualte offsets for the next character
    pub fn write_char(&mut self, x_offset: i32, y_offset: i32, char: char) -> Point<usize> {
        let mut graphics_device_ref = self.graphic_device.borrow_mut();
        let pixel_map = match self.render_cache.get(&char) {
            Some(map) => map,
            None => {
                let pixel_map = self.font.rasterize(char);
                self.render_cache.insert(char, pixel_map);

                // UNWRAP: we just inserted this into the map, so we know it exists
                self.render_cache.get(&char).unwrap()
            }
        };

        pixel_map.iter().for_each(|Pixel { position, color }| {
            let x = (position.x + x_offset) as usize;
            let y = (position.y + y_offset) as usize;

            graphics_device_ref.set_pixel(x, y, Color::Rgb(*color))
        });

        pixel_map.dimensions
    }
}

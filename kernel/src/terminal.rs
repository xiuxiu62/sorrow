use crate::graphics::{Color, Font, GraphicsDevice, PixelMap};
use alloc::{collections::BTreeMap, rc::Rc};
use core::cell::RefCell;

pub struct Terminal<'a> {
    graphic_device: Rc<RefCell<dyn GraphicsDevice>>,
    font: Font<'a>,
    background: Color,
    foreground: Color,
    bitmap_cache: BTreeMap<char, PixelMap>,
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
            bitmap_cache: BTreeMap::new(),
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
    pub fn write_char(&mut self, x_offset: usize, y_offset: usize, char: char) -> (usize, usize) {
        let mut graphics_device_ref = self.graphic_device.borrow_mut();
        let pixel_map = match self.bitmap_cache.get(&char) {
            Some(map) => map,
            None => {
                let pixel_map = self.font.rasterize(char);
                self.bitmap_cache.insert(char, pixel_map);

                // UNWRAP: we just inserted this into the map, so we know it exists
                self.bitmap_cache.get(&char).unwrap()
            }
        };

        pixel_map
            .as_ref()
            .chunks(pixel_map.width)
            .enumerate()
            .for_each(|(y, row)| {
                row.iter()
                    .enumerate()
                    .filter(|(_, alpha)| **alpha > 0)
                    .for_each(|(x, pixel)| {
                        graphics_device_ref.set_pixel(
                            x + x_offset,
                            y + y_offset,
                            Color::Rgb(*pixel),
                        )
                    });
            });

        (pixel_map.width, pixel_map.height)
    }
}

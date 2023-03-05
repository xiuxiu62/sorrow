use alloc::vec::Vec;
use rusttype::{Point, PositionedGlyph, Scale};

// A 2-dimensional map of grayscale pixels
pub struct PixelMap {
    inner: Vec<u32>,
    pub width: usize,
    pub height: usize,
}

impl PixelMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            inner: vec![0; width * height],
            width,
            height,
        }
    }

    pub fn as_ref(&self) -> &[u32] {
        &self.inner
    }
}

pub struct Font<'a> {
    inner: rusttype::Font<'a>,
    size: usize,
    scale: Scale,
    offset: Point<f32>,
}

impl<'a> Font<'a> {
    pub fn new(font_data: &'a [u8], size: usize) -> Option<Self> {
        rusttype::Font::try_from_bytes(font_data).map(|inner| {
            let scale = Scale {
                x: size as f32,
                y: size as f32,
            };
            let v_metrics = inner.v_metrics(scale);
            let offset = rusttype::point(0.0, v_metrics.ascent);

            Self {
                inner,
                scale,
                offset,
                size,
            }
        })
    }

    pub fn update_size(&mut self, size: usize) {
        let scale = Scale {
            x: size as f32,
            y: size as f32,
        };
        let v_metrics = self.inner.v_metrics(scale);

        self.offset = rusttype::point(0.0, v_metrics.ascent);
        self.size = size;
    }

    pub fn rasterize(&self, char: char) -> PixelMap {
        let glyph = self
            .inner
            .glyph(char)
            .scaled(self.scale)
            .positioned(self.offset);
        let width = (glyph.position().x + glyph.unpositioned().h_metrics().advance_width) as u32;

        let mut pixel_map = PixelMap::new(width as usize, self.size);
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|mut x, mut y, v| {
                let alpha = (v * 255.0) as u8;
                let color = (alpha as u32) << 24 | (alpha as u32) << 16 | (alpha as u32) << 8;

                x += bounding_box.min.x as u32;
                y += bounding_box.min.y as u32;
                pixel_map.inner[(x + y * width) as usize] = color;
            });
        }

        pixel_map
    }

    // Finds the most visually pleasing width to display from a slice of glyphs
    fn ideal_glyph_width(glyphs: &[PositionedGlyph]) -> usize {
        glyphs
            .iter()
            .rev()
            .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
            .next()
            .unwrap_or(0.0) as usize
    }
}

impl<'a> Default for Font<'a> {
    fn default() -> Self {
        Self::new(include_bytes!("../../../data/fonts/Roboto-Regular.ttf"), 12).unwrap()
    }
}

// let glyphs: Vec<PositionedGlyph> = font.layout("hello world", scale, offset).collect();

// Find the most visually pleasing width to display
// let width = glyphs
//     .iter()
//     .rev()
//     .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
//     .next()
//     .unwrap_or(0.0) as usize;

// .ceil();
// }

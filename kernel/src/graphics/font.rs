use alloc::{slice, vec::Vec};
use rusttype::{Point, Scale};

// A 2-dimensional map of grayscale pixels
#[derive(Debug, Clone)]
pub struct PixelMap {
    pub dimensions: Point<usize>,
    inner: Vec<Pixel>,
}

impl PixelMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            dimensions: Point {
                x: width,
                y: height,
            },
            inner: vec![],
        }
    }

    pub fn push(&mut self, x: i32, y: i32, color: u32) {
        self.inner.push(Pixel::new(x, y, color))
    }

    pub fn iter(&self) -> slice::Iter<Pixel> {
        self.inner.iter()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    pub position: Point<i32>,
    pub color: u32,
}

impl Pixel {
    pub fn new(x: i32, y: i32, color: u32) -> Self {
        Self {
            position: Point { x, y },
            color,
        }
    }
}

#[derive(Debug)]
pub struct Font<'a> {
    inner: rusttype::Font<'a>,
    height: usize,
    scale: Scale,
    offset: Point<f32>,
}

impl<'a> Font<'a> {
    pub fn new(font_data: &'a [u8], height: usize) -> Option<Self> {
        rusttype::Font::try_from_bytes(font_data).map(|inner| {
            let scale = Scale {
                x: height as f32,
                y: height as f32,
            };
            let v_metrics = inner.v_metrics(scale);
            let offset = rusttype::point(0.0, v_metrics.ascent);

            Self {
                inner,
                height,
                scale,
                offset,
            }
        })
    }

    pub fn update_height(&mut self, height: usize) {
        let scale = Scale {
            x: height as f32,
            y: height as f32,
        };
        let v_metrics = self.inner.v_metrics(scale);

        self.height = height;
        self.scale = scale;
        self.offset = rusttype::point(0.0, v_metrics.ascent);
    }

    pub fn rasterize(&self, char: char) -> PixelMap {
        let glyph = self
            .inner
            .glyph(char)
            .scaled(self.scale)
            .positioned(self.offset);
        let width = (glyph.position().x + glyph.unpositioned().h_metrics().advance_width) as u32;

        let mut pixel_map = PixelMap::new(width as usize, self.height);
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                let alpha = (v * 255.0) as u8;
                if alpha > 0 {
                    let color = (alpha as u32) << 24 | (alpha as u32) << 16 | (alpha as u32) << 8;

                    let x = x as i32 + bounding_box.min.x;
                    let y = y as i32 + bounding_box.min.y;

                    pixel_map.push(x, y, color);
                }
            });
        }

        pixel_map
    }

    pub fn height(&self) -> usize {
        self.height
    }

    // Finds the most visually pleasing width to display from a slice of glyphs
    // fn ideal_glyph_width(glyphs: &[PositionedGlyph]) -> usize {
    //     glyphs
    //         .iter()
    //         .rev()
    //         .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
    //         .next()
    //         .unwrap_or(0.0) as usize
    // }
}

impl<'a> Default for Font<'a> {
    fn default() -> Self {
        Self::new(
            include_bytes!("../../../data/fonts/open-sans/OpenSans-Regular.ttf"),
            12,
        )
        .unwrap()
    }
}

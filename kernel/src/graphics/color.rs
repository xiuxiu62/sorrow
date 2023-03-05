#[derive(Clone, Copy)]
pub enum Color {
    White,
    Black,
    Red,
    Green,
    Blue,
    Purple,
    Rgb(u32),
    Bgr(u32),
}

impl Color {
    // const COLOR_MASK: u32 = 0x00ff00ff;

    pub fn as_rgb(&self) -> u32 {
        match *self {
            Self::White => 0xffffff00,
            Self::Black => 0x00000000,
            Self::Red => 0xff000000,
            Self::Green => 0x00ff0000,
            Self::Blue => 0x0000ff00,
            Self::Purple => 0xaa00aa00,
            Self::Rgb(value) => value,
            Self::Bgr(value) => {
                // let blue = (value & 0xff000000) >> 16;
                // let red = (value & 0x0000ff00) << 16;

                // value & Self::COLOR_MASK & blue & red

                let blue = (value >> 24) as u8 as u32;
                let green = (value >> 16) as u8 as u32;
                let red = (value >> 8) as u8 as u32;
                let alpha = (value >> 0) as u8;

                (red as u32) << 24 | (green as u32) << 16 | (blue as u32) << 8 | (alpha as u32) << 0
            }
        }
    }

    pub fn as_bgr(&self) -> u32 {
        match *self {
            Self::White => 0xffffff00,
            Self::Black => 0x00000000,
            Self::Red => 0x0000ff00,
            Self::Green => 0x00ff0000,
            Self::Blue => 0xff000000,
            Self::Purple => 0xaa00aa00,
            Self::Rgb(value) => {
                // let red = (value & 0xff000000) >> 16;
                // let blue = (value & 0x0000ff00) << 16;

                // value & Self::COLOR_MASK & red & blue

                let red = (value >> 24) as u8;
                let green = (value >> 16) as u8;
                let blue = (value >> 8) as u8;
                let alpha = (value >> 0) as u8;

                (blue as u32) << 24 | (green as u32) << 16 | (red as u32) << 8 | (alpha as u32) << 0
            }
            Self::Bgr(value) => value,
        }
    }
}

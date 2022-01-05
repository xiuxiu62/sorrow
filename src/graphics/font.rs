use alloc::vec::Vec;

const PSF_FONT_MAGIC_NUMBER: u32 = 0x864ab572;

// const extern _binary_font_psf_start: u32;
// extern const _binary_font_psf_start;

struct PSFFont {
    magic_number: u32,    // Magic identifier
    version: u32,         // Zero
    header_size: u32,     // Offset of bitmaps in file
    flags: u32,           // 0 if there is no unicode table
    glyph_count: u32,     // Number of glyphs
    bytes_per_glyph: u32, // Size of each glyph
    height: u32,          // Height in pixels
    width: u32,           // Width in pixels
}

// impl PSFFont {
//     pub fn new(input_stream: Vec<u8>) -> Self {
//         let glyph = 0_u32;
//         // let psf_font
//     }
// }

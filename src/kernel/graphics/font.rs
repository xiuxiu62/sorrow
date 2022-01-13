




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

// pub unsafe fn load_default_font() -> [u8; kb_to_b!(4)] {
//     // in: edi=4k buffer
//     // out: buffer filled with font
//     // clear even/odd mode

//     asm!(
//         "mov dx, 03ceh",
//         "mov ax, 5",
//         "out dx, ax",
//         // map VGA memory to 0A0000h
//         "mov ax, 0406h",
//         "out dx, ax",
//         // set bitplane 2
//         "mov dx, 03c4h",
//         "mov ax, 0402h",
//         "out dx, ax",
//         // clear even/odd mode (the other way, don't ask why)
//         "mov ax, 0604h",
//         "out dx, ax",
//         // copy charmap
//         "mov esi, 0A0000h",
//         "mov ecx, 256",
//         // copy 16 bytes to bitmap
//         "@@: movsd",
//         "movsd",
//         "movsd",
//         "movsd",
//         // skip another 16 bytes
//         "add esi, 16",
//         "loop @b",
//         // restore VGA state to normal operation
//         "mov ax, 0302h",
//         "out dx, ax",
//         "mov ax, 0204h",
//         "out dx, ax",
//         "mov dx, 03ceh",
//         "mov ax, 1005h",
//         "out dx, ax",
//         "mov ax, 0E06h",
//         "out dx, ax"
//     );
// }

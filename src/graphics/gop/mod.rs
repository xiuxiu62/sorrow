use bootloader::boot_info::{FrameBuffer, FrameBufferInfo, Optional};
use lazy_static::lazy_static;
// use spin::Mutex;

struct Position {
    x: usize,
    y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

pub struct Writer<'a> {
    frame_buffer: &'a mut FrameBuffer,
    info: FrameBufferInfo,
    position: Position,
}

impl<'a> Writer<'a> {
    pub fn new(frame_buffer: &'a mut FrameBuffer) -> Self {
        let info = frame_buffer.info();
        Self {
            frame_buffer,
            info,
            position: Position::default(),
        }
    }

    pub fn try_new(frame_buffer: &'a mut Optional<FrameBuffer>) -> Result<Self, &'a str> {
        match frame_buffer {
            Optional::Some(frame_buffer) => Ok(Self::new(frame_buffer)),
            Optional::None => Err("Failed to acquire frame buffer handle"),
        }
    }

    pub fn fill(&mut self, color: u8) {
        self.as_mut().iter_mut().for_each(|byte| *byte = color);
    }

    pub fn clear(&mut self) {
        self.fill(0x00)
    }
}

impl<'a> AsRef<[u8]> for Writer<'a> {
    fn as_ref(&self) -> &[u8] {
        self.frame_buffer.buffer()
    }
}

impl<'a> AsMut<[u8]> for Writer<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.frame_buffer.buffer_mut()
    }
}

// turn the screen gray
// if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
//     for byte in framebuffer.buffer_mut() {
//         *byte = 0x90;
//     }
// };

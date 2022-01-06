use super::buffer::{Buffer, Color, ColorCode, Coordinates};
use bootloader::boot_info::FrameBufferInfo;
use core::{
    fmt::{self, Write},
    ptr,
};
use font8x8::UnicodeFonts;
use spin::Mutex;
// use lazy_static::lazy_static;
// use spin::Mutex;

pub struct Writer<'a>(Buffer<'a>);

// impl<'a> Writer<'a> {
//     fn new(frame_buffer: &'static mut [u8], info: FrameBufferInfo) ->
// }

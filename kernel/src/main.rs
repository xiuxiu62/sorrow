#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, custom_test_frameworks, error_in_core)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_run"]

mod gdt;
mod graphics;
mod idt;
mod mem;
mod terminal;

#[macro_use]
extern crate alloc;

use alloc::rc::Rc;
use bootloader_api::{config::Mapping, entry_point, BootInfo, BootloaderConfig};
use core::{cell::RefCell, panic::PanicInfo};
use graphics::{Color, GopDevice, GraphicsDevice};
use mem::{alloc::BootInfoFrameAllocator, heap, MemoryResult};
use rusttype::Point;
use terminal::Terminal;
use x86_64::instructions;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);

    config
};

entry_point!(main, config = &BOOTLOADER_CONFIG);

fn main(boot_info: &'static mut BootInfo) -> ! {
    #[cfg(test)]
    test_run();

    // TODO: handle errors
    let runtime = Runtime::new(boot_info).unwrap();
    let mut terminal = Terminal::new(runtime.gop_device.clone(), 20);

    (0..5).for_each(|_| {
        terminal.clear();
        test_writes(&mut terminal);
    });

    terminal.update_font_size(36);
    (0..5).for_each(|_| {
        terminal.clear();
        test_writes(&mut terminal);
    });

    halt_loop()
}

fn test_writes(terminal: &mut Terminal) {
    let mut x_offset = 0;
    let mut y_offset = 0;
    let mut max_height = 0;
    (0..20).for_each(|_| {
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890!@#$%^&*()`~[]{},.<>/?;:'\"-_\\|".chars().for_each(|char| {
                let Point {
                    x: width,
                    y: height,
                } = terminal.write_char(x_offset as i32 + 10, y_offset as i32 + 10, char);
                x_offset += width;
                max_height = max_height.max(height);
            });

            x_offset = 0;
            y_offset += max_height;
        });
}

struct Runtime {
    gop_device: Rc<RefCell<dyn GraphicsDevice>>,
}

impl Runtime {
    fn new(boot_info: &'static mut BootInfo) -> MemoryResult<Self> {
        gdt::initialize();
        idt::initialize();
        let mut memory_mapper =
            unsafe { mem::initialize(boot_info.physical_memory_offset.as_ref())? };
        let mut frame_allocator = unsafe { BootInfoFrameAllocator::new(&boot_info.memory_regions) };
        heap::initialize(&mut memory_mapper, &mut frame_allocator)?;
        let gop_device = Rc::new(RefCell::new(
            GopDevice::new(boot_info.framebuffer.as_mut()).unwrap(),
        ));

        Ok(Self { gop_device })
    }
}

pub fn halt_loop() -> ! {
    loop {
        instructions::hlt()
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    halt_loop()
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    // println!("running {} tests", tests.len());
    tests.for_each(|test| test())
}

#[test_case]
fn example_test() {
    assert_eq!(1, 1);
}

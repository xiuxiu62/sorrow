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

    // let frame_buffer = core::mem::take(boot_info.framebuffer.as_mut());
    // initialize_hardware(boot_info).unwrap();
    // let gop_device = Rc::new(RefCell::new(GopDevice::new(frame_buffer).unwrap()));

    // TODO: handle errors
    let runtime = Runtime::new(boot_info).unwrap();
    // let font = Font::default();
    // runtime.execute_sync(draw_things);
    // let terminal = Terminal::new(runtime.gop_device.clone(), font);
    let terminal = Terminal::new(runtime.gop_device.clone());
    terminal.clear();
    let mut x = 0;
    let mut y = 0;
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890+!@#$%^&*()"
        .chars()
        .for_each(|char| {
            terminal.write_char(x * 50 + 20, y * 50 + 20, char);

            x += 1;
            if x == 15 {
                x = 0;
                y += 1;
            };
        });
    // terminal.write_char(0, 0, 'A');
    // terminal.write_char(50, 0, 'B');
    // terminal.write_char(100, 0, 'C');
    // terminal.write_char(150, 0, 'D');
    // terminal.write_char(200, 0, 'E');
    // terminal.test_char();

    // runtime.execute_sync(draw_things);

    halt_loop()
}

fn draw_things(runtime: &mut Runtime) {
    runtime.gop_device.borrow_mut().fill(Color::Black);

    // runtime.gop_device.fill(Color::Purple);
    // (0..12).for_each(|x| {
    // (0..7).for_each(|y| {
    // runtime.gop_device.borrow_mut().draw_square(
    // x * 100 + 50,
    // y * 100 + 50,
    // 50,
    // Color::Black,
    // )
    // })
    // });
}

struct Runtime {
    // memory_mapper: OffsetPageTable<'a>,
    // allocator: Box<dyn FrameAllocator<Size4KiB>>,
    // gop_device: GopDevice<'a>,
    gop_device: Rc<RefCell<dyn GraphicsDevice>>,
}

// pub fn initialize_hardware(boot_info: &'static BootInfo) -> MemoryResult<()> {
//     gdt::initialize();
//     idt::initialize();
//     let mut memory_mapper = unsafe { mem::initialize(boot_info.physical_memory_offset.as_ref())? };
//     let mut frame_allocator = unsafe { BootInfoFrameAllocator::new(&boot_info.memory_regions) };
//     heap::initialize(&mut memory_mapper, &mut frame_allocator)?;
//     let gop_device = Rc::new(RefCell::new(
//         GopDevice::new(boot_info.framebuffer.as_mut()).unwrap(),
//     ));

//     Ok(())
// }

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

    fn execute_sync(&mut self, f: fn(&mut Self)) {
        f(self)
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

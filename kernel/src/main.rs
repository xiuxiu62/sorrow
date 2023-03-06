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
use graphics::GopDevice;
use mem::{alloc::BootInfoFrameAllocator, heap, MemoryResult};
use x86_64::instructions;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    // config.kernel_stack_size = 1024 * 1000 * 1000;
    config.mappings.physical_memory = Some(Mapping::Dynamic);

    config
};

entry_point!(main, config = &BOOTLOADER_CONFIG);

fn main(boot_info: &'static mut BootInfo) -> ! {
    #[cfg(test)]
    test_run();

    // TODO: handle errors
    initialize_hardware(boot_info).unwrap();
    clear!();

    println!("/home/xiuxiu/documents > ls");
    println!("test.txt");
    println!("hello-world.txt");

    halt_loop()
}

fn initialize_hardware(boot_info: &'static mut BootInfo) -> MemoryResult<()> {
    gdt::initialize();
    idt::initialize();
    let mut memory_mapper = unsafe { mem::initialize(boot_info.physical_memory_offset.as_ref())? };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::new(&boot_info.memory_regions) };
    heap::initialize(&mut memory_mapper, &mut frame_allocator)?;
    let gop_device = Rc::new(RefCell::new(
        GopDevice::new(boot_info.framebuffer.as_mut()).unwrap(),
    ));
    crate::terminal::initialize(gop_device);

    Ok(())
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

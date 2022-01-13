#![no_std]
#![cfg_attr(test, no_main)]
#![feature(
    abi_x86_interrupt,
    alloc_error_handler,
    const_mut_refs,
    custom_test_frameworks
)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
extern crate alloc;

use core::panic::PanicInfo;
use x86_64::instructions;

pub mod allocator;
pub mod devices;
pub mod gdt;
pub mod graphics;
pub mod interrupts;
pub mod io;
pub mod memory;
pub mod serial;
pub mod storage;
pub mod task;

pub fn hlt_loop() -> ! {
    loop {
        instructions::hlt();
    }
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Alloc error: {layout:?}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use bootloader::{entry_point, BootInfo};

    entry_point!(test_kernel_main);

    /// Entry point for `cargo ktest`
    fn test_kernel_main(_boot_info: &'static mut BootInfo) -> ! {
        test_main();
        hlt_loop();
    }

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        test_panic_handler(info)
    }
}

// #[cfg(test)]
// use bootloader::BootInfo;

// /// Entry point for `cargo ktest`
// #[cfg(test)]
// fn test_kernel_main(_boot_info: &'static mut BootInfo) -> ! {
//     test_main();
//     hlt_loop();
// }

// #[cfg(test)]
// mod tests {
//     use super::{test_kernel_main, test_panic_handler, PanicInfo};
//     use bootloader::entry_point;

//     entry_point!(test_kernel_main);

//     #[panic_handler]
//     fn panic(info: &PanicInfo) -> ! {
//         test_panic_handler(info)
//     }
// }

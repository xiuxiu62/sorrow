#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lib_sorrow::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]

extern crate alloc;

use alloc::string::ToString;
use alloc::{format, string::String};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use lib_sorrow::serial_println;
use lib_sorrow::{
    self,
    memory::BootInfoFrameAllocator,
    // devices::{self, keyboard::Keyboard},
    println,
    storage::drive::Drive,
    // task::{executor::Executor, Task},
};
// use pc_keyboard::{layouts::Us104Key, HandleControl, ScancodeSet1};
// use spin::Mutex;

// lazy_static! {
//     static ref KEYBOARD: Arc<Mutex<Keyboard<Us104Key, ScancodeSet1>>> = Arc::new(Mutex::new(
//         Keyboard::new(Us104Key, ScancodeSet1, HandleControl::Ignore),
//     ));
// }

const TASK_QUEUE_SIZE: usize = 100;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    match kernel_run(boot_info) {
        Ok(()) => lib_sorrow::hlt_loop(),
        Err(message) => panic!("{message}"),
    }
}

fn kernel_run(boot_info: &'static mut BootInfo) -> Result<(), String> {
    kernel_init(boot_info)?;

    // Initialize drive and read some data
    // let drive = Drive::default();
    // let data = drive.read(0, 1);

    // let formatted_data = data
    //     .iter()
    //     .map(|w| [(w >> 8) as u8, *w as u8])
    //     .flatten()
    //     .filter(|b| *b > 31 && *b < 127)
    //     .map(|b| b as char)
    //     .fold(String::new(), |acc, c| acc + format!("{c}").as_str());

    println!("hello world");
    // println!("Some drive data: {:?}", formatted_data);

    Ok(())

    // Create and spawn tasks
    // let mut executor = Executor::new(TASK_QUEUE_SIZE);
    // let tasks = vec![
    //     Task::new(print_number(42)),
    //     Task::new(lib_sorrow::devices::keyboard::listen()),
    // ];
    // for task in tasks {
    //     executor.spawn(task)?;
    // }

    // executor.run();
}

fn kernel_init(boot_info: &'static mut BootInfo) -> Result<(), String> {
    serial_println!("test");
    lib_sorrow::gdt::init();
    lib_sorrow::interrupts::idt_init();
    lib_sorrow::interrupts::pics_init();
    // lib_sorrow::interrupts::enable();

    // Initialize paging
    let mut mapper = unsafe { lib_sorrow::memory::init(boot_info.physical_memory_offset) }?;
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };
    if lib_sorrow::allocator::init_heap(&mut mapper, &mut frame_allocator).is_err() {
        return Err("Failed to initialize heap".to_string());
    };

    lib_sorrow::graphics::gop::init(&mut boot_info.framebuffer, 2)?;

    #[cfg(test)]
    test_main();

    Ok(())
}

async fn print_number(n: u32) {
    println!("{n}");
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC:");
    println!("{info}");
    lib_sorrow::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    lib_sorrow::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

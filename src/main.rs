#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lib_sorrow::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use lib_sorrow::{
    self,
    devices::keyboard::Keyboard,
    interrupts, println,
    storage::drive::Drive,
    task::{executor::Executor, Task},
};
use pc_keyboard::{layouts::Us104Key, HandleControl, ScancodeSet1};

static TASK_QUEUE_SIZE: usize = 100;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    match lib_sorrow::init(boot_info) {
        Ok(console) => console,
        Err(err) => panic!("{err}"),
    };

    // Initialize devices
    let drive = Drive::new(0);
    let mut keyboard = {
        unsafe { interrupts::PICS.lock().initialize() };
        Keyboard::new(Us104Key, ScancodeSet1, HandleControl::Ignore)
    };

    let data = {
        let mut buf = [0_u16; 512];
        unsafe { drive.read_sector(1, &mut buf) };
        Box::new(buf)
    };

    println!("hello world");
    println!("hello world");
    println!("hello world");
    println!("hello world");
    println!("hello world");

    #[cfg(test)]
    test_main();

    lib_sorrow::hlt_loop();

    // let mut executor = Executor::new(TASK_QUEUE_SIZE);
    // Create and spawn tasks
    // vec![Task::new(keyboard.listen(&mut console))]
    //     .into_iter()
    //     .for_each(|task| {
    //         if let Err(task_id) = executor.spawn(task) {
    //             panic!("Task {task_id} failed to execute")
    //         }
    //     });

    // executor.run();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // println!("KERNEL PANIC:");
    // println!("{info}");
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

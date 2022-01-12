#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lib_sorrow::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{
    format,
    string::{String, ToString},
    vec::{self, Vec},
};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use lib_sorrow::{
    self,
    devices::{self, keyboard::Keyboard},
    println,
    storage::drive::Drive,
    task::{executor::Executor, Task},
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
    lib_sorrow::init(boot_info)?;

    // Initialize drive and read some data
    let drive = Drive::default();
    let data = drive.read(0, 1);

    let formatted_data = data[0..256]
        .iter()
        .map(|w| [(w >> 8) as u8, *w as u8])
        .flatten()
        .filter(|b| *b > 31 && *b < 127)
        .map(|b| b as char)
        .fold(String::new(), |acc, c| acc + format!("{c}").as_str());

    println!("hello world");
    println!("Some drive data: {:?}", formatted_data);

    #[cfg(test)]
    test_main();

    Ok(())

    // Create and spawn tasks
    // let mut executor = Executor::new(TASK_QUEUE_SIZE);
    // let tasks = vec![
    // Task::new(print_number(42)),
    // // Task::new(crate::devices::keyboard::listen()),
    // ];
    // for task in tasks {
    //     executor.spawn(task)?;
    // }

    // executor.run();
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

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
    devices::{self, keyboard::Keyboard},
    println,
    storage::drive::Drive,
    task::{executor::Executor, Task},
};
use pc_keyboard::{layouts::Us104Key, HandleControl, ScancodeSet1};
use spin::Mutex;

// lazy_static! {
//     static ref KEYBOARD: Arc<Mutex<Keyboard<Us104Key, ScancodeSet1>>> = Arc::new(Mutex::new(
//         Keyboard::new(Us104Key, ScancodeSet1, HandleControl::Ignore),
//     ));
// }

const TASK_QUEUE_SIZE: usize = 100;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Err(err) = lib_sorrow::init(boot_info) {
        panic!("{err}");
    }

    // Initialize devices
    let drive = Drive::new(0);

    let mut buf = [0_u16; 512];
    unsafe { drive.read_sector(1, &mut buf) };

    // static KEYBOARD: Keyboard<Us104Key, ScancodeSet1> =
    //     Keyboard::new(Us104Key, ScancodeSet1, HandleControl::Ignore);

    println!("hello world");

    #[cfg(test)]
    test_main();

    // lib_sorrow::hlt_loop();

    // Create and spawn tasks
    let mut executor = Executor::new(TASK_QUEUE_SIZE);
    vec![
        // Task::new(crate::devices::keyboard::listen()),
        Task::new(print_number(42)),
    ]
    .into_iter()
    .for_each(|task| {
        if let Err(task_id) = executor.spawn(task) {
            panic!("Task {task_id} failed to execute")
        }
    });

    executor.run();
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

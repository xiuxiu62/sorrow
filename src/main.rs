#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lib_sorrow::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::vec;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use lib_sorrow::{
    self, allocator,
    devices::keyboard,
    memory::{self, BootInfoFrameAllocator},
    println,
    task::{executor::Executor, Task},
};
use x86_64::VirtAddr;

static TASK_QUEUE_SIZE: usize = 100;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    lib_sorrow::init(boot_info);

    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Error: {err:?}");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new(TASK_QUEUE_SIZE);
    let tasks = vec![
        Task::new(example_task()),
        Task::new(keyboard::handle_keypresses()),
    ];

    tasks.into_iter().for_each(|task| {
        if let Err(task_id) = executor.spawn(task) {
            panic!("Task {task_id} failed to execute")
        }
    });

    executor.run();
}

async fn async_number() -> u32 {
    42 * 10
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
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

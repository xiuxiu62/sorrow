#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lib_sorrow::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::vec;
use bootloader::{boot_info::Optional, entry_point, BootInfo};
use core::panic::PanicInfo;
use lib_sorrow::{
    self, allocator,
    devices::keyboard,
    graphics::gop::{self, Color, ColorCode, Coordinates},
    memory::{self, BootInfoFrameAllocator},
    println,
    storage::disk,
    task::{executor::Executor, Task},
};

static TASK_QUEUE_SIZE: usize = 100;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // lib_sorrow::init();
    let lba = disk::LBA::new(0);

    let mut buf = [0_u16; 512];
    unsafe { lba.disk_read_sector(1, &mut buf) };

    let mut gop_writer = match gop::Writer::try_new(&mut boot_info.framebuffer) {
        Ok(writer) => writer,
        Err(err) => panic!("{err}"),
    };

    gop_writer.clear();
    gop_writer.draw_rectangle(
        Coordinates::new(0, 0),
        Coordinates::new(80, 30),
        Color::from(ColorCode::Magenta),
    );

    gop_writer.draw_rectangle(
        Coordinates::new(20, 20),
        Coordinates::new(30, 80),
        Color::from(ColorCode::Blue),
    );

    gop_writer.draw_rectangle(
        Coordinates::new(0, 0),
        Coordinates::new(5, 50),
        Color::from(ColorCode::Red),
    );

    #[cfg(test)]
    test_main();

    lib_sorrow::hlt_loop();

    // let physical_memory_offset = match memory::try_get_physical_memory_offset(boot_info) {
    //     Ok(offset) => offset,
    //     Err(err) => panic!("{err}"),
    // };

    // let mut mapper = unsafe { memory::init(physical_memory_offset) };
    // let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    // allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Error: {err:?}");

    // let mut executor = Executor::new(TASK_QUEUE_SIZE);
    // Create and spawn tasks
    // vec![Task::new(keyboard::handle_keypresses())]
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

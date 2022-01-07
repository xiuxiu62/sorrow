#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lib_sorrow::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, format, vec};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use lib_sorrow::{
    self, allocator,
    devices::keyboard,
    gdt,
    graphics::gop::writer::TextWriter,
    interrupts,
    memory::{self, BootInfoFrameAllocator},
    storage::drive::Drive,
    task::{executor::Executor, Task},
};

static TASK_QUEUE_SIZE: usize = 100;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    gdt::init();
    interrupts::init_idt();
    // interrupts::disable();

    // Try to initialize paging
    match memory::try_get_physical_memory_offset(boot_info.physical_memory_offset) {
        Ok(offset) => unsafe {
            let mut mapper = memory::init(offset);
            let mut frame_allocator = BootInfoFrameAllocator::init(&boot_info.memory_regions);
            allocator::init_heap(&mut mapper, &mut frame_allocator);
        },
        Err(err) => panic!("{err}"),
    };

    unsafe { interrupts::PICS.lock().initialize() };
    // interrupts::enable();

    let drive = Drive::new(0);

    let data = {
        let mut buf = [0_u16; 512];
        unsafe { drive.read_sector(1, &mut buf) };
        Box::new(buf)
    };

    // let mut gop_writer = match Buffer::try_new(&mut boot_info.framebuffer) {
    //     Ok(writer) => writer,
    //     Err(err) => panic!("{err}"),
    // };

    let mut console = match TextWriter::try_new(&mut boot_info.framebuffer) {
        Ok(writer) => writer,
        Err(err) => panic!("{err}"),
    };

    console.clear();
    console.write_str("hello world\n");
    console.write_str("hello world\n");
    console.write_str("hello world\n");
    console.shift();

    lib_sorrow::hlt_loop();

    #[cfg(test)]
    test_main();

    // let mut executor = Executor::new(TASK_QUEUE_SIZE);
    // Create and spawn tasks
    // vec![Task::new(keyboard::handle_keypresses(&mut console))]
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

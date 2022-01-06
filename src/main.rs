#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lib_sorrow::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec};
use bootloader::{boot_info::Optional, entry_point, BootInfo};
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
    console.write("hello world\n");
    console.write("hello world again");

    // let background = Color::from(ColorCode::White);
    // let foreground = Color::from(ColorCode::Black);

    // gop_writer.fill(background);

    // // Visualize lba read data
    // buf.iter().enumerate().for_each(|(x, w)| {
    //     let color = if *w > 1 { background } else { foreground };
    //     (0..gop_writer.info.vertical_resolution)
    //         .for_each(|y| gop_writer.draw(Coordinates::new(x, y), color));
    // });

    // draw_some_rectangles(&mut gop_writer);

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

// fn draw_some_rectangles(writer: &mut gop::Writer) {
//     writer.draw_rectangle(
//         Coordinates::new(0, 0),
//         Coordinates::new(200, 200),
//         Color::from(ColorCode::Blue),
//     );

//     writer.draw_rectangle(
//         Coordinates::new(writer.info.horizontal_resolution - 200, 0),
//         Coordinates::new(200, 200),
//         Color::from(ColorCode::Green),
//     );

//     writer.draw_rectangle(
//         Coordinates::new(0, writer.info.vertical_resolution - 200),
//         Coordinates::new(200, 200),
//         Color::from(ColorCode::Red),
//     );

//     writer.draw_rectangle(
//         Coordinates::new(
//             writer.info.horizontal_resolution - 200,
//             writer.info.vertical_resolution - 200,
//         ),
//         Coordinates::new(200, 200),
//         Color::from(ColorCode::Magenta),
//     );
// }

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

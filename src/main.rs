#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lib_sorrow::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use lib_sorrow::{
    self, allocator,
    memory::{self, BootInfoFrameAllocator},
    println,
};
use x86_64::VirtAddr;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    lib_sorrow::init(boot_info);

    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    if let Err(err) = allocator::init_heap(&mut mapper, &mut frame_allocator) {
        println!("Error: {err:?}");
        lib_sorrow::hlt_loop();
    };

    // allocate something on the heap

    let heap_value_1 = Box::new(vec![420, 421, 422, 423, 424]);
    let heap_value_2 = Box::new(69);
    core::mem::drop(heap_value_2);

    let rc_value = Rc::new(heap_value_1.leak());
    let cloned_rc_value = rc_value.clone();
    println!(
        "current reference count: {}",
        Rc::strong_count(&cloned_rc_value)
    );

    core::mem::drop(rc_value);
    println!(
        "current reference count: {}",
        Rc::strong_count(&cloned_rc_value)
    );

    #[cfg(test)]
    test_main();

    lib_sorrow::hlt_loop();
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

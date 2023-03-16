pub mod alloc;
mod error;
pub mod heap;

pub use error::{
    Error as MemoryError, FrameError, PhysicalMemoryOffsetError, Result as MemoryResult,
};
use error::{Error, Result};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{OffsetPageTable, PageTable},
    PhysAddr, VirtAddr,
};

pub unsafe fn initialize(physical_memory_offset: Option<&u64>) -> Result<OffsetPageTable<'static>> {
    let physical_memory_offset = match physical_memory_offset {
        Some(offset) => VirtAddr::new(*offset),
        None => return Err(Error::PhysicalMemoryOffset(PhysicalMemoryOffsetError)),
    };
    let level_4_table = active_level_4_table(physical_memory_offset);

    Ok(OffsetPageTable::new(level_4_table, physical_memory_offset))
}

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_page_table, _cr3_flags) = Cr3::read();
    let physical_address = level_4_page_table.start_address();
    let virtual_address = physical_memory_offset + physical_address.as_u64();
    let page_table_pointer = virtual_address.as_mut_ptr();

    &mut *page_table_pointer
}

/// Translates the given virtual address to the mapped physical address, or
/// `None` if the address is not mapped.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`.
pub unsafe fn translate_address(
    address: VirtAddr,
    physical_memory_offset: VirtAddr,
) -> Result<PhysAddr> {
    let (level_4_table_frame, _cr3_flags) = Cr3::read();

    let table_indexes = [
        address.p4_index(),
        address.p3_index(),
        address.p2_index(),
        address.p1_index(),
    ];
    let mut frame = level_4_table_frame;

    // traverse the multi-level page table
    for index in table_indexes {
        // convert the frame into a page table reference
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        // read the page table entry and update `frame`
        frame = table[index].frame()?;
    }

    // calculate the physical address by adding the page offset
    Ok(frame.start_address() + u64::from(address.page_offset()))
}

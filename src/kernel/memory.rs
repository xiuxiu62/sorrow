use crate::kb;
use bootloader::boot_info::{MemoryRegionKind, MemoryRegions, Optional};
use x86_64::{
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init<'a>(
    physical_memory_offset: Optional<u64>,
) -> Result<OffsetPageTable<'static>, &'a str> {
    let offset = try_get_physical_memory_offset(physical_memory_offset)?;
    let (l4_table, _) = get_active_l4_table(offset);
    Ok(OffsetPageTable::new(l4_table, offset))
}

/// Attempts to parse a Virtual address from and Optional u64
fn try_get_physical_memory_offset<'a>(
    physical_memory_offset: Optional<u64>,
) -> Result<VirtAddr, &'a str> {
    match physical_memory_offset {
        Optional::Some(offset) => Ok(VirtAddr::new(offset)),
        Optional::None => Err("Failed to acquire physical memory offset"),
    }
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn get_active_l4_table(
    physical_memory_offset: VirtAddr,
) -> (&'static mut PageTable, Cr3Flags) {
    let (table_frame, cr3_flags) = Cr3::read();
    let phys = table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    (&mut *page_table_ptr, cr3_flags) // unsafe
}

/// A FrameAllocator that always returns `None`.
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_regions: &'static MemoryRegions) -> Self {
        BootInfoFrameAllocator {
            memory_regions,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let usable_regions = self
            .memory_regions
            .iter()
            .filter(|region| region.kind == MemoryRegionKind::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions.map(|region| region.start..region.end);
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|region| region.step_by(kb!(4)));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

#[macro_export]
macro_rules! kb {
    ($n:expr) => {
        $n * 1024
    };
}

#[macro_export]
macro_rules! mb {
    ($n:expr) => {
        $n * (1024 ^ 2)
    };
}

#[macro_export]
macro_rules! gb {
    ($n:expr) => {
        $n * (1024 ^ 3)
    };
}

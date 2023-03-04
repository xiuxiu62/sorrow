use crate::idt::DOUBLE_FAULT_IST_INDEX;
use lazy_static::lazy_static;
use x86_64::{
    instructions::tables,
    registers::segmentation::{Segment, CS},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

lazy_static! {
    static ref GLOBAL_DESCRIPTOR_TABLE_DATA: GDTData = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tts_selector = gdt.add_entry(Descriptor::tss_segment(&TASK_STATE_SEGMENT));

        GDTData::new(gdt, Selectors::new(code_selector, tts_selector))
    };
    static ref TASK_STATE_SEGMENT: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        tss
    };
}

struct GDTData {
    gdt: GlobalDescriptorTable,
    selectors: Selectors,
}

impl GDTData {
    pub fn new(gdt: GlobalDescriptorTable, selectors: Selectors) -> Self {
        Self { gdt, selectors }
    }
}

struct Selectors {
    code: SegmentSelector,
    tts: SegmentSelector,
}

impl Selectors {
    fn new(code: SegmentSelector, tts: SegmentSelector) -> Self {
        Self { code, tts }
    }
}

pub fn initialize() {
    GLOBAL_DESCRIPTOR_TABLE_DATA.gdt.load();

    let selectors = &GLOBAL_DESCRIPTOR_TABLE_DATA.selectors;
    unsafe {
        CS::set_reg(selectors.code);
        tables::load_tss(selectors.tts)
    }
}

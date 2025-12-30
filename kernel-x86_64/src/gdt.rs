use core::ptr::addr_of;
use kernel_core::sync::init::InitData;
use x86_64::VirtAddr;
use x86_64::instructions::segmentation::{SS, Segment};
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::CS;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static GLOBAL_DESCRIPTOR: InitData<GlobalDescriptor> = InitData::uninit();
static TSS: InitData<TaskStateSegment> = InitData::uninit();

pub struct GlobalDescriptor {
    pub table: GlobalDescriptorTable,
    pub tss: SegmentSelector,
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub user_data: SegmentSelector,
}

pub fn get_gdt<'a>() -> &'a GlobalDescriptor {
    GLOBAL_DESCRIPTOR.get()
}

pub fn get_tss<'a>() -> &'a TaskStateSegment {
    TSS.get()
}

pub unsafe fn init() {
    unsafe {
        TSS.init(build_tss());
    }

    let gdt = unsafe {
        let mut table = GlobalDescriptorTable::new();

        let tss = table.append(Descriptor::tss_segment(get_tss()));

        let kernel_code = table.append(Descriptor::kernel_code_segment());
        let kernel_data = table.append(Descriptor::kernel_data_segment());

        let user_code = table.append(Descriptor::user_code_segment());
        let user_data = table.append(Descriptor::user_data_segment());

        GLOBAL_DESCRIPTOR.init(GlobalDescriptor {
            table,
            tss,
            kernel_code,
            kernel_data,
            user_code,
            user_data,
        })
    };

    gdt.table.load();

    unsafe {
        CS::set_reg(gdt.kernel_code);
        SS::set_reg(gdt.kernel_data);
        load_tss(gdt.tss);
    }
}

unsafe fn build_tss() -> TaskStateSegment {
    let mut tss = TaskStateSegment::new();

    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        let stack_start = VirtAddr::from_ptr(addr_of!(STACK));
        let stack_end = stack_start + STACK_SIZE as u64;
        stack_end
    };

    tss.privilege_stack_table[0] = {
        const RING0_STACK_SIZE: usize = 4096 * 5;
        static mut RING0_STACK: [u8; RING0_STACK_SIZE] = [0; RING0_STACK_SIZE];

        let stack_start = VirtAddr::from_ptr(addr_of!(RING0_STACK));
        let stack_end = stack_start + RING0_STACK_SIZE as u64;
        stack_end
    };

    tss.iomap_base = (size_of::<TaskStateSegment>() - 1) as u16;

    tss
}

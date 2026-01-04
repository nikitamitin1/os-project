use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

static mut TSS: TaskStateSegment = TaskStateSegment::new();
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const DOUBLE_FAULT_IST_INDEX_FOR_IDT: u16 = DOUBLE_FAULT_IST_INDEX + 1;
const DF_STACK_SIZE: usize = 4096 * 5;
static mut DF_STACK: [u8; DF_STACK_SIZE] = [0; DF_STACK_SIZE];


#[allow(static_mut_refs)]
pub fn init() {

    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};

    // init tss
    // Avoid creating references to a mutable static; get a raw pointer instead.
    let stack_ptr = unsafe { core::ptr::addr_of_mut!(DF_STACK) as *mut u8 };
    let stack_start = VirtAddr::from_ptr(stack_ptr);
    let stack_end = stack_start + (DF_STACK_SIZE as u64);
    
    unsafe {TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_end; };


    // Build and load GDT, then load TSS
    unsafe {
        let gdt = &mut GDT;
        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        let tss_selector  = gdt.append(Descriptor::tss_segment(&TSS));
        gdt.load();
        CS::set_reg(code_selector);
        load_tss(tss_selector);
    }
}   

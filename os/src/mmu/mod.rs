mod addr_types;
mod phys_frame_allocator;
mod page_directory;
mod kernel_heap;
mod mem_section;

pub use mem_section::{
    MemArea,
    MemSection,
    MemMapType,
    MemSectionPermission,
    KERNEL_SPACE,
    print_kernel_info,
};
pub use page_directory::{
    translated_byte_buffer,
    PageDirectory,
    translated_refmut,
    translated_ref,
    translated_str,
};
pub use phys_frame_allocator::{
    FrameItem
};

pub use addr_types::{
    PhysPageNumber,
    PhysAddr,
    VirtAddr,
    VirtPageNumber,
};

pub fn init() {
    kernel_heap::init_heap();
    phys_frame_allocator::init_frame_allocator();
    mem_section::print_kernel_info();
    KERNEL_SPACE.lock().activate();
}
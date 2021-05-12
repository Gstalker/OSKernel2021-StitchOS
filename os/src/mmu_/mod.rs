mod addr_types;
mod phys_frame_allocator;
mod page_table;
mod kernel_heap;
pub use phys_frame_allocator::init_frame_allocator;
pub use kernel_heap::init_heap;
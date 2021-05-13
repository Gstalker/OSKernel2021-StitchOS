mod addr_types;
mod phys_frame_allocator;
mod page_table;
mod kernel_heap;

pub use kernel_heap::init_heap;  //初始化内核堆
pub use phys_frame_allocator::init_frame_allocator;  //初始化物理内存页分配器
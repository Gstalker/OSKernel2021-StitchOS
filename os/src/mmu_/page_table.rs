use super::addr_types::*;
use super::phys_frame_allocator::{
    phys_frame_alloc,
    FrameItem,
};
use alloc::vec::Vec;
use alloc::vec;

//数据结构：页表
#[repr(C)]
struct PageDirectory{
    pub root_page : PhysPageNumber,
    pub objs : Vec<FrameItem>,
}

impl PageDirectory{
    fn new() -> Self{ 
        if let Some(page) = phys_frame_alloc(){
            Self{
                root_page : page.ppn,
                objs : vec![page]
            }
        }
        else{
            // 换出换入算法需要在这里完善
            panic!("ERROR: Failed to allocate a phys page as a PageDirectory");
        }
    }
    //在当前页目录中映射物理地址到虚拟地址
    fn map(&mut self,virt_page_num : VirtPageNumber,phys_page_num : PhysPageNumber,flags : PDEFlags){
        
    }

    // 删除虚拟地址为VirtPageNumber的键值对
    fn unmap(&mut self,virt_page_num : VirtPageNumber){

    }

}

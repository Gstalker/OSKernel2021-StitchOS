use super::addr_types::*;
use super::phys_frame_allocator::{
    phys_frame_alloc,
    FrameItem,
};
use crate::config::{
    PAGE_SIZE,
    PAGE_SIZE_BITS,
};
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;

//数据结构：根页表
//root_page指向页框
//objs是位于内核堆上的一个数组，记录了这个页表树使用的物理页框
//页表不负责回收给用户进程的数据物理页
#[repr(C)]
pub struct PageDirectory{
    pub root_page : PhysPageNumber,
    pub objs : Vec<FrameItem>,
}

impl PageDirectory{
    pub fn new() -> Self{ 
        if let Some(page) = phys_frame_alloc(){
            DEBUG!("allocated new page {:?}", page.ppn.0);
            Self{
                root_page : page.ppn,
                objs : vec![page]
            }
        }
        else{
            // 换出换入算法需要在这里完善
            panic!("Failed to allocate a phys page as a PageDirectory");
        }
    }
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_page: PhysPageNumber::from((satp & ((1usize << 44) - 1)) << PAGE_SIZE_BITS),
            objs: Vec::new(),
        }
    }
    pub fn token(&self) -> usize {
        (8usize << 60) | (self.root_page.0 >> PAGE_SIZE_BITS)
    }
    //根据虚拟地址找到物理页框。并建立页表映射
    pub fn find_pte_create(&mut self,virt_page_num : VirtPageNumber) -> Option<&mut PageDirectoryEntry>{
        let idx = virt_page_num.indexes();
        let mut ppn = self.root_page;
        let mut result : Option<&mut PageDirectoryEntry> = None;
        for i in 0..3 {
            //找到Page Dir Entry，也就是页目录中的一个元素
            let pde = &mut ppn.get_pte_array()[idx[i]];
            if i == 2{
                result = Some(pde);
                break
            }
            if !pde.is_valid() {
                if let Some(frame) = phys_frame_alloc(){
                    DEBUG!("test-------{}", frame.ppn.0);
                    *pde = PageDirectoryEntry::new(frame.ppn,PDEFlags::V);
                    self.objs.push(frame);
                    
                }
                else{
                    //物理页耗尽，如何处理？
                    panic!("Failed to allocate phys_frame! at find_pte_create");
                }
            }
            ppn = pde.get_page_number();
        }
        result
    }

    //在当前页目录中映射物理地址到虚拟地址
    pub fn map(&mut self,virt_page_num : VirtPageNumber,phys_page_num : PhysPageNumber,flags : PDEFlags){
        if let Some(pde) = self.find_pte_create(virt_page_num){
            assert!(
                !pde.is_valid(),
                "virtual address {:X} has been mapped!",
                virt_page_num.0
            );
            // let temp = PageDirectoryEntry::new(phys_page_num, flags | PDEFlags::V);
            // *pde = temp;
            DEBUG!("find page--------------- {}", phys_page_num.0);
            *pde = PageDirectoryEntry::new(phys_page_num, flags | PDEFlags::V)
        }
        else{
            panic!("Error in maping a phys_page to a virtual_addr");
        } 
    }

    // 删除虚拟地址为VirtPageNumber的键值对
    pub fn unmap(&mut self,virt_page_num : VirtPageNumber){
        if let Some(pde) = self.find_pte_create(virt_page_num){
            assert!(
                pde.is_valid(),
                "virtual address {:X} has already been unmapped!",
                virt_page_num.0
            );
            *pde = PageDirectoryEntry::empty();
        }
        else{
            panic!("Error in unmaping a phys_page to a virtual_addr.at page_directory.rs,pub fn unmap()");
        }
    }
    pub fn find_pte(&self,virt_page_num : VirtPageNumber) -> Option<&PageDirectoryEntry>{
        let idx = virt_page_num.indexes();
        let mut ppn = self.root_page;
        let mut result : Option<&PageDirectoryEntry> = None;
        DEBUG!("idx: {:?} {:?}",idx, ppn.0);
        for i in 0..3 {
            //找到Page Dir Entry，也就是页目录中的一个元素
            DEBUG!("addr check {}", ppn.0);
            let pde = &ppn.get_pte_array()[idx[i]];
            DEBUG!("pde: {:?}", pde.is_valid());
            DEBUG!("len {}", ppn.get_pte_array().len());
            if i == 2{
                DEBUG!("any output?");
                DEBUG!("pde: {:X}",pde.item);
                DEBUG!("any output?");
                result = Some(pde);
                break
            }
            if !pde.is_valid() {
                return None;
            }
            //下面是报错位置
            ppn = pde.get_page_number();
        }
        DEBUG!("out of {}", result.unwrap().item);
        result
    }
    
    //根据虚拟地址找到物理页框
    pub fn get_phys_frame_by_vpn(&self,vpn : VirtPageNumber) -> Option<PhysPageNumber>{
        if let Some(pde) = self.find_pte(vpn){
            let ppn = pde.get_page_number();
            let a : usize = ppn.into();
            Some(ppn)
        }
        else{
            None
        }
    }
    pub fn get_phys_addr_by_va(&self,va : VirtAddr) -> Option<PhysAddr>{
        let vpn = va.floor();
        if let Some(ppn) = self.get_phys_frame_by_vpn(vpn){
            let temp : usize = ppn.into();
            Some(PhysAddr(temp + va.get_page_offset()))
        }
        else{
            None
        }
    }
}

pub fn translated_ref<T>(token: usize, ptr: *const T) -> &'static T {
    let page_table = PageDirectory::from_token(token);
    page_table.get_phys_addr_by_va(VirtAddr::from(ptr as usize)).unwrap().get_ref()
}

pub fn translated_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
    let page_table = PageDirectory::from_token(token);
    let va = ptr as usize;
    page_table.get_phys_addr_by_va(VirtAddr::from(va)).unwrap().get_mut()
}

/// Load a string from other address spaces into kernel space without an end `\0`.
pub fn translated_str(token: usize, ptr: *const u8) -> String {
    let page_table = PageDirectory::from_token(token);
    let mut string = String::new();
    let mut va = ptr as usize;
    loop {
        let ch: u8 = *(page_table.get_phys_addr_by_va(VirtAddr::from(va)).unwrap().get_mut());
        if ch == 0 {
            break;
        }
        string.push(ch as char);
        va += 1;
    }
    string
}



pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let mut pd = PageDirectory::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn : usize = start_va.floor().into();
        let ppn = pd
            .get_phys_frame_by_vpn(vpn.into())
            .unwrap();
        vpn += PAGE_SIZE;
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.get_page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.get_page_offset()..]);
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.get_page_offset()..end_va.get_page_offset()]);
        }
        start = end_va.into();
    }
    v
}
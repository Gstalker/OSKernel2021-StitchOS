// memsection.rs:
// 内存分段管理
use super::addr_types::*;
use super::phys_frame_allocator::{
    phys_frame_alloc,
};
use super::page_directory::{
    PageDirectory,
};
use super::FrameItem;
use crate::config::{
    MEMORY_END,
    PAGE_SIZE,
    TRAMPOLINE,
    USER_STACK_SIZE,
    TRAP_CONTEXT,
};
use bitflags::*;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use riscv::register::satp;
use alloc::sync::Arc;
use spin::Mutex;
use lazy_static::*;

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<Mutex<MemArea>> = Arc::new(Mutex::new(
        MemArea::new_from_kernel()
    ));
}

struct HeapSection{
    mem_sect : MemSection,
    start_va : VirtAddr,
    end_va   : VirtAddr,
}

impl HeapSection{
    pub fn new(s : VirtAddr,e : VirtAddr , pd : &mut PageDirectory) -> Self{
        let mut mem_sect = MemSection::new(
            s.into(),
            e.into(),
            MemSectionPermission::R | MemSectionPermission::W | MemSectionPermission::U,
            MemMapType::FRAMED,
        );
        // mem_sect.map(pd);
        Self{
            mem_sect,
            start_va : s,
            end_va : e,
        }
    }

    //缺少逻辑：减少BRK大小的机制
    pub fn brk(&mut self,expend_size : usize ,pd : &mut PageDirectory) -> Option<isize>{
        //当参数为0时，返回heap_base
        if expend_size == 0{
            return Some(self.start_va.0 as isize)
        }
        let new_end_va : usize = self.end_va.0 + expend_size;
        if new_end_va < self.start_va.0{
            return None
        }
        //如果能成功从页表中找到这个va对应的物理地址，则说明不需要expand
        // else if new_end_va < self.mem_sect.end_vpn.into(){}wowoxian
        //无法从页表中找到va对应的物理地址，则map_single_frame
        else{
            let mut i : usize = self.mem_sect.end_vpn.into();
            while i < new_end_va{
                self.mem_sect.map_single_frame(pd, i.into());
                i += PAGE_SIZE;
                self.mem_sect.end_vpn = i.into();
            }
        }
        self.end_va = new_end_va.into();
        return Some(new_end_va as isize);
    }
}

//一个进程的地址空间
pub struct MemArea{
    page_directory : PageDirectory,
    mem_sections   : Vec<MemSection>,
    heap_section   : Option<HeapSection>, // 第一次调用BRK前，这里为空
}

impl MemArea{
    //刚新建的MemArea里头什么都没有，需要慢慢添加
    pub fn new() -> Self{
        Self{
            page_directory: PageDirectory::new(),
            mem_sections  : Vec::new(),
            heap_section  : None,
        }
    }
    // 返回MemArea ,uesr stack pointer , program entry point
    pub fn new_from_elf(elf_file : &[u8]) -> (Self,usize,usize){
        let mut ma = Self::new();
        ma.map_trampoline();
        let elf = xmas_elf::ElfFile::new(elf_file).unwrap();
        let elf_header = elf.header;
        // /x7fELF，elf文件的header
        assert_eq!(elf_header.pt1.magic,[0x7f,0x45,0x4c,0x46],"invalid elf magic");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNumber(0);
        for i in 0..ph_count{
            DEBUG!("ph count {}", i);
            let ph = elf.program_header(i).unwrap();
            DEBUG!("ph count {:?}", ph);
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MemSectionPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() { map_perm |= MemSectionPermission::R; }
                if ph_flags.is_write() { map_perm |= MemSectionPermission::W; }
                if ph_flags.is_execute() { map_perm |= MemSectionPermission::X; }
                DEBUG!("va {:X} {:X}", start_va.0, end_va.0);
                let map_area = MemSection::new(
                    start_va,
                    end_va,
                    map_perm,
                    MemMapType::FRAMED,
                );
                
                max_end_vpn = map_area.end_vpn;
                ma.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize])
                );
            }
        }
        DEBUG!("out of");
        // map user stack with U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        // guard page
        user_stack_bottom += PAGE_SIZE * 0x10;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        ma.push(MemSection::new(
            user_stack_bottom.into(),
            user_stack_top.into(),
            MemSectionPermission::R | MemSectionPermission::W | MemSectionPermission::U,
            MemMapType::FRAMED,
        ), None);
        // map TrapContext

        // map heap
        //guard page
        let heap_start = user_stack_bottom + PAGE_SIZE * 0x10000;
        let heap_end = heap_start;
        DEBUG!("mapp heap_section");
        ma.heap_section = Some(HeapSection::new(
            heap_start.into(), 
            heap_end.into(),
            &mut ma.page_directory,
        ));
        DEBUG!("mapp heap_section complete!");

        ma.push(MemSection::new(
            TRAP_CONTEXT.into(),
            TRAMPOLINE.into(),
            MemSectionPermission::R | MemSectionPermission::W,
            MemMapType::FRAMED,
        ), None);
        DEBUG!("elf finish");
        (ma, user_stack_top, elf.header.pt2.entry_point() as usize)
    }
    fn map_trampoline(&mut self){
        DEBUG!("strampoline {:X} {:X} {:X}", strampoline as usize, VirtAddr(TRAMPOLINE as usize).floor().0, TRAMPOLINE as usize);
        self.page_directory.map(
            VirtAddr(TRAMPOLINE as usize).floor(),
            PhysAddr(strampoline as usize).into(),
            PDEFlags::R | PDEFlags::X,
        );
    }
    pub fn new_from_mem_area(tar:&MemArea) -> Self{
        let mut mem_area = MemArea::new();
        mem_area.map_trampoline();
        //copy data from target. include:
        // sections,trap_content,user_stack
        for i in tar.mem_sections.iter(){
            let copied_section = MemSection::new_from_mem_section(i);
            mem_area.push(copied_section,None);
            let mut j : usize = i.start_vpn.into();
            while j < i.end_vpn.into(){
                let src : PhysPageNumber = tar.get_phys_frame_by_vpn(j.into()).unwrap().into();
                let dst : PhysPageNumber = mem_area.get_phys_frame_by_vpn(j.into()).unwrap().into();
                dst.get_bytes_array().copy_from_slice(src.get_bytes_array());
                j += PAGE_SIZE;
            }
        }
        mem_area
    }
    pub fn new_from_kernel() -> Self{ 
        let mut ma = MemArea::new();
        //map trampoline
        ma.map_trampoline();
        //map .text section
        ma.push(
            MemSection::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MemSectionPermission::X | MemSectionPermission::R,
                MemMapType::DIRECT,
            ),
            None,
        );
        //map .rodata
        ma.push(
            MemSection::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MemSectionPermission::R,
                MemMapType::DIRECT,
            ),
            None,
        );
        //map .data
        ma.push(
            MemSection::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MemSectionPermission::W | MemSectionPermission::R,
                MemMapType::DIRECT,
            ),
            None,
        );
        //map .bss
        ma.push(
            MemSection::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MemSectionPermission::W | MemSectionPermission::R,
                MemMapType::DIRECT,
            ),
            None,
        );
        //map remaining phys memory
        ma.push(
            MemSection::new(
                (ekernel as usize).into(),
                (MEMORY_END as usize).into(),
                MemSectionPermission::W | MemSectionPermission::R,
                MemMapType::DIRECT,
            ),
            None,
        );
        use crate::config::MMIO;
        DEBUG!("mapping memory-mapped registers");
        for pair in MMIO {
            ma.push(MemSection::new(
                (*pair).0.into(),
                ((*pair).0 + (*pair).1).into(),
                MemSectionPermission::R | MemSectionPermission::W,
                MemMapType::DIRECT,
            ), None);
        }
        ma
    }

    pub fn brk(&mut self,expend_size : usize) -> Option<isize>{
        if let Some(heap_sect) = &mut self.heap_section{
            heap_sect.brk(expend_size,&mut self.page_directory)
        }
        else{
            None
        }
    }

    // 添加一个分页的段
    pub fn add_framed_section(
        &mut self,
        s        : VirtAddr,
        e        : VirtAddr, // 段的大小。根据该数值来分配物理页数量
        perm     : MemSectionPermission,
        sect_buf : Option<&[u8]>,//指针 ： 指向数据，初始化时直接拷贝到对应物理页里头
    ){
        // 录入Section信息
        let ms = MemSection::new(
            s,
            e,
            perm,
            MemMapType::FRAMED
        );
        // //为该内存段分配内存空间
        // ms.map(&mut self.page_directory);
        // //复制数据到该内存段
        // if let Some(ptr) = sect_buf{
        //     ms.copy_data(ptr,&mut self.page_directory);
        // }
        // //记录该内存段
        // self.mem_sections.push(ms);
        self.push(ms,sect_buf);
    }
    pub fn remove_framed_section_by_start_vpn(&mut self,vpn : VirtPageNumber){
        for i in 0..self.mem_sections.len(){
            if self.mem_sections[i].start_vpn == vpn{
                self.mem_sections[i].unmap(&mut self.page_directory);
                self.mem_sections.remove(i);
                break;
            }
        }
    }
    pub fn token(&self) -> usize {
        self.page_directory.token()
    }
    //提升代码复用率
    fn push(&mut self,mut ms : MemSection, sect_buf : Option<&[u8]>){
        DEBUG!("start map");
        ms.map(&mut self.page_directory);
        DEBUG!("finish map");
        //复制数据到该内存段
        if let Some(ptr) = sect_buf{
            DEBUG!("cp");
            ms.copy_data(ptr,&mut self.page_directory);
            DEBUG!("cpc");
        }
        else{}
        //记录该内存段
        self.mem_sections.push(ms);
    }
    pub fn activate(&self) {
        let satp = self.page_directory.token();
        // LOG!("satp : {:X}",satp);
        // WARN!("START SATP");
        unsafe {
            satp::write(satp);
            // WARN!("WRITE SATP");
            llvm_asm!("sfence.vma" :::: "volatile");
        }
        // LOG!("SUCC SATP");
    }
    pub fn recycle_data_pages(&mut self) {
        self.mem_sections.clear();
    }

    pub fn get_phys_frame_by_vpn(&self,vpn : VirtPageNumber) -> Option<PhysPageNumber>{ 
        self.page_directory.get_phys_frame_by_vpn(vpn)
    }
    pub fn get_phys_addr_by_va(&self,va : VirtAddr) -> Option<PhysAddr>{
        self.page_directory.get_phys_addr_by_va(va)
    }
}
//结构体 内存段
pub struct MemSection{
    start_vpn : VirtPageNumber,
    end_vpn   : VirtPageNumber,
    permission      : MemSectionPermission,
    map_type        : MemMapType,
    frames          : BTreeMap<VirtPageNumber, FrameItem>,
}

// 段页面类型，是物理内存映射到相同的虚拟地址，还是用户态的映射方法
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MemMapType{
    DIRECT,  // VPN == PPN
    FRAMED,  // map PhysPage to virtual addr
}

bitflags! {
    pub struct MemSectionPermission : u8{
        const R = 1 << 1; // readable
        const W = 1 << 2; // writable
        const X = 1 << 3; // executable
        const U = 1 << 4;
    }
}

impl MemSection{
    pub fn new(
        s_addr : VirtAddr,
        e_addr : VirtAddr,
        perm : MemSectionPermission,
        t : MemMapType
    ) -> Self {
        Self{
            start_vpn : s_addr.floor(),
            end_vpn : e_addr.celi(),
            permission : perm,
            map_type : t,
            frames : BTreeMap::new(),
        }
    }

    pub fn new_from_mem_section(tar : &MemSection) -> Self {
        Self{
            start_vpn : tar.start_vpn,
            end_vpn : tar.end_vpn,
            permission : tar.permission,
            map_type : tar.map_type,
            frames : BTreeMap::new(),
        }

    }
    //实现loader的关键函数之一
    //将数据拷贝到段中
    fn copy_data(&self, data: &[u8],pd : &mut PageDirectory){ 
        //实现思路： 循环遍历虚拟页，逐页面复制
        let mut virt_addr_iter : usize = self.start_vpn.into();
        let mut i : usize= 0;
        let mut size = data.len();
        while i < data.len(){
            let src_slice_data = &data[i..size.min(i + PAGE_SIZE)];
            if let Some(ppn) = pd.get_phys_frame_by_vpn(virt_addr_iter.into()){
                DEBUG!("write ppn {}", ppn.0);
                let ptr = &mut ppn.get_bytes_array()[..src_slice_data.len()];
                ptr.copy_from_slice(src_slice_data);
            }
            else{
                panic!("Failed in find a physpage by vpn! in mem_section.rs,copy_data()");
            }
            DEBUG!("i : {:X}",i);
            DEBUG!("virtaddr : {:X}",virt_addr_iter);
            DEBUG!("data.len: {:X} {:X}",data.len(), PAGE_SIZE);
            i += PAGE_SIZE;
            virt_addr_iter += PAGE_SIZE;
        }
    }
    fn unmap_single_frame(&mut self,pd : &mut PageDirectory,vpn : VirtPageNumber){
        pd.unmap(vpn);
        match self.map_type {
            MemMapType::FRAMED => {
                self.frames.remove(&vpn);
            }
            MemMapType::DIRECT => {
                //整个内存都是内核的，内核：关我屁事
            }
        }
    }
    fn map_single_frame(&mut self,pd :&mut PageDirectory,vpn : VirtPageNumber){
        let ppn : PhysPageNumber;
        match self.map_type{
            MemMapType::DIRECT =>{
                ppn = PhysPageNumber(vpn.0);
            }
            MemMapType::FRAMED =>{
                let pf = phys_frame_alloc().unwrap();
                ppn = pf.ppn;
                self.frames.insert(vpn,pf);
            }
        }
        let flags : PDEFlags = PDEFlags::from_bits(self.permission.bits).unwrap();
        pd.map(vpn,ppn,flags);
    }
    // 将这个Section映射到内存中
    fn map(&mut self, pd : &mut PageDirectory){
        let mut i : usize = self.start_vpn.into();
        let end : usize = self.end_vpn.into();
        while i < self.end_vpn.into(){
            self.map_single_frame(
                pd,
                VirtPageNumber(i)
            );
            i += PAGE_SIZE;
        }
    }
    //从内存中回收该section
    fn unmap(&mut self,pd : &mut PageDirectory){
        let mut i : usize = self.start_vpn.into();
        while i < self.end_vpn.into(){
            self.unmap_single_frame(
                pd,
                VirtPageNumber(i)
            );
            i += PAGE_SIZE;
        }
    }
}

pub fn print_kernel_info(){
    println!(".text : {:X} - {:X}",stext as usize,etext as usize);
    println!(".rodata : {:X} - {:X}",srodata as usize,erodata as usize);
    println!(".data : {:X} - {:X}",sdata as usize,edata as usize);
    println!(".bss : {:X} - {:X}",sbss_with_stack as usize,ebss as usize);
    println!(".ekernel : {:X} ",ekernel as usize);
}
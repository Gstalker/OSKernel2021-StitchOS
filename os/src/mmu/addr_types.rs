use bitflags::*;

use crate::config::{PAGE_SIZE,PAGE_SIZE_BITS};
//mmu变量类型
//约定：PPN和VPN都保留高位，低位不会保留

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNumber(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNumber(pub usize);

//类型转换
impl From<usize> for PhysAddr{
    fn from(s: usize) -> Self{ Self(s) }
}

impl From<usize> for PhysPageNumber{
    fn from(s: usize) -> Self{ Self(s) }
}

impl From<usize> for VirtAddr{
    fn from(s: usize) -> Self{ Self(s) }
}

impl From<usize> for VirtPageNumber{
    fn from(s: usize) -> Self{ Self(s) }
}

impl From<VirtAddr> for usize{
    fn from(s: VirtAddr) -> Self{ s.0 }
}
impl From<PhysAddr> for usize{
    fn from(s: PhysAddr) -> Self{ s.0 }
}
impl From<PhysPageNumber> for usize{
    fn from(s: PhysPageNumber) -> Self{ s.0 }
}
impl From<VirtPageNumber> for usize{
    fn from(s: VirtPageNumber) -> Self{ s.0 }
}

impl From<VirtAddr> for VirtPageNumber{
    fn from(s: VirtAddr) -> Self{
        assert!(s.is_aligened());
        Self(s.0)
    }
}

impl From<VirtPageNumber> for VirtAddr{
    fn from(s: VirtPageNumber) -> Self{
        Self(s.0)
    }
}

impl From<PhysAddr> for PhysPageNumber{
    fn from(s: PhysAddr) -> Self{
        assert!(s.is_aligened());
        Self(s.0)
    }
}

impl From<PhysPageNumber> for PhysAddr{
    fn from(s: PhysPageNumber) -> Self{
        Self(s.0)
    }
}


// PhysAddr的接口函数
impl PhysAddr{
    pub fn floor(&self) -> usize{
        self.0 & !(PAGE_SIZE - 1)
    }
    pub fn celi(&self) -> usize{ 
        (self.0 - 1 + PAGE_SIZE) & !(PAGE_SIZE - 1)
    }
    pub fn is_aligened(&self) -> bool { 
        (self.0 & (PAGE_SIZE - 1)) == 0
    }
    pub fn get_page_offset(&self) -> usize{ 
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn get_ref<T>(&self) -> &'static T {
        unsafe {
            (self.0 as *const T).as_ref().unwrap()
        }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe {
            (self.0 as *mut T).as_mut().unwrap()
        }
    }
}

//PhysAddrPage类型的接口 主要是指针操作
impl PhysPageNumber{
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = self.clone().into();
        unsafe {
            //单个页表项
            #[repr(C)]
            #[derive(Copy,Clone)]
            pub struct PageDirectoryEntry(pub usize);
            core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096)
        }
    }
    pub fn get_pte_array(&self) -> &'static mut[PageDirectoryEntry]{
        println!("addr {}", self.0);
        let pa: PhysAddr = self.clone().into();
        unsafe{
            core::slice::from_raw_parts_mut(pa.0 as *mut PageDirectoryEntry, 512)
        }
    }
    pub fn get_mut<T>(&self) -> &'static mut T{
        let pa : PhysAddr = self.clone().into();
        // here needs a fix alg
        unsafe{
            //单个页表项
            #[repr(C)]
            #[derive(Copy,Clone)]
            pub struct PageDirectoryEntry(pub usize);
            (pa.0 as * mut T).as_mut().unwrap()
        }
    }
}

impl VirtAddr{
    pub fn floor(&self) -> VirtPageNumber{
        VirtPageNumber(self.0 & !(PAGE_SIZE - 1))
    }
    pub fn celi(&self) -> VirtPageNumber{ 
        VirtPageNumber((self.0 - 1 + PAGE_SIZE) & !(PAGE_SIZE - 1))
    }
    pub fn is_aligened(&self) -> bool { 
        (self.0 & (PAGE_SIZE - 1)) == 0
    }
    pub fn get_page_offset(&self) -> usize{ 
        self.0 & (PAGE_SIZE - 1)
    }
}

impl VirtPageNumber{
    // 三级页表索引的数组下标
    pub fn indexes(&self) -> [usize;3]{
        let mut vpn = self.0 >> PAGE_SIZE_BITS;
        let mut idx = [0usize; 3];
        for i in (0..3).rev(){
            idx[i] = vpn & (512 - 1);
            vpn >>=  9;
        }
        idx
    }
}

// 页目录低位的FLAGS，参见RISC-V的SV39多级页表机制
bitflags! {
    pub struct PDEFlags: u8 {
        const V = 1 << 0; // valid
        const R = 1 << 1; // readable
        const W = 1 << 2; // writable
        const X = 1 << 3; // executable
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}


//单个页表项
#[repr(C)]
#[derive(Copy,Clone)]
pub struct PageDirectoryEntry{
    pub item : usize,
}

impl PageDirectoryEntry{
    pub fn new(ppn: PhysPageNumber, flags: PDEFlags) -> Self{ 
        PageDirectoryEntry{
            item : (ppn.0 >> 2) | (flags.bits() as usize)
        }
    }
    pub fn get_page_number(&self) -> PhysPageNumber {
        let pn : usize = self.item;
        println!("self pn {}", pn);
        ((pn << 2) & !(PAGE_SIZE - 1)).into()
    }
    pub fn get_flags(&self) -> PDEFlags {
        PDEFlags::from_bits(self.item as u8).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        (self.get_flags() & PDEFlags::V) != PDEFlags::empty()
    }
    pub fn is_readable(&self) -> bool {
        (self.get_flags() & PDEFlags::R) != PDEFlags::empty()
    }
    pub fn is_writable(&self) -> bool {
        (self.get_flags() & PDEFlags::W) != PDEFlags::empty()
    }
    pub fn is_executable(&self) -> bool {
        (self.get_flags() & PDEFlags::X) != PDEFlags::empty()
    }
    pub fn empty() -> Self{ 
        Self{
            item : 0 as usize,
        }
    }
}
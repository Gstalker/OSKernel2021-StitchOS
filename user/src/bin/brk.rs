#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{brk};

#[no_mangle]
pub fn main() -> i32 {
    println!("brk test");
    println!("brk(0)     {:X}",brk(0));
    println!("brk(32)    {:X}",brk(32));
    println!("brk(0x1000){:X}",brk(0x1000));
    println!("brk(0x2048){:X}",brk(0x2048));
    println!("brk(0)     {:X}",brk(0));
    let ptr = brk(0) as usize;
    
    let prt = unsafe{core::slice::from_raw_parts_mut(ptr as *mut u8, 0x2099)};
    for i in 0..0x2048{
        prt[i] = 0xab;
    }
    println!("Write test : ptr[0x2021] = {:X}",prt[0x2021]);
    println!("brk test complete!");
    0
}


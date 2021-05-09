#![no_std]
#![no_main]
#![feature(llvm_asm)]

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    println!("Hello, world!");
    // unsafe {
    //     llvm_asm!("sret");
    // }
    
    // #  issue:
    // #  https://github.com/luojia65/rustsbi/issues/10
    0
}
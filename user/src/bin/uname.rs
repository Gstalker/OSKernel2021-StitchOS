#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::{
    utsname,
    uname,
};

#[no_mangle]
pub fn main() -> i32 {
    let mut sysname:[u8;65] = [0u8;65];
    let mut nodename:[u8;65] = [0u8;65];
    let mut release:[u8;65] = [0u8;65];
    let mut version:[u8;65] = [0u8;65];
    let mut machine:[u8;65] = [0u8;65];
    let mut domainname:[u8;65] = [0u8;65];
    let mut uts = utsname{
        sysname ,
        nodename,
        release ,
        version ,
        machine ,
        domainname,
    };
    println!("uname test");
    uname(&mut uts);
    println!("uname : {:?}", uts);
    0
}
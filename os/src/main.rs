#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(panic_info_message)]
#![feature(const_in_array_repeat_expressions)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
extern crate bitflags;

#[macro_use]
mod console;
mod lang_items;
mod sbi;
mod syscall;
mod trap;
mod loader;
mod config;
mod task;
mod timer;
mod fs;
mod mmu;
mod drivers;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    LOG!("Hello, world!");
    ERROR!("ERROR test");
    WARN!("WARN test");
    println!("[kernel] Hello, world!");
    mmu::init();
    println!("[kernel] back to world!");
    //mmu::remap_test();
    task::add_initproc();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();

    // test codes for fat32 file system -- disable this if not in use

    let mut root = fs::fat32::fat32_root_dir();
    println!("{:?}", root.ls());
    println!("{}", root.create_file("blank.txt"));

    let dir = root.child("dir2").unwrap();

    println!("inner {:?}", dir);
    println!("inner files {:?}", dir.ls());
    println!("into first task");
    loader::list_apps();

    // end of file system test

    //task::run_first_task();
    println!("second maybe");
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
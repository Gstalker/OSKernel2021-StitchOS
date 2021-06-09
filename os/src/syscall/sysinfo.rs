use crate::task::{
    suspend_current_and_run_next,
    exit_current_and_run_next,
    current_task,
    current_user_token,
    add_task,
};
use crate::mmu::{
    translated_str,
    translated_refmut,
    translated_ref,
};
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::string::String;
use crate::loader::get_app_data_by_name;
use crate::timer::get_time_ms;
use lazy_static::*;


#[repr(C)]
pub struct utsname{
    pub sysname : [u8;65],
    pub nodename : [u8;65],
    pub release : [u8;65],
    pub version : [u8;65],
    pub machine : [u8;65],
    pub domainname : [u8;65],
}


// 缺少校验逻辑：如果这个指针是无效指针呢？
pub fn sys_uname(us : *mut utsname) -> isize{
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    let buffer = translated_refmut(inner.mem_area.token(), us);
    buffer.sysname[0..6].copy_from_slice("rCore\x00".as_bytes());
    buffer.nodename[0..9].copy_from_slice("StitchOS\x00".as_bytes());
    buffer.release[0..20].copy_from_slice("Tropical Depression\x00".as_bytes());
    buffer.version[0..14].copy_from_slice("StitchOS 0.11\x00".as_bytes());
    buffer.machine[0..14].copy_from_slice("Kendryte K210\x00".as_bytes());
    buffer.domainname[0..13].copy_from_slice("CSU/StitchOS\x00".as_bytes());
    0
}
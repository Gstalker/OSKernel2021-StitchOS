use alloc::vec::Vec;
use lazy_static::*;
use crate::fs::*;

pub fn get_num_app() -> usize {
    extern "C" { fn _num_app(); }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    extern "C" { fn _num_app(); }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe {
        core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1)
    };
    assert!(app_id < num_app);
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id]
        )
    }
}

lazy_static! {
    static ref APP_NAMES: Vec<&'static str> = {
        let num_app = get_num_app();
        extern "C" { fn _app_names(); }
        let mut start = _app_names as usize as *const u8;
        let mut v = Vec::new();
        unsafe {
            for _ in 0..num_app {
                let mut end = start;
                while end.read_volatile() != '\0' as u8 {
                    end = end.add(1);
                }
                let slice = core::slice::from_raw_parts(start, end as usize - start as usize);
                let str = core::str::from_utf8(slice).unwrap();
                v.push(str);
                start = end.add(1);
            }
        }
        v
    };
}


#[allow(unused)]
pub fn get_app_data_by_name(path: &str) -> Option<Vec<u8>> {
    // let num_app = get_num_app();
    // (0..num_app)
    //     .find(|&i| APP_NAMES[i] == name)
    //     .map(|i| get_app_data(i))
    let root_dir = fat32::fat32_root_dir();
    if let Some(app_file) = root_dir.open_file(path){
        let app_size = app_file.len();
        let mut app_data : Vec<u8> = Vec::with_capacity(app_size);
        unsafe{app_data.set_len(app_size)};
        WARN!("App _data length : {:X}",app_size);
        app_file.read(app_data.as_mut_slice()).unwrap();
        Some(app_data)
    }
    else{
        None
    }
}

pub fn list_apps() {
    println!("/**** APPS ****");
    for app in APP_NAMES.iter() {
        println!("{}", app);
    }
    println!("**************/")
}
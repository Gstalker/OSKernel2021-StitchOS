use alloc::vec::Vec;
use lazy_static::*;
use crate::fs::*;


#[allow(unused)]
pub fn get_app_data_by_name(path: &str) -> Option<Vec<u8>> {
    let root_dir = fat32::fat32_path(path,None);
    let app_file = match root_dir.open(){
        Ok(file) => file,
        Err(err) => {
            WARN!("Couldn't open file {}",err);
            return None;
        }
    };
    let app_size = app_file.len();
    let mut app_data : Vec<u8> = Vec::with_capacity(app_size);
    unsafe{app_data.set_len(app_size)};
    app_file.read(app_data.as_mut_slice()).unwrap();
    Some(app_data)
}
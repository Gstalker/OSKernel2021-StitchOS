
use kfat32::block_dev::BlockDevice;
use kfat32::volume::Volume;
use kfat32::entry::*;
use kfat32::dir;
use crate::drivers::storage::BlockDeviceImpl;
use alloc::sync::Arc;
use super::inode::*;
use alloc::vec;
use alloc::vec::*;

#[derive(Copy, Clone, Debug)]
pub struct DummyDevice(usize);

const SECTOR_LEN : usize = 512;

// Dummy Device that selects SD card as block device and with a default 2048 sectors' offset against FAT32 in MBR
impl BlockDevice for DummyDevice {
    type Error = ();

    fn read(&self, buf: &mut [u8], address: usize, number_of_blocks: usize) -> Result<usize, Self::Error> {
        unsafe {
            let v =  &*(self.0 as *const BlockDeviceImpl);
            v.read(buf, address + SECTOR_LEN * 2048, number_of_blocks)
        }
    }
    fn write(&self, buf: &[u8], address: usize, number_of_blocks: usize) -> Result<usize, Self::Error> {
        unsafe {
            let v =  &*(self.0 as *const BlockDeviceImpl);
            v.write(buf, address + SECTOR_LEN * 2048, number_of_blocks)
        }
    }
}
use lazy_static::*;
use kfat32::entry::EntryType;

#[derive(Debug)]
pub struct Inode{
    pub(crate) dir: Option<dir::Dir<'static, DummyDevice>>,
    pub(crate) entry: Option<Entry>
}

impl Inode {

    pub fn child(&self, name : &str) -> Option<Inode> {
        self.dir.map(|dir| {
            dir.exist(name).map(|entry| {
                println!("child entry {:?} {:?}", entry, entry.item_type);
                match entry.item_type {
                    EntryType::File => {
                        println!("as file {:?}", entry);
                        Some(Inode{ dir: None, entry: Some(entry)})
                    }
                    EntryType::Dir => {
                        // we knew it's a directory, so unwrap
                        println!("as dir {:?}", entry);
                        Some(Inode{ dir: Some(dir.cd_entry(entry).unwrap()), entry: None})
                    }
                    _ => None
                }
            })
        }).flatten().flatten()
    }

    pub fn is_dir(&self) -> bool {
        if let Some(_) = self.dir {
            true
        } else {
            false
        }
    }

    pub fn is_file(&self) -> bool {
        return !self.is_dir()
    }

    pub fn open_sub_inode<'a>(&self, node: &Inode) -> Option<kfat32::file::File<'a, DummyDevice>> {
        self.dir.map(|dir| {
            node.entry.map(|entry| {
                dir.open_file_entry(entry).ok()
            })
        }).flatten().flatten()
    }

    pub fn open_file<'a>(&self, name: &str) -> Option<kfat32::file::File<'a, DummyDevice>> {
        self.dir.map(|dir| {
            dir.open_file(name).ok()
        }).flatten()
    }

    pub fn ls(&self) -> Vec<Inode> {
        println!("into ls");
        if let Some(dir) = self.dir {
            println!("into ls is dir");
            let mut result = vec![];
            for entry in dir.list_files() {
                println!("entry {:?}", entry);
                match entry.item_type {
                    EntryType::File => result.push(Inode{ dir: None, entry: Some(entry)}),
                    EntryType::Dir => {
                        // we knew it's a directory, so unwrap
                        result.push(Inode{ dir: Some(dir.cd_entry(entry).unwrap()), entry: None});
                    }
                    _ => {}
                }
            }
            result
        } else {
            vec![]
        }
    }

    /// Delete Dir
    pub fn delete_dir(&mut self, file: &str) -> bool {
        self.dir.map(|mut dir| {
            dir.delete_dir(file).is_ok()
        }).unwrap_or(false)
    }

    /// Delete File
    pub fn delete_file(&mut self, file: &str) -> bool  {
        self.dir.map(|mut dir| {
            dir.delete_file(file).is_ok()
        }).unwrap_or(false)
    }

    /// Create Dir
    pub fn create_dir(&mut self, file: &str) -> bool  {
        self.dir.map(|mut dir| {
            dir.create_dir(file).is_ok()
        }).unwrap_or(false)
    }

    /// Create File
    pub fn create_file(&mut self, file: &str) -> bool {
        self.dir.map(|mut dir| {
            dir.create_file(file).is_ok()
        }).unwrap_or(false)
    }
}

lazy_static! {
    pub static ref GLOBAL_VOLUME: Volume<DummyDevice> = {
        let dev = crate::drivers::storage::BLOCK_DEVICE.clone();
        Volume::new(DummyDevice(Arc::as_ptr(&dev) as *const _ as usize))
    };
}

pub fn fat32_label() -> &'static str {
    GLOBAL_VOLUME.volume_label()
}

pub fn fat32_root_dir() -> Inode {
    Inode { entry: None, dir: Some(GLOBAL_VOLUME.root_dir()) }
}

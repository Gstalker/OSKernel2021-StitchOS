use super::inode::*;
use crate::drivers::storage::BlockDeviceImpl;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::*;
use kfat32::block_dev::BlockDevice;
use kfat32::dir;
use kfat32::entry::*;
use kfat32::volume::Volume;

// 0 - handle to physical device, 1 - offset of partition
#[derive(Copy, Clone, Debug)]
pub struct PartitionDevice(usize, usize);

const SECTOR_LEN: usize = 512;

// Dummy Device that selects SD card as block device and with a default 2048 sectors' offset against FAT32 in MBR
impl BlockDevice for PartitionDevice {
    type Error = ();

    fn read(
        &self,
        buf: &mut [u8],
        address: usize,
        number_of_blocks: usize,
    ) -> Result<usize, Self::Error> {
        unsafe {
            let v = &*(self.0 as *const BlockDeviceImpl);
            v.read(buf, address + self.1 * 512, number_of_blocks)
        }
    }
    fn write(
        &self,
        buf: &[u8],
        address: usize,
        number_of_blocks: usize,
    ) -> Result<usize, Self::Error> {
        unsafe {
            let v = &*(self.0 as *const BlockDeviceImpl);
            v.write(buf, address + self.1 * 512, number_of_blocks)
        }
    }
}
use alloc::boxed::*;
use kfat32::entry::EntryType;
use lazy_static::*;

#[derive(Debug, Clone)]
pub struct Inode {
    pub(crate) dir: Option<dir::Dir<'static, PartitionDevice>>,
    pub(crate) entry: Option<Entry>,
    pub(crate) parent: Option<Box<Inode>>,
}

impl Inode {
    pub fn child(&self, name: &str) -> Option<Inode> {
        self.dir
            .map(|dir| {
                dir.exist(name).map(|entry| {
                    DEBUG!("child entry {:?} {:?}", entry, entry.item_type);
                    match entry.item_type {
                        EntryType::File => {
                            DEBUG!("as file {:?}", entry);
                            Some(Inode {
                                dir: None,
                                entry: Some(entry),
                                parent: Some(Box::new(self.clone())),
                            })
                        }
                        EntryType::Dir => {
                            // we knew it's a directory, so unwrap
                            DEBUG!("as dir {:?}", entry);
                            Some(Inode {
                                dir: Some(dir.cd_entry(entry).unwrap()),
                                entry: None,
                                parent: Some(Box::new(self.clone())),
                            })
                        }
                        _ => None,
                    }
                })
            })
            .flatten()
            .flatten()
    }

    pub fn is_dir(&self) -> bool {
        if let Some(_) = self.dir {
            true
        } else {
            false
        }
    }

    pub fn is_file(&self) -> bool {
        return !self.is_dir();
    }

    pub fn open_sub_inode<'a>(
        &self,
        node: &Inode,
    ) -> Option<kfat32::file::File<'a, PartitionDevice>> {
        self.dir
            .map(|dir| node.entry.map(|entry| dir.open_file_entry(entry).ok()))
            .flatten()
            .flatten()
    }

    pub fn open_file<'a>(&self, name: &str) -> Option<kfat32::file::File<'a, PartitionDevice>> {
        self.dir.map(|dir| dir.open_file(name).ok()).flatten()
    }

    pub fn open<'a>(&self) -> Option<kfat32::file::File<'a, PartitionDevice>> {
        self.parent
            .as_ref()
            .map(|parent| parent.open_sub_inode(self))
            .flatten()
    }

    pub fn parent(&self) -> Option<Inode> {
        self.parent.clone().map(|par| *par)
    }

    pub fn ls(&self) -> Vec<Inode> {
        DEBUG!("into ls");
        if let Some(dir) = self.dir {
            DEBUG!("into ls is dir");
            let mut result = vec![];
            for entry in dir.list_files() {
                DEBUG!("entry {:?}", entry);
                match entry.item_type {
                    EntryType::File => result.push(Inode {
                        dir: None,
                        entry: Some(entry),
                        parent: Some(Box::new(self.clone())),
                    }),
                    EntryType::Dir => {
                        // we knew it's a directory, so unwrap
                        result.push(Inode {
                            dir: Some(dir.cd_entry(entry).unwrap()),
                            entry: None,
                            parent: Some(Box::new(self.clone())),
                        });
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
        self.dir
            .map(|mut dir| dir.delete_dir(file).is_ok())
            .unwrap_or(false)
    }

    /// Delete File
    pub fn delete_file(&mut self, file: &str) -> bool {
        self.dir
            .map(|mut dir| dir.delete_file(file).is_ok())
            .unwrap_or(false)
    }

    /// Create Dir
    pub fn create_dir(&mut self, file: &str) -> bool {
        self.dir
            .map(|mut dir| dir.create_dir(file).is_ok())
            .unwrap_or(false)
    }

    /// Create File
    pub fn create_file(&mut self, file: &str) -> bool {
        self.dir
            .map(|mut dir| dir.create_file(file).is_ok())
            .unwrap_or(false)
    }
}
use super::ProgramBuffer;
use crate::syscall::fs::OpenFlags;

pub struct SysFile<'a>(kfat32::file::File<'a, PartitionDevice>, OpenFlags);

impl<'a> SysFile<'a> {
    fn new(file: kfat32::file::File<'a, PartitionDevice>, wt: OpenFlags) -> Self {
        SysFile(file, wt)
    }
}

impl<'a> crate::fs::File for SysFile<'a> {
    fn read(&self, buf: ProgramBuffer) -> usize {
        let mut len: usize = 0;
        for buffer in buf.buffers {
            len += self.0.read(buffer).unwrap();
            if len < buffer.len() {
                break;
            }
        }
        len
    }
    fn write(&mut self, buf: ProgramBuffer) -> usize {
        let mut len: usize = 0;
        for buffer in buf.buffers {
            len += buffer.len();
            self.0.write(buffer, self.1.into()).unwrap();
            if len < buffer.len() {
                break;
            }
        }
        len
    }
}

use super::mbr::*;
#[macro_use]
use crate::console;
use alloc::str;

fn create_volume_from_part(id: usize) -> Volume<PartitionDevice> {
    let mut sector: Vec<u8> = Vec::with_capacity(512);
    unsafe {
        sector.set_len(512);
    }
    DEBUG!("Initializing SD Block Device");
    let dev = crate::drivers::storage::BLOCK_DEVICE.clone();
    dev.read(sector.as_mut_slice(), 0, 1).unwrap();
    DEBUG!("Detecting Raw Fat32 System ...");
    let header = str::from_utf8(&sector[0x52..0x57]).unwrap_or("fail");
    let offset = if header.to_lowercase().eq("fat32") {
        DEBUG!("Raw Fat32 File System Detected ... why not partition it =.=?");
        0
    } else {
        let mbr = MasterBootRecord::from_sector(sector.as_slice());
        let active = mbr.partitions[0].is_active();
        DEBUG!(
            "Master Boot Record Read, partition {} status => {}",
            id,
            if active { "active" } else { "inactive" }
        );
        if !active {
            ERROR!("No active partition found in block device, the system might fail!")
        } else {
            DEBUG!(
                "Disk partition file system is {:?}, with {} sectors allocated",
                mbr.partitions[0].fs,
                mbr.partitions[0].size
            );
        }
        mbr.partitions[0].start_sector as usize
    };
    Volume::new(PartitionDevice(
        Arc::as_ptr(&dev) as *const _ as usize,
        offset,
    ))
}

lazy_static! {
    pub static ref GLOBAL_VOLUME: Volume<PartitionDevice> = create_volume_from_part(0);
}

pub fn fat32_label() -> &'static str {
    GLOBAL_VOLUME.volume_label()
}

pub fn fat32_root_dir() -> Inode {
    Inode {
        entry: None,
        dir: Some(GLOBAL_VOLUME.root_dir()),
        parent: None,
    }
}
/// If cwd is none, then it is equal to '/'
pub fn fat32_path(path: &str, cwd: Option<Inode>) -> Path {
    let root = fat32_root_dir();
    let cwd = cwd.unwrap_or(root.clone());
    Path::new(cwd, root, String::from(path))
}

use alloc::string::*;

#[derive(Clone, Debug)]
pub struct Path {
    location: Inode,
    root: Inode,
    path: String,
}

/// lazy resolved path struct, avoid using this as much as you can
impl Path {
    pub fn new(cwd: Inode, root: Inode, path: String) -> Self {
        Self {
            location: cwd,
            root,
            path,
        }
    }

    pub fn relative(&self, path: &str) -> Result<Path, &'static str> {
        resolve_path(&self.location, &self.root, self.path.as_str(), false)
            .map(|res| Path::new(res, self.root.clone(), String::from(path)))
    }

    pub fn mkdirs(&self) -> Result<(), &'static str> {
        resolve_path(&self.location, &self.root, self.path.as_str(), true).map(|_| ())
    }

    pub fn exists(&self) -> bool {
        resolve_path(&self.location, &self.root, self.path.as_str(), false).is_ok()
    }

    pub fn get_inode(&self) -> Result<Inode, &'static str> {
        resolve_path(&self.location, &self.root, self.path.as_str(), false)
    }

    pub fn open<'a>(&self) -> Result<kfat32::file::File<'a, PartitionDevice>, &'static str> {
        resolve_path(&self.location, &self.root, self.path.as_str(), false).and_then(|node| {
            match node.open() {
                Some(file) => Ok(file),
                _ => Err("cannot open file"),
            }
        })
    }

    pub fn ls(&self) -> Result<Vec<Inode>, &'static str> {
        resolve_path(&self.location, &self.root, self.path.as_str(), false).map(|entry| entry.ls())
    }
}

fn resolve_path(
    cwd: &Inode,
    root: &Inode,
    path: &str,
    mkmode: bool,
) -> Result<Inode, &'static str> {
    let mut cur = if path.chars().nth(0).unwrap_or('-') == '/' {
        cwd.clone()
    } else {
        root.clone()
    };
    for seg in path.split("/") {
        match seg {
            "." | "" => {}
            ".." => match cur.parent {
                Some(node) => {
                    cur = *node;
                }
                None => return Err("reach root dir for '..'"),
            },
            value => match cur.child(value) {
                Some(node) => {
                    cur = node;
                }
                None => {
                    if mkmode {
                        if cur.create_dir(value) {
                            cur = cur.child(value).unwrap()
                        } else {
                            return Err("cannot create dir");
                        }
                    } else {
                        return Err("filename not found");
                    }
                }
            },
        }
    }
    Ok(cur)
}

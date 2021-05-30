// Disk MBR Support

#[derive(Copy, Clone, Debug)]
pub struct MasterBootRecord {
    pub partitions: [PartitionRecord; 4]
}

impl MasterBootRecord {
    pub(crate) fn from_sector(sector: &[u8]) -> Self {
        Self {
            partitions: [
                PartitionRecord::new(&sector[0x1be..0x1ce]),
                PartitionRecord::new(&sector[0x1ce..0x1de]),
                PartitionRecord::new(&sector[0x1de..0x1ee]),
                PartitionRecord::new(&sector[0x1ee..0x1fe])
            ]
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum FileSystem {
    Unknown = 0x00,
    FAT12 = 0x01,
    FAT32 = 0x0b,
    FAT32Ex = 0x0c,
    NTFS = 0x07
}

impl FileSystem {
    fn from_byte(byte: u8) -> Self {
        match byte {
            0x01 => Self::FAT12,
            0x0b => Self::FAT32,
            0x0c => Self::FAT32Ex,
            0x07 => Self::NTFS,
            _ => Self::Unknown
        }
    }
}

pub const PARTITION_ACTIVE : u8 = 0x80;

#[derive(Copy, Clone, Debug)]
pub struct PartitionRecord {
    status: u8,
    start_chs: [u8; 3],
    pub fs: FileSystem,
    end_chs: [u8; 3],
    pub start_sector: u32,
    pub size: u32
}

// note that this is store in a reversed endian
fn read_u32(bytes: &[u8]) -> u32 {
    (bytes[3] as u32) << 24 |
    (bytes[2] as u32) << 16 | 
    (bytes[1] as u32) << 8 | 
    (bytes[0] as u32)
}

impl PartitionRecord {
    fn new(record: &[u8]) -> Self {
        let mut start_chs: [u8; 3] = [0; 3];
        let mut end_chs: [u8; 3] = [0; 3];
        start_chs.copy_from_slice(&record[1..4]);
        end_chs.copy_from_slice(&record[1..4]);
        Self {
            status: record[0],
            start_chs,
            fs: FileSystem::from_byte(record[4]),
            end_chs,
            start_sector: read_u32(&record[8..12]),
            size: read_u32(&record[12..16])
        }
    }

    pub fn is_active(&self) -> bool {
        self.status & PARTITION_ACTIVE != 0x0
    }
}


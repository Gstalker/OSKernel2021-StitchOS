//mod virtio_blk;
mod sdcard;

use lazy_static::*;
use alloc::sync::Arc;
use kfat32::block_dev::BlockDevice;

//#[cfg(feature = "board_k210")]
pub type BlockDeviceImpl = sdcard::SDCardWrapper;

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<BlockDeviceImpl> = Arc::new(BlockDeviceImpl::new());
}

#[allow(unused)]
pub fn block_device_test() {
    let block_device = BLOCK_DEVICE.clone();
    let mut write_buffer = [0u8; 512];
    let mut read_buffer = [0u8; 512];
    for i in 0..512 {
        for byte in write_buffer.iter_mut() { *byte = i as u8; }
        block_device.write(&write_buffer, i * 512, 1);
        block_device.read(&mut read_buffer, i * 512, 0);
        assert_eq!(write_buffer, read_buffer);
    }
    DEBUG!("block device test passed!");
}
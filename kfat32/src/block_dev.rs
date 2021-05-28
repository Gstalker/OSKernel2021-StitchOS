use core::any::Any;

pub trait BlockDevice : Send + Sync + Any {
    
    type Error = ();

    fn read(&self, buf: &mut [u8], address: usize, number_of_blocks: usize) -> Result<usize, Self::Error>;
    fn write(&self, buf: &[u8], address: usize, number_of_blocks: usize) -> Result<usize, Self::Error>;
}

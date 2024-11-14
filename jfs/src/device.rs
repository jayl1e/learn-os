use super::types::*;
pub trait BlkDev: Sync + Send {
    fn read(&self, blk: usize, buf: &mut [u8]) -> IOResult<()>;
    fn write(&self, blk: usize, buf: &[u8]) -> IOResult<()>;
}

#[cfg(test)]
pub mod test {
    use super::BlkDev;
    use alloc::vec::Vec;

    use super::super::types::*;
    pub struct MemoryBlockInner {
        pub blocks: Vec<[u8; BlockSize]>,
        pub write_cnt: usize,
        pub read_cnt: usize,
    }

    pub struct MemoryBlock {
        pub inner: *mut MemoryBlockInner,
    }
    unsafe impl Sync for MemoryBlock {}
    unsafe impl Send for MemoryBlock {}

    impl BlkDev for MemoryBlock {
        fn read(&self, blk: usize, buf: &mut [u8]) -> IOResult<()> {
            let inner = unsafe { &mut *self.inner };
            if blk >= inner.blocks.len() {
                return Err(IOError::NoSuchBlock);
            }
            if buf.len() != BlockSize {
                return Err(IOError::BadBufSize);
            }
            buf.copy_from_slice(&inner.blocks[blk]);
            inner.read_cnt += 1;
            Ok(())
        }
        fn write(&self, blk: usize, buf: &[u8]) -> IOResult<()> {
            let inner = unsafe { &mut *self.inner };
            if blk >= inner.blocks.len() {
                return Err(IOError::NoSuchBlock);
            }
            if buf.len() != BlockSize {
                return Err(IOError::BadBufSize);
            }
            inner.blocks[blk].copy_from_slice(buf);
            inner.write_cnt += 1;
            Ok(())
        }
    }
}

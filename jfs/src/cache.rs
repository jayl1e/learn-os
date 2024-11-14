use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use spin::Mutex;

use lazy_static::lazy_static;

use log::error;

use super::device::BlkDev;
use super::types::*;

pub struct BlockCache {
    blk_id: usize,
    buf: [u8; BlockSize],
    dirty: bool,
    dev: Arc<dyn BlkDev>,
}

impl BlockCache {
    pub fn new(blk_id: usize, dev: Arc<dyn BlkDev>) -> Result<Self, IOError> {
        let mut buf = [0u8; BlockSize];
        dev.read(blk_id, &mut buf)?;
        Ok(Self {
            blk_id,
            dev,
            buf: buf,
            dirty: false,
        })
    }

    pub fn write_back(&mut self) -> Result<(), IOError> {
        if self.dirty {
            self.dev.write(self.blk_id, &self.buf)?;
            self.dirty = false;
        }
        Ok(())
    }

    pub fn read<T>(&self, offset: usize, Fn: impl FnOnce(&T)) {
        assert!(offset + size_of::<T>() <= BlockSize);
        let ptr = &self.buf[offset] as *const u8 as *const T;
        unsafe { Fn(&*ptr) }
    }

    pub fn write<T>(&mut self, offset: usize, func: impl FnOnce(&mut T)) {
        func(self.ref_mut(offset));
    }
    pub fn ref_mut<T>(&mut self, offset: usize) -> &mut T {
        assert!(offset + size_of::<T>() <= BlockSize);
        let ptr = &mut self.buf[offset] as *mut u8 as *mut T;
        self.dirty = true;
        unsafe { &mut *ptr }
    }
}

impl Drop for BlockCache {
    fn drop(&mut self) {
        if let Err(_e) = self.write_back() {
            error!("write back block failed: blk_id={}", self.blk_id);
        }
    }
}

struct CacheManager {
    index: BTreeMap<usize, Arc<Mutex<BlockCache>>>,
    lru: Vec<usize>,
    limit: usize,
}

impl CacheManager {
    fn new(limit: usize) -> Self {
        let mut lru = Vec::new();
        lru.reserve(limit);
        Self {
            index: BTreeMap::new(),
            lru: lru,
            limit,
        }
    }
    fn get_block(
        &mut self,
        blk_id: usize,
        dev: Arc<dyn BlkDev>,
    ) -> Result<Arc<Mutex<BlockCache>>, IOError> {
        match self.index.get(&blk_id).map(|blk| Arc::clone(blk)) {
            Some(blk) => {
                self.update_access(blk_id);
                Ok(blk)
            }
            None => {
                if self.index.len() >= self.limit {
                    self.expire_oldest();
                }
                let blk = Arc::new(Mutex::new(BlockCache::new(blk_id, dev)?));
                self.index.insert(blk_id, Arc::clone(&blk));
                self.lru.push(blk_id);
                Ok(blk)
            }
        }
    }
    fn update_access(&mut self, blk_id: usize) {
        if let Some(pos) = self.lru.iter().position(|&x| x == blk_id) {
            self.lru.remove(pos);
        }
        self.lru.push(blk_id);
    }
    fn expire_oldest(&mut self) {
        let blk = self.lru.remove(0);
        let blk = self.index.remove(&blk).unwrap();
        drop(blk);
    }
    fn sync(&mut self) -> IOResult<()> {
        for (_i, blk) in self.index.iter_mut() {
            blk.lock().write_back()?;
        }
        Ok(())
    }
}

lazy_static! {
    static ref CACHE_MGR: Mutex<CacheManager> = { Mutex::new(CacheManager::new(32)) };
}

pub fn get_block(blk_id: usize, dev: Arc<dyn BlkDev>) -> IOResult<Arc<Mutex<BlockCache>>> {
    CACHE_MGR.lock().get_block(blk_id, dev)
}
pub fn sync_blocks() -> IOResult<()> {
    CACHE_MGR.lock().sync()
}

#[cfg(test)]
mod test {
    use alloc::sync::Arc;
    use alloc::vec;

    use super::CacheManager;
    use crate::device::test::{MemoryBlock, MemoryBlockInner};
    use crate::device::BlkDev;
    use crate::types::*;

    #[test]
    fn test_cache() {
        let mut blk_inner = MemoryBlockInner {
            blocks: vec![[1u8; BlockSize]; 32],
            read_cnt: 0,
            write_cnt: 0,
        };
        let dev: Arc<dyn BlkDev> = Arc::new(MemoryBlock {
            inner: &raw mut blk_inner,
        });
        let mut mgr = CacheManager::new(2);
        let c = mgr.get_block(1, dev.clone()).unwrap();
        let mut l = c.lock();
        assert_eq!(1, l.blk_id);
        l.read(1, |c: &u8| {
            assert_eq!(1, *c);
        });
        l.write(1, |c: &mut u8| {
            *c += 1;
        });
        drop(l);
        let _c = mgr.get_block(2, dev.clone()).unwrap();
        let _c = mgr.get_block(3, dev.clone()).unwrap();
        let _c = mgr.get_block(4, dev.clone()).unwrap();
        assert_eq!(4, blk_inner.read_cnt);
        assert_eq!(0, blk_inner.write_cnt);
        drop(c);
        assert_eq!(1, blk_inner.write_cnt);
    }
}

use alloc::sync::Arc;
use spin::Mutex;

use super::types::*;
use crate::{
    cache::{get_block, BlockCache},
    device::BlkDev,
};

const MAGIC: [u8; 4] = [b'\x18', b'j', b'f', b's'];
const InodeSize: usize = 128;
const InodePerBlock: usize = BlockSize / InodeSize;
const BlockInBlock: usize = BlockSize / size_of::<u32>();
#[repr(C)]
struct SuperBlock {
    magic: [u8; 4],
    version: u32,
    pub total_blocks: u32,
    pub inode_blocks: u32,
    pub data_blocks: u32,
}

impl SuperBlock {
    fn is_valid(&self) -> bool {
        self.magic == MAGIC
    }
    fn init(&mut self, total_blocks: u32, inode_blocks: u32, data_blocks: u32) {
        *self = Self {
            magic: MAGIC,
            version: 1,
            total_blocks,
            inode_blocks,
            data_blocks,
        }
    }
}

struct DiskPos {
    block_id: u32,
    offset: usize,
}

const DIRECT_BLOCKS: usize = 28;
#[derive(Debug)]
#[repr(C)]
struct DiskInode {
    file_type: FileType,
    size: u32,
    block0s: [u32; DIRECT_BLOCKS],
    block1: u32,
    block2: u32,
}

const _: () = assert!(InodeSize == size_of::<DiskInode>());

#[test]
fn test_size() {
    assert_eq!(InodeSize, size_of::<DiskInode>());
}

#[derive(Debug, PartialEq, Eq)]
enum FileType {
    IdleHead = 0,
    BlockGC = 1,
    File = 2,
    Directory = 3,
}

struct JFS {
    inode_start_block: u32,
    data_start_block: u32,
    data_end_block: u32,
    dev: Arc<dyn BlkDev>,
}

struct Inode {
    pos: DiskPos,
    fs: Arc<JFS>,
}

enum BlockPosition<'a> {
    InINode(&'a mut u32),
    Block(DiskPos),
}

struct BlockIndicator<'a> {
    pos: BlockPosition<'a>,
    is_block_table: bool,
}

impl DiskInode {
    fn inc_size(&mut self, sz: u32, new_blocks: Vec<u32>, jfs: &JFS) -> IOResult<()> {
        assert!(sz > self.size);
        let cur_block = Self::total_blocks(self.size);
        for (i, block) in new_blocks.into_iter().enumerate() {
            self.emplace_block(cur_block + i, block, jfs)?
        }
        self.size = sz;
        Ok(())
    }
    fn dec_size(&mut self, sz: u32, jfs: &JFS) -> IOResult<Vec<u32>> {
        assert!(sz < self.size);
        let mut rt = Vec::new();
        let cur_block = Self::total_blocks(self.size);
        let new_block = Self::total_blocks(sz);
        rt.reserve(cur_block - new_block);
        for b in (new_block..cur_block).rev() {
            let poped = self.pop_block(b, jfs)?;
            if poped == 0 {
                return Err(IOError::CorruptedFS);
            }
            rt.push(poped);
        }
        Ok(rt)
    }
    // block: when block=0, it will remove block, otherwise, it will add block
    fn replace_block_at(&mut self, at: usize, block: u32, fs: &JFS) -> IOResult<u32> {
        let mut old = 0;
        if at < DIRECT_BLOCKS {
            old = self.block0s[at];
            self.block0s[at] = block;
            return Ok(old);
        }
        if at == DIRECT_BLOCKS {
            old = self.block1;
            self.block1 = block;
            if block != 0 {
                let blk_lk = fs.get_block(block)?;
                blk_lk.lock().write(0, |bs: &mut [u32; BlockInBlock]| {
                    bs.fill(0);
                });
            }
            return Ok(old);
        }
        if at < DIRECT_BLOCKS + BlockInBlock + 1 {
            let blk_lk = fs.get_block(self.block1)?;
            blk_lk.lock().write(
                (at - DIRECT_BLOCKS - 1) * size_of::<u32>(),
                |b: &mut u32| {
                    old = *b;
                    *b = block;
                },
            );
            return Ok(old);
        }
        const L1_BLOCK_LIMIT: usize = DIRECT_BLOCKS + BlockInBlock + 1;
        if at == L1_BLOCK_LIMIT {
            old = self.block2;
            self.block2 = block;
            if block != 0 {
                let blk_lk = fs.get_block(block)?;
                blk_lk.lock().write(0, |bs: &mut [u32; BlockInBlock]| {
                    bs.fill(0);
                });
            }
            return Ok(old);
        }
        const L2_BLOCK_LIMIT: usize = L1_BLOCK_LIMIT + 1 + BlockInBlock * (BlockInBlock + 1);
        if at >= L2_BLOCK_LIMIT {
            return Err(IOError::DiskFull);
        };
        let pos_in_l2 = at - L1_BLOCK_LIMIT - 1;
        let l2_bt = pos_in_l2 / (1 + BlockInBlock);
        let off_in_l2_l1_p1 = pos_in_l2 % (1 + BlockInBlock); // offset in l2->l1, plus 1
        if off_in_l2_l1_p1 == 0 {
            if block != 0 {
                let blk_lk = fs.get_block(block)?;
                blk_lk.lock().write(0, |bs: &mut [u32; BlockInBlock]| {
                    bs.fill(0);
                });
            }
            let l2_blk = fs.get_block(self.block2)?;
            l2_blk
                .lock()
                .write(l2_bt * size_of::<u32>(), |b: &mut u32| {
                    old = *b;
                    *b = block;
                });
            Ok(old)
        } else {
            let l2_blk = fs.get_block(self.block2)?;
            let mut l2_l1_block = 0;
            l2_blk.lock().read(l2_bt * size_of::<u32>(), |b: &u32| {
                l2_l1_block = *b;
            });
            let l2_l1_blk = fs.get_block(l2_l1_block)?;
            l2_l1_blk
                .lock()
                .write((off_in_l2_l1_p1 - 1) * size_of::<u32>(), |b: &mut u32| {
                    old = *b;
                    *b = block;
                });
            Ok(old)
        }
    }

    fn emplace_block(&mut self, at: usize, block: u32, fs: &JFS) -> IOResult<()> {
        let old = self.replace_block_at(at, block, fs)?;
        if old != 0 {
            return Err(IOError::CorruptedFS);
        }
        Ok(())
    }
    fn pop_block(&mut self, at: usize, fs: &JFS) -> IOResult<u32> {
        self.replace_block_at(at, 0, fs)
    }

    fn total_blocks(sz: u32) -> usize {
        let eblocks = (sz as usize).div_ceil(BlockSize);
        if eblocks <= DIRECT_BLOCKS {
            return eblocks;
        }
        if eblocks <= DIRECT_BLOCKS + BlockInBlock {
            return eblocks + 1;
        }
        eblocks + 1 + 1 + (eblocks - DIRECT_BLOCKS - BlockInBlock).div_ceil(BlockInBlock)
    }
}

impl JFS {
    fn mkfs(dev: Arc<dyn BlkDev>, total_blocks: u32, inode_blocks: u32) -> IOResult<Self> {
        let data_blocks = total_blocks - inode_blocks - 1;
        let s = Self {
            inode_start_block: 1,
            data_start_block: inode_blocks + 1,
            data_end_block: total_blocks,
            dev: dev,
        };
        let super_block = s.get_block(0)?;
        super_block.lock().write(0, |su: &mut SuperBlock| {
            su.init(total_blocks, inode_blocks, data_blocks);
        });
        {
            let idle_head = s.idle_head_pos();
            s.get_block(idle_head.block_id)?.lock().write(
                idle_head.offset,
                |idle_root: &mut DiskInode| {
                    idle_root.file_type = FileType::IdleHead;
                    idle_root.block1 = 0;
                },
            );
        }
        for inode_id in (2..s.inode_cnt()).rev() {
            s.dealloc_inode(inode_id)?;
        }
        {
            let free_pos = s.block_gc_pos();
            s.get_block(free_pos.block_id)?.lock().write(
                free_pos.offset,
                |free: &mut DiskInode| {
                    free.file_type = FileType::BlockGC;
                    free.size = 0;
                    free.block0s.fill(0);
                    free.block1 = 0;
                    free.block2 = 0;
                },
            );
        }
        for block_id in s.data_start_block..s.data_end_block {
            s.dealloc_block(block_id)?;
        }
        {
            let root_dir = s.root_dir_pos();
            s.get_block(root_dir.block_id)?.lock().write(
                root_dir.offset,
                |root: &mut DiskInode| {
                    root.file_type = FileType::Directory;
                    root.block0s.fill(0);
                    root.block1 = 0;
                    root.block2 = 0;
                },
            );
        }
        Ok(s)
    }

    fn inode_cnt(&self) -> u32 {
        (self.data_start_block - self.inode_start_block) * InodePerBlock as u32
    }

    fn root_dir(self: Arc<Self>) -> Inode {
        let pos = self.get_inode_pos(1);
        Inode { pos, fs: self }
    }

    fn from_dev(dev: Arc<dyn BlkDev>) -> IOResult<Self> {
        let mut s = Self {
            inode_start_block: 0,
            data_start_block: 0,
            data_end_block: 0,
            dev: dev,
        };
        let su = s.get_block(0)?;
        su.lock().read(0, |sb: &SuperBlock| {
            s.inode_start_block = 1;
            s.data_start_block = sb.inode_blocks + 1;
            s.data_end_block = sb.total_blocks;
        });
        Ok(s)
    }

    fn get_block(&self, blk_id: u32) -> IOResult<Arc<Mutex<BlockCache>>> {
        get_block(blk_id as usize, Arc::clone(&self.dev))
    }
    fn get_inode_pos(&self, id: u32) -> DiskPos {
        let block_id = self.inode_start_block + id / InodePerBlock as u32;
        let offset = (id as usize % InodePerBlock) * InodeSize;
        DiskPos { block_id, offset }
    }

    fn idle_head_pos(&self) -> DiskPos {
        DiskPos {
            block_id: 0,
            offset: 2 * InodeSize,
        }
    }

    fn block_gc_pos(&self) -> DiskPos {
        DiskPos {
            block_id: 0,
            offset: 3 * InodeSize,
        }
    }
    fn root_dir_pos(&self) -> DiskPos {
        self.get_inode_pos(0)
    }

    fn alloc_inode(&self) -> Result<u32, IOError> {
        let head_pos = self.idle_head_pos();
        let head_blk_lk = self.get_block(head_pos.block_id)?;
        let mut next = 0;
        let mut head_blk = head_blk_lk.lock();
        head_blk.read(head_pos.offset, |inode: &DiskInode| {
            next = inode.block1;
        });
        if next == 0 {
            return Err(IOError::DiskFull);
        }
        let next_pos = self.get_inode_pos(next);
        let mut next_next = 0;
        if next_pos.block_id != head_pos.block_id {
            let nxt_blk = self.get_block(next_pos.block_id)?;
            nxt_blk.lock().read(next_pos.offset, |inode: &DiskInode| {
                next_next = inode.block1;
            });
        } else {
            head_blk.read(next_pos.offset, |inode: &DiskInode| {
                next_next = inode.block1;
            });
        }
        head_blk.write(head_pos.offset, |inode: &mut DiskInode| {
            inode.block1 = next_next;
        });
        Ok(next)
    }

    fn dealloc_inode(&self, inode_id: u32) -> Result<(), IOError> {
        let head_pos = self.idle_head_pos();
        let head_blk_lk = self.get_block(head_pos.block_id)?;
        let mut next_next = 0;
        let mut head_blk = head_blk_lk.lock();
        head_blk.read(head_pos.offset, |inode: &DiskInode| {
            next_next = inode.block1;
        });

        let next_pos = self.get_inode_pos(inode_id);
        if next_pos.block_id != head_pos.block_id {
            let nxt_blk = self.get_block(next_pos.block_id)?;
            nxt_blk
                .lock()
                .write(next_pos.offset, |inode: &mut DiskInode| {
                    inode.file_type = FileType::IdleHead;
                    inode.block1 = next_next;
                });
        } else {
            head_blk.write(next_pos.offset, |inode: &mut DiskInode| {
                inode.file_type = FileType::IdleHead;
                inode.block1 = next_next;
            });
        }
        head_blk.write(head_pos.offset, |inode: &mut DiskInode| {
            inode.block1 = inode_id;
        });
        Ok(())
    }

    fn alloc_block(&self) -> IOResult<u32> {
        for _ in 0..5 {
            match self._alloc_block() {
                Ok(0) => {
                    continue;
                }
                other => {
                    return other;
                }
            }
        }
        panic!("should return after 5 times alloc retry")
    }
    fn _alloc_block(&self) -> IOResult<u32> {
        let pos = self.block_gc_pos();
        let blk_lk = self.get_block(pos.block_id)?;
        let mut blk = blk_lk.lock();
        let mut rt = Err(IOError::Unknown);
        blk.write(pos.offset, |free: &mut DiskInode| {
            if free.size == 0 {
                rt = Err(IOError::DiskFull);
                return;
            }
            for b in free.block0s.iter_mut() {
                if *b != 0 {
                    rt = Ok(*b);
                    *b = 0;
                    free.size -= BlockSize as u32;
                    return;
                }
            }
            if free.block1 != 0 {
                let blk = self.get_block(free.block1);
                if let Err(e) = blk {
                    rt = Err(e);
                    return;
                }
                let blk_lk = blk.unwrap();
                let mut blk = blk_lk.lock();
                let mut current_index = 0;
                let fill_cnt = free.block0s.len() / 2;
                for block0_ref in free.block0s[0..fill_cnt].iter_mut() {
                    if current_index >= BlockInBlock {
                        break;
                    }
                    blk.write(0, |block_id_array: &mut [u32; BlockInBlock]| {
                        while current_index <= BlockInBlock {
                            if current_index == BlockInBlock {
                                *block0_ref = free.block1;
                                break;
                            }
                            if block_id_array[current_index] == 0 {
                                current_index += 1;
                                continue;
                            }
                            *block0_ref = block_id_array[current_index];
                            block_id_array[current_index] = 0;
                            current_index += 1;
                        }
                    });
                }
                rt = Ok(0);
                return;
            }
            if free.block2 != 0 {
                let blk_lk = self.get_block(free.block2);
                if let Err(e) = blk_lk {
                    rt = Err(e);
                    return;
                }
                let blk_lk = blk_lk.unwrap();
                let mut blk = blk_lk.lock();
                blk.write(0, |blk_id_array: &mut [u32; BlockInBlock]| {
                    for blk in blk_id_array.iter_mut() {
                        if *blk != 0 {
                            free.block1 = *blk;
                            *blk = 0;
                            break;
                        }
                    }
                });
                if free.block1 == 0 {
                    free.block1 = free.block2;
                    free.block2 = 0;
                }
                rt = Ok(0)
            }
            panic!("should not be empty")
        });
        rt
    }

    fn dealloc_block(&self, block_id: u32) -> IOResult<()> {
        let pos = self.block_gc_pos();
        let blk_lk = self.get_block(pos.block_id)?;
        let mut blk = blk_lk.lock();
        let free: &mut DiskInode = blk.ref_mut(pos.offset);
        for block in free.block0s.iter_mut() {
            if *block == 0 {
                *block = block_id;
                free.size += BlockSize as u32;
                return Ok(());
            }
        }
        if free.block1 == 0 {
            let blk = self.get_block(block_id)?;
            blk.lock().write(0, |blk_arr: &mut [u32; BlockInBlock]| {
                blk_arr[..free.block0s.len()].copy_from_slice(&free.block0s);
                blk_arr[free.block0s.len()..].fill(0);
            });
            free.block1 = block_id;
            free.block0s.fill(0);
            free.size += BlockSize as u32;
            return Ok(());
        } else {
            let blk1_lk = self.get_block(free.block1)?;
            let mut blk1 = blk1_lk.lock();
            let blk1_arr: &mut [u32; BlockInBlock] = blk1.ref_mut(0);
            for blk in blk1_arr.iter_mut() {
                if *blk == 0 {
                    *blk = block_id;
                    free.size += BlockSize as u32;
                    return Ok(());
                }
            }
        }
        if free.block2 == 0 {
            let blk = self.get_block(block_id)?;
            blk.lock().write(0, |blk_arr: &mut [u32; BlockInBlock]| {
                blk_arr[0] = free.block1;
                blk_arr[1..].fill(0);
            });
            free.block2 = block_id;
            free.block1 = 0;
            free.size += BlockSize as u32;
            return Ok(());
        } else {
            let blk2_lk = self.get_block(free.block2)?;
            let mut blk2 = blk2_lk.lock();
            let blk2_arr: &mut [u32; BlockInBlock] = blk2.ref_mut(0);
            for blk in blk2_arr.iter_mut() {
                if *blk == 0 {
                    *blk = free.block1;
                    free.block1 = 0;
                    free.size += BlockSize as u32;
                    return Ok(());
                }
            }
        }
        Err(IOError::DiskFull)
    }
}

#[cfg(test)]
mod test {
    use crate::cache::sync_blocks;
    use crate::device::test::{MemoryBlock, MemoryBlockInner};
    use crate::device::BlkDev;
    use crate::jfs::{BlockInBlock, DiskInode, FileType, MAGIC};
    use crate::types::*;
    use alloc::sync::Arc;
    use alloc::vec;

    use super::JFS;

    #[test]
    fn test_mkfs() {
        let mut blk_inner = MemoryBlockInner {
            blocks: vec![[0u8; BlockSize]; 2048],
            read_cnt: 0,
            write_cnt: 0,
        };
        let dev: Arc<dyn BlkDev> = Arc::new(MemoryBlock {
            inner: &raw mut blk_inner,
        });
        let fs = JFS::mkfs(dev, 2048, 31).unwrap();
        sync_blocks().unwrap();
        assert_eq!(MAGIC, blk_inner.blocks[0][..4]);
        let inode = fs.root_dir_pos();
        let inode_blk = fs.get_block(inode.block_id).unwrap();
        let sz = (BlockSize * 29) as u32;
        let blocks_needed = DiskInode::total_blocks(sz);
        assert_eq!(30, blocks_needed);
        let mut blocks = vec![];
        for _ in 0..blocks_needed {
            blocks.push(fs.alloc_block().unwrap());
        }
        println!("blocks {:?}", blocks);
        panic!("block not valid"); //todo fix this
        inode_blk
            .lock()
            .write(inode.offset, |inode: &mut DiskInode| {
                inode.inc_size(sz, blocks, &fs).unwrap();
            });
        print_blocks(0, &fs);
        print_blocks(-1, &fs);
        print_blocks(-2, &fs);
    }
    fn print_blocks(inode_id: i32, fs: &JFS) {
        let free_pos = match inode_id {
            -1 => fs.block_gc_pos(),
            -2 => fs.idle_head_pos(),
            other => fs.get_inode_pos(other as u32),
        };
        let block = fs.get_block(free_pos.block_id).unwrap();
        let mut block1 = 0;
        let mut block2 = 0;
        let mut is_idle = false;
        block.lock().read(free_pos.offset, |inode: &DiskInode| {
            println!("inode: {}: {:?}", inode_id, inode);
            block1 = inode.block1;
            block2 = inode.block2;
            is_idle = inode.file_type == FileType::IdleHead;
        });
        if is_idle || block1 == 0 {
            return;
        }
        let blk1 = fs.get_block(block1).unwrap();
        blk1.lock().read(0, |bb: &[u32; BlockInBlock]| {
            println!("block 1 at {}: {:?}", block1, bb);
        });
        if block2 == 0 {
            return;
        }
        let blk2 = fs.get_block(block2).unwrap();
        let mut free_blk2_0 = 0;
        blk2.lock().read(0, |bb: &[u32; BlockInBlock]| {
            println!("block 2 at {}: {:?}", block2, bb);
            free_blk2_0 = bb[0];
        });
        let blk2_0 = fs.get_block(free_blk2_0).unwrap();
        blk2_0.lock().read(0, |bb: &[u32; BlockInBlock]| {
            println!("block 2.0 at {}: {:?}", free_blk2_0, bb);
        });
    }
}

use alloc::vec;
use alloc::vec::Vec;
use bitflags::bitflags;

use super::{
    address::{PhysPageNum, VirtPageNum},
    frame_allocator::{frame_new, FrameGuard},
};

const STAP_PPN_BIT: usize = 44;

bitflags! {
    pub struct PTEFlags:u8{
        const V = 1<<0;
        const R = 1<<1;
        const W = 1<<2;
        const X = 1<<3;
        const U = 1<<4;
        const G = 1<<5;
        const A = 1<<6;
        const D = 1<<7;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry {
    bits: usize,
}

impl PageTableEntry {
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    fn empty() -> Self {
        Self { bits: 0 }
    }
    fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        Self {
            bits: ppn.0 << 10 | flags.bits() as usize,
        }
    }
    fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V)
    }
    pub fn readable(&self) -> bool {
        self.flags().contains(PTEFlags::R)
    }
    pub fn writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
    }
    pub fn executable(&self) -> bool {
        self.flags().contains(PTEFlags::X)
    }
}

pub struct PageTable {
    root: PhysPageNum,
    // the frames is only pages that use to store page table
    frames: Vec<FrameGuard>,
}

impl PageTable {
    pub fn new() -> Self {
        let root = frame_new().unwrap();
        Self {
            root: root.ppn,
            frames: vec![root],
        }
    }
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_or_create_pte(vpn).unwrap();
        assert_eq!(
            false,
            pte.is_valid(),
            "map an already valid page: vpn {:?}",
            vpn
        );
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        match self.find_pte(vpn) {
            Some(pte) if pte.is_valid() => {
                *pte = PageTableEntry::empty();
            }
            _ => {
                panic!("unmap an invalid page: vpn {:?}", vpn)
            }
        }
    }
    fn find_or_create_pte(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.index();
        let mut ppn = self.root;
        let mut rt = None;
        for i in 0..idxs.len() {
            let pte = &mut ppn.pte_array_mut()[idxs[i]];
            if i + 1 == idxs.len() {
                rt = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let f = frame_new().unwrap();
                *pte = PageTableEntry::new(f.ppn, PTEFlags::V);
                self.frames.push(f);
            }
            ppn = pte.ppn();
        }
        rt
    }

    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.index();
        let mut ppn = self.root;
        let mut rt = None;
        for i in 0..idxs.len() {
            let pte = &mut ppn.pte_array_mut()[idxs[i]];
            if i + 1 == idxs.len() {
                rt = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            } else {
                ppn = pte.ppn();
            }
        }
        rt
    }

    pub fn from_token(satp: usize) -> Self {
        Self {
            root: PhysPageNum(satp & ((1 << STAP_PPN_BIT) - 1)),
            frames: vec![],
        }
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }

    pub fn token(&self) -> usize {
        // 8 means sv39
        8usize << 60 | self.root.0
    }
}

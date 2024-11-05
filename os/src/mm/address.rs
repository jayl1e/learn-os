use core::{ops::AddAssign, slice};

use super::page_table::PageTableEntry;

const PAGE_SIZE_BIT: usize = 12;
const PA_WIDTH_SV39: usize = 56;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BIT;
pub const PAGE_SIZE: usize = 1 << PAGE_SIZE_BIT;
const RV39_INDEX_CNT: usize = 3;
const RV39_INDEX_BIT: usize = 9;
const RV39_INDEX_MASK: usize = (1 << RV39_INDEX_BIT) - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtAddress(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysAddress(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtPageNum(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysPageNum(pub usize);

impl VirtPageNum {
    pub fn index(&self) -> [usize; RV39_INDEX_CNT] {
        let mut vpn = self.0;
        let mut idx = [0; RV39_INDEX_CNT];
        for i in idx.iter_mut().rev() {
            *i = vpn & RV39_INDEX_MASK;
            vpn >>= RV39_INDEX_BIT;
        }
        idx
    }
}

impl AddAssign<usize> for VirtPageNum {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}
impl From<usize> for VirtAddress {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<usize> for PhysAddress {
    fn from(value: usize) -> Self {
        Self(value & ((1 << PA_WIDTH_SV39) - 1))
    }
}

impl From<usize> for PhysPageNum {
    fn from(value: usize) -> Self {
        Self(value & ((1 << PPN_WIDTH_SV39) - 1))
    }
}

impl Into<usize> for PhysAddress {
    fn into(self) -> usize {
        self.0
    }
}

impl Into<usize> for PhysPageNum {
    fn into(self) -> usize {
        self.0
    }
}

impl PhysAddress {
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 >> PAGE_SIZE_BIT)
    }
    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE - 1) >> PAGE_SIZE_BIT)
    }
}

impl VirtAddress {
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 >> PAGE_SIZE_BIT)
    }
    pub fn ceil(&self) -> VirtPageNum {
        VirtPageNum((self.0 - 1 + PAGE_SIZE) >> PAGE_SIZE_BIT)
    }
}
impl From<VirtAddress> for VirtPageNum {
    fn from(value: VirtAddress) -> Self {
        assert_eq!(0, value.page_offset());
        value.floor()
    }
}

impl From<VirtPageNum> for VirtAddress {
    fn from(value: VirtPageNum) -> Self {
        Self(value.0 << PAGE_SIZE_BIT)
    }
}

impl From<PhysAddress> for PhysPageNum {
    fn from(value: PhysAddress) -> Self {
        assert_eq!(0, value.page_offset());
        value.floor()
    }
}

impl From<PhysPageNum> for PhysAddress {
    fn from(value: PhysPageNum) -> Self {
        Self(value.0 << PAGE_SIZE_BIT)
    }
}

impl PhysPageNum {
    pub fn bytes_mut(&self) -> &'static mut [u8] {
        let start: usize = PhysAddress::from(*self).into();
        unsafe { slice::from_raw_parts_mut(start as *mut u8, PAGE_SIZE) }
    }
    pub fn pte_array_mut(self) -> &'static mut [PageTableEntry] {
        let pa: usize = PhysAddress::from(self).into();
        unsafe {
            slice::from_raw_parts_mut(
                pa as *mut PageTableEntry,
                PAGE_SIZE / size_of::<PageTableEntry>(),
            )
        }
    }
    pub fn get_mut<T>(self) -> &'static mut T {
        let pa: usize = PhysAddress::from(self).into();
        unsafe { &mut *(pa as *mut T) }
    }
}

#[derive(Debug, Clone)]
pub struct VPNRange {
    pub l: VirtPageNum,
    pub r: VirtPageNum,
}

impl VPNRange {
    pub fn new(l: VirtPageNum, r: VirtPageNum) -> Self {
        Self { l, r }
    }
}

impl IntoIterator for &VPNRange {
    type IntoIter = VPNRangeIter;
    type Item = VirtPageNum;
    fn into_iter(self) -> Self::IntoIter {
        VPNRangeIter {
            c: self.l,
            r: self.r,
        }
    }
}

pub struct VPNRangeIter {
    c: VirtPageNum,
    r: VirtPageNum,
}

impl Iterator for VPNRangeIter {
    type Item = VirtPageNum;
    fn next(&mut self) -> Option<Self::Item> {
        if self.c < self.r {
            let c = self.c;
            self.c += 1;
            Some(c)
        } else {
            None
        }
    }
}

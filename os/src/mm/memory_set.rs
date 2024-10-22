use core::{
    arch::asm,
    cmp::{max, min},
};

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use bitflags::bitflags;
use lazy_static::lazy_static;
use log::debug;
use riscv::register;
use xmas_elf;

use crate::{
    config::{KERNEL_STACK_LIMIT, USER_STACK_LIMIT},
    mm::address::PhysAddress,
    println,
    sync::up::UPSafeCell,
};

use super::{
    address::{VPNRange, VirtAddress, VirtPageNum, PAGE_SIZE},
    frame_allocator::{frame_new, FrameGuard, MEMORY_END},
    page_table::{PTEFlags, PageTable},
};

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

struct MapArea {
    vpns: VPNRange,
    frames: BTreeMap<VirtPageNum, FrameGuard>,
    map_type: MapType,
    map_perm: MapPermission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MapType {
    Identical,
    Framed,
}

bitflags! {
    pub struct MapPermission:u8{
        const R = 1<<1;
        const W = 1<<2;
        const X = 1<<3;
        const U = 1<<4;
    }
}

impl MapArea {
    fn new(start: VirtAddress, end: VirtAddress, tp: MapType, perm: MapPermission) -> Self {
        Self {
            vpns: VPNRange::new(start.floor(), end.ceil()),
            frames: BTreeMap::new(),
            map_type: tp,
            map_perm: perm,
        }
    }
    fn map(&mut self, pt: &mut PageTable) {
        for vpn in &self.vpns {
            self.map_one(vpn, pt)
        }
    }
    fn unmap(&mut self, pt: &mut PageTable) {
        for vpn in &self.vpns {
            self.unmap_one(vpn, pt)
        }
    }
    fn map_one(&mut self, vpn: VirtPageNum, pt: &mut PageTable) {
        let ppn = match self.map_type {
            MapType::Framed => {
                let frame = frame_new().unwrap();
                let ppn = frame.ppn;
                self.frames.insert(vpn, frame);
                ppn
            }
            MapType::Identical => vpn.0.into(),
        };
        let flags = PTEFlags::from_bits(self.map_perm.bits()).unwrap();
        pt.map(vpn, ppn, flags);
    }

    fn unmap_one(&mut self, vpn: VirtPageNum, pt: &mut PageTable) {
        match self.map_type {
            MapType::Framed => {
                self.frames.remove(&vpn).unwrap();
            }
            MapType::Identical => {}
        }
        pt.unmap(vpn);
    }

    fn copy_data(&mut self, data: &[u8], pt: &PageTable) {
        assert_eq!(MapType::Framed, self.map_type);
        let len = data.len();
        let mut wrote = 0;
        for vpn in &self.vpns {
            let to_wrote = min(PAGE_SIZE, len - wrote);
            if to_wrote <= 0 {
                break;
            }
            let ppn = pt.translate(vpn).unwrap().ppn();
            let dst = &mut ppn.bytes_mut()[..to_wrote];
            let src = &data[wrote..wrote + to_wrote];
            dst.copy_from_slice(src);
            wrote += to_wrote
        }
        assert_eq!(len, wrote)
    }
}

pub struct MemorySet {
    pub page_table: PageTable,
    areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn bare_new() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    fn push(&mut self, mut area: MapArea, data: Option<&[u8]>) {
        area.map(&mut self.page_table);
        if let Some(data) = data {
            area.copy_data(data, &self.page_table)
        }
        self.areas.push(area);
    }

    pub fn insert_frame(&mut self, start: VirtAddress, end: VirtAddress, perm: MapPermission) {
        let area = MapArea::new(start, end, MapType::Framed, perm);
        self.push(area, None);
    }

    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            register::satp::write(satp);
            asm!("sfence.vma");
        }
    }

    fn map_trampoline(&mut self) {
        extern "C" {
            fn strampoline();
        }
        self.page_table.map(
            VirtAddress(TRAMPOLINE).into(),
            PhysAddress(strampoline as usize).into(),
            PTEFlags::X | PTEFlags::R,
        );
    }
    pub fn new_app_from_elf(elf: &[u8]) -> (Self, usize, usize) {
        let mut ms = MemorySet::bare_new();
        ms.map_trampoline();
        let elf = xmas_elf::ElfFile::new(elf).unwrap();
        let magic = elf.header.pt1.magic;
        assert_eq!([0x7f, 0x45, 0x4c, 0x46], magic, "bad elf file");
        let mut max_end_vpn: VirtPageNum = VirtPageNum(0);
        for header in elf.program_iter() {
            if header.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddress = (header.virtual_addr() as usize).into();
                let end_va: VirtAddress = (start_va.0 + header.mem_size() as usize).into();
                let mut flag = MapPermission::U;
                let hf = header.flags();
                if hf.is_read() {
                    flag |= MapPermission::R;
                }
                if hf.is_write() {
                    flag |= MapPermission::W;
                }
                if hf.is_execute() {
                    flag |= MapPermission::X;
                }
                let area = MapArea::new(start_va, end_va, MapType::Framed, flag);
                max_end_vpn = max(max_end_vpn, area.vpns.r);
                let data = &elf.input
                    [header.offset() as usize..(header.offset() + header.file_size()) as usize];
                ms.push(area, Some(data))
            }
        }

        // +1 to set gap page
        max_end_vpn += 1;
        // user stack
        let stack_bottom: VirtAddress = max_end_vpn.into();
        let stack_top = VirtAddress(stack_bottom.0 + USER_STACK_LIMIT);
        ms.push(
            MapArea::new(
                stack_bottom,
                stack_top,
                MapType::Framed,
                MapPermission::U | MapPermission::R | MapPermission::W,
            ),
            None,
        );
        // map the trap context page
        ms.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        (ms, stack_top.0, elf.header.pt2.entry_point() as usize)
    }
}

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
}

fn new_kernel_map() -> MemorySet {
    debug!("new memory set");
    let mut ms = MemorySet::bare_new();
    debug!("map trapoline");
    ms.map_trampoline();
    debug!("map text: [{:#x},{:#x}]", stext as usize, etext as usize);
    ms.push(
        MapArea::new(
            (stext as usize).into(),
            (etext as usize).into(),
            MapType::Identical,
            MapPermission::X | MapPermission::R,
        ),
        None,
    );
    debug!("map rodata");
    ms.push(
        MapArea::new(
            (srodata as usize).into(),
            (erodata as usize).into(),
            MapType::Identical,
            MapPermission::R,
        ),
        None,
    );
    debug!("map data");
    ms.push(
        MapArea::new(
            (sdata as usize).into(),
            (edata as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ),
        None,
    );
    debug!("map bss");
    ms.push(
        MapArea::new(
            (sbss_with_stack as usize).into(),
            (ebss as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ),
        None,
    );
    debug!("map endkernel");
    ms.push(
        MapArea::new(
            (ekernel as usize).into(),
            (MEMORY_END as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ),
        None,
    );
    ms
}

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> =
        Arc::new(unsafe { UPSafeCell::new(new_kernel_map()) });
}

#[allow(unused)]
fn test_kernel_remap() {
    let mut kernel = KERNEL_SPACE.exclusive_access();
    let text_seg = kernel
        .page_table
        .translate(VirtAddress(stext as usize).into())
        .unwrap();
    assert_eq!(true, text_seg.executable());
    assert_eq!(true, text_seg.readable());
    assert_eq!(false, text_seg.writable());
    let rodata = kernel
        .page_table
        .translate(VirtAddress(srodata as usize).into())
        .unwrap();
    assert_eq!(true, rodata.readable());
    assert_eq!(false, rodata.writable());
    assert_eq!(false, rodata.executable());
    println!("test kernel map permission passed")
}

pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_LIMIT + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_LIMIT;
    (bottom, top)
}

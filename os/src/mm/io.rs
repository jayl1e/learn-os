use core::cmp::min;

use super::{page_table::PageTable, PhysPageNum, VirtAddress};

pub struct IOError {
    pub msg: &'static str,
}
pub trait Reader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError>;
}

pub trait Writer {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IOError>;
}

pub struct UserBuf {
    start: usize,
    end: usize,
    pt: PageTable,
}
impl UserBuf {
    pub fn new(satp: usize, ptr: *const u8, len: usize) -> Self {
        Self {
            start: ptr as usize,
            end: ptr as usize + len,
            pt: PageTable::from_token(satp),
        }
    }
}
impl Reader for UserBuf {
    fn read(&mut self, mut buf: &mut [u8]) -> Result<usize, IOError> {
        let end_ppn = self.pt.translate(VirtAddress::from(self.end).floor());
        let end_ppn = match end_ppn {
            Some(p) => p.ppn(),
            None => {
                return Err(IOError {
                    msg: "bad page mapping",
                });
            }
        };
        let start_va = VirtAddress::from(self.start);
        let mut readed = 0;
        while self.start < self.end {
            let start_ppn = match self.pt.translate(start_va.floor()) {
                Some(p) => p.ppn(),
                None => {
                    return Err(IOError {
                        msg: "bad page mapping",
                    });
                }
            };
            let read_buf = if start_ppn == end_ppn {
                let end_va = VirtAddress::from(self.end);
                &mut start_ppn.bytes_mut()[start_va.page_offset()..end_va.page_offset()]
            } else {
                &mut start_ppn.bytes_mut()[start_va.page_offset()..]
            };
            let to_read = min(buf.len(), read_buf.len());
            buf[..to_read].copy_from_slice(&read_buf[..to_read]);
            readed += to_read;
            buf = &mut buf[to_read..];
            self.start += to_read;
            if buf.len() == 0 {
                break;
            }
        }
        Ok(readed)
    }
}

pub struct UserBufMut(UserBuf);

impl UserBufMut {
    pub fn new(satp: usize, ptr: *mut u8, len: usize) -> Self {
        Self(UserBuf {
            start: ptr as usize,
            end: ptr as usize + len,
            pt: PageTable::from_token(satp),
        })
    }
}

impl Reader for UserBufMut {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
        self.0.read(buf)
    }
}

impl Writer for UserBufMut {
    fn write(&mut self, mut buf: &[u8]) -> Result<usize, IOError> {
        let s = &mut self.0;
        let end_ppn = s.pt.translate(VirtAddress::from(s.end).floor());
        let end_ppn = match end_ppn {
            Some(p) => p.ppn(),
            None => {
                return Err(IOError {
                    msg: "bad page mapping",
                });
            }
        };
        let start_va = VirtAddress::from(s.start);
        let mut written = 0;
        while s.start < s.end {
            let start_ppn = match s.pt.translate(start_va.floor()) {
                Some(p) => p.ppn(),
                None => {
                    return Err(IOError {
                        msg: "bad page mapping",
                    });
                }
            };
            let write_buf = if start_ppn == end_ppn {
                let end_va = VirtAddress::from(s.end);
                &mut start_ppn.bytes_mut()[start_va.page_offset()..end_va.page_offset()]
            } else {
                &mut start_ppn.bytes_mut()[start_va.page_offset()..]
            };
            let to_write = min(buf.len(), write_buf.len());
            write_buf[..to_write].copy_from_slice(&buf[..to_write]);
            written += to_write;
            buf = &buf[to_write..];
            s.start += to_write;
            if buf.len() == 0 {
                break;
            }
        }
        Ok(written)
    }
}

pub fn iter_from_user_ptr(ptr: *const u8, token: usize)->BytePtrIter{
    BytePtrIter{
        pt: PageTable::from_token(token),
        ptr: VirtAddress(ptr as usize),
        ppn:None
    }
}

pub fn translate_ptr_mut<T>(ptr: *mut T, token: usize)->Option<&'static mut T>{
    let pt = PageTable::from_token(token);
    let va = VirtAddress::from(ptr as usize);
    match  pt.translate(va.floor()){
        None=>None,
        Some(e)=>{
            if !e.writable(){
                None
            }else{
                Some(e.ppn().get_mut_at_offset(va.page_offset()))
            }
        }
        
    }
}

pub struct BytePtrIter{
    pt: PageTable,
    ptr: VirtAddress,
    ppn: Option<PhysPageNum>
}

impl Iterator for BytePtrIter {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.ppn.is_none(){
            let entry = self.pt.translate(self.ptr.floor());
            match entry {
                None=>{
                    return None;
                },
                Some(e)=>{
                    self.ppn = Some(e.ppn())
                }
            }
        }
        let b :u8= *self.ppn.unwrap().get_mut_at_offset(self.ptr.page_offset());
        self.ptr.0+=1;
        if self.ptr.page_offset() == 0{
            self.ppn=None
        }
        Some(b)
    }
}
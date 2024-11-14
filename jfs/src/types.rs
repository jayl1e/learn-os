pub const BlockSize: usize = 512;

#[derive(Debug)]
pub enum IOError {
    Unknown,
    NoSuchBlock,
    BadBufSize,
    DiskFull,
    CorruptedFS,
    DeviceBusy,
}

pub type IOResult<T> = core::result::Result<T, IOError>;

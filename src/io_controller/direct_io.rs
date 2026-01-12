use std::alloc::{alloc, dealloc, Layout};
use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind};
use std::path::Path;
use std::ptr::NonNull;

#[cfg(windows)]
const FILE_FLAG_NO_BUFFERING: u32 = 0x20000000;
#[cfg(windows)]
const FILE_FLAG_WRITE_THROUGH: u32 = 0x80000000;

pub(super) fn align_up(value: usize, alignment: usize) -> usize {
    if alignment == 0 {
        return value;
    }
    (value + alignment - 1) / alignment * alignment
}

pub(super) fn align_down_u64(value: u64, alignment: u64) -> u64 {
    if alignment == 0 {
        return value;
    }
    value / alignment * alignment
}

pub(super) fn resolve_block_size(block_size: usize) -> io::Result<usize> {
    if block_size == 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "Block size must be greater than zero.",
        ));
    }
    Ok(align_up(block_size, super::DIRECT_IO_ALIGNMENT))
}

pub(super) fn open_direct_write(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(FILE_FLAG_NO_BUFFERING | FILE_FLAG_WRITE_THROUGH);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_DIRECT);
    }
    options.open(path)
}

pub(super) fn open_direct_read(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(FILE_FLAG_NO_BUFFERING);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_DIRECT);
    }
    options.open(path)
}

pub(super) struct AlignedBuffer {
    ptr: NonNull<u8>,
    len: usize,
    alignment: usize,
}

impl AlignedBuffer {
    pub(super) fn new(len: usize, alignment: usize) -> io::Result<Self> {
        let layout = Layout::from_size_align(len, alignment).map_err(|_| {
            io::Error::new(ErrorKind::InvalidInput, "Invalid alignment for buffer.")
        })?;
        let ptr = unsafe { alloc(layout) };
        let ptr = NonNull::new(ptr).ok_or_else(|| {
            io::Error::new(ErrorKind::Other, "Failed to allocate aligned buffer.")
        })?;
        Ok(Self {
            ptr,
            len,
            alignment,
        })
    }

    pub(super) fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        let Ok(layout) = Layout::from_size_align(self.len, self.alignment) else {
            return;
        };
        unsafe {
            dealloc(self.ptr.as_ptr(), layout);
        }
    }
}

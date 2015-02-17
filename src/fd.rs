
use std::ffi;
use std::os;
use libc::{open, read, write, close, c_int, size_t, mode_t, c_void};
use err::Errno;

macro_rules! errno_check {
    ($status:expr, $success:expr) => (
        if $status == -1 { Err(Errno::new(os::errno())) } else { Ok($success) }
    )
}

pub type SysResult<T> = Result<T, Errno>;

// open() flags on my x86-64 Linux system
// Not intended to be portable!
bitflags! {
    flags OpenFlags: c_int {
        const O_ACCMODE   = 0b0000_0000_0000_0000_0000_0011,
        const O_RDONLY    = 0b0000_0000_0000_0000_0000_0000,
        const O_WRONLY    = 0b0000_0000_0000_0000_0000_0001,
        const O_RDWR      = 0b0000_0000_0000_0000_0000_0010,
        const O_CREAT     = 0b0000_0000_0000_0000_0100_0000,
        const O_EXCL      = 0b0000_0000_0000_0000_1000_0000,
        const O_NOCTTY    = 0b0000_0000_0000_0001_0000_0000,
        const O_TRUNC     = 0b0000_0000_0000_0010_0000_0000,
        const O_APPEND    = 0b0000_0000_0000_0100_0000_0000,
        const O_NONBLOCK  = 0b0000_0000_0000_1000_0000_0000,
        const O_NDELAY    = 0b0000_0000_0000_1000_0000_0000,
        const O_DSYNC     = 0b0000_0000_0001_0000_0000_0000,
        const O_DIRECT    = 0b0000_0000_0100_0000_0000_0000,
        const O_LARGEFILE = 0b0000_0000_1000_0000_0000_0000,
        const O_DIRECTORY = 0b0000_0001_0000_0000_0000_0000,
        const O_NOFOLLOW  = 0b0000_0010_0000_0000_0000_0000,
        const O_NOATIME   = 0b0000_0100_0000_0000_0000_0000,
        const O_CLOEXEC   = 0b0000_1000_0000_0000_0000_0000,
        const O_SYNC      = 0b0001_0000_0001_0000_0000_0000,
        const O_PATH      = 0b0010_0000_0000_0000_0000_0000,
        const O_TMPFILE   = 0b0100_0001_0000_0000_0000_0000,
    }
}

bitflags! {
    flags FilePerms: mode_t {
        const S_ISUID = 0b100_000_000_000,
        const S_ISGID = 0b010_000_000_000,
        const S_ISVTX = 0b001_000_000_000,
        const S_IRUSR = 0b000_100_000_000,
        const S_IWUSR = 0b000_010_000_000,
        const S_IXUSR = 0b000_001_000_000,
        const S_IRGRP = 0b000_000_100_000,
        const S_IWGRP = 0b000_000_010_000,
        const S_IXGRP = 0b000_000_001_000,
        const S_IROTH = 0b000_000_000_100,
        const S_IWOTH = 0b000_000_000_010,
        const S_IXOTH = 0b000_000_000_001,
        const S_IRWXU = S_IRUSR.bits | S_IWUSR.bits | S_IXUSR.bits,
        const S_IRWXG = S_IRGRP.bits | S_IWGRP.bits | S_IXGRP.bits,
        const S_IRWXO = S_IROTH.bits | S_IWOTH.bits | S_IXOTH.bits,
    }
}

pub struct FileDescriptor(c_int);

impl FileDescriptor {

    pub fn open(
        path: String, flags: OpenFlags, mode: FilePerms
    ) -> SysResult<FileDescriptor> {
        let cstring_path = ffi::CString::from_vec(path.into_bytes()).as_ptr();
        let fd = unsafe { open(cstring_path, flags.bits(), mode.bits()) };
        errno_check!(fd, FileDescriptor(fd))
    }

    pub fn read(&self, buf: &mut [u8]) -> SysResult<usize> {
        let buf_ptr = buf.as_mut_ptr() as *mut c_void;
        let buf_len = buf.len() as size_t;
        let bytes_read = unsafe { read(self.raw(), buf_ptr, buf_len) };
        errno_check!(bytes_read, bytes_read as usize)
    }

    pub fn write(&self, buf: &[u8]) -> SysResult<usize> {
        let buf_ptr = buf.as_ptr() as *const c_void;
        let buf_len = buf.len() as size_t;
        let bytes_written = unsafe { write(self.raw(), buf_ptr, buf_len) };
        errno_check!(bytes_written, bytes_written as usize)
    }

    // We cannot safely provide a Drop impl to handle this automatically;
    // it does not provide a mechanism for handling errors
    pub fn close(self) -> SysResult<()> {
        let status = unsafe { close(self.raw()) };
        errno_check!(status, ())
    }

    fn raw(&self) -> c_int {
        let &FileDescriptor(fd) = self;
        fd
    }

}


//! Provides operations on file descriptors.

use std::ffi;
use std::os;
use libc::{open, read, write, close, c_int, size_t, mode_t, c_void};
use err::Errno;

/// The result of a system call.
pub type SysResult<T> = Result<T, Errno>;

/// Newtype for file descriptors.
///
/// Does not implement `Copy` so that `FileDescriptor::close()` can
/// take ownership, preventing file descriptors from being used
/// afterwards.
pub struct FileDescriptor {
    raw: c_int,
}

/// Factors out the common operation of creating a `SysResult` based
/// on a syscall return value and `errno`.
macro_rules! errno_check {
    ($status:expr, $success:expr) => (
        if $status == -1 { Err(Errno::new(os::errno())) } else { Ok($success) }
    )
}

impl FileDescriptor {

    /// The `open` system call.
    ///
    /// ## Arguments
    ///
    /// - `path`: the pathname of the file to open. Ownership is
    /// transferred to this function so that it can be converted into
    /// a C-style string.
    /// - `flags`: specifies the access mode, file creation flags, and
    /// file status flags using a dedicated type.
    /// - `mode`: specifies the permissions to give the file if it is
    /// being created, using a dedicated type. If the file is not
    /// being created, an empty set of flags should be supplied.
    ///
    /// Returns a new file descriptor on success, or an appropriate
    /// `Errno` on failure.
    ///
    /// Consult the manpage (command `man 2 open`) for further
    /// details.
    pub fn open(
        path: String, flags: OpenFlags, mode: FilePerms
    ) -> SysResult<FileDescriptor> {
        let cstring_path = ffi::CString::from_vec(path.into_bytes()).as_ptr();
        let fd = unsafe { open(cstring_path, flags.bits(), mode.bits()) };
        errno_check!(fd, FileDescriptor { raw: fd, })
    }

    pub fn read(&self, buf: &mut [u8]) -> SysResult<usize> {
        let buf_ptr = buf.as_mut_ptr() as *mut c_void;
        let buf_len = buf.len() as size_t;
        let bytes_read = unsafe { read(self.raw, buf_ptr, buf_len) };
        errno_check!(bytes_read, bytes_read as usize)
    }

    pub fn write(&self, buf: &[u8]) -> SysResult<usize> {
        let buf_ptr = buf.as_ptr() as *const c_void;
        let buf_len = buf.len() as size_t;
        let bytes_written = unsafe { write(self.raw, buf_ptr, buf_len) };
        errno_check!(bytes_written, bytes_written as usize)
    }

    // We cannot safely provide a Drop impl to handle this automatically;
    // it does not provide a mechanism for handling errors
    pub fn close(self) -> SysResult<()> {
        let status = unsafe { close(self.raw) };
        errno_check!(status, ())
    }

}

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

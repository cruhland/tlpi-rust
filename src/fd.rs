
//! Provides operations on file descriptors.

use std::ffi;
use std::io;
use libc::{open, read, write, close, lseek, ftruncate};
use libc::{c_int, size_t, mode_t, c_void, off_t};
use libc::{STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO};
use err::Errno;

/// The result of a system call.
pub type SysResult<T> = Result<T, Errno>;

/// Newtype for file descriptors.
///
/// Does not implement `Copy` so that `FileDescriptor::close()` can
/// take ownership, preventing file descriptors from being used
/// afterwards.
pub struct FileDescriptor(c_int);

/// File descritor for standard input
pub const STDIN: FileDescriptor = FileDescriptor(STDIN_FILENO);

/// File descriptor for standard output
pub const STDOUT: FileDescriptor = FileDescriptor(STDOUT_FILENO);

/// File descriptor for standard error
pub const STDERR: FileDescriptor = FileDescriptor(STDERR_FILENO);

/// Factors out the common operation of creating a `SysResult` based
/// on a syscall return value and `errno`.
macro_rules! errno_check {
    ($status:expr, $success:expr) => (
        {
            let errno = io::Error::last_os_error().raw_os_error().unwrap();
            if $status == -1 { Err(Errno::new(errno)) } else { Ok($success) }
        }
    )
}

impl FileDescriptor {

    /// The `open()` system call.
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
    /// Consult the man page (command `man 2 open`) for further
    /// details.
    pub fn open(
        path: String, flags: OpenFlags, mode: FilePerms
    ) -> SysResult<FileDescriptor> {
        // Panic if `path` contains nul chars; crude but good enough
        let cstring_path = ffi::CString::new(path).unwrap().as_ptr();
        let fd = unsafe { open(cstring_path, flags.bits(), mode.bits()) };
        errno_check!(fd, FileDescriptor(fd))
    }

    /// The `read()` system call.
    ///
    /// Copies up to `buf.len()` bytes from the file into `buf`,
    /// returning the number of bytes read.
    ///
    /// Consult the man page (command `man 2 read`) for further
    /// details.
    pub fn read(&self, buf: &mut [u8]) -> SysResult<usize> {
        let buf_ptr = buf.as_mut_ptr() as *mut c_void;
        let buf_len = buf.len() as size_t;
        let bytes_read = unsafe { read(self.0, buf_ptr, buf_len) };
        errno_check!(bytes_read, bytes_read as usize)
    }

    /// The `write()` system call.
    ///
    /// Attempts to copy `buf` to the file, returning the actual
    /// number of bytes copied (which may be smaller than `buf.len()`
    /// in rare circumstances).
    ///
    /// Consult the man page (command `man 2 write`) for further
    /// details.
    pub fn write(&self, buf: &[u8]) -> SysResult<usize> {
        let buf_ptr = buf.as_ptr() as *const c_void;
        let buf_len = buf.len() as size_t;
        let bytes_written = unsafe { write(self.0, buf_ptr, buf_len) };
        errno_check!(bytes_written, bytes_written as usize)
    }

    /// The `close()` system call.
    ///
    /// Cleans up kernel resources for the file descriptor; it can no
    /// longer be used after this call returns. To enforce this at the
    /// Rust level, the file descriptor is moved into this method and
    /// is not moved out.
    ///
    /// We cannot safely provide a `Drop` impl to handle this
    /// automatically; it does not provide a mechanism for handling
    /// errors.
    ///
    /// Consult the man page (command `man 2 close`) for further
    /// details.
    pub fn close(self) -> SysResult<()> {
        let status = unsafe { close(self.0) };
        errno_check!(status, ())
    }

    /// The `lseek()` system call.
    ///
    /// Adjusts the offset of the file to the value of `offset` under
    /// the interpretation of `whence`, returning the resulting
    /// absolute offset.
    ///
    /// Consult the man page (command `man 2 lseek`) for further
    /// details.
    pub fn lseek(&self, offset: i64, whence: OffsetBase) -> SysResult<u64> {
        let abs_offset = unsafe {
            lseek(self.0, offset as off_t, whence as i32)
        };
        errno_check!(abs_offset, abs_offset as u64)
    }

    /// The `ftruncate()` system call.
    ///
    /// Changes the size of the file to `length` bytes.
    ///
    /// Consult the man page (command `man 2 ftruncate`) for further
    /// details.
    pub fn ftruncate(&self, length: i64) -> SysResult<()> {
        let status = unsafe { ftruncate(self.0, length as off_t) };
        errno_check!(status, ())
    }

}

bitflags! {
    #[doc = "Access mode, file creation, and file status flags for `open()`"]
    #[doc = "and related system calls."]
    #[doc = ""]
    #[doc = "Consult `man 2 open` for details on each flag."]
    #[doc = ""]
    #[doc = "Taken from C header files on an x86-64 Linux system; not"]
    #[doc = "intended to be portable!"]
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
    #[doc = "File permissions flags."]
    #[doc = ""]
    #[doc = "Consult `man 2 stat` for details. The values are portable"]
    #[doc = "across Unix-based systems."]
    flags FilePerms: mode_t {
        #[doc = "set-user-ID bit"]
        const S_ISUID = 0b100_000_000_000,
        #[doc = "set-group-ID bit"]
        const S_ISGID = 0b010_000_000_000,
        #[doc = "sticky bit"]
        const S_ISVTX = 0b001_000_000_000,
        #[doc = "owner has read permission"]
        const S_IRUSR = 0b000_100_000_000,
        #[doc = "owner has write permission"]
        const S_IWUSR = 0b000_010_000_000,
        #[doc = "owner has execute permission"]
        const S_IXUSR = 0b000_001_000_000,
        #[doc = "group has read permission"]
        const S_IRGRP = 0b000_000_100_000,
        #[doc = "group has write permission"]
        const S_IWGRP = 0b000_000_010_000,
        #[doc = "group has execute permission"]
        const S_IXGRP = 0b000_000_001_000,
        #[doc = "others have read permission"]
        const S_IROTH = 0b000_000_000_100,
        #[doc = "others have write permission"]
        const S_IWOTH = 0b000_000_000_010,
        #[doc = "others have execute permission"]
        const S_IXOTH = 0b000_000_000_001,
        #[doc = "mask for file owner permissions"]
        const S_IRWXU = S_IRUSR.bits | S_IWUSR.bits | S_IXUSR.bits,
        #[doc = "mask for group permissions"]
        const S_IRWXG = S_IRGRP.bits | S_IWGRP.bits | S_IXGRP.bits,
        #[doc = "mask for permissions for others (not in group)"]
        const S_IRWXO = S_IROTH.bits | S_IWOTH.bits | S_IXOTH.bits,
    }
}

/// Interpretations for the `offset` argument of `lseek()`.
pub enum OffsetBase {
    /// The offset is set to `offset` bytes.
    SeekSet  = 0,
    /// The offset is set to its current location plus `offset`
    /// bytes.
    SeekCur  = 1,
    /// The offset is set to the size of the file plus `offset`
    /// bytes.
    SeekEnd  = 2,
    /// Adjust the file offset to the next location in the file ≥
    /// `offset` which contains data.
    ///
    /// If `offset` points to data, then the file offset is set to
    /// `offset`.
    ///
    /// Available since Linux version 3.1.
    SeekData = 3,
    /// Adjust the file offset to the next hole in the file ≥
    /// `offset`.
    ///
    /// If `offset` points into the middle of a hole, then the file
    /// offset is set to `offset`. If there is no hole past `offset`,
    /// then the file offset is adjusted to the end of the file (i.e.,
    /// there is an implicit hole at the end of any file).
    ///
    /// Available since Linux version 3.1.
    SeekHole = 4,
}

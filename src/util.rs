
use std::fmt;
use std::old_io as io;
use std::ffi;
use std::os;
use libc::{open, read, write, close, c_int, size_t, mode_t, c_void};

#[macro_export]
macro_rules! usage_err {
    ($($arg:tt)*) => (
        tlpi_rust::util::usage_err_fmt(format_args!($($arg)*))
    )
}

#[macro_export]
macro_rules! err_exit {
    ($errno:expr, $($arg:tt)*) => (
        tlpi_rust::util::err_exit_fmt($errno, format_args!($($arg)*))
    )
}

#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => (
        tlpi_rust::util::fatal_fmt(format_args!($($arg)*))
    )
}

macro_rules! errno_check {
    ($status:expr, $success:expr) => (
        if $status == -1 { Err(Errno(os::errno())) } else { Ok($success) }
    )
}

static ENAME: [&'static str; 134] = [
    "",
    "EPERM", "ENOENT", "ESRCH", "EINTR", "EIO", "ENXIO",
    "E2BIG", "ENOEXEC", "EBADF", "ECHILD",
    "EAGAIN/EWOULDBLOCK", "ENOMEM", "EACCES", "EFAULT",
    "ENOTBLK", "EBUSY", "EEXIST", "EXDEV", "ENODEV",
    "ENOTDIR", "EISDIR", "EINVAL", "ENFILE", "EMFILE",
    "ENOTTY", "ETXTBSY", "EFBIG", "ENOSPC", "ESPIPE",
    "EROFS", "EMLINK", "EPIPE", "EDOM", "ERANGE",
    "EDEADLK/EDEADLOCK", "ENAMETOOLONG", "ENOLCK", "ENOSYS",
    "ENOTEMPTY", "ELOOP", "", "ENOMSG", "EIDRM", "ECHRNG",
    "EL2NSYNC", "EL3HLT", "EL3RST", "ELNRNG", "EUNATCH",
    "ENOCSI", "EL2HLT", "EBADE", "EBADR", "EXFULL", "ENOANO",
    "EBADRQC", "EBADSLT", "", "EBFONT", "ENOSTR", "ENODATA",
    "ETIME", "ENOSR", "ENONET", "ENOPKG", "EREMOTE",
    "ENOLINK", "EADV", "ESRMNT", "ECOMM", "EPROTO",
    "EMULTIHOP", "EDOTDOT", "EBADMSG", "EOVERFLOW",
    "ENOTUNIQ", "EBADFD", "EREMCHG", "ELIBACC", "ELIBBAD",
    "ELIBSCN", "ELIBMAX", "ELIBEXEC", "EILSEQ", "ERESTART",
    "ESTRPIPE", "EUSERS", "ENOTSOCK", "EDESTADDRREQ",
    "EMSGSIZE", "EPROTOTYPE", "ENOPROTOOPT",
    "EPROTONOSUPPORT", "ESOCKTNOSUPPORT",
    "EOPNOTSUPP/ENOTSUP", "EPFNOSUPPORT", "EAFNOSUPPORT",
    "EADDRINUSE", "EADDRNOTAVAIL", "ENETDOWN", "ENETUNREACH",
    "ENETRESET", "ECONNABORTED", "ECONNRESET", "ENOBUFS",
    "EISCONN", "ENOTCONN", "ESHUTDOWN", "ETOOMANYREFS",
    "ETIMEDOUT", "ECONNREFUSED", "EHOSTDOWN", "EHOSTUNREACH",
    "EALREADY", "EINPROGRESS", "ESTALE", "EUCLEAN",
    "ENOTNAM", "ENAVAIL", "EISNAM", "EREMOTEIO", "EDQUOT",
    "ENOMEDIUM", "EMEDIUMTYPE", "ECANCELED", "ENOKEY",
    "EKEYEXPIRED", "EKEYREVOKED", "EKEYREJECTED",
    "EOWNERDEAD", "ENOTRECOVERABLE", "ERFKILL", "EHWPOISON"
];

pub type TlpiErr<'a> = Fn(&mut io::Writer) -> () + 'a;

pub fn write_err<'a, F>(fmt: fmt::Arguments, err: F)
    where F: Fn(&mut io::Writer) -> () + 'a
{
    io::stdio::stdout().flush().unwrap();

    let mut stderr = io::stdio::stderr();
    err(&mut stderr);
    stderr.write_fmt(fmt).unwrap();
    write!(&mut stderr, "\n").unwrap();
    stderr.flush().unwrap();
}

pub fn usage_err_fmt(fmt: fmt::Arguments) -> bool {
    write_err(fmt, |&: writer| {
        write!(writer, "Usage: ").unwrap();
    });
    false
}

pub fn err_exit_fmt(errno: Errno, fmt: fmt::Arguments) -> bool {
    let Errno(err) = errno;
    let error_name =
        if err > 0 && err < ENAME.len() { ENAME[err] } else { "?UNKNOWN?" };
    let io_error = io::IoError::from_errno(err, true);
    let detail = match io_error.detail {
        Some(ref d) => format!(" ({})", d),
        _ => String::new()
    };

    write_err(fmt, |&: writer| {
        write!(
            writer, "ERROR [{} ({:?}); {}{}] ", error_name, io_error.kind,
            io_error.desc, detail
        ).unwrap();
    });
    false
}

pub fn fatal_fmt(fmt: fmt::Arguments) -> bool {
    write_err(fmt, |&: writer| {
        write!(writer, "ERROR: ").unwrap();
    });
    false
}

#[derive(Copy, Debug)]
pub struct Errno(usize);

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

pub struct FileDescriptor(c_int);

impl FileDescriptor {

    pub fn open(
        path: String, flags: OpenFlags, mode: mode_t
    ) -> SysResult<FileDescriptor> {
        let cstring_path = ffi::CString::from_vec(path.into_bytes());
        let fd = unsafe { open(cstring_path.as_ptr(), flags.bits(), mode) };
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


use std::fmt;
use std::old_io as io;
use std::ffi;
use std::os;
use libc::funcs::posix88::fcntl::open;
use libc::funcs::posix88::unistd::{read, write};
use libc::types::os::arch::c95::{c_int, size_t};
use libc::types::os::arch::posix88::mode_t;
use libc::types::common::c95::c_void;

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

pub fn err_exit_fmt(err: usize, fmt: fmt::Arguments) -> bool {
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

pub fn open_wip(path: String, oflag: c_int, mode: mode_t) -> Result<u32, usize> {
    let cstring_path = ffi::CString::from_vec(path.into_bytes());
    let fd = unsafe { open(cstring_path.as_ptr(), oflag, mode) };
    if fd == -1 { Err(os::errno()) } else { Ok(fd as u32) }
}

pub fn read_wip(fd: u32, buf: &mut [u8]) -> Result<u32, usize> {
    let buf_ptr = buf.as_mut_ptr() as *mut c_void;
    let buf_len = buf.len() as size_t;
    let bytes_read = unsafe { read(fd as c_int, buf_ptr, buf_len) };
    if bytes_read == -1 { Err(os::errno()) } else { Ok(bytes_read as u32) }
}

pub fn write_wip(fd: u32, buf: &[u8]) -> Result<u32, usize> {
    let buf_ptr = buf.as_ptr() as *const c_void;
    let buf_len = buf.len() as size_t;
    let bytes_written = unsafe { write(fd as c_int, buf_ptr, buf_len) };
    if bytes_written == -1 { Err(os::errno()) } else { Ok(bytes_written as u32) }
}

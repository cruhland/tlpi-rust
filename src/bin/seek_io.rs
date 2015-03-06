
#![feature(collections, core, env, libc)]

#[macro_use]
extern crate tlpi_rust;

use std::env;
use tlpi_rust::fd::*;
use std::num;
use Command::*;
use ReadFormat::*;

fn main() {
    set_exit_status!(main_with_result());
}

fn main_with_result() -> bool {
    let argv: Vec<_> = env::args().collect();

    if argv.len() < 3 || argv[1] == "--help" {
        return usage_err!(
            "{} file {{r<length>|R<length>|w<string>|s<offset>}}...", argv[0]
        );
    }

    let flags = O_RDWR | O_CREAT;
    // rw-rw-rw
    let perms = S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH | S_IWOTH;

    let fd = match FileDescriptor::open(argv[1].clone(), flags, perms) {
        Ok(fd) => fd,
        Err(errno) => return err_exit!(errno, "open")
    };

    argv.iter().skip(2).all(|arg| {
        match Command::parse(arg) {
            Ok(Read { byte_count, format }) => {
                read_file(&fd, arg, byte_count, format)
            },
            Ok(Write { text }) => write_file(&fd, arg, text),
            Ok(Seek { offset }) => seek_file(&fd, arg, offset),
            Err(message) => cmd_line_err!("{}", message),
        }
    })
}

enum Command<'a> {
    Read { byte_count: usize, format: ReadFormat },
    Write { text: &'a str },
    Seek { offset: i64 },
}

enum ReadFormat { Text, Hex }

impl<'a> Command<'a> {

    fn parse(s: &str) -> Result<Command, String> {
        match s.slice_shift_char() {
            Some(('r', arg)) => {
                parse_int(arg, "length")
                    .map(|count| Read { byte_count: count, format: Text })
            },
            Some(('R', arg)) => {
                parse_int(arg, "length")
                    .map(|count| Read { byte_count: count, format: Hex })
            },
            Some(('w', arg)) => Ok(Write { text: arg }),
            Some(('s', arg)) => {
                parse_int(arg, "offset").map(|offset| Seek { offset: offset })
            },
            _ => Err(format!("Argument must start with [rRws]: {:?}", s)),
        }
    }

}

fn parse_int<T>(
    s: &str, into_what: &str
) -> Result<T, String> where T: num::FromStrRadix {
    let parsed = num::from_str_radix(s, 10);
    parsed.map_err(|_| format!("Invalid {}: {}", into_what, s))
}

fn read_file(
    fd: &FileDescriptor, arg: &str, byte_count: usize, format: ReadFormat
) -> bool {
    let mut buf = vec![0u8; byte_count];
    let num_read = match fd.read(buf.as_mut_slice()) {
        Ok(count) => count,
        Err(errno) => return err_exit!(errno, "read"),
    };

    print!("{}: ", arg);
    if num_read == 0 {
        println!("end-of-file");
    } else {
        display_bytes(&buf[..num_read], format);
    }

    true
}

fn display_bytes(bytes: &[u8], format: ReadFormat) {
    match format {
        Text => {
            // TODO Handle UTF-8 error gracefully
            let buf_str = std::str::from_utf8(bytes).unwrap();
            for c in buf_str.chars() {
                let out_char = if c.is_control() { '?' } else { c };
                print!("{}", out_char);
            }
        },
        Hex => {
            for byte in bytes {
                print!("{:0>2x} ", byte);
            }
        }
    };
    println!("");
}

fn write_file(fd: &FileDescriptor, arg: &str, text: &str) -> bool {
    let num_written = match fd.write(text.as_bytes()) {
        Ok(bytes) => bytes,
        Err(errno) => return err_exit!(errno, "write")
    };
    println!("{}: wrote {} bytes", arg, num_written);
    true
}

fn seek_file(fd: &FileDescriptor, arg: &str, offset: i64) -> bool {
    match fd.lseek(offset, OffsetBase::SeekSet) {
        Err(errno) => return err_exit!(errno, "lseek"),
        _ => {}
    };

    println!("{}: seek succeeded", arg);
    true
}

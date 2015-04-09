
#![feature(core, exit_status, libc, str_char)]

#[macro_use]
extern crate tlpi_rust;

extern crate core;

use std::env;
use tlpi_rust::fd::*;
use tlpi_rust::err::*;
use Command::*;
use ReadFormat::*;

fn main() {
    set_exit_status!(main_with_result());
}

fn main_with_result() -> TlpiResult<()> {
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

    let result = {
        // Need to put this in a block so that `fd` is not borrowed
        // for the call to `close()` below
        let result_iter = argv.iter().skip(2).map(|arg| {
            Command::parse(arg).and_then(|command| command.execute(&fd))
        });

        std::result::fold(result_iter, (), |v, _| v)
    };

    match fd.close() {
        Err(errno) => err_exit!(errno, "close"),
        _ => result,
    }
}

#[derive(Clone, Copy)]
enum Command<'a> {
    Read { byte_count: usize, format: ReadFormat },
    Write { text: &'a str },
    Seek { offset: i64 },
}

#[derive(Clone, Copy)]
enum ReadFormat { Text, Hex }

impl<'a> Command<'a> {

    fn parse(s: &str) -> TlpiResult<Command> {
        match s.slice_shift_char() {
            Some((mode @ 'r', arg)) | Some((mode @ 'R', arg)) => {
                match usize::from_str_radix(arg, 10).ok() {
                    Some(count) => {
                        let format = if mode == 'r' { Text } else { Hex };
                        Ok(Read { byte_count: count, format: format })
                    },
                    _ => cmd_line_err!("Invalid length: {}", s),
                }
            },
            Some(('w', arg)) => Ok(Write { text: arg }),
            Some(('s', arg)) => {
                match i64::from_str_radix(arg, 10).ok() {
                    Some(offset) => Ok(Seek { offset: offset }),
                    _ => cmd_line_err!("Invalid offset: {}", s),
                }
            },
            _ => cmd_line_err!("Argument must start with [rRws]: {:?}", s),
        }
    }

    fn execute(self, fd: &FileDescriptor) -> TlpiResult<()> {
        match self {
            Read { byte_count, format } => {
                let mut buf = vec![0u8; byte_count];
                let num_read = match fd.read(&mut buf[..]) {
                    Ok(count) => count,
                    Err(errno) => return err_exit!(errno, "read"),
                };

                print!("{}: ", self);
                if num_read == 0 {
                    println!("end-of-file");
                } else {
                    display_bytes(&buf[..num_read], format);
                }
            },
            Write { text } => {
                let num_written = match fd.write(text.as_bytes()) {
                    Ok(bytes) => bytes,
                    Err(errno) => return err_exit!(errno, "write")
                };
                println!("{}: wrote {} bytes", self, num_written);
            },
            Seek { offset } => {
                match fd.lseek(offset, OffsetBase::SeekSet) {
                    Err(errno) => return err_exit!(errno, "lseek"),
                    _ => {}
                };

                println!("{}: seek succeeded", self);
            },
        };
        Ok(())
    }

}

impl<'a> core::fmt::Display for Command<'a> {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> core::fmt::Result {
        match self {
            &Read { byte_count, format } => {
                 let command_char = match format {
                    Text => 'r',
                    Hex => 'R',
                };
                write!(f, "{}{}", command_char, byte_count)
            },
            &Write { text } => write!(f, "w{}", text),
            &Seek { offset } => write!(f, "s{}", offset),
        }
    }

}

fn display_bytes(bytes: &[u8], format: ReadFormat) {
    match format {
        Text => {
            let buf_str = std::string::String::from_utf8_lossy(bytes);
            for c in buf_str.chars() {
                let out_char = if c.is_control() { '\u{FFFD}' } else { c };
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

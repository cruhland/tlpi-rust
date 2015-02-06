
#![feature(libc, os)]

#[macro_use]
extern crate tlpi_rust;

extern crate libc;

use std::os;
use tlpi_rust::util;
use libc::consts::os::posix88::*;

const BUF_SIZE: usize = 1024;

fn main() {
    if !main_with_io() {
        os::set_exit_status(libc::consts::os::c95::EXIT_FAILURE as isize);
    }
}

fn main_with_io() -> bool {
    let argv = os::args();

    if argv.len() != 3 || argv[1] == "--help" {
        return usage_err!("{} old-file new-file", argv[0]);
    }

    // Open input and output files

    let src_path = argv[1].clone();
    let input_fd = match util::open_wip(src_path, O_RDONLY, 0) {
        Ok(fd) => fd,
        Err(errno) => return err_exit!(errno, "opening file {}", argv[1])
    };

    let open_flags = O_CREAT | O_WRONLY | O_TRUNC;

    let file_perms = 0o666; // rw-rw-rw

    let dst_path = argv[2].clone();
    let output_fd = match util::open_wip(dst_path, open_flags, file_perms) {
        Ok(fd) => fd,
        Err(errno) => return err_exit!(errno, "opening file {}", argv[2])
    };

    // Transfer data until we encounter end of input or an error

    let mut buf = [0u8; BUF_SIZE];
    loop {
        match util::read_wip(input_fd, buf.as_mut_slice()) {
            Ok(0) => break,
            Ok(num_read) => {
                match util::write_wip(output_fd, &buf[..num_read as usize]) {
                    Ok(num_written) if num_read == num_written => {},
                    Ok(_) => return fatal!("couldn't write whole buffer"),
                    Err(errno) => {
                        return err_exit!(errno, "writing file {}", argv[2])
                    }
                };
            },
            Err(errno) => return err_exit!(errno, "reading file {}", argv[1])
        };
    }

    // TODO close() files

    true
}

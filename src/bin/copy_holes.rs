
#![feature(libc, exit_status)]

#[macro_use]
extern crate tlpi_rust;

use std::env;
use tlpi_rust::fd::*;
use tlpi_rust::err::*;

const BUF_SIZE: usize = 1 << 16; // 64k

fn main() {
    set_exit_status!(main_with_io());
}

fn main_with_io() -> TlpiResult<()> {
    let argv: Vec<_> = env::args().collect();

    if argv.len() != 3 || argv[1] == "--help" {
        return usage_err!("{} old-file new-file", argv[0]);
    }

    // Open input and output files

    let src_path = argv[1].clone();
    let empty_perms = FilePerms::empty();
    let input_fd = match FileDescriptor::open(src_path, O_RDONLY, empty_perms) {
        Ok(fd) => fd,
        Err(errno) => return err_exit!(errno, "opening file {}", argv[1])
    };

    let open_flags = O_CREAT | O_WRONLY | O_TRUNC;
    let file_perms = S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH;
    let dst_path = argv[2].clone();
    let output_fd = match FileDescriptor::open(dst_path, open_flags, file_perms) {
        Ok(fd) => fd,
        Err(errno) => return err_exit!(errno, "opening file {}", argv[2])
    };

    // Transfer data until we encounter end of input or an error

    let mut buf = [0u8; BUF_SIZE];
    let mut length = 0;
    let mut ends_with_hole = false;

    loop {
        let bytes_read = match input_fd.read(buf.as_mut_slice()) {
            Ok(0) => break,
            Ok(bytes) => bytes,
            Err(errno) => return err_exit!(errno, "reading file {}", argv[1])
        };
        length += bytes_read;

        let write_buf = &buf[..bytes_read];
        let desc = &argv[2][..];
        ends_with_hole = try!(write_with_holes(&output_fd, write_buf, desc));
    }

    if ends_with_hole {
        match output_fd.ftruncate(length as i64) {
            Err(errno) => return err_exit!(errno, "extending file {}", argv[2]),
            _ => {},
        }
    }

    // Clean up

    match input_fd.close() {
        Err(errno) => return err_exit!(errno, "close input"),
        _ => {}
    };

    match output_fd.close() {
        Err(errno) => return err_exit!(errno, "close output"),
        _ => {}
    };

    Ok(())
}

fn write_with_holes(
    fd: &FileDescriptor, buf: &[u8], desc: &str
) -> TlpiResult<bool> {
    let mut iter = buf.iter();
    let len = buf.len();
    let mut region_start = try!(seek_to_data(fd, desc, &mut iter, len));
    let mut ends_with_hole = region_start == len;

    while region_start < len {
        let region_end = match iter.position(|&byte| byte == 0) {
            Some(pos) => region_start + pos + 1,
            _ => buf.len(),
        };
        println!("region_start = {:?}", region_start);
        println!("region_end = {:?}", region_end);
        let slice = &buf[region_start..region_end];
        println!("About to write slice with len {:?}", slice.len());
        match fd.write(slice) {
            Ok(byte_count) if slice.len() == byte_count => {},
            Ok(_) => return fatal!(
                "couldn't write entire region [{}..{}] of file {}",
                region_start,
                region_end,
                desc
            ),
            Err(errno) => return err_exit!(
                errno,
                "writing region [{}..{}] of file {}",
                region_start,
                region_end,
                desc
            ),
        };
        ends_with_hole = false;
        if region_end >= len { break };
        let remaining_len = len - region_end;
        region_start =
            region_end + try!(seek_to_data(fd, desc, &mut iter, remaining_len));
        ends_with_hole = region_start == remaining_len;
        println!("region_start at end of loop: {:?}", region_start);
        println!("remaining_len at end of loop: {:?}", remaining_len);
    }

    Ok(ends_with_hole)
}

fn seek_to_data<'a>(
    fd: &FileDescriptor,
    desc: &str,
    iter: &mut std::slice::Iter<'a, u8>,
    max_amount: usize,
) -> TlpiResult<usize> {
    let seek_amount = iter.position(|&byte| byte != 0).unwrap_or(max_amount);
    println!("seek_amount = {:?}", seek_amount);
    match fd.lseek(seek_amount as i64, OffsetBase::SeekCur) {
        Err(errno) => return err_exit!(
            errno, "lseek by amount {} in file {}", seek_amount, desc
        ),
        _ => Ok(seek_amount),
    }
}

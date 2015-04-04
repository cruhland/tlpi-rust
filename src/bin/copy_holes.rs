
#![feature(libc, exit_status, collections)]

#[macro_use]
extern crate tlpi_rust;

use std::env;
use tlpi_rust::fd::*;
use tlpi_rust::err::*;
use Region::*;

const BUF_SIZE: usize = 1 << 16; // 64k

fn main() {
    set_exit_status!(main_with_io());
}

fn main_with_io() -> TlpiResult<()> {
    let argv: Vec<_> = env::args().collect();

    if argv.len() != 3 || argv[1] == "--help" {
        return usage_err!("{} old-file new-file", argv[0]);
    }

    let input_fd = try!(open_input(&argv[1][..]));
    let output_fd = try!(open_output(&argv[2][..]));

    try!(copy_with_holes(&input_fd, &output_fd));

    try!(clean_up(input_fd, "input"));
    try!(clean_up(output_fd, "output"));

    Ok(())
}

fn open_input(path: &str) -> TlpiResult<FileDescriptor> {
    let empty_perms = FilePerms::empty();
    FileDescriptor::open(String::from_str(path), O_RDONLY, empty_perms)
        .or_else(|errno| err_exit!(errno, "opening input file {}", path))
}

fn open_output(path: &str) -> TlpiResult<FileDescriptor> {
    let open_flags = O_CREAT | O_WRONLY | O_TRUNC;
    let file_perms = S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH;
    FileDescriptor::open(String::from_str(path), open_flags, file_perms)
        .or_else(|errno| err_exit!(errno, "opening output file {}", path))
}

fn copy_with_holes(
    input_fd: &FileDescriptor, output_fd: &FileDescriptor
) -> TlpiResult<()> {
    let mut reader = RegionReader::attach(input_fd);
    let mut writer = BulkWriter::attach(output_fd);

    loop {
        let region = match try!(reader.read()) {
            Some(r) => r,
            _ => break,
        };

        match region {
            Data(data) => try!(writer.write(data)),
            Hole(size) => writer.extend(size as u64),
        };
    }

    try!(writer.detach());

    Ok(())
}

enum Region<'a> {
    Data(&'a [u8]),
    Hole(usize),
}

struct RegionReader<'a> {
    fd: &'a FileDescriptor,
    buffer: [u8; BUF_SIZE],
    next_index: usize,
    bytes_read: usize,
}

impl<'a> RegionReader<'a> {

    fn attach(fd: &FileDescriptor) -> RegionReader {
        RegionReader {
            fd: fd, buffer: [0; BUF_SIZE], next_index: 0, bytes_read: 0
        }
    }

    fn read(&mut self) -> TlpiResult<Option<Region>> {
        if self.next_index == self.bytes_read {
            self.bytes_read = match self.fd.read(&mut self.buffer[..]) {
                Ok(0) => return Ok(None),
                Ok(bytes) => bytes,
                Err(errno) => return err_exit!(errno, "reading input file"),
            };
            self.next_index = 0;
        }

        let current_region_start = self.next_index;
        let region = if self.buffer[current_region_start] == 0 {
            self.next_index = self.next_region(|&byte| byte != 0);
            Hole(self.next_index - current_region_start)
        } else {
            self.next_index = self.next_region(|&byte| byte == 0);
            Data(&self.buffer[current_region_start..self.next_index])
        };

        Ok(Some(region))
    }

    fn next_region<P>(&self, predicate: P) -> usize
        where P: FnMut(&u8) -> bool
    {
        let mut iter = self.buffer[self.next_index..self.bytes_read].iter();
        match iter.position(predicate) {
            Some(pos) => self.next_index + pos,
            _ => self.bytes_read,
        }
    }

}

struct BulkWriter<'a> {
    fd: &'a FileDescriptor,
    buffer: Vec<u8>,
    pending_extend: u64,
    bytes_added: u64,
}

impl<'a> BulkWriter<'a> {

    fn attach(fd: &FileDescriptor) -> BulkWriter {
        BulkWriter {
            fd: fd,
            buffer: Vec::with_capacity(BUF_SIZE),
            pending_extend: 0,
            bytes_added: 0,
        }
    }

    fn write(&mut self, data: &[u8]) -> TlpiResult<()> {
        if self.pending_extend > 0 && data.len() > 0 {
            if self.buffer.len() > 0 {
                try!(self.flush_writes());
            }

            try!(self.flush_extends());
        }

        let mut bytes_buffered = 0;
        while bytes_buffered < data.len() {
            if self.remaining() == 0 {
                try!(self.flush_writes());
            }

            let capacity_index = bytes_buffered + self.buffer.capacity();
            let end = std::cmp::min(capacity_index, data.len());
            let slice = &data[bytes_buffered..end];
            self.buffer.push_all(slice);
            bytes_buffered += slice.len();
        }

        Ok(())
    }

    fn extend(&mut self, amount: u64) {
        self.pending_extend += amount;
    }

    fn detach(mut self) -> TlpiResult<()> {
        if self.buffer.len() > 0 {
            try!(self.flush_writes());
        }

        if self.pending_extend > 0 {
            let file_length = self.bytes_added + self.pending_extend;
            let result = self.fd.ftruncate(file_length as i64);
            try!(result.or_else(|errno| err_exit!(errno, "ftruncate")));
        }

        Ok(())
    }

    fn remaining(&self) -> usize {
        self.buffer.capacity() - self.buffer.len()
    }

    fn flush_writes(&mut self) -> TlpiResult<()> {
        match self.fd.write(&self.buffer[..]) {
            Ok(byte_count) => {
                self.bytes_added += byte_count as u64;
                if self.buffer.len() != byte_count {
                    return fatal!("wrote partial data");
                }
            },
            Err(errno) => return err_exit!(errno, "write failure"),
        };

        self.buffer.clear();
        Ok(())
    }

    fn flush_extends(&mut self) -> TlpiResult<()> {
        match self.fd.lseek(self.pending_extend as i64, OffsetBase::SeekCur) {
            Err(errno) => return err_exit!(
                errno,
                "lseek by amount {} in output file",
                self.pending_extend,
            ),
            _ => self.bytes_added += self.pending_extend,
        };

        self.pending_extend = 0;
        Ok(())
    }

}

fn clean_up(fd: FileDescriptor, desc: &str) -> TlpiResult<()> {
    fd.close().or_else(|errno| err_exit!(errno, "close {}", desc))
}

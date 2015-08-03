
#![feature(libc)]

#[macro_use]
extern crate tlpi_rust;

use std::env;
use tlpi_rust::fd::*;
use tlpi_rust::err::*;
use Region::*;

/// Capacity of buffers for reading and writing file data.
const BUF_SIZE: usize = 1 << 16;

fn main() {
    exit_with_status!(main_with_io());
}

fn main_with_io() -> TlpiResult<()> {
    let argv: Vec<_> = env::args().collect();

    if argv.len() != 3 || argv[1] == "--help" {
        return usage_err!("{} old-file new-file", argv[0]);
    }

    let input_fd = try!(open_input(&argv[1]));
    let output_fd = try!(open_output(&argv[2]));

    try!(copy_with_holes(&input_fd, &output_fd));

    try!(clean_up(input_fd, "input"));
    try!(clean_up(output_fd, "output"));

    Ok(())
}

fn open_input(path: &str) -> TlpiResult<FileDescriptor> {
    let empty_perms = FilePerms::empty();
    FileDescriptor::open(String::from(path), O_RDONLY, empty_perms)
        .or_else(|errno| err_exit!(errno, "opening input file {}", path))
}

fn open_output(path: &str) -> TlpiResult<FileDescriptor> {
    let open_flags = O_CREAT | O_WRONLY | O_TRUNC;
    let file_perms = S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH;
    FileDescriptor::open(String::from(path), open_flags, file_perms)
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

/// A contiguous, non-empty segment of a stream of bytes.
enum Region<'a> {
    /// Segment containing non-zero bytes.
    Data(&'a [u8]),

    /// Segment containing only zero bytes.
    Hole(usize),
}

/// Reads files as a sequence of `Region`s.
struct RegionReader<'a> {
    /// The file to read from.
    fd: &'a FileDescriptor,

    /// The buffer to read into.
    buffer: [u8; BUF_SIZE],

    /// The start index in `buffer` of the next region.
    next_index: usize,

    /// The number of bytes in `buffer` from the most recent read on `fd`.
    bytes_read: usize,
}

impl<'a> RegionReader<'a> {

    /// Create an empty reader from an existing file descriptor.
    ///
    /// The file offset of the descriptor is not changed.
    fn attach(fd: &FileDescriptor) -> RegionReader {
        RegionReader {
            fd: fd, buffer: [0; BUF_SIZE], next_index: 0, bytes_read: 0
        }
    }

    /// Extracts the next region from the file.
    ///
    /// Any `Data` regions must be consumed before calling this method
    /// again.
    ///
    /// Returns `Ok(None)` at end-of-file.
    fn read(&mut self) -> TlpiResult<Option<Region>> {
        // Have we reached the end of the buffer?
        if self.next_index == self.bytes_read {
            // Try to get more data from the file
            self.bytes_read = match self.fd.read(&mut self.buffer) {
                Ok(0) => return Ok(None),
                Ok(bytes) => bytes,
                Err(errno) => return err_exit!(errno, "reading input file"),
            };
            self.next_index = 0;
        }

        // Find the next region's start so the current one can be
        // returned
        let current_region_start = self.next_index;
        let region = if self.buffer[current_region_start] == 0 {
            self.next_index = self.next_region(|&byte| byte != 0);
            Hole(self.next_index - current_region_start)
        } else {
            self.next_index = self.next_region(|&byte| byte == 0);

            // Don't copy the data on the assumption it will be used before
            // another call to this method
            Data(&self.buffer[current_region_start..self.next_index])
        };

        Ok(Some(region))
    }

    /// Find the first index in `buffer` at or beyond `next_index`
    /// that satisfies the given `predicate`.
    ///
    /// This is a helper method that finds region boundaries for
    /// `read()`.
    ///
    /// If no index was found, returns `bytes_read`, i.e. the index
    /// just beyond the data in `buffer`.
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

/// Writes `Region`s to a file.
struct BulkWriter<'a> {
    /// The file to write to.
    fd: &'a FileDescriptor,

    /// Accumulates data from calls to `write()`.
    buffer: Vec<u8>,

    /// Accumulates length extensions from calls to `extend()`.
    pending_extend: u64,

    /// The total number of bytes written to `fd`.
    bytes_added: u64,
}

impl<'a> BulkWriter<'a> {

    /// Create an empty writer from an existing file descriptor.
    ///
    /// The file offset of the descriptor is not changed.
    fn attach(fd: &FileDescriptor) -> BulkWriter {
        BulkWriter {
            fd: fd,
            buffer: Vec::with_capacity(BUF_SIZE),
            pending_extend: 0,
            bytes_added: 0,
        }
    }

    /// Writes the given data to file.
    ///
    /// Any pending length extensions of the file are flushed prior to
    /// writing. Depending on the size of the data, some or all of it
    /// may be buffered and written to file later.
    fn write(&mut self, data: &[u8]) -> TlpiResult<()> {
        // If we're actually writing data, we need to flush pending
        // length extensions to move the file offset
        if self.pending_extend > 0 && data.len() > 0 {
            // Pending writes go before length extensions
            if self.buffer.len() > 0 {
                try!(self.flush_writes());
            }

            try!(self.flush_extends());
        }

        // Copy the data to `buffer`, flushing to file if more space
        // is needed
        let mut bytes_buffered = 0;
        while bytes_buffered < data.len() {
            if self.remaining() == 0 {
                try!(self.flush_writes());
            }

            let capacity_index = bytes_buffered + self.buffer.capacity();
            let end = std::cmp::min(capacity_index, data.len());
            let slice = &data[bytes_buffered..end];
            self.buffer.extend(slice);
            bytes_buffered += slice.len();
        }

        Ok(())
    }

    /// Increase the length of the file by the given amount of bytes.
    ///
    /// The length extensions are buffered to minimize I/O
    /// operations. When actually written to file, the length
    /// extensions appear as file holes and/or zero bytes.
    fn extend(&mut self, amount: u64) {
        self.pending_extend += amount;
    }

    /// Flush any buffered data and/or length extensions to file and
    /// consume this writer.
    fn detach(mut self) -> TlpiResult<()> {
        if self.buffer.len() > 0 {
            try!(self.flush_writes());
        }

        if self.pending_extend > 0 {
            // We can't just advance the file offset here, because
            // without data to write after it, the file hole will not
            // be created.
            let file_length = self.bytes_added + self.pending_extend;
            let result = self.fd.ftruncate(file_length as i64);
            try!(result.or_else(|errno| err_exit!(errno, "ftruncate")));
        }

        Ok(())
    }

    /// Helper method; the number of unused bytes of capacity in
    /// `buffer`.
    fn remaining(&self) -> usize {
        self.buffer.capacity() - self.buffer.len()
    }

    /// Helper method; writes all buffered data to file.
    fn flush_writes(&mut self) -> TlpiResult<()> {
        match self.fd.write(&self.buffer) {
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

    /// Helper method; writes all buffered length extensions to file.
    ///
    /// Assumes that data will follow the length extensions!
    fn flush_extends(&mut self) -> TlpiResult<()> {
        // This only works if data will later be written to the file
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

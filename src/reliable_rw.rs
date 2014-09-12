// Copyright 2014 Stacey Ell <stacey.ell@gmail.com>

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![feature(macro_rules)]

use std::io::{IoResult, IoError};

use sha256::{Sha256, Digest};
mod sha256;


/// Magic number at the beginning of the stream
pub static MAGIC_HEADER: &'static [u8] = b"reliable-encap";

/// We won't emit any pieces longer than this
pub static PIECE_SIZE: uint = 32 * 1024;  // 32kB

/// We won't accept any pieces longer than this
pub static MAX_PIECE_SIZE: uint = 256 * 1024;  // 256kB


pub enum ReliableWriteError {
    IntegrityError,
    ProtocolError,
    ReadError(IoError),
    WriteError(IoError)
}


pub type ReliableWriteResult<T> = Result<T, ReliableWriteError>;


pub struct ReliableEncap<'a> {
    digest: Sha256,
    output: &'a mut Writer+'a,
}


impl<'a> ReliableEncap<'a> {
    pub fn new<'a>(output: &'a mut Writer) -> IoResult<ReliableEncap<'a>> {
        let rv = ReliableEncap {
            digest: Sha256::new(),
            output: output
        };
        try!(rv.output.write(MAGIC_HEADER));
        Ok(rv)
    }

    pub fn update(&mut self, buf: &Vec<u8>) -> IoResult<()> {
        match self.output.write_be_u32(buf.len() as u32) {
            Ok(()) => (),
            Err(err) => return Err(err)
        }

        self.digest.input(buf.as_slice());
        match self.output.write(buf.as_slice()) {
            Ok(()) => (),
            Err(err) => return Err(err)
        }

        let hasher_res = self.digest.result_bytes();

        match self.output.write(hasher_res.as_slice()) {
            Ok(()) => (),
            Err(err) => return Err(err)
        }
        Ok(())
    }

    pub fn finish_write(&mut self) -> IoResult<()> {
        match self.output.write_be_u32(0) {
            Ok(()) => (),
            Err(err) => return Err(err)
        }
        match self.output.write(self.digest.result_bytes().as_slice()) {
            Ok(()) => (),
            Err(err) => return Err(err)
        }
        Ok(())
    }

    pub fn finalize(&mut self) -> IoResult<()> {
        // self.digest.input(MAGIC_HEADER);
        try!(self.output.write(self.digest.result_bytes().as_slice()));
        self.output.flush()
    }
}


pub fn copy_out(input: &mut Reader, output: &mut Writer) -> ReliableWriteResult<()> {
    let mut hasher: Box<Digest> = box Sha256::new();

    match input.read_exact(MAGIC_HEADER.len()) {
        Ok(_) => (),
        Err(err) => return Err(ReadError(err))
    }

    loop {
        let n = match input.read_be_u32() {
            Ok(n) => {
                let n = n as uint;
                if MAX_PIECE_SIZE < n {
                    return Err(ProtocolError);
                }
                n
            },
            Err(err) => return Err(ReadError(err))
        };
        let data = match input.read_exact(n) {
            Ok(data) => data,
            Err(err) => return Err(ReadError(err))
        };

        hasher.input(data.as_slice());

        match output.write(data.as_slice()) {
            Ok(_) => (),
            Err(err) => return Err(WriteError(err))
        };

        let hash_data = match input.read_exact(hasher.output_bits() / 8) {
            Ok(data) => data,
            Err(err) => return Err(ReadError(err))
        };
        // IntegrityError
        if hash_data != hasher.result_bytes() {
            return Err(IntegrityError);
        }

        if n == 0 {
            break;
        }
    }
    let hash_data = match input.read_exact(hasher.output_bits() / 8) {
        Ok(data) => data,
        Err(err) => return Err(ReadError(err))
    };
    // IntegrityError
    if hash_data != hasher.result_bytes() {
        return Err(IntegrityError);
    }

    Ok(())
}
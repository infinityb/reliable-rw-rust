#![feature(macro_rules)]
// Copyright 2014 Stacey Ell <stacey.ell@gmail.com>

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::os;
use std::io::{stdin, File, Open, Write, Reader, Writer, IoResult};
use std::io::fs::{unlink, rename};

use reliable_rw_common::MAGIC_HEADER;
use sha256::{Sha256, Digest};

mod sha256;
mod reliable_rw_common;

static MAX_PIECE_SIZE: uint = 256 * 1024;  // 256kB


fn print_usage(program: &str) {
    println!("{} filename", program);
}

fn copy_out(
        input: &mut Reader,
        output: &mut Writer
) -> IoResult<()> {
    let mut hasher: Box<Digest> = box Sha256::new();

    match input.read_exact(MAGIC_HEADER.len()) {
        Ok(_) => (),
        Err(err) => return Err(err)
    }

    loop {
        let n = match input.read_be_u32() {
            Ok(n) => {
                let n = n as uint;
                if MAX_PIECE_SIZE < n {
                    // We won't clean up our temp file if this happens!
                    // StreamProtocolError
                    fail!("excessive piece size, {}", n);
                }
                if n == 0 {
                    break;
                }
                n
            },
            Err(err) => return Err(err)
        };
        let data = match input.read_exact(n) {
            Ok(data) => data,
            Err(err) => return Err(err)
        };

        hasher.input(data.as_slice());
        match output.write(data.as_slice()) {
            Ok(_) => (),
            Err(err) => return Err(err)
        };

        let hash_data = match input.read_exact(hasher.output_bits() / 8) {
            Ok(data) => data,
            Err(err) => return Err(err)
        };
        // IntegrityError
        assert!(hash_data.as_slice() == hasher.result_bytes().as_slice());
    }
    Ok(())
}

fn main() {
    let args = os::args();

    let program_name = args[0].as_slice().clone();
    if args.len() < 2 {
        print_usage(program_name);
        os::set_exit_status(1);
        return;
    }
    let output_path = Path::new(args[1].as_slice().clone());
    let output_path_tmp = Path::new(output_path.clone().into_vec() + b".tmp");

    let mut input = stdin();
    let mut output = match File::open_mode(&output_path_tmp, Open, Write) {
        Ok(f) => f,
        Err(e) => fail!("file error: {}", e),
    };

    match copy_out(&mut input, &mut output) {
        Ok(_) => {
            // is `output' flushed at this point in time?
            rename(&output_path_tmp, &output_path);
        },
        Err(err) => {
            assert!(unlink(&output_path_tmp).is_ok());        
            fail!("{}", err);
        }
    }
}

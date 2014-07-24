#![feature(macro_rules)]
// Copyright 2014 Stacey Ell <stacey.ell@gmail.com>

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate debug;

use std::os;
use std::io::{stdin, File, Open, Write, Reader, Writer};
use std::io::fs::{rename};

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
) -> bool {
    let mut hasher: Box<Digest> = box Sha256::new();

    match input.read_exact(MAGIC_HEADER.len()) {
        Ok(data) => {
            println!("data = {:?}", data);
        },
        Err(err) => fail!("{}", err)
    }

    loop {
        let n = match input.read_be_u32() {
            Ok(n) => {
                let n = n as uint;
                println!("n = {}", n);
                if MAX_PIECE_SIZE < n {
                    fail!("excessive piece size, {}", n);
                }
                if n == 0 {
                    break;
                }
                n
            },
            Err(err) => fail!("{}", err)
        };
        match input.read_exact(n) {
            Ok(data) => {
                hasher.input(data.as_slice());
                assert!(output.write(data.as_slice()).is_ok());
                println!("data = {:?}", data);
            },
            Err(err) => fail!("{}", err)
        }
        match input.read_exact(hasher.output_bits() / 8) {
            Ok(data) => {
                assert!(data.as_slice() == hasher.result_bytes().as_slice());
            },
            Err(err) => fail!("{}", err)
        }
    }
    true
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
    if copy_out(&mut input, &mut output) {
        match rename(&output_path_tmp, &output_path) {
            Ok(_) => {},
            Err(err) => fail!("{}", err)
        };
    }
}

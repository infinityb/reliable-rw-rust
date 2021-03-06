#![feature(macro_rules)]
// Copyright 2014 Stacey Ell <stacey.ell@gmail.com>

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate reliable_rw;

use std::os;
use std::io::{stdin, stderr, File, Open, Write, Writer};
use std::io::fs::{unlink, rename};

use reliable_rw::{
    copy_out,
    ReliableWriteError,
};


fn print_usage(program: &[u8]) {
    let mut stderr = stderr();
    let mut output = Vec::new();
    output.extend(program.iter().map(|x| x.clone()));
    output.extend(b" filename\n".iter().map(|x| x.clone()));
    assert!(stderr.write(output.as_slice()).is_ok());
}


fn main() {
    let args = os::args_as_bytes();

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
        Err(e) => panic!("file error: {}", e),
    };

    match copy_out(&mut input, &mut output) {
        Ok(_) => {
            // is `output' flushed at this point in time?
            assert!(rename(&output_path_tmp, &output_path).is_ok())
        },
        Err(ReliableWriteError::IntegrityError) => {
            assert!(unlink(&output_path_tmp).is_ok());
            panic!("IntegrityError");
        },
        Err(ReliableWriteError::ProtocolError) => {
            assert!(unlink(&output_path_tmp).is_ok());
            panic!("ProtocolError");
        },
        Err(ReliableWriteError::ReadError(err)) => {
            assert!(unlink(&output_path_tmp).is_ok());
            panic!("ReadError: {}", err);
        },
        Err(ReliableWriteError::WriteError(err)) => {
            assert!(unlink(&output_path_tmp).is_ok());
            panic!("WriteError: {}", err);
        },
    }
}

#![feature(macro_rules)]
// Copyright 2014 Stacey Ell <stacey.ell@gmail.com>

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate libc;

use std::os;
use std::io;
use std::io::{Command, IoError, EndOfFile};
use std::io::process::{InheritFd, ExitStatus, ExitSignal};

use reliable_rw_common::MAGIC_HEADER;
use sha256::{Sha256, Digest};

mod sha256;
mod reliable_rw_common;


// We won't emit any pieces longer than this
pub static PIECE_SIZE: uint = 32 * 1024;  // 32kB


fn print_usage(program: &str) {
    println!("{} [--] command", program);
}

fn main() {
    let args = os::args();
    let program_name = args[0].as_slice().clone();
    if args.len() < 2 {
        print_usage(program_name);
        os::set_exit_status(1);
        return;
    }

    let mut cmd_args: &[String] = args.tail();

    let head = cmd_args.get(0);
    if head.is_some() && head.unwrap().as_slice() == "--" {
        cmd_args = cmd_args.tail();
    } else {
        let mut stderr = io::stderr();
        let warning = "Warning: please include -- before the command name\n";
        assert!(stderr.write(warning.as_bytes()).is_ok());
    }

    let head = cmd_args.get(0);
    if head.is_none() {
        print_usage(program_name);
        os::set_exit_status(1);
        return;
    }

    let child_executable = head.unwrap();
    let mut command = Command::new(child_executable.as_slice());
    for arg in cmd_args.tail().iter() {
        command.arg(arg.as_slice());
    }
    command.stdin(InheritFd(libc::STDIN_FILENO));
    command.stderr(InheritFd(libc::STDERR_FILENO));

    let mut process = match command.spawn() {
        Ok(p) => p,
        Err(e) => fail!("failed to execute process: {}", e),
    };

    let mut output = io::stdout();
    let max_read_len = 32 * 1024;

    let mut buf: Vec<u8> = Vec::with_capacity(max_read_len);
    let mut hasher: Box<Digest> = box Sha256::new();
    assert!(output.write(MAGIC_HEADER).is_ok());

    loop {
        buf.clear();
        match process.stdout.get_mut_ref().push(PIECE_SIZE, &mut buf) {
            // Don't forget to import the different IoError kinds
            // if you are going to catch them.  Otherwise you'll get
            // an E0001 unreachable pattern.
            Ok(n) => {
                let out_slice = buf.as_slice();
                assert!(buf.len() == n);
                assert!(output.write_be_u32(n as u32).is_ok());
                assert!(output.write(out_slice).is_ok());
                hasher.input(out_slice);
                assert!(output.write(hasher.result_bytes().as_slice()).is_ok());
            },
            Err(IoError { kind: EndOfFile, .. }) => {
                assert!(output.write_be_u32(0).is_ok());
                assert!(output.write(hasher.result_bytes().as_slice()).is_ok());
                break;
            },
            Err(err) => fail!("{}", err)
        };
    };

    match process.wait() {
        Ok(ExitStatus(0)) => {
            hasher.input(MAGIC_HEADER);
            match output.write(hasher.result_bytes().as_slice()) {
                Ok(_) => (),
                Err(err) => fail!("{}", err)
            }
            // At this point, even if we fail, the write is complete.
            match output.flush() {
                Ok(_) => (),
                Err(err) => fail!("{}", err)
            }
        },
        Ok(ExitStatus(n)) => {
            os::set_exit_status(n);
        },
        Ok(ExitSignal(_)) => {
            os::set_exit_status(1);
        },
        Err(_) => {
            os::set_exit_status(1);
        }
    }
}

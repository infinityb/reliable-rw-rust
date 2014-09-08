#![feature(macro_rules)]
#![allow(unused_must_use)]
// Copyright 2014 Stacey Ell <stacey.ell@gmail.com>
// 
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate libc;
extern crate reliable_rw_common;

use std::os;
use std::io;
use std::io::{Command, IoError, EndOfFile};
use std::io::process::{InheritFd, ExitStatus, ExitSignal};
use reliable_rw_common::ReliableEncap;


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

    let max_read_len = 32 * 1024;
    let mut encap_output = io::stdout();
    let mut encapper = ReliableEncap::new(&mut encap_output);

    let mut buf: Vec<u8> = Vec::with_capacity(max_read_len);

    loop {
        buf.clear();
        match process.stdout.as_mut().unwrap().push(PIECE_SIZE, &mut buf) {
            // Don't forget to import the different IoError kinds
            // if you are going to catch them.  Otherwise you'll get
            // an E0001 unreachable pattern.
            Ok(n) => {
                assert_eq!(buf.len(), n);
                assert!(encapper.update(&buf).is_ok())
            },
            Err(IoError { kind: EndOfFile, .. }) => {
                assert!(encapper.finish_write().is_ok())
                break;
            },
            Err(err) => fail!("{}", err)
        };
    };

    match process.wait() {
        Ok(ExitStatus(0)) => {
            assert!(encapper.finalize().is_ok())
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

reliable-rw
===========

## Usage

Currently compatible with the Python implementation.

    reliable-encap cat somefile | ssh somehost reliable-write somefile


## Why does this exist?

I needed a way to guarantee streamed file writes to a remote server either
completed or failed without leaving files on disk.

## Current State
This project is definitely not ready for use.  Neither `reliable-encap` or
`reliable-write` are sufficiently implemented to provide the target guarantees.

* `reliable-encap` serialises a correct bytestream in the absence of failures
* `reliable-write` does not
* No library yet

## License
Distributed under the same terms as the Rust project (dual licensed MIT and Apache 2)

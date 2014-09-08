reliable-rw
===========

## Usage

Currently compatible with the Python implementation.

    reliable-encap -- cat somefile | ssh somehost reliable-write somefile


## Why does this exist?

I needed a way to guarantee streamed file writes to a remote server either
completed or failed without leaving files on disk.

## Current State
Might be ready to use.
* `reliable-encap` serialises a correct bytestream, even under failure conditions
* `reliable-write` seems to deserialise the stream robustly
* Beginning to resemble a library

## License
Distributed under the same terms as the Rust project (dual licensed MIT and Apache 2)

# Port Sodigy

If you're to port Sodigy to another platform, please read this document!

On Linux, Mac and Windows, all you need is a Rust compiler (for now, it's only tested on Linux). If your platform is Rust tier-1 supported, it's 99% guaranteed that Sodigy just works. If it doesn't build, please read the list below and check what's missing.

## MSRV

Sodigy only runs on nightly Rust. For now, it's tested on Rust 1.77.0-nightly (3cdd004e5 2023-12-29), on Ubuntu 23.10.

## Std

Below is the list of Rust std funcs/structs used in this project.

- std::collections::{HashMap, HashSet, hash_map}
  - `hash_map` is for its hash function.
- std::ffi::OsString
  - It's used to handle some file io errors.
- std::fmt
  - Many structs implement `fmt::Display` and `fmt::Debug`.
- std::fs::*
  - for file io
- std::hash::{Hash, Hasher}
- std::io::{self, Read, Write}
  - mostly for file io
- std::path::PathBuf
  - for file io
- std::sync::Mutex
- std::env::args

## External Crates

### hmath

It's used to handle arbitrary width integers.

### smallvec

It's purely for optimization purpose. Removing all the `smallvec`s (and using `vec`s instead) doesn't change anything, but its performance.

### colored

It's used to pretty-print the compiler output. I'm planning for an uncolored version of the compiler, which doesn't use this crate at all.

### lazy_static

It's also for an optimization, but it's quite critical. I might replace it with `once_cell` someday.

### rand

It's used ro generate uids, and I can't think of any alternative. If this crate doesn't work on your platform, you should write a function that returns a random 128 bit integer on your platform.

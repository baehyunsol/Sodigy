# Build Instructions

I work on Ubuntu 23.10 (Intel), and rarely on MacOS 14 (Apple Sillicon). If you're using either platform, you can just follow the remaining instructions. If you're on Windows, I'm sorry. I haven't tested anything on Windows.

## Requirements

It needs Rust build toolchain. You can download Rust compiler and its package manager [here](https://rustup.rs/). Sodigy only runs on nightly Rust.`rustup default nightly` changes the default setting to nightly. Make sure to always keep up with the latest version.

Clone this repository and run `cargo build --release` in the cloned directory. That's it. You'll find the result at `./target/release/sodigy`.

### Crates used in Sodigy

If something goes wrong with Rust, and you're on an unusual platform (like haiku, Serenity, Windows or very old versions of Linux) the problem could be due to one of its dependencies. Below are the libraries used by Sodigy, including the Rust std lib. Please make sure that all the libraries are available on your platform.

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
- [hmath](https://github.com/baehyunsol/hmath)
  - It's used to handle arbitrary width integers. 
- [smallvec](https://github.com/servo/rust-smallvec)
  - For performance reasons.
- [colored](https://github.com/colored-rs/colored)
  - It's used to pretty-print the compiler output. I'm planning for an uncolored version of the compiler, which doesn't use this crate at all.
- [lazy_static](https://github.com/rust-lang-nursery/lazy-static.rs)
  - It's also for an optimization, but it's quite critical. I might replace it with `once_cell` someday.
- [rand](https://github.com/rust-random/rand)
  - It's used to generate uids, and I can't think of any alternative. If this crate doesn't work on your platform, you should write a function that returns a random 128 bit integer on your platform.

## Tests

In order to run tests, you need [nushell](https://www.nushell.sh/). `nu tests.nu` in `sodigy/` will run the tests.

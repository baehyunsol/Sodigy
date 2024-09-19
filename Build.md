# Build Instructions

You need git and [rust](https://rustup.rs). It only runs on nightly rust. In order to use the nightly version, you have to run `rustup default nightly` after installing `rustup`.

1. clone this repository
2. `cd sodigy`
3. run `cargo build --release`, you'll find the binary at `./target/release/sodigy`

## MSRV

Use the latest version. Please run `rustup update` before building Sodigy.

## Tests

Run `python3 tests.py` to see if it passes all the tests.

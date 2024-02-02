# Build Instructions

You need git and [rust](https://rustup.rs). It only runs on nightly rust. In order to use the nightly version, you have to run `rustup default nightly` after downloading `rustup`.

1. clone this repository
2. `cd sodigy`
3. run `cargo build --release`

If you want to run tests, you need [nushell](https://nushell.sh). Run `nu tests.nu` at `sodigy/`.

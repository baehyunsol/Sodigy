# Build Instructions

You need git and [rust](https://rustup.rs). It only runs on nightly rust. In order to use the nightly version, you have to run `rustup default nightly` after downloading `rustup`.

1. clone this repository
2. `cd sodigy`
3. run `cargo build --release; cp ./target/release/sodigy .`

## MSRV

Use the latest version. Please run `rustup update` before building Sodigy.

## Tests

Once you built Sodigy, you should run tests to see if the build was successful. The test script requires [nushell](https://nushell.sh) to run.

After installing the shell, run `nu tests.nu` to run the tests.

If the test fails, please run `nu clean.nu` manually to remove temporary files.

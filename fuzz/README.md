# SETUP

```sh
rustup default nightly
cargo install cargo-fuzz

# TODO: I want it to be inside `sodigy/tests/fuzzer/`, but I can't do that
# It's already done and is inside the repo, so you don't have to do this.
# cargo fuzz init

cargo fuzz run <fuzz name>
```

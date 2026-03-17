# SETUP

```sh
rustup default nightly
cargo install cargo-fuzz

# TODO: I want it to be inside `sodigy/tests/fuzzer/`, but I can't do that
# It's already done and is inside the repo, so you don't have to do this.
# cargo fuzz init

cargo fuzz run <fuzz name>
```

# Coverage

Once you run the test runner (binary in `tests/runner/`), you'll have test files in `fuzz/corpus/cnr/`. You can run `cargo +nightly fuzz coverage cnr` to generate the coverage data of the test files.

You have to run `rustup component add llvm-tools-preview --toolchain nightly` and `cargo install cargo-binutils rustfilt` in order to visualize the coverage data.

```
/Users/baehyunsol/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/llvm-cov show fuzz/target/aarch64-apple-darwin/coverage/aarch64-apple-darwin/release/cnr -format=html -instr-profile=fuzz/coverage/cnr/coverage.profdata -show-line-counts-or-regions show-instantiations -output-dir=coverage
```

-> It works, but I'm not sure whether it's the best way... There must be a better way :(
-> The coverage in the report is too small... There must be some cnr cases that successfully compiles, but inter-mir and inter-hir's coverage are 0%.
-> It doesn't work on Linux...

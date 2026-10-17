rm -r it;
cargo build --release;
cp target/release/sodigy ~/.cargo/bin/sodigy;
sodigy new it;
cd it;
sodigy build --test --std-assertions --emit=bytecode -o=obc;
sodigy run --bytecode=obc;

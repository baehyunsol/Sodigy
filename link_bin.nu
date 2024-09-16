cargo build --release

# TODO: it doesn't work on Windows
ln -f ./target/release/sodigy ./sodigy
chmod +x ./sodigy

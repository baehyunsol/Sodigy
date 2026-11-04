fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    sodigy_driver::main_(args);
}


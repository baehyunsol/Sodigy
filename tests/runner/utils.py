import os

def goto_root():
    while "crates" not in (ll := os.listdir()) and "Cargo.toml" not in ll:
        os.chdir("..")

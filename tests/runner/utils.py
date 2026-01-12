import os

def goto_root():
    while "crates" not in (ll := os.listdir()) or "Cargo.toml" not in ll:
        os.chdir("..")

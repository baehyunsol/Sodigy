# TODO: rewrite this in Sodigy
#
# You have to run this code in `crates/error-gen/`
# It requires `crates/error-gen/errors.txt`, which is created by running the proc_macro in the crate.
#
# It iterates all the files in the Sodigy compiler, and creates a map of error kinds.
import os
import re

def count_errors(map, errors, path):
    if path == "crates/error-gen/":
        return

    for d in os.listdir():
        if os.path.isdir(d):
            if d == "target":
                continue

            os.chdir(d)
            count_errors(map, errors, path + d + "/")
            os.chdir("..")

        elif d.endswith(".rs"):
            with open(d, "r") as f:
                code = f.read()

            for name, _, _ in errors:
                c = code.count(f"ErrorKind::{name}")

                if c > 0:
                    map[name] = map.get(name, []) + [(path + d, c)]

errors = []

with open("errors.txt", "r") as f:
    for line in f.read().split("\n"):
        name, index, level = line.split("/")
        index = int(index)
        errors.append((name, index, level))

map = { name: [] for name, _, _ in errors }

os.chdir("../..")
count_errors(map, errors, "")
print(map)

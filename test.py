# TODO: rewrite it in Sodigy

import os
import re
import shutil
import subprocess
import sys

args = sys.argv
no_clean = "--no-clean" in args
no_std = "--no-std" in args
debug_bytecode = "--debug-bytecode" in args

# It's always enabled!
# debug_heap = "--debug-heap" in args
debug_heap = True

args = [arg for arg in args if not arg.startswith("-")]
filter = args[1] if len(args) > 1 else None
sample_files = []

for file in os.listdir("sample"):
    if file.endswith(".sdg"):
        sample_files.append(file)

if filter is not None and filter != "all":
    sample_files = [sample for sample in sample_files if filter in sample]

sample_files.sort()
result = {}

if os.path.exists("sodigy-test"):
    shutil.rmtree("sodigy-test")

features = (["debug-bytecode"] if debug_bytecode else []) + (["debug-heap"] if debug_heap else [])
features = ["--features=" + ",".join(features)] if features else []
subprocess.run(["cargo", "build", *features], check=True)
subprocess.run(["target/debug/sodigy", "new", "sodigy-test"], capture_output=True, check=True)
os.chdir("sodigy-test/src")

try:
    for file in sample_files:
        print(f"running sample/{file}")
        status = "compiling"

        try:
            with open(f"../../sample/{file}", "r") as f:
                sample = f.read()

            with open("lib.sdg", "w") as f:
                f.write(sample)

            flags = ["--no-std"] if no_std else []
            flags += ["--emit-irs"]
            p = subprocess.run(["../../target/debug/sodigy", "test", *flags], timeout=999)

            if p.returncode == 0:
                status = "success"

            else:
                status = "fail"

        except subprocess.TimeoutExpired:
            status = "timeout"

        color = 31 if status == "fail" else 33 if status == "timeout" else 32
        print(f"{file}: \033[{color}m{status}\033[0m")
        result[status] = result.get(status, 0) + 1

finally:
    os.chdir("../..")

    if not no_clean:
        shutil.rmtree("sodigy-test")

print(result)

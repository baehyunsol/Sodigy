# TODO: rewrite it in Sodigy

import os
import re
import subprocess

sample_files = []

for file in os.listdir("sample"):
    if file.endswith(".sdg"):
        sample_files.append(f"sample/{file}")

sample_files.sort()
result = {}

for file in sample_files:
    status = "compiling"

    try:
        p = subprocess.run(["cargo", "run", file], capture_output=True, timeout=20, text=True)

        if p.returncode == 0:
            status = "compile-success"

        else:
            status = "compile-fail"

    except subprocess.TimeoutExpired:
        status = "compile-timeout"

    if status == "compile-success":
        p = subprocess.run(["python3", "sample/target/run.py"], capture_output=True, timeout=20, text=True)
        status = "test-success" if p.returncode == 0 else "test-fail"

    color = 31 if status == "compile-fail" else 33 if status == "test-fail" else 32
    print(f"{file}: \033[{color}m{status}\033[0m")
    result[status] = result.get(status, 0) + 1

print(result)

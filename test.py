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

subprocess.run(["cargo", "build"], capture_output=True)

for file in sample_files:
    status = "compiling"

    try:
        p = subprocess.run(["target/debug/sodigy", "test", file], capture_output=False, timeout=20, text=True)

        if p.returncode == 0:
            status = "success"

        else:
            status = "fail"

    except subprocess.TimeoutExpired:
        status = "timeout"

    color = 31 if status == "fail" else 33 if status == "timeout" else 32
    print(f"{file}: \033[{color}m{status}\033[0m")
    result[status] = result.get(status, 0) + 1

print(result)

from error import parse_errors
import os
from run_result import RunResult, parse_expectation
import shutil
import subprocess
from typing import Optional
from utils import goto_root

def single_files(
    filter: Optional[str],
    no_std: bool,
    debug_bytecode: bool,
    debug_heap: bool,

    # seconds
    timeout: int = 20,
):
    goto_root()

    features = (["debug-bytecode"] if debug_bytecode else []) + (["debug-heap"] if debug_heap else [])
    features = ["--features=" + ",".join(features)] if features else []
    subprocess.run(["cargo", "build", *features], check=True)

    files = [file for file in os.listdir("tests/single-file/") if file.endswith(".sdg")]

    if filter is not None:
        files = [file for file in files if filter in file]

    files.sort()
    succ, fail = 0, 0

    if len(files) == 0:
        raise ValueError(f"There's no test that matches `{filter}`")

    for file in files:
        print(f"running `{file}`...")
        error = single_file(file, no_std, "target/debug/sodigy", timeout)
        color, status = (32, "success") if error is None else (31, "fail")
        print(f"{file}: \033[{color}m{status}\033[0m")

        if error is not None:
            print(error)
            fail += 1

        else:
            succ += 1

    print(f"succ: {succ}, fail: {fail}")
    return succ, fail

def single_file(
    # just a file name, without directories
    file: str,

    no_std: bool,

    # the path has to be absolute, or relative to the repository root
    sodigy_binary: str = "target/debug/sodigy",

    # seconds
    timeout: int = 20,
) -> Optional[str]:  # If there's an error, it returns the error message.
    goto_root()

    if os.path.exists("sodigy-test/"):
        shutil.rmtree("sodigy-test/")

    subprocess.run([sodigy_binary, "new", "sodigy-test"], capture_output=True, check=True)
    file = os.path.join("tests/single-file/", file)

    with open(file, "r") as f:
        code = f.read()

    expectation = parse_expectation(code)

    with open("sodigy-test/src/lib.sdg", "w") as f:
        f.write(code)

    flags = ["--no-std"] if no_std else []
    flags += ["--emit-irs"]

    try:
        os.chdir("sodigy-test")
        p = subprocess.run([os.path.join("..", sodigy_binary), "test", *flags], timeout=timeout)
        errors, warnings = parse_errors(p.stderr)
        status = "success" if p.returncode == 0 else "test-fail" if p.returncode == 10 else "compile-fail" if p.returncode == 11 else "misc-error"
        result = RunResult(status, errors, warnings)

    except subprocess.TimeoutExpired:
        result = RunResult("timeout", [], [])

    try:
        result.expect(expectation)
        return None

    except Exception as e:
        return str(e)

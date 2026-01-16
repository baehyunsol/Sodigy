from error import parse_errors
import os
from run_result import RunResult, parse_expectation
import shutil
import subprocess
import time
from typing import Optional
from utils import get_file_hash, goto_root

def single_files(
    filter: Optional[str],
    no_std: bool,
    debug_bytecode: bool,
    debug_heap: bool,

    # batch | dump
    # batch: run and return the result
    # dump: run and dump the result to stdout/stderr
    mode: str = "batch",

    # If it's `batch` mode, this function will return
    # a list of results, and each result has captured stdout/stderr.
    # If you want to keep the colors (ANSI terminal colors), set this flag.
    save_with_color: bool = True,

    # seconds
    timeout: int = 20,
):
    goto_root()
    result_all = []

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
        print("\n\n")
        print(f"running `single-file/{file}`...")
        result = single_file(
            file,
            no_std,
            "target/debug/sodigy",
            ["capture_and_parse", "capture_and_save"] if mode == "batch" else ["capture_and_parse", "dump"],
            save_with_color,
            timeout,
        )
        error = result["error"]
        color, status = (32, "success") if error is None else (31, "fail")
        result_all.append(result)
        print(f"{file}: \033[{color}m{status}\033[0m")

        if error is not None:
            print(error)
            fail += 1

        else:
            succ += 1

    print("---------------------------")
    print(f"succ: {succ}, fail: {fail}")
    return result_all, succ, fail

def single_file(
    # just a file name, without directories
    file: str,

    # It compiles the sodigy code with `--no-std` option.
    no_std: bool,

    # the path has to be absolute, or relative to the repository root
    sodigy_binary: str,

    # "capture_and_parse"
    # "capture_and_save"
    # "dump"
    modes: list[str],

    save_with_color: bool,

    # seconds
    timeout: int = 20,
):
    goto_root()
    result = {
        "name": file,
        "error": None,
        "stdout": None,
        "stderr": None,
        "compile-errors": [],
        "compile-warnings": [],
        "hash": None,
    }

    if os.path.exists("sodigy-test/"):
        shutil.rmtree("sodigy-test/")

    subprocess.run([sodigy_binary, "new", "sodigy-test"], capture_output=True, check=True)
    file = os.path.join("tests/single-file/", file)
    result["hash"] = get_file_hash(file)

    with open(file, "r") as f:
        code = f.read()

    expectation = parse_expectation(code)

    with open("sodigy-test/src/lib.sdg", "w") as f:
        f.write(code)

    os.chdir("sodigy-test")
    error = None
    stdout = None
    stderr = None

    for mode in modes:
        flags = ["--no-std"] if no_std else []
        flags += ["--emit-irs"]

        if mode == "capture_and_parse" or mode == "capture_and_save" and not save_with_color:
            flags += ["--color=never"]

        else:
            flags += ["--color=always"]

        kwargs = {
            "capture_output": True,
            "text": True,
            "timeout": timeout,
        }

        if mode == "dump":
            kwargs.pop("capture_output")
            kwargs.pop("text")

        try:
            started_at = time.time()
            p = subprocess.run(
                [os.path.join("..", sodigy_binary), "test", *flags],
                **kwargs,
            )
            elapsed = int((time.time() - started_at) * 1000)

            if mode == "capture_and_parse":
                status = "success" if p.returncode == 0 else "test-error" if p.returncode == 10 else "compile-error" if p.returncode == 11 else "misc-error"
                errors, warnings = parse_errors(p.stderr) if status != "misc-error" else ([], [])
                run_result = RunResult(status, errors, warnings)
                result["compile-errors"] = [error.to_dict() for error in errors]
                result["compile-warnings"] = [warning.to_dict() for warning in warnings]

            elif mode == "capture_and_save":
                result["stdout"] = p.stdout
                result["stderr"] = p.stderr

        except subprocess.TimeoutExpired:
            run_result = RunResult("timeout", [], [])

        if mode == "capture_and_parse":
            try:
                run_result.expect(expectation)

            except Exception as e:
                result["error"] = str(e)

    return result

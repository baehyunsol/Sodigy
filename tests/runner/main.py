import crates
import json
import os
from single_file import single_files
import sys
import time
from utils import check_repo_clean, get_file_name, get_meta, goto_root

args = sys.argv
command = args[1] if len(args) > 1 else None
args = args[1:]

if command == "single-file":
    no_std = "--no-std" in args
    debug_bytecode = "--debug-bytecode" in args
    all = "--all" in args

    # It's always enabled!
    # debug_heap = "--debug-heap" in args
    debug_heap = True

    args = [arg for arg in args if not arg.startswith("-")]
    filter = None if all else args[1]

    if all and len(args) > 1:
        raise ValueError(f"Unexpected cli argument: {args[1].__repr__()}")

    single_files(
        filter,
        no_std,
        debug_bytecode,
        debug_heap,
        "batch" if all else "dump",
    )

elif command == "all":
    force = "--force" in args

    if not force and not check_repo_clean():
        raise ValueError("Git repository is not clean. Please commit your changes beforehand. If you want to override the status check, run it with `--force`")

    meta = get_meta()
    sf_started_at = time.time()
    sf_result, sf_succ, sf_fail = single_files(
        filter=None,
        no_std=False,
        debug_bytecode=False,
        debug_heap=True,
        mode="batch",
    )
    sf_elapsed = int((time.time() - sf_started_at) * 1000)
    c_started_at = time.time()
    c_result, c_succ, c_fail = crates.run_all()
    c_elapsed = int((time.time() - c_started_at) * 1000)
    result = {
        "meta": meta,
        "crate-test": c_result,
        "single-file-test": sf_result,
        "stat": {
            "crate-test": {
                "total": c_succ + c_fail,
                "success": c_succ,
                "fail": c_fail,
                "elapsed": c_elapsed,
            },
            "single-file-test": {
                "total": sf_succ + sf_fail,
                "success": sf_succ,
                "fail": sf_fail,
                "elapsed": sf_elapsed,
            },
            "elapsed": c_elapsed + sf_elapsed,
        },
    }

    goto_root()
    os.chdir("tests")

    if not os.path.exists("results/"):
        os.mkdir("results/")

    with open(f"results/{get_file_name(result)}", "w") as f:
        f.write(json.dumps(result, ensure_ascii=False, indent=4))

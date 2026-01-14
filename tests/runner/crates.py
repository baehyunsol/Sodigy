import os
import subprocess
import time
from utils import goto_root

def run_all():
    goto_root()
    os.chdir("crates")
    result_all = []
    succ, fail = 0, 0

    for crate in sorted(os.listdir()):
        print(f"testing crates/{crate}...")
        result = run_test(crate)
        result_all.append(result)

        if result["debug"]["error"] is not None or result["release"]["error"] is not None or result["doc"]["error"] is not None:
            fail += 1

        else:
            succ += 1

    print("---------------------------")
    print(f"succ: {succ}, fail: {fail}")
    return result_all, succ, fail

def run_test(crate: str) -> dict[str, str]:
    result = {
        "name": crate,
        "debug": {},
        "release": {},
        "doc": {},
    }
    goto_root()
    os.chdir(f"crates/{crate}")

    subprocess.run(["cargo", "clean"], check=True)

    started_at = time.time()
    r = subprocess.run(["cargo", "test"], capture_output=True, text=True)
    elapsed = time.time() - started_at
    subprocess.run(["cargo", "clean"], check=True)
    result["debug"]["error"] = r.stderr if r.returncode != 0 else None
    result["debug"]["elapsed"] = int(elapsed * 1000)

    started_at = time.time()
    r = subprocess.run(["cargo", "test", "--release"], capture_output=True, text=True)
    elapsed = time.time() - started_at
    subprocess.run(["cargo", "clean"], check=True)
    result["release"]["error"] = r.stderr if r.returncode != 0 else None
    result["release"]["elapsed"] = int(elapsed * 1000)

    started_at = time.time()
    r = subprocess.run(["cargo", "doc"], capture_output=True, text=True)
    elapsed = time.time() - started_at
    subprocess.run(["cargo", "clean"], check=True)
    result["doc"]["error"] = r.stderr if r.returncode != 0 else None
    result["doc"]["elapsed"] = int(elapsed * 1000)

    return result

if __name__ == "__main__":
    import json
    print(json.dumps(run_all(), indent=4))

import os
import subprocess
from utils import goto_root

def run_all():
    goto_root()
    os.chdir("crates")
    result_all = {}

    for crate in sorted(os.listdir()):
        print(f"testing crates/{crate}...")
        result = run_test(crate)

        for key, error in result.items():
            result_all[f"{crate} ({key})"] = error

    return result_all

def run_test(crate: str) -> dict[str, str]:
    errors = {}
    goto_root()
    os.chdir(f"crates/{crate}")

    subprocess.run(["cargo", "clean"], check=True)
    r1 = subprocess.run(["cargo", "test"], capture_output=True, text=True)
    r2 = subprocess.run(["cargo", "test", "--release"], capture_output=True, text=True)
    r3 = subprocess.run(["cargo", "doc"], capture_output=True, text=True)
    subprocess.run(["cargo", "clean"], check=True)

    if r1.returncode != 0:
        errors["debug"] = r1.stderr

    if r2.returncode != 0:
        errors["release"] = r2.stderr

    if r3.returncode != 0:
        errors["doc"] = r3.stderr

    return errors

if __name__ == "__main__":
    import json
    print(json.dumps(run_all(), indent=4))

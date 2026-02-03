import os

def goto_root():
    while "crates" not in (ll := os.listdir()) or "Cargo.toml" not in ll:
        os.chdir("..")

def check_repo_clean():
    import subprocess
    status = subprocess.run(["git", "status", "--porcelain"], capture_output=True, text=True, check=True).stdout

    for line in status.split("\n"):
        if "M" in line[:2] or "A" in line[:2] or "D" in line[:2]:
            return False

    return True

# Python's try-catch statement sucks. Sodigy's is superior.
def get_meta():
    import platform
    import subprocess

    goto_root()
    result = {}

    try:
        result["sodigy-commit-hash"] = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, text=True, check=True).stdout.strip()

    except Exception as e:
        result["sodigy-commit-hash"] = f"cannot get sodigy-commit-hash: {e}"

    try:
        result["cargo-version"] = subprocess.run(["cargo", "version"], capture_output=True, text=True, check=True).stdout.strip()

    except Exception as e:
        result["cargo-version"] = f"cannot get cargo-version: {e}"

    try:
        result["rustc-version"] = subprocess.run(["rustc", "--version"], capture_output=True, text=True, check=True).stdout.strip()

    except Exception as e:
        result["rustc-version"] = f"cannot get rustc-version: {e}"

    try:
        result["python-version"] = platform.python_version()

    except Exception as e:
        result["python-version"] = f"cannot get python-version: {e}"

    try:
        result["platform"] = platform.platform()

    except Exception as e:
        result["platform"] = f"cannot get platform: {e}"

    return result

# r: result of `python3 main.py all`
def get_file_name(r: dict) -> str:
    hash = r["meta"]["sodigy-commit-hash"]
    hash = ("?" * 9) if hash.startswith("cannot get") else hash
    os = r["meta"]["platform"].lower()
    os = "windows" if "windows" in os else "mac" if "mac" in os else "linux" if "linux" in os else None
    return f"result-{hash[:9]}-{os}.json"

# returns the hash of the content of the file
# I'm using git's hash function because I'll rewrite the entire test framework in Sodigy someday,
# and I want a universal hash function, not Python-specific one.
def get_file_hash(file: str) -> str:
    import subprocess
    return subprocess.run(["git", "hash-object", file], check=True, capture_output=True, text=True).stdout.strip()

import os
import subprocess
import sys
import time

def goto_root_dir():
    while "tests.py" not in (dirs := os.listdir()) and ".gitignore" not in dirs:
        os.chdir("..")

def clean():
    goto_root_dir()

    for file in os.listdir():
        if file.startswith("__tmp_") or file.startswith("__error_"):
            os.remove(os.path.join(os.getcwd(), file))

def draw_depgraph():
    depgraph_process = subprocess.Popen(["cargo", "depgraph"], stdout = subprocess.PIPE)
    dot_process = subprocess.Popen(["dot", "-Tpng"], stdin=depgraph_process.stdout, stdout = subprocess.PIPE)

    depgraph_process.stdout.close()  # allow depgraph_process to receive a SIGPIPE if dot_process exits.
    output, _ = dot_process.communicate()

    with open("./dep_graph.png", "wb") as f:
        f.write(output)

def main(
    depgraph: bool = False,
):
    started_at = time.time()
    goto_root_dir()
    env = os.environ.copy()
    env["RUST_LOG"] = "trace"

    test_commands = [
        ["cargo", "doc"],
        ["cargo", "test"],
        ["cargo", "test", "--release"],
    ]

    aux_commands = [
        ["cargo", "run", "--release", "--", "--raw-input", "let main = \"Hello, World!\";"],
        ["cargo", "run", "--", "--raw-input", "let main = \"Hello, World!\";"],
    ]

    try:
        for command in test_commands:
            assert subprocess.run(command, env=env).returncode == 0

        os.chdir("./crates")

        for crate in os.listdir():
            if not os.path.isdir(crate):
                continue

            os.chdir(crate)

            for command in test_commands:
                assert subprocess.run(command, env=env).returncode == 0

            os.chdir("..")

        goto_root_dir()

        for command in aux_commands:
            assert subprocess.run(command, env=env).returncode == 0

        # TODO: run `./sodigy --test XXX.sdg` here

    except AssertionError:
        print(f"test failed: running `{' '.join(command)}` at {os.getcwd()}")

    if depgraph:
        draw_depgraph()

    clean()
    print(f"Complete! It took {int(time.time() - started_at)} seconds...")

if __name__ == "__main__":
    main(
        depgraph="--dep-graph" in sys.argv,
    )

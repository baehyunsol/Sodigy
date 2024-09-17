import os
import subprocess
import sys
import time

def goto_root_dir():
    dirs = os.listdir()

    while "tests.py" not in dirs and ".gitignore" not in dirs:
        os.chdir("..")

def clean():
    # TODO: clean garbages from tests, if any
    pass

def draw_depgraph():
    depgraph_process = subprocess.Popen(["cargo", "depgraph"], stdout=subprocess.PIPE)
    dot_process = subprocess.Popen(["dot", "-Tpng"], stdin=depgraph_process.stdout, stdout=subprocess.PIPE)

    depgraph_process.stdout.close()  # Allow depgraph_process to receive a SIGPIPE if dot_process exits.
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

    # TODO: it doesn't run on windows
    aux_commands = [
        ["sh", "./link_bin.sh"],
        ["./sodigy", "--raw-input", "let main = \"Hello, World!\";"],
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

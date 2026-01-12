from single_file import single_files
import sys

args = sys.argv
no_std = "--no-std" in args
debug_bytecode = "--debug-bytecode" in args

# It's always enabled!
# debug_heap = "--debug-heap" in args
debug_heap = True

args = [arg for arg in args if not arg.startswith("-")]
filter = args[1] if len(args) > 1 else None

single_files(
    filter,
    no_std,
    debug_bytecode,
    debug_heap,
    20,
)

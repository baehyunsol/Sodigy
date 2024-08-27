# Impure Functions

There are 2 classes of impurity: IO and debug. Note that panicking (aborting the entire process) is 100% pure operation.

## IO

There are 2 pure types for IO operations: `IOAction` and `IOResult`. Being pure, any function can do anything with the types.

There's an impure function that takes `IOAction` as an input and returns `IOResult`. This is the only impure function in this language, and only the main function can call this function.

```
let run_io(action: IOAction): IOResult = @@__very_dangerous_function(action);

let struct IOAction = {
    #> IOResult also has an `id` field, telling which action the result is from.
    #> It's users' responsibility to keep it unique.
    id: Int,
    kind: IOActionKind,
};

let enum IOActionKind = {
    Print(String),
    ReadStdin,
    ReadStdinLine,
    GetTime,
    RandomInt,
    Sleep(Int),  # milliseconds
    ReadFile {
        path: String,
        mode: ReadMode,
    },
    WriteFile {
        path: String,
        content: List(Byte),
        mode: WriteMode,
    },
};

let struct IOResult = {
    id: Int,
    kind: IOResultKind,
};

let enum IOResultKind = {
    FileNotFoundError { path: String },
    PermissionError { path: String },
};
```

## Debug

For debugging purpose, you can call impure functions in any context. These functions are not supposed to change the behavior of the program (it's a bug if so), and can be opt out when compiled with optimization (which is not implemented yet).

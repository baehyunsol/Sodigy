# Impure Functions

There are 2 classes of impurity: IO and debug. Note that panicking (aborting the entire process) is 100% pure operation.

## IO

There are 2 pure types for IO operations: `IOAction` and `IOResult`. Being pure, any function can do anything with the types.

```
let main(world: List(IOResult)): List(IOAction) = # Your implementation goes here
```

Above is how the main function deals with impurity.

1. It returns impure actions to run.
2. The actions are run outside Sodigy.
3. When the results are ready, the main function is called again with the new results.
4. It's called over and over until you quit.
5. It doesn't mean the type signature of the main function has to be like that.
  - If it returns an `Int`, it's the same as quitting with an exit code. An integer greater than 255 or less than 0 are converted to 1.

```
#> Every `IOAction` generates exactly one `IOResult`, except `Quit`.
#> If it's synchronous, the result is given at the next call of `main`.
let struct IOAction = {
    #> IOResult also has an `id` field, telling which action the result is from.
    #> It's users' responsibility to keep it unique.
    id: Int,
    kind: IOActionKind,

    # TODO: is it possible to impl async for all IOAction?
    #> Any IOAction can be asynchronous.
    async: Bool,
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
    Quit { code: Int },
};

let struct IOResult = {
    id: Int,
    kind: IOResultKind,
};

let enum IOResultKind = {
    FileNotFoundError { path: String },
    PermissionError { path: String },
    # TODO: many more variants...
};
```

## Debug

For debugging purpose, you can call impure functions in any context. These functions are not supposed to change the behavior of the program (it's a bug if so), and can be opt out when compiled with optimization (which is not implemented yet).

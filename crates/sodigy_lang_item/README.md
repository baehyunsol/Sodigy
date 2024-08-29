# Lang Items

Sodigy Compiler sometimes have to create new tokens while compiling. If the token has to use a name (whether it's in std or a new function defined by the compiler), it uses lang items.

The lang items are prefixed with `@@lang_item_`, so that user code cannot override lang items.

The lang items are later converted to normal Sodigy identifiers, when there's no worry for name confusions

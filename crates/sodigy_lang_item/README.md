# Lang Items

Sodigy Compiler sometimes have to create new tokens while compiling. If the token has to access names in std, it uses lang-item.

The lang items are prefixed with `@@__lang_item_`, so that user code cannot override lang items.

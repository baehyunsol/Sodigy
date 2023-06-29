pub enum StmtKind {

    // 'def' NAME '(' ARGS ')' ':' TYPE '=' EXPR ';'
    FuncDef,

    // 'def' NAME ':' TYPE '=' EXPR ';'
    ConstDef,

    // has many aliases
    // 'use' PATH 'as' NAME ';'
    UseName,
}
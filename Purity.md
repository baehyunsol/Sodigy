# Impure Functions

There are 2 classes of impurity: IO and debug.

## IO

TODO: it has to be designed from scratch

my idea is to use a state-machine like approach

1. There are many states.
2. The entry of a program and the termination of a progrom are states.
3. State itself is pure. A state can eval arbitrary Sodigy functions and decide which state to go next.
  - The only difference is that normal Sodigy functions cannot call state-related functions. Only state-related functions can.
4. Impure things can happen BETWEEN states. A state lists which impure actions to run. The actions are run while the transition. The results of the actions are sent to the next state.
5. Async states? Multiple states in parallel?

## Debug

For debugging purpose, you can call impure functions in any context. These functions are not supposed to change the behavior of the program (it's a bug if so), and can be opt out when compiled with optimization (which is not implemented yet).

TODO: is `panic()` pure?

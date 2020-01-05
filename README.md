# Brassfuck
A simple Brainfuck interpreter written in Rust.
Performs basic optimizations such as reducing `+++++` into a single `Add(5)` expression,
and `[-]` into a `Clear` expression that sets the current cell to 0.

## Implementation Details
- Memory consists of 65536 8-bit cells
- Incrementation and decrementation operations wrap around
- EOF returns 0
    - TODO: rot13.bf expects 255, maybe this should be configurable?

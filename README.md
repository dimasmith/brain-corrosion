# brainfuck

A library to implement the brainfuck language interpreter.

Contains a simple virtual machine that can execute brainfuck opcodes directly.
The source file parser and translator is also a part of the package.

## Virtual Machine

Operations:

| Mnemonic | Symbol | Description |
| :-- | :-- | :-- |
| Inc | `+` | Increments value at current memory cell by 1 |
| Dec | `-` | Decrements value at current memory cell by 1 |
| Next | `>` | Moves memory pointer to the next cell |
| Prev | `<` | Moves memory pointer to the previous cell |
| LoopForward | `[` | Jump past the matching ] if the cell at the pointer is 0 |
| LoopBack | `]` | Jump back to the matching [ if the cell at the pointer is nonzero |
| In | `,` | Read a character and store in the current memory cell |
| Out | `.` | Write a byte from the current memory cell |

The virtual machine starts with 30k bytes of memory.
The memory buffer is cyclic - when pointer moves before the start of after the end - it is wrapped.

Incrementing memory value beyond 0xff and decrementing beyond 0 wraps the value.

## Interpreter

The package contains a `bf` executable to run source files.

The `bf` reads program from file, parses and translates it and executes on a standard virtual machine.
The virtual machine uses standard input and output.

```shell
bf rot13.bf
```

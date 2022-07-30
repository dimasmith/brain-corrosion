//! Library implementing brainfuck language and runtime.
//!
//! It contains a simple virtual machine able to run brainfuck code directly.
//! It also has a parser and translator to prepare brainfuck source for the virtual machine.
pub mod parser;
pub mod vm;

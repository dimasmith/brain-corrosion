//! Brainfuck virtual machines definitions and implementations.

use std::{rc::Rc, cell::RefCell, io::{Read, Write, BufReader, stdin, stdout, BufWriter}};

pub mod standard;

/// A trait for brainfuck virtual machine.
/// 
/// Accepts a boxed array of operations and expect to return error when something goes wrong.
pub trait Vm {
    type Operation;
    type Error;

    /// Loads and immediately run the program.
    fn execute(&mut self, program: Box<[Self::Operation]>) -> Result<(), Self::Error>;

    /// Loads the program into the virtual machine.
    fn load(&mut self, program: Box<[Self::Operation]>) -> Result<(), Self::Error>;

    /// Runs previously loaded program
    fn run(&mut self) -> Result<(), Self::Error>;
}

/// Input reader reference for virtual machine.
pub type Input = Rc<RefCell<dyn Read>>;

/// Create reference to the standard input.
pub fn standard_input() -> Input {
    Rc::new(RefCell::new(BufReader::new(stdin())))
}

/// Output writer reference for virtual machine.
pub type Output = Rc<RefCell<dyn Write>>;

/// Create reference to the standard output.
pub fn standard_output() -> Output {
    Rc::new(RefCell::new(BufWriter::new(stdout())))
}


//! Brainfuck virtual machines definitions and implementations.

use std::{
    cell::RefCell,
    io::{stdin, stdout, BufReader, BufWriter, Read, Write},
    rc::Rc,
};

pub mod standard;

/// A trait for brainfuck virtual machine.
///
/// Accepts a boxed array of operations and expect to return error when something goes wrong.
pub trait Vm {
    type Operation;
    type Error;

    /// Run the program in a virtual machine.
    fn run(&mut self, program: Box<[Self::Operation]>) -> Result<(), Self::Error>;
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

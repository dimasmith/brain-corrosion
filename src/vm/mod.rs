//! Brainfuck virtual machines definitions and implementations.

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

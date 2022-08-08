//! Virtual machine to directly execute brainfuck code.
//!
//! The virtual machine starts with a specified amount of memory (30_000 bytes by default).
//! By default it use standard input and output but that can be changed.
//! The standard machine is only capable of executing the default set of brainfuck commands.
//! It handles only 8 commands described in the language specification.

use std::{
    cell::RefCell,
    io::{ErrorKind, Read, Write},
    rc::Rc,
};

use crate::vm::{Vm, Output, Input, standard_output, standard_input};

const DEFAULT_MEMORY_SIZE: usize = 30000;


/// Executable instruction for virtual machine.
///
/// The set of operations is exactly the same as in brainfuck languate definition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    /// `+` - Increment the memory cell at the pointer
    Inc,
    /// `-` - Decrement the memory cell at the pointer
    Dec,
    /// `>` - Move the pointer to the right
    Next,
    /// `<` - Move the pointer to the left
    Prev,
    /// `,` - Input a character and store it in the cell at the pointer
    In,
    /// `.` - Output the character signified by the cell at the pointer
    Out,
    /// `[` - Jump past the matching ] if the cell at the pointer is 0
    LoopForward,
    /// `]` - Jump back to the matching [ if the cell at the pointer is nonzero
    LoopBack,
}

/// Errors thrown by the virtual machine during execution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VmError {
    /// Unmatched `[` instruction
    NoLoopEnd,
    /// Unmatched `]` instruction
    NoLoopStart,
    /// Instruction pointer outside of the program memory.
    /// This should not happen and indicates a programming error.
    OutOfProgramMemory,
    /// Error during IO operation.
    /// Contains error kind for root cause analysis.
    IoError(ErrorKind),
}

/// Virtual machine for direct brainfuck execution.
///
/// The machine is strictly standard and only executes brainfuck code without any optimizations.
/// It uses 30000 bytes of operative memory.
/// The machine contains:
/// - A set of operations
/// - An instruction counter (ip)
/// - An array for memory
/// - Current memory cell pointer (mp)
pub struct StandardVm {
    memory: Box<[u8]>,
    mp: usize,
    program: Box<[Operation]>,
    ip: usize,
    output: Output,
    input: Input,
}

impl StandardVm {

    /// Create vm with standard parameters.
    pub fn new() -> Self {
        let mem = vec![0; DEFAULT_MEMORY_SIZE];
        StandardVm {
            memory: mem.into_boxed_slice(),
            mp: 0,
            program: vec![].into_boxed_slice(),
            ip: 0,
            output: standard_output(),
            input: standard_input(),
        }
    }

    /// Create VM with custom input and output.
    pub fn with_io(output: Output, input: Input) -> Self {
        StandardVm {
            output,
            input,
            ..StandardVm::new()
        }
    }

    fn reset(&mut self) {
        self.memory.fill(0);
        self.mp = 0;
        self.ip = 0;
    }

    fn operation(&self) -> Option<Operation> {
        if self.ip >= self.program.len() {
            return None;
        }
        Some(self.program[self.ip])
    }

    fn inc(&mut self) -> usize {
        let (v, _) = self.get().overflowing_add(1);
        self.memory[self.mp] = v;
        self.ip + 1
    }

    fn dec(&mut self) -> usize {
        let (v, _) = self.get().overflowing_sub(1);
        self.memory[self.mp] = v;
        self.ip + 1
    }

    fn mem_next(&mut self) -> usize {
        self.mp += 1;
        if self.mp == self.memory.len() {
            self.mp = 0;
        }
        self.ip + 1
    }

    fn mem_prev(&mut self) -> usize {
        self.mp = match self.mp {
            0 => self.memory.len() - 1,
            x => x - 1,
        };
        self.ip + 1
    }

    fn read(&mut self) -> Result<usize, VmError> {
        let mut buf: [u8; 1] = [0; 1];
        self.input
            .borrow_mut()
            .read_exact(&mut buf)
            .map_err(|ioe| VmError::IoError(ioe.kind()))?;
        self.put(buf[0]);
        Ok(self.ip + 1)
    }

    fn write(&mut self) -> Result<usize, VmError> {
        let data: [u8; 1] = [self.get()];
        self.output
            .borrow_mut()
            .write_all(&data)
            .map_err(|ioe| VmError::IoError(ioe.kind()))?;
        Ok(self.ip + 1)
    }

    fn loop_zero(&mut self) -> Result<usize, VmError> {
        if self.get() != 0 {
            return Ok(self.ip + 1);
        }
        self.ip_next().map_err(|_| VmError::NoLoopEnd)?;
        let mut nested = 0;
        loop {
            match self.operation() {
                Some(Operation::LoopForward) => nested += 1,
                Some(Operation::LoopBack) => {
                    if nested == 0 {
                        return Ok(self.ip + 1);
                    }
                    nested -= 1;
                }
                _ => {}
            }
            self.ip_next().map_err(|_| VmError::NoLoopEnd)?;
        }
    }

    fn loop_back_nz(&mut self) -> Result<usize, VmError> {
        if self.get() == 0 {
            return Ok(self.ip + 1);
        }
        self.ip_prev().map_err(|_| VmError::NoLoopStart)?;
        let mut nested = 0;
        loop {
            match self.operation() {
                Some(Operation::LoopBack) => nested += 1,
                Some(Operation::LoopForward) => {
                    if nested == 0 {
                        return Ok(self.ip + 1);
                    }
                    nested -= 1;
                }
                _ => {}
            }
            self.ip_prev().map_err(|_| VmError::NoLoopStart)?;
        }
    }

    fn get(&self) -> u8 {
        self.memory[self.mp]
    }

    fn put(&mut self, v: u8) {
        self.memory[self.mp] = v;
    }

    fn ip_prev(&mut self) -> Result<usize, VmError> {
        if self.ip > 0 {
            self.ip -= 1;
            return Ok(self.ip);
        }
        Err(VmError::OutOfProgramMemory)
    }

    fn ip_next(&mut self) -> Result<usize, VmError> {
        if self.ip < self.program.len() - 1 {
            self.ip += 1;
            return Ok(self.ip);
        }
        Err(VmError::OutOfProgramMemory)
    }
}

impl Vm for StandardVm {
    type Operation = Operation;

    type Error = VmError;

    fn run(&mut self, program: Box<[Self::Operation]>) -> Result<(), Self::Error> {
        self.reset();
        self.program = program;

        while let Some(op) = self.operation() {
            let ip = match op {
                Operation::Inc => self.inc(),
                Operation::Dec => self.dec(),
                Operation::Next => self.mem_next(),
                Operation::Prev => self.mem_prev(),
                Operation::In => self.read()?,
                Operation::Out => self.write()?,
                Operation::LoopForward => self.loop_zero()?,
                Operation::LoopBack => self.loop_back_nz()?,
            };
            self.ip = ip;
        }

        Ok(())
    }
}

impl Default for StandardVm {
    /// Create VM with the 30_000 bytes of memory and standard input and output.
    fn default() -> Self {
        StandardVm::new()
    }
}

/// Builder for standard VM.
///
/// # Example
/// ```
/// # use brain_corrosion::vm::standard::vm::StandardVmBuilder;
///
/// let compact_vm = StandardVmBuilder::new()
///     .with_memory_size(0xff)
///     .build();
/// ```
pub struct StandardVmBuilder {
    mem_size: Option<usize>,
    output: Option<Output>,
    input: Option<Input>,
}

impl StandardVmBuilder {
    /// Create virtual machine builder.
    pub fn new() -> Self {
        StandardVmBuilder {
            mem_size: None,
            output: None,
            input: None,
        }
    }

    /// Set amount of memory to be used with the virtual machine.
    pub fn with_memory_size(mut self, size: usize) -> Self {
        self.mem_size = Some(size);
        self
    }

    /// Set reference to output writer.
    pub fn with_output(mut self, output: Rc<RefCell<dyn Write>>) -> Self {
        self.output = Some(output);
        self
    }

    /// Set reference to input reader.
    pub fn with_input(mut self, input: Rc<RefCell<dyn Read>>) -> Self {
        self.input = Some(input);
        self
    }

    /// Build configured virtual machine.
    ///
    /// The method sets any non-specified configuration parameters
    /// to their default values.
    pub fn build(self) -> StandardVm {
        let memory = vec![0; self.mem_size.unwrap_or(DEFAULT_MEMORY_SIZE)].into_boxed_slice();
        let output = self.output.unwrap_or_else(|| standard_output());
        let input = self.input.unwrap_or_else(|| standard_input());
        StandardVm {
            memory,
            output,
            input,
            ..StandardVm::default()
        }
    }
}

impl Default for StandardVmBuilder {
    fn default() -> Self {
        StandardVmBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_default_memory_value() {
        let vm = StandardVm::default();

        assert_eq!(vm.get(), 0, "default value of memory cell is 0");
    }

    #[test]
    fn set_current_cell_value() {
        let mut vm = StandardVm::default();

        vm.put(0x8d);

        assert_eq!(vm.get(), 0x8d, "value must be set");
    }

    #[test]
    fn increment_current_memory_cell() {
        let mut vm = StandardVm::default();

        vm.inc();

        assert_eq!(vm.get(), 1, "memory value must be incremented");
    }

    #[test]
    fn decrement_current_memory_cell() {
        let mut vm = StandardVm::default();
        vm.put(0xf);

        vm.dec();

        assert_eq!(vm.get(), 0xe, "memory value must be decremented");
    }

    #[test]
    fn decrement_carry_over() {
        let mut vm = StandardVm::default();

        vm.dec();

        assert_eq!(vm.get(), 255, "decrementing zero cell must carry over");
    }

    #[test]
    fn increment_carry_over() {
        let mut vm = StandardVm::default();
        vm.put(u8::MAX); // set value to maximum

        vm.inc();

        assert_eq!(vm.get(), 0, "incrementing maximum value carries over to 0");
    }

    #[test]
    fn default_memory_cell() {
        let vm = StandardVm::default();

        assert_eq!(vm.mp, 0, "default memory cell is 0");
    }

    #[test]
    fn next_memory_cell() {
        let mut vm = StandardVm::default();

        vm.mem_next();

        assert_eq!(vm.mp, 1, "should move to next cell");
    }

    #[test]
    fn previous_memory_cell() {
        let mut vm = StandardVm::default();
        vm.mem_next();

        vm.mem_prev();

        assert_eq!(vm.mp, 0, "should move to previous cell");
    }

    #[test]
    fn rotate_memory_on_bounds() {
        let mut vm = StandardVm::default();

        vm.mem_prev();

        assert_eq!(
            vm.mp,
            vm.memory.len() - 1,
            "should rotate over to last cell"
        );

        vm.mem_next();

        assert_eq!(vm.mp, 0, "should rotate over to first cell");
    }

    #[test]
    fn run_empty_program() {
        let mut vm = StandardVm::default();

        let result = vm.run(Box::new([]));

        assert_eq!(vm.get(), 0, "memory must not change");
        assert_eq!(
            vm.ip, 0,
            "no instruction in the program. pointer should not move"
        );
        assert!(result.is_ok(), "empty program is valid");
    }

    #[test]
    fn move_instruction_pointer() {
        let mut vm = StandardVm::default();

        vm.run(Box::new([Operation::Inc])).unwrap();

        assert_eq!(
            vm.ip, 1,
            "elementary operations increments instruction pointer"
        );
    }

    #[test]
    fn zero_cell() {
        let mut vm = StandardVm::default();
        vm.put(5);

        vm.run(Box::new([
            Operation::LoopForward,
            Operation::Dec,
            Operation::LoopBack,
        ]))
        .unwrap();

        assert_eq!(vm.get(), 0, "cell must be zeroed");
        assert_eq!(vm.ip, 3, "instruction pointer must be at end");
    }

    #[test]
    fn simple_loop() {
        let mut vm = StandardVm::default();

        vm.run(Box::new([
            Operation::Inc,
            Operation::Inc,
            Operation::Inc,
            Operation::Inc,
            Operation::Inc,
            Operation::LoopForward,
            Operation::Next,
            Operation::Inc,
            Operation::Inc,
            Operation::Prev,
            Operation::Dec,
            Operation::LoopBack,
            Operation::Next,
            Operation::Inc,
        ]))
        .unwrap();

        assert_eq!(vm.get(), 11, "2n+1 should be calculated");
        assert_eq!(vm.ip, 14, "instruction pointer must be at end");
    }

    #[test]
    fn unclosed_loop() {
        let mut vm = StandardVm::default();
        let program = Box::new([Operation::LoopForward]);

        let result = vm.run(program);

        assert!(result.is_err(), "the loop is not closed and VM must fail");
        assert_eq!(result.err().unwrap(), VmError::NoLoopEnd);
    }

    #[test]
    fn loop_without_opening() {
        let mut vm = StandardVm::default();
        let program = Box::new([Operation::Inc, Operation::LoopBack]);

        let result = vm.run(program);

        assert!(result.is_err(), "the loop is not closed and VM must fail");
        assert_eq!(result.err().unwrap(), VmError::NoLoopStart);
    }
}

//! Virtual machine to directly execute brainfuck code.
//!
//! The virtual machine starts with a specified amount of memory (30_000 bytes by default).

use std::{
    cell::RefCell,
    io::{stdin, stdout, BufReader, BufWriter, Error, Read, Write},
    rc::Rc,
};

const DEFAULT_MEMORY_SIZE: usize = 30000;

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Inc,
    Dec,
    Next,
    Prev,
    In,
    Out,
    LoopForward,
    LoopBack,
}

#[derive(Debug, Clone, Copy)]
pub enum Err {
    UnclosedLoop,
    IoError,
}

/// Simple virtual machine for direct brainfuck execution.
///
/// It uses 30000 bytes of operative memory.
/// The machine contains:
/// - A set of operations
/// - An instruction counter (ip)
/// - An array for memory
/// - Current memory cell pointer (mp)
pub struct BrainfuckVm {
    memory: Box<[u8]>,
    mp: usize,
    program: Box<[Operation]>,
    ip: usize,
    output: Rc<RefCell<dyn Write>>,
    input: Rc<RefCell<dyn Read>>,
}

impl BrainfuckVm {
    pub fn io(output: Rc<RefCell<dyn Write>>, input: Rc<RefCell<dyn Read>>) -> Self {
        BrainfuckVm {
            output,
            input,
            ..Default::default()
        }
    }

    pub fn run(&mut self, program: Box<[Operation]>) -> Result<(), Err> {
        self.reset();
        self.program = program;

        let mut op = self.operation();
        while op.is_some() {
            let ip = match op.unwrap() {
                Operation::Inc => self.inc(),
                Operation::Dec => self.dec(),
                Operation::Next => self.mem_next(),
                Operation::Prev => self.mem_prev(),
                Operation::In => self.read().map_err(|_| Err::IoError)?,
                Operation::Out => self.write().map_err(|_| Err::IoError)?,
                Operation::LoopForward => self.loop_zero()?,
                Operation::LoopBack => self.loop_back_nz()?,
            };
            self.ip = ip;
            op = self.operation();
        }

        Ok(())
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

    fn read(&mut self) -> Result<usize, Error> {
        let mut buf: [u8; 1] = [0; 1];
        self.input.borrow_mut().read_exact(&mut buf)?;
        self.put(buf[0]);
        Ok(self.ip + 1)
    }

    fn write(&mut self) -> Result<usize, Error> {
        let data: [u8; 1] = [self.get()];
        self.output.borrow_mut().write_all(&data)?;
        Ok(self.ip + 1)
    }

    fn loop_zero(&mut self) -> Result<usize, Err> {
        if self.get() != 0 {
            return Ok(self.ip + 1);
        }
        self.ip += 1;
        let mut nested = 0;
        while let Some(op) = self.operation() {
            match op {
                Operation::LoopForward => nested += 1,
                Operation::LoopBack => {
                    if nested == 0 {
                        return Ok(self.ip + 1);
                    }
                    nested -= 1;
                }
                _ => {}
            }
            self.ip += 1;
        }
        Err(Err::UnclosedLoop)
    }

    fn loop_back_nz(&mut self) -> Result<usize, Err> {
        if self.get() == 0 {
            return Ok(self.ip + 1);
        }
        self.ip -= 1;
        let mut nested = 0;
        while let Some(op) = self.operation() {
            match op {
                Operation::LoopBack => nested += 1,
                Operation::LoopForward => {
                    if nested == 0 {
                        return Ok(self.ip + 1);
                    }
                    nested -= 1;
                }
                _ => {}
            }
            self.ip -= 1;
        }
        Err(Err::UnclosedLoop)
    }

    fn get(&self) -> u8 {
        self.memory[self.mp]
    }

    fn put(&mut self, v: u8) {
        self.memory[self.mp] = v;
    }
}

impl Default for BrainfuckVm {
    fn default() -> Self {
        let mem = vec![0; DEFAULT_MEMORY_SIZE];
        BrainfuckVm {
            memory: mem.into_boxed_slice(),
            mp: 0,
            program: vec![].into_boxed_slice(),
            ip: 0,
            output: Rc::new(RefCell::new(BufWriter::new(stdout()))),
            input: Rc::new(RefCell::new(BufReader::new(stdin()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_default_memory_value() {
        let vm = BrainfuckVm::default();

        assert_eq!(vm.get(), 0, "default value of memory cell is 0");
    }

    #[test]
    fn set_current_cell_value() {
        let mut vm = BrainfuckVm::default();

        vm.put(0x8d);

        assert_eq!(vm.get(), 0x8d, "value must be set");
    }

    #[test]
    fn increment_current_memory_cell() {
        let mut vm = BrainfuckVm::default();

        vm.inc();

        assert_eq!(vm.get(), 1, "memory value must be incremented");
    }

    #[test]
    fn decrement_current_memory_cell() {
        let mut vm = BrainfuckVm::default();
        vm.put(0xf);

        vm.dec();

        assert_eq!(vm.get(), 0xe, "memory value must be decremented");
    }

    #[test]
    fn decrement_carry_over() {
        let mut vm = BrainfuckVm::default();

        vm.dec();

        assert_eq!(vm.get(), 255, "decrementing zero cell must carry over");
    }

    #[test]
    fn increment_carry_over() {
        let mut vm = BrainfuckVm::default();
        vm.put(u8::MAX); // set value to maximum

        vm.inc();

        assert_eq!(vm.get(), 0, "incrementing maximum value carries over to 0");
    }

    #[test]
    fn default_memory_cell() {
        let vm = BrainfuckVm::default();

        assert_eq!(vm.mp, 0, "default memory cell is 0");
    }

    #[test]
    fn next_memory_cell() {
        let mut vm = BrainfuckVm::default();

        vm.mem_next();

        assert_eq!(vm.mp, 1, "should move to next cell");
    }

    #[test]
    fn previous_memory_cell() {
        let mut vm = BrainfuckVm::default();
        vm.mem_next();

        vm.mem_prev();

        assert_eq!(vm.mp, 0, "should move to previous cell");
    }

    #[test]
    fn rotate_memory_on_bounds() {
        let mut vm = BrainfuckVm::default();

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
        let mut vm = BrainfuckVm::default();

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
        let mut vm = BrainfuckVm::default();

        vm.run(Box::new([Operation::Inc])).unwrap();

        assert_eq!(
            vm.ip, 1,
            "elementary operations increments instruction pointer"
        );
    }

    #[test]
    fn zero_cell() {
        let mut vm = BrainfuckVm::default();
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
        let mut vm = BrainfuckVm::default();

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
}

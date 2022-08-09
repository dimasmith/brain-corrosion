use std::{env, fs};
use std::io::{BufReader, Read, stdin};

use brain_corrosion::parser;
use brain_corrosion::vm::standard::translator;
use brain_corrosion::vm::standard::vm::StandardVm;
use brain_corrosion::vm::Vm;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let source_path_str = &args[1];
        let source = fs::read_to_string(source_path_str).expect("cannot read source file");
        let source_reader = BufReader::new(source.as_bytes());
        execute_on_standard_vm(source_reader);
    } else {
        execute_on_standard_vm(stdin());
    }
}

fn execute_on_standard_vm<R: Read>(input: R) {
    let reader = BufReader::new(input);
    let tokens = parser::parse(reader).expect("source parsing failed");
    let program = translator::translate(tokens.as_ref());
    let mut vm = StandardVm::new();
    vm.load(program).expect("cannot load program");
    vm.run().expect("program execution failed");
}
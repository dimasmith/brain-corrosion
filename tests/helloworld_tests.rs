use brain_corrosion::parser::parse;
use brain_corrosion::vm::standard::translator;
use brain_corrosion::vm::standard::vm::StandardVmBuilder;
use brain_corrosion::vm::Vm;
use std::{
    cell::RefCell,
    io::{BufReader, Write},
    rc::Rc,
};

struct TestOut {
    buf: Vec<u8>,
}

impl Write for TestOut {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl ToString for TestOut {
    fn to_string(&self) -> String {
        let mut output = String::new();
        for b in self.buf.iter() {
            output.push(*b as char);
        }
        output
    }
}

impl Default for TestOut {
    fn default() -> Self {
        Self { buf: vec![] }
    }
}

fn run_program(source: &str) -> String {
    let mut program_input = BufReader::new(source.as_bytes());
    let tokens = parse(&mut program_input).unwrap();
    let ops = translator::translate(tokens.as_ref());
    let output = Rc::new(RefCell::new(TestOut::default()));
    {
        let mut vm = StandardVmBuilder::new().with_output(output.clone()).build();
        vm.run(ops).unwrap();
    }
    output.clone().as_ref().borrow().to_string()
}

#[test]
fn base_helloworld() {
    let result = run_program("++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.");

    assert_eq!("Hello World!\n", result);
}

#[test]
fn overflow_helloworld() {
    let result = run_program(
        ">++++++++[-<+++++++++>]<.>>+>-[+]++>++>+++[>[->+++<<+++>]<<]>-----.>->
    +++..+++.>-.<<+[>[+>+]>>]<--------------.>>.+++.------.--------.>+.>+.",
    );

    assert_eq!("Hello World!\n", result);
}

#[test]
fn short_helloworld() {
    let result = run_program(
        "--<-<<+[+[<+>--->->->-<<<]>]<<--.<++++++.<<-..<<.<+.>>.>>.<<<.+++.>>.>>-.<<<+.",
    );

    assert_eq!("Hello, World!", result);
}

#[test]
fn shortest_helloworld() {
    let result =
        run_program("+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+.");

    assert_eq!("Hello, World!", result);
}

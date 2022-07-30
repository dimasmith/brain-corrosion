//! Traslates parser output to the virtual machine operations.

use crate::{parser::Token, vm::Operation};

pub fn translate(tokens: &[Token]) -> Box<[Operation]> {
    let ops: Vec<Operation> = tokens
        .iter()
        .map(|token| match *token {
            Token::Inc => Operation::Inc,
            Token::Dec => Operation::Dec,
            Token::Shl => Operation::Prev,
            Token::Shr => Operation::Next,
            Token::Stl => Operation::LoopForward,
            Token::Endl => Operation::LoopBack,
            Token::In => Operation::In,
            Token::Out => Operation::Out,
        })
        .collect();
    ops.into_boxed_slice()
}

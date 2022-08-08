//! Traslates parser output to the operations of standard virtual machine.

use crate::{parser::Token, vm::standard::vm::Operation};

/// Translates source code tokens to virtual machine instructions.
///
/// ```
/// use brain_corrosion::vm::standard::translator;
/// use brain_corrosion::vm::standard::vm::Operation;
/// use brain_corrosion::parser::Token;
///
/// let program = translator::translate(&[
///     Token::Inc,
///     Token::Stl,
///     Token::Dec,
///     Token::Endl,
/// ]);
///
/// assert_eq!(
///     *program,
///     [
///         Operation::Inc,
///         Operation::LoopForward,
///         Operation::Dec,
///         Operation::LoopBack,
///     ]
/// )
/// ```
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_all_tokens() {
        let program = translate(&[
            Token::Inc,
            Token::Dec,
            Token::Stl,
            Token::Endl,
            Token::Shl,
            Token::Shr,
            Token::In,
            Token::Out,
        ]);

        assert_eq!(
            *program,
            [
                Operation::Inc,
                Operation::Dec,
                Operation::LoopForward,
                Operation::LoopBack,
                Operation::Prev,
                Operation::Next,
                Operation::In,
                Operation::Out,
            ]
        )
    }
}

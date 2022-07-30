//! Parser of the brainfuck source code.
//!
//! Produces an array of tokens representing the source.

use std::{
    io::{Error, Read},
    vec,
};

/// Source code token.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    Inc,
    Dec,
    Shl,
    Shr,
    In,
    Out,
    Stl,
    Endl,
}

/// Parse brainfuck program source into the list of tokens.
///
/// ```
/// # use brainfuck::parser;
/// # use std::io::BufReader;
///
/// let source = "[-]";
/// let mut input = BufReader::new(source.as_bytes());
/// let program = parser::parse(&mut input).unwrap();
/// ```
pub fn parse(input: &mut dyn Read) -> Result<Box<[Token]>, Error> {
    let mut code = vec![];
    let mut buf = vec![];
    input.read_to_end(&mut buf)?;
    for b in buf.iter() {
        let ch = *b as char;
        match ch {
            '+' => code.push(Token::Inc),
            '-' => code.push(Token::Dec),
            '>' => code.push(Token::Shr),
            '<' => code.push(Token::Shl),
            '.' => code.push(Token::Out),
            ',' => code.push(Token::In),
            '[' => code.push(Token::Stl),
            ']' => code.push(Token::Endl),
            _ => {}
        }
    }
    Ok(code.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn parse_all_tokens() {
        let source = "+-.,<>[]";
        let mut input = BufReader::new(source.as_bytes());

        let program = parse(&mut input).unwrap();

        assert_eq!(
            *program,
            [
                Token::Inc,
                Token::Dec,
                Token::Out,
                Token::In,
                Token::Shl,
                Token::Shr,
                Token::Stl,
                Token::Endl
            ]
        );
    }
}

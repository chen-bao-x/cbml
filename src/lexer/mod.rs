use crate::parser::ParserError;

mod lexer;

pub mod token;

pub fn tokenizer(file_path: &str, code: &str) -> Result<Vec<token::Token>, ParserError> {
    let mut lexer = lexer::Lexer::new(file_path, code);
    let tokens = lexer.tokenize()?;
    return Ok(tokens);
}

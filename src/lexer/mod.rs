mod lexer;

pub mod token;

pub fn tokenizer(code: &str) -> Result<Vec<token::Token>, String> {
    let mut lexer = lexer::Lexer::new(code);
    let tokens = lexer.tokenize()?;
    return Ok(tokens);
}

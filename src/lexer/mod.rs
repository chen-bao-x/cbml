pub mod lexer;
pub mod token;

/// 词法分析.
pub fn tokenize(file_path: &str, code: &str) -> crate::lexer::lexer::LexerResult {
    let mut lexer = lexer::Lexer::new(file_path, code);
    let re = lexer.tokenize();
    return re;
}

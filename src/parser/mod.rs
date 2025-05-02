use ast::stmt::Stmt;
use cbml_parser::CbmlParser;

use crate::lexer::token::Token;

mod ast;
mod cbml_parser;
mod typedef;

/// 解析 Token 列表并返回 AST
pub fn parse(source: &[Token]) -> Result<Vec<Stmt>, Vec<ParserError>> {
    let mut parser = CbmlParser::new(source);

    return parser.parse();
}

#[derive(Debug, Clone)]
pub struct ParserError {
    pub message: String,
    pub token: Option<Token>,
}

impl ParserError {
    fn new(message: String, token: Option<Token>) -> Self {
        ParserError { message, token }
    }

    fn message_only(message: String) -> Self {
        ParserError {
            message,
            token: None,
        }
    }
}

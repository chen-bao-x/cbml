use crate::lexer::token::Token;
use ast::stmt::Stmt;
pub use ast::stmt::StmtKind;
pub use cbml_parser::CbmlParser;
use parser_error::ParserError;

pub mod ast;
pub mod cbml_parser;
pub mod parser_error;

/// 解析 Token 列表并返回 AST
// pub fn parse(file_path: String, source: &[Token]) -> Result<Vec<StmtKind>, Vec<ParserError>> {
pub fn parse(file_path: String, source: &[Token]) -> Result<Vec<Stmt>, Vec<ParserError>> {
    let mut parser = CbmlParser::new(file_path, source);

    return parser.parse();
}

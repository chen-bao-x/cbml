use crate::lexer::token::Token;
pub use ast::stmt::StmtKind;
pub use cbml_parser::CbmlParser;
use cbml_parser::ParserResult;
use parser_error::CbmlError;

pub mod ast;
pub mod cbml_parser;
pub mod parser_error;

/// 解析 Token 列表并返回 AST
// pub fn parse(file_path: String, source: &[Token]) -> Result<Vec<StmtKind>, Vec<ParserError>> {
pub fn parse(file_path: String, source: &[Token]) -> ParserResult {
    let mut parser = CbmlParser::new(file_path, source);

    return parser.parse();
}

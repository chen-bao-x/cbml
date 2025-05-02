// use crate::lexer::token::{Token, TokenKind};

// use super::{ast::stmt::Stmt, ParserError};

// /// cbml 解析器
// pub struct TypedefParser<'a> {
//     tokens: &'a [Token],
//     current: usize,
//     eof: Token,
// }

// impl<'a> TypedefParser<'a> {
//     /// 创建一个新的 Parser 实例，接受一个 Token 列表
//     pub fn new(tokens: &'a [Token]) -> Self {
//         Self {
//             tokens,
//             current: 0,
//             eof: Token::new(crate::lexer::token::TokenKind::EOF, 0, 0),
//         }
//     }

//     /// 解析 Token 列表，直到结束并返回 AST
//     pub fn parse(&mut self) -> Result<Vec<Stmt>, Vec<ParserError>> {
//         let mut statements = Vec::new();
//         let mut errors = Vec::new();

//         while !self.is_at_end() {
//             println!("parse(&mut self) current: {:?}", self.peek());
//             _ = self.eat_zeor_or_multy(TokenKind::NewLine);

//             let re = self.parse_statement();
//             match re {
//                 Ok(s) => statements.push(s),
//                 Err(e) => {
//                     errors.push(e);
//                     self.current += 1; // 移动到下一个 Token
//                 }
//             }
//             _ = self.eat_zeor_or_multy(TokenKind::NewLine);
//         }

//         if errors.is_empty() {
//             return Ok(statements);
//         } else {
//             return Err(errors);
//         }
//     }
// }

// impl<'a> TypedefParser<'a> {
//     /// 检查是否到达 Token 列表的末尾
//     fn is_at_end(&self) -> bool {
//         self.current >= self.tokens.len() || self.peek().clone().kind.kind_is(&TokenKind::EOF)
//     }

//     fn eat_zeor_or_multy(&mut self, kind: TokenKind) -> Result<Vec<Token>, ParserError> {
//         let mut eated = Vec::<Token>::new();

//         while !self.is_at_end() {
//             if self.peek().kind.kind_is(&kind) {
//                 let a = self.consume(kind.clone())?;
//                 eated.push(a.clone());
//             } else {
//                 break;
//             }
//         }

//         return Ok(eated);
//     }

//     /// 查看当前 Token
//     fn peek(&self) -> &Token {
//         let re = self.tokens.get(self.current);
//         match re {
//             Some(_x) => _x,
//             None => &self.eof,
//         }
//     }

//     /// 消费一个期望的 Token，如果当前 Token 不匹配则返回错误
//     fn consume(&mut self, kind: TokenKind) -> Result<&Token, ParserError> {
//         if self.check(&kind) {
//             self.current += 1;
//             // println!("消耗掉了一个: {:?}", &self.tokens[self.current - 1]);
//             Ok(&self.tokens[self.current - 1])
//         } else {
//             Err(ParserError::new(
//                 format!("Expected token: {:?}, but found: {:?}", kind, self.peek()),
//                 Some(self.peek().clone()),
//             ))
//         }
//     }

//     /// 检查当前 Token 是否与期望的 Token 匹配
//     fn check(&self, kind: &TokenKind) -> bool {
//         if self.is_at_end() {
//             return false;
//         }
//         self.tokens[self.current].kind.kind_is(kind)
//     }

//     /// 查看还为解析的 Token,
//     /// offset: 偏移量, 0 表示查看当前 Token;
//     fn peek_next(&self, offset: usize) -> &Token {
//         let re = self.tokens.get(self.current + offset);
//         match re {
//             Some(_x) => _x,
//             None => &self.eof,
//         }
//     }

//     /// 语句结尾符
//     fn consume_stmt_end_token(&mut self) -> Result<Token, ParserError> {
//         let tok = self.peek().clone();
//         println!("consume_stmt_end_token: {:?}", tok);
//         match &tok.kind {
//             TokenKind::NewLine => {
//                 self.consume(TokenKind::NewLine)?;
//                 return Ok(tok);
//             }
//             TokenKind::EOF => {
//                 return Ok(tok);
//             }

//             _ => {
//                 return Err(ParserError::new(
//                     format!(
//                         "need: {:?}, but found: {:?}",
//                         TokenKind::NewLine,
//                         self.peek()
//                     ),
//                     Some(self.peek().clone()),
//                 ));
//             }
//         }
//     }
// }

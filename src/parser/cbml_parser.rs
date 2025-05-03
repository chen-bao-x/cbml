use super::{
    ParserError,
    ast::stmt::{CbmlType, Literal, Stmt, StructFieldDefinition, StructTy},
};
use crate::{
    lexer::token::{Token, TokenKind},
    parser::ast::stmt::{EnumFieldDefinition, EnumTy},
};
use std::collections::HashMap;

/// cbml 解析器
pub struct CbmlParser<'a> {
    tokens: &'a [Token],
    current: usize,
    eof: Token,
}

impl<'a> CbmlParser<'a> {
    /// 创建一个新的 Parser 实例，接受一个 Token 列表
    pub fn new(tokens: &'a [Token]) -> Self {
        CbmlParser {
            tokens,
            current: 0,
            eof: Token::new(crate::lexer::token::TokenKind::EOF, 0, 0),
        }
    }

    /// 解析 Token 列表，直到结束并返回 AST
    pub fn parse(&mut self) -> Result<Vec<Stmt>, Vec<ParserError>> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            println!("parse(&mut self) current: {:?}", self.peek());
            _ = self.eat_zeor_or_multy(TokenKind::NewLine);

            let re = self.parse_statement();
            match re {
                Ok(s) => statements.push(s),
                Err(e) => {
                    errors.push(e);
                    self.current += 1; // 移动到下一个 Token
                }
            }
            _ = self.eat_zeor_or_multy(TokenKind::NewLine);
        }

        if errors.is_empty() {
            return Ok(statements);
        } else {
            return Err(errors);
        }
    }

    fn parse_type_sign(&mut self) -> Result<CbmlType, ParserError> {
        println!("parse_type_sign(&mut self)");
        // 解析类型声明
        let tok = self.peek();
        match tok.kind.clone() {
            TokenKind::QuestionMark => {
                // 可选类型
                self.consume(TokenKind::QuestionMark)?;
                let inner_type = self.parse_type_sign()?;
                return Ok(CbmlType::Optional {
                    ty: Box::new(inner_type),
                    default: None,
                });
            }
            TokenKind::Identifier(name) => {
                self.consume(TokenKind::Identifier(name.clone()))?;

                return Ok(CbmlType::Custom {
                    type_name: name.clone(),
                    default: None,
                });
            }
            TokenKind::Any => {
                self.consume(TokenKind::Any)?;
                return Ok(CbmlType::Any { default: None });
            }
            TokenKind::StringTy => {
                self.consume(TokenKind::StringTy)?;
                return Ok(CbmlType::String { default: None });
            }
            TokenKind::NumberTy => {
                self.consume(TokenKind::NumberTy)?;
                return Ok(CbmlType::Number { default: None });
            }
            TokenKind::BooleanTy => {
                self.consume(TokenKind::BooleanTy)?;
                return Ok(CbmlType::Boolean { default: None });
            }
            TokenKind::LBracket => {
                // 数组类型
                self.consume(TokenKind::LBracket)?;

                let inner_type = self.parse_type_sign()?;

                self.consume(TokenKind::RBracket)?;

                return Ok(CbmlType::Array {
                    inner_type: Box::new(inner_type),
                    default: None,
                });
            }
            TokenKind::LBrace => {
                // 解析匿名结构体.

                // 结构体类型
                self.consume(TokenKind::LBrace)?;

                let mut fields: Vec<StructFieldDefinition> = vec![];
                let mut count = 0;

                while !self.is_at_end() {
                    if count > 0 {
                        let k = self.peek().kind.clone();
                        match k {
                            TokenKind::Comma => {
                                self.consume(TokenKind::Comma)?;
                            }
                            TokenKind::NewLine => {
                                self.consume(TokenKind::NewLine)?;
                            }

                            _ => {
                                break;
                            }
                        }
                    }

                    _ = self.eat_zeor_or_multy(TokenKind::NewLine)?;

                    if let TokenKind::RBrace = self.peek().kind {
                        break;
                    }

                    let field = self.parse_struct_field_def()?;
                    fields.push(field);

                    count += 1;
                }

                self.consume(TokenKind::RBrace)?;
                let t = CbmlType::Struct(fields);
                return Ok(t);
            }

            _ => {
                println!("parse_type_sign error: unkonow token {:?}", tok);
                todo!();
                return Err(ParserError::new(
                    format!("parse_type_sign error: unkonow token {:?}", tok),
                    Some(tok.clone()),
                ));
            }
        }
    }

    fn parse_literal(&mut self) -> Result<Literal, ParserError> {
        // 解析字面量
        let tok = self.peek();
        match tok.kind.clone() {
            TokenKind::String(s) => {
                let a = Literal::String(s.clone());
                self.consume(TokenKind::String(s.clone()))?;
                return Ok(a);
            }
            TokenKind::Number(n) => {
                self.consume(TokenKind::Number(n.clone()))?;

                let a = Literal::Number(n.clone());
                return Ok(a);
            }
            TokenKind::True => {
                self.consume(TokenKind::True)?;
                return Ok(Literal::Boolean(true));
            }
            TokenKind::False => {
                self.consume(TokenKind::False)?;
                return Ok(Literal::Boolean(false));
            }
            TokenKind::None => {
                self.consume(TokenKind::None)?;
                return Ok(Literal::None);
            }

            TokenKind::Todo => {
                self.consume(TokenKind::Todo)?;
                return Ok(Literal::Todo);
            }
            TokenKind::Default => {
                self.consume(TokenKind::Default)?;
                return Ok(Literal::Default);
            }
            TokenKind::LBracket => {
                // 数组字面量
                self.consume(TokenKind::LBracket)?;
                self.eat_zeor_or_multy(TokenKind::NewLine)?; // 可有可无的换行符.

                let mut array = Vec::new();

                let mut count = 0;

                while !self.is_at_end() {
                    self.eat_zeor_or_multy(TokenKind::NewLine)?; // 可有可无的换行符.
                    match self.peek().kind {
                        TokenKind::RBracket => {
                            // 数组结束
                            break;
                        }

                        _ => {
                            if count > 0 {
                                // 解析逗号分隔的数组元素
                                self.consume(TokenKind::Comma)?;
                            }
                            self.eat_zeor_or_multy(TokenKind::NewLine)?; // 可有可无的换行符.

                            let value = self.parse_literal()?;
                            array.push(value);

                            count += 1;
                        }
                    }
                }

                self.consume(TokenKind::RBracket)?;
                return Ok(Literal::Array(array));
            }
            TokenKind::LBrace => {
                // 结构体字面量
                self.consume(TokenKind::LBrace)?;

                self.eat_zeor_or_multy(TokenKind::NewLine)?; // 可有可无的换行符.

                let mut fields: HashMap<String, Literal> = HashMap::new();

                let mut count = 0;
                while !self.is_at_end() {
                    // 上一个字段结束
                    if count > 0 {
                        let k = self.peek().kind.clone();
                        if k.kind_is(&TokenKind::Comma) {
                            self.consume(TokenKind::Comma)?;
                        } else if k.kind_is(&TokenKind::NewLine) {
                            self.consume(TokenKind::NewLine)?;
                        }
                    }

                    _ = self.eat_zeor_or_multy(TokenKind::NewLine)?; // 可有可无的换行符.

                    let tok = self.peek();
                    match tok.kind.clone() {
                        TokenKind::RBrace => {
                            // 结构体结束

                            break;
                        }

                        TokenKind::Identifier(_) => {
                            // 解析结构体字段

                            let name_tok = self.consume(TokenKind::Identifier("".into()))?.clone();

                            if let TokenKind::Identifier(name) = name_tok.kind.clone() {
                                self.consume(TokenKind::Asign)?;

                                let value = self.parse_literal()?;

                                let re = fields.insert(name.clone(), value);
                                match re {
                                    Some(_) => {
                                        return Err(ParserError::new(
                                            format!("duplicate field: {:?}", name),
                                            Some(name_tok.clone()),
                                        ));
                                    }
                                    None => {}
                                }
                            }
                        }
                        _ => {
                            println!("parse_literal error: unkonow token {:?}", tok);
                            todo!();
                        }
                    }

                    count += 1;
                }

                self.consume(TokenKind::RBrace)?;

                return Ok(Literal::Struct(fields));
            }
            _ => {
                println!("parse_literal error: unkonow token {:?}", tok);
                todo!();
                return Err(ParserError::new(
                    format!("parse_literal error: unkonow token {:?}", tok),
                    Some(tok.clone()),
                ));
            }
        }
    }

    /// name = "hello"
    fn parse_asignment(&mut self) -> Result<Stmt, ParserError> {
        // 解析赋值语句

        let name_tok = self.consume(TokenKind::Identifier("".into()))?.kind.clone();

        if let TokenKind::Identifier(name) = name_tok {
            println!("parse_asignment(&mut self)");

            self.consume(TokenKind::Asign)?;

            let value = self.parse_literal()?;
            println!("parse_asignment(&mut self) value: {:?}", value);

            self.consume_stmt_end_token()?;

            return Ok(Stmt::Asignment {
                field_name: name,
                value,
            });
        } else {
            println!(
                "parse_asignment error: unkonow token {:?}",
                self.peek().clone()
            );
            todo!();
            return Err(ParserError::new(
                format!("parse_asignment error: unkonow token {:?}", self.peek()),
                Some(self.peek().clone()),
            ));
        };
    }

    fn parse_default_value(&mut self) -> Result<Option<Literal>, ParserError> {
        // 解析默认值
        let tok = self.peek();
        match tok.kind.clone() {
            TokenKind::Default => {
                self.consume(TokenKind::Default)?;

                let value = self.parse_literal()?;
                return Ok(Some(value));
            }
            _ => {
                return Ok(None);
            }
        }
    }
    fn parse_struct_field_def(&mut self) -> Result<StructFieldDefinition, ParserError> {
        // 解析字段定义
        let name_tok = self.consume(TokenKind::Identifier("".into()))?;
        if let TokenKind::Identifier(name) = name_tok.kind.clone() {
            self.consume(TokenKind::Colon)?;

            let field_type = self.parse_type_sign()?;

            // 默认值
            let default_value = self.parse_default_value()?;

            // self.consume_stmt_end_token()?;

            return Ok(StructFieldDefinition {
                field_name: name,
                ty: field_type,
                default: default_value,
            });
        } else {
            println!(
                "parse_field_def error: unkonow token {:?}",
                self.peek().clone()
            );
            todo!();
            return Err(ParserError::new(
                format!("parse_field_def error: unkonow token {:?}", self.peek()),
                Some(self.peek().clone()),
            ));
        }
    }

    /// 解析使用 struct name { } 这种方式定义的结构体.
    fn parse_struct_def(&mut self) -> Result<Stmt, ParserError> {
        // 解析结构体定义

        self.consume(TokenKind::Struct)?;

        if let TokenKind::Identifier(name) =
            self.consume(TokenKind::Identifier("".into()))?.kind.clone()
        {
            self.consume(TokenKind::LBrace)?;

            let mut fields: Vec<StructFieldDefinition> = vec![];
            let mut count = 0;

            while !self.is_at_end() {
                if count > 0 {
                    let k = self.peek().kind.clone();
                    match k {
                        TokenKind::Comma => {
                            self.consume(TokenKind::Comma)?;
                        }
                        TokenKind::NewLine => {
                            self.consume(TokenKind::NewLine)?;
                        }

                        _ => {
                            break;
                        }
                    }
                }

                _ = self.eat_zeor_or_multy(TokenKind::NewLine)?;

                if let TokenKind::RBrace = self.peek().kind {
                    break;
                }

                let field = self.parse_struct_field_def()?;
                fields.push(field);
                count += 1;
            }

            self.consume(TokenKind::RBrace)?;

            return Ok(Stmt::StructDef(StructTy {
                name,
                fields: fields,
            }));
        } else {
            println!(
                "parse_struct_def error: unkonow token {:?}",
                self.peek().clone()
            );
            todo!();
            return Err(ParserError::new(
                format!("parse_struct_def error: unkonow token {:?}", self.peek()),
                Some(self.peek().clone()),
            ));
        };
    }

    fn parse_union_def(&mut self) -> Result<Stmt, ParserError> {
        // 解析联合体定义
        self.consume(TokenKind::Union)?;
        if let TokenKind::LParen = self.peek().kind {
            // 解析联合体的基本类型
            self.consume(TokenKind::LParen)?;
            let base_type = self.parse_literal()?;
            self.consume(TokenKind::RParen)?;
        } else {
            return Err(ParserError {
                message: format!(
                    "parse_statement error: need {:?}, but found token {:?}",
                    TokenKind::LParen,
                    self.peek()
                ),
                token: self.peek().clone().into(),
            });
        }

        if let TokenKind::Identifier(name) =
            self.consume(TokenKind::Identifier("".into()))?.kind.clone()
        {
            self.consume_stmt_end_token()?;
            return Ok(Stmt::UnionDef {
                base_type: todo!(),
                alowd_values: todo!(),
            });
        } else {
            return Err(ParserError {
                message: format!(
                    "parse_statement error: need {:?}, but found token {:?}",
                    TokenKind::Identifier("".into()),
                    self.peek()
                ),
                token: self.peek().clone().into(),
            });
        }
    }

    fn parse_use(&mut self) -> Result<Stmt, ParserError> {
        // 解析 use 语句
        self.consume(TokenKind::Use)?;
        if let TokenKind::String(name) = self.consume(TokenKind::String("".into()))?.kind.clone() {
            self.consume_stmt_end_token()?;
            return Ok(Stmt::Use(name));
        } else {
            return Err(ParserError {
                message: format!(
                    "parse_statement error: need {:?}, but found token {:?}",
                    TokenKind::String("".into()),
                    self.peek()
                ),
                token: self.peek().clone().into(),
            });
        }
    }

    /// 解析单个语句，根据当前 Token 类型决定解析方式
    fn parse_statement(&mut self) -> Result<Stmt, ParserError> {
        use crate::lexer::token::TokenKind::*;
        println!("parse_statement(&mut self)");

        _ = self.eat_zeor_or_multy(NewLine);

        let tok = self.peek();
        match tok.kind {
            TokenKind::Identifier(_) => {
                // 解析赋值语句
                let next_tok = self.peek_next(1);
                if next_tok.kind.kind_is(&TokenKind::Asign) {
                    return self.parse_asignment();
                } else if next_tok.kind.kind_is(&TokenKind::Colon) {
                    let field_def = self.parse_struct_field_def()?;
                    return Ok(Stmt::FieldDef(field_def));
                } else {
                    return Err(ParserError {
                        message: format!(
                            "need {:?} or {:?}, but found {:?}",
                            TokenKind::Asign,
                            TokenKind::Colon,
                            next_tok.kind
                        ),
                        token: next_tok.clone().into(),
                    });
                }
            }
            TokenKind::Use => self.parse_use(),
            TokenKind::Struct => return self.parse_struct_def(),
            TokenKind::Union => self.parse_union_def(),
            TokenKind::Enum => {
                return Ok(self.parse_enum_def().unwrap());
            }
            _ => {
                println!("parse_statement error: unkonow token {:?}", tok);
                todo!();
                return Err(ParserError::new(
                    format!("parse_statement error: unkonow token {:?}", tok),
                    Some(tok.clone()),
                ));
            }
        }
    }

    fn parse_enum_def(&mut self) -> Result<Stmt, ParserError> {
        // enum identifier LBrace newline{0,} enum_field{0,} RBrace
        // enum_field = newline{0,} identifier LParent typedef RParent newline

        println!("parse_enum_def(&mut self)");

        self.consume(TokenKind::Enum)?; // enum

        let enum_name_tok = self.consume(TokenKind::Identifier("".into()))?; // identifier

        if let TokenKind::Identifier(enum_name) = enum_name_tok.kind.clone() {
            self.consume(TokenKind::LBrace)?; // LBrace

            self.eat_zeor_or_multy(TokenKind::NewLine)?; // newline{0,}

            let mut fields: Vec<EnumFieldDefinition> = vec![];

            {
                // enum_field{0,}

                let mut count = 0;
                while !self.is_at_end() {
                    if let TokenKind::RBrace = self.peek().kind.clone() {
                        break;
                    }

                    // let field_name_tok = self.consume(TokenKind::Identifier("".into()))?;

                    let asdf = self.parse_enum_field()?;
                    fields.push(asdf);

                    
                    // let field_name_tok = self.consume(TokenKind::Identifier("".into())).unwrap();
                    // if let TokenKind::Identifier(field_name) = field_name_tok.kind.clone() {
                    //     self.consume(TokenKind::LParen)?;

                    //     let ty = self.parse_type_sign()?;

                    //     self.consume(TokenKind::RParen)?;

                    //     let e = EnumFieldDefinition {
                    //         name: field_name,
                    //         ty,
                    //     };
                    //     println!("{:?}", e);
                    //     fields.push(e);
                    // } else {
                    //     panic!("这是逻辑上不可能出现的错误.")
                    // }
                }
            }

            self.consume(TokenKind::RBrace)?;

            let enum_type = EnumTy {
                name: enum_name,
                fields,
            };
            return Ok(Stmt::EnumDef(enum_type));
        } else {
            panic!("这是逻辑上不可能出现的错误.")
            // return Err(ParserError {
            //     message: format!("这是逻辑上不可能出现的错误"),
            //     token: Some(enum_name_tok.clone()),
            // });
        }
    }

    fn parse_enum_field(&mut self) -> Result<EnumFieldDefinition, ParserError> {
        // enum_field =   identifier LParent typedef RParent newline

        // let field_name_tok = self.consume(TokenKind::Identifier("".into()))?; // identifier
        let field_name_tok = self.consume(TokenKind::Identifier("".into())).unwrap();

        if let TokenKind::Identifier(field_name) = field_name_tok.kind.clone() {
            self.consume(TokenKind::LParen)?; // LParent

            let ty = self.parse_type_sign()?; // typedef

            self.consume(TokenKind::RParen)?;

            self.consume(TokenKind::NewLine)?;

            let e = EnumFieldDefinition {
                name: field_name,
                ty,
            };
            println!("{:?}", e);
            return Ok(e);
        } else {
            panic!("这是逻辑上不可能出现的错误.")
        }
    }
}

impl<'a> CbmlParser<'a> {
    /// 检查是否到达 Token 列表的末尾
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.peek().clone().kind.kind_is(&TokenKind::EOF)
    }

    fn eat_zeor_or_multy(&mut self, kind: TokenKind) -> Result<Vec<Token>, ParserError> {
        let mut eated = Vec::<Token>::new();

        while !self.is_at_end() {
            if self.peek().kind.kind_is(&kind) {
                let a = self.consume(kind.clone())?;
                eated.push(a.clone());
            } else {
                break;
            }
        }

        return Ok(eated);
    }

    /// 查看当前 Token
    fn peek(&self) -> &Token {
        let re = self.tokens.get(self.current);
        match re {
            Some(_x) => _x,
            None => &self.eof,
        }
    }

    /// 消费一个期望的 Token，如果当前 Token 不匹配则返回错误
    fn consume(&mut self, kind: TokenKind) -> Result<&Token, ParserError> {
        if self.check(&kind) {
            self.current += 1;
            // println!("消耗掉了一个: {:?}", &self.tokens[self.current - 1]);
            Ok(&self.tokens[self.current - 1])
        } else {
            Err(ParserError::new(
                format!("Expected token: {:?}, but found: {:?}", kind, self.peek()),
                Some(self.peek().clone()),
            ))
        }
    }

    /// 检查当前 Token 是否与期望的 Token 匹配
    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.tokens[self.current].kind.kind_is(kind)
    }

    /// 查看还为解析的 Token,
    /// offset: 偏移量, 0 表示查看当前 Token;
    fn peek_next(&self, offset: usize) -> &Token {
        let re = self.tokens.get(self.current + offset);
        match re {
            Some(_x) => _x,
            None => &self.eof,
        }
    }

    /// 语句结尾符
    fn consume_stmt_end_token(&mut self) -> Result<Token, ParserError> {
        let tok = self.peek().clone();
        println!("consume_stmt_end_token: {:?}", tok);
        match &tok.kind {
            TokenKind::NewLine => {
                self.consume(TokenKind::NewLine)?;
                return Ok(tok);
            }
            TokenKind::EOF => {
                return Ok(tok);
            }

            _ => {
                return Err(ParserError::new(
                    format!(
                        "need: {:?}, but found: {:?}",
                        TokenKind::NewLine,
                        self.peek()
                    ),
                    Some(self.peek().clone()),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::TokenKind;
    use crate::lexer::tokenizer;

    #[test]
    fn test_parser() {
        // let code = std::fs::read_to_string("/Users/chenbao/Documents/GitHub/cbml/examples/1.cmml").unwrap();
        let code =
            std::fs::read_to_string("/Users/chenbao/Documents/GitHub/cbml/examples/1.typedef.cbml")
                .unwrap();
        // let code = CODE;

        let tokens = tokenizer(&code).unwrap();
        println!("tokens: {:?}", tokens);
        let mut parser = CbmlParser::new(&tokens);
        let re = parser.parse();
        match re {
            Ok(statements) => {
                statements.iter().for_each(|s| {
                    println!("statement: {:?}", s);
                });
            }
            Err(e) => {
                e.iter().for_each(|s| {
                    println!("message: {:?}", s.message);
                    println!("tok: {:?}", s.token);
                });
            }
        }
    }
}

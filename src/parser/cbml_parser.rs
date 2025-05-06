use super::{
    ParserError,
    ast::stmt::{
        AsignmentStmt, CbmlType, LiteralKind, LiteralWithSpan, StmtKind, StructDef,
        StructFieldDefStmt, UnionDef, UseStmt,
    },
};
use crate::{
    dp,
    lexer::token::{Position, Span, TokenKind as tk},
};
use crate::{
    lexer::token::Token,
    parser::ast::stmt::{EnumDef, EnumField},
};

/// cbml 解析器
pub struct CbmlParser<'a> {
    file_path: String,
    tokens: &'a [Token],
    current_position: usize,
    eof: Token,
}

impl<'a> CbmlParser<'a> {
    /// 创建一个新的 Parser 实例，接受一个 Token 列表
    pub fn new(file_path: String, tokens: &'a [Token]) -> Self {
        CbmlParser {
            tokens,
            current_position: 0,

            eof: Token::new(
                tk::EOF,
                Span {
                    start: Position::new(0, 0, 0),
                    end: Position::new(0, 0, 0),
                },
            ),
            file_path: file_path,
        }
    }

    /// 解析 Token 列表，直到结束并返回 AST
    pub fn parse(&mut self) -> Result<Vec<StmtKind>, Vec<ParserError>> {
        let mut statements = Vec::new();
        let errors = Vec::new();

        while !self.is_at_end() {
            // dp(format!("parse(&mut self) current: {:?}", self.peek()));

            _ = self.eat_zeor_or_multy(tk::NewLine);

            let re = self.parse_statement();
            match re {
                Ok(s) => statements.push(s),
                Err(e) => {
                    // errors.push(e);
                    // self.current_position += 1; // 移动到下一个 Token

                    println!("{:?}", e);

                    panic!();
                }
            }
            _ = self.eat_zeor_or_multy(tk::NewLine);
        }

        if errors.is_empty() {
            return Ok(statements);
        } else {
            return Err(errors);
        }
    }

    /// 解析单个语句，根据当前 Token 类型决定解析方式
    fn parse_statement(&mut self) -> Result<StmtKind, ParserError> {
        // dp(format!("parse_statement(&mut self)"));

        _ = self.eat_zeor_or_multy(tk::NewLine);

        let tok = self.peek().kind.clone();
        match tok {
            tk::Identifier(_) => {
                let next_tok = self.peek_next(1);
                if next_tok.kind.kind_is(&tk::Asign) {
                    // 解析赋值语句

                    return self.parse_asignment();
                } else if next_tok.kind.kind_is(&tk::Colon) {
                    // 解析结构体成员定义.

                    let field_def = self.parse_struct_field_def()?;
                    return Ok(StmtKind::FileFieldStmt(field_def));
                } else if next_tok.kind.kind_is(&tk::LParen) {
                    // 解析 enum literal

                    let sdaf = self.parse_asignment()?;
                    return Ok(sdaf);
                } else {
                    return Err(ParserError::new(
                        self.file_path.clone(),
                        format!(
                            "need {:?} or {:?}, but found {:?}",
                            tk::Asign,
                            tk::Colon,
                            next_tok.kind
                        ),
                        next_tok.span.clone(),
                    ));
                }
            }
            tk::Use => self.parse_use(),
            tk::Struct => return self.parse_struct_def(),
            tk::Union => self.parse_union_def(),
            tk::Enum => self.parse_enum_def(),
            tk::LineComment(s) => {
                self.consume(tk::LineComment("".into()))?;
                return Ok(StmtKind::LineComment(s));
            }
            tk::BlockComment(s) => {
                self.consume(tk::BlockComment("".into()))?;
                Ok(StmtKind::BlockComment(s))
            }
            tk::DocComment(s) => {
                self.consume(tk::DocComment("".into()))?;
                Ok(StmtKind::DocComment(s))
            }
            _ => {
                // dp(format!("parse_statement error: unkonow token {:?}", tok));
                // todo!();
                return Err(ParserError::new(
                    self.file_path.clone(),
                    format!("parse_statement error: unkonow token {:?}", tok),
                    self.peek().span.clone(),
                ));
            }
        }
    }

    /// 类型标注
    fn parse_type_sign(&mut self) -> Result<CbmlType, ParserError> {
        // any | string | number | bool | identifier | Anonymous_optinal  | Anonymous_array | Anonymous_struct | Anonymous_union

        // dp(format!("parse_type_sign(&mut self)"));

        // 解析类型声明
        let tok = self.peek();
        match tok.kind.clone() {
            tk::Any => {
                self.consume(tk::Any)?;
                return Ok(CbmlType::Any);
            }
            tk::StringTy => {
                self.consume(tk::StringTy)?;
                return Ok(CbmlType::String);
            }
            tk::NumberTy => {
                self.consume(tk::NumberTy)?;
                return Ok(CbmlType::Number);
            }
            tk::BooleanTy => {
                self.consume(tk::BooleanTy)?;
                return Ok(CbmlType::Boolean);
            }
            tk::Identifier(name) => {
                self.consume(tk::Identifier("".into()))?;

                return Ok(CbmlType::Custom(name));
            }
            tk::QuestionMark => {
                // 可选类型
                self.consume(tk::QuestionMark)?;
                let inner_type = self.parse_type_sign()?;
                return Ok(CbmlType::Optional {
                    inner_type: Box::new(inner_type),
                });
            }
            tk::LBracket => {
                // 匿名数组
                self.consume(tk::LBracket)?;

                let inner_type = self.parse_type_sign()?;

                self.consume(tk::RBracket)?;

                return Ok(CbmlType::Array {
                    inner_type: Box::new(inner_type),
                });
            }
            tk::LBrace => {
                // 解析匿名结构体.

                // 结构体类型
                self.consume(tk::LBrace)?;

                let mut fields: Vec<StructFieldDefStmt> = vec![];
                let mut count = 0;

                while !self.is_at_end() {
                    if count > 0 {
                        let k = self.peek().kind.clone();
                        match k {
                            tk::Comma => {
                                self.consume(tk::Comma)?;
                            }
                            tk::NewLine => {
                                self.consume(tk::NewLine)?;
                            }

                            _ => {
                                break;
                            }
                        }
                    }

                    _ = self.eat_zeor_or_multy(tk::NewLine)?;

                    if let tk::RBrace = self.peek().kind {
                        break;
                    }

                    let field = self.parse_struct_field_def()?;
                    fields.push(field);

                    count += 1;
                }

                self.consume(tk::RBrace)?;
                let t = CbmlType::Struct(fields);
                return Ok(t);
            }

            x => {
                match x {
                    tk::Pipe | tk::Number(_) | tk::String(_) => {
                        // 解析匿名 union.
                        let alowd_values = self.parse_union_fields()?;

                        let base_type: CbmlType = LiteralKind::union_base_type(&alowd_values);
                        return Ok(CbmlType::Union {
                            base_type: Box::new(base_type),
                            alowd_values,
                        });
                    }

                    _ => {}
                };

                match self.peek_next(1).kind.clone() {
                    tk::Pipe => {
                        // 解析匿名 union.
                        let alowd_values = self.parse_union_fields()?;

                        let base_type: CbmlType = LiteralKind::union_base_type(&alowd_values);
                        return Ok(CbmlType::Union {
                            base_type: Box::new(base_type),
                            alowd_values,
                        });
                    }
                    _ => {}
                };

                #[cfg(debug_assertions)]
                {
                    dp(format!("parse_type_sign error: unkonow token {:?}", tok));
                    todo!();
                }

                return Err(ParserError::new(
                    self.file_path.clone(),
                    format!("parse_type_sign error: unkonow token {:?}", tok),
                    tok.span.clone(),
                ));
            }
        }
    }

    fn parse_array_literal(&mut self) -> Result<Vec<LiteralKind>, ParserError> {
        // array_literal = LBracket elements Coma{0,1} RBracket
        // elements = Newline{0,} first_element tail_elements{0,}
        // first_element = Newline{0,} literal
        // tail_elements = splitor literal
        // splitor = Newline{0,} Coma Newline{0,}
        // LBracket = "["
        // RBracket = "]"
        // Coma = ","

        self.consume(tk::LBracket)?; // LBracket

        let mut elements: Vec<LiteralKind> = Vec::<LiteralKind>::new();

        if let tk::RBracket = self.peek().kind {
            //  空数组  [ ]

            self.consume(tk::RBracket)?;
            return Ok(elements);
        }

        {
            enum State {
                NeedLiteral,
                NeedComa,
            }
            let mut s: State = State::NeedLiteral;
            while !self.is_at_end() {
                self.eat_zeor_or_multy(tk::NewLine)?;
                if let tk::RBracket = self.peek().kind {
                    break; // array literal ends.
                }

                match s {
                    State::NeedLiteral => {
                        let literal = self.parse_literal()?;
                        elements.push(literal.kind);

                        s = State::NeedComa;
                    }
                    State::NeedComa => {
                        self.consume(tk::Comma)?;
                        s = State::NeedLiteral;
                    }
                }
            }
        }

        self.eat_zeor_or_multy(tk::NewLine)?; // NewLine{0,}
        _ = self.consume(tk::Comma); // Coma{0,1}
        self.eat_zeor_or_multy(tk::NewLine)?; // NewLine{0,}
        self.consume(tk::RBracket)?; // RBracket

        return Ok(elements);
    }
 
    fn parse_literal(&mut self) -> Result<LiteralWithSpan, ParserError> {
        // 解析字面量
        let tok = self.peek().clone();
        match tok.kind {
            tk::String(s) => {
                let a = LiteralKind::String(s);
                self.consume(tk::String("".into()))?;

                return Ok(LiteralWithSpan {
                    kind: a,
                    span: tok.span,
                });
            }
            tk::Number(n) => {
                self.consume(tk::Number(n.clone()))?;

                let a = LiteralKind::Number(n.clone());
             
                return Ok(LiteralWithSpan {
                    kind: a,
                    span: tok.span,
                });
            }
            tk::True => {
                self.consume(tk::True)?;
          

                return Ok(LiteralWithSpan {
                    kind: LiteralKind::Boolean(true),
                    span: tok.span,
                });
            }
            tk::False => {
                self.consume(tk::False)?;
                

                return Ok(LiteralWithSpan {
                    kind: LiteralKind::Boolean(false),
                    span: tok.span,
                });
            }
            tk::TkNone => {
                self.consume(tk::TkNone)?;

                return Ok(LiteralWithSpan {
                    kind: LiteralKind::LiteralNone,
                    span: tok.span,
                });
            }

            tk::Todo => {
                self.consume(tk::Todo)?;

                return Ok(LiteralWithSpan {
                    kind: LiteralKind::Todo,
                    span: tok.span,
                });
            }
            tk::Default => {
                self.consume(tk::Default)?;

                return Ok(LiteralWithSpan {
                    kind: LiteralKind::Default,
                    span: tok.span,
                });
            }
            tk::LBracket => {
                // LBracket literal element{0,} coma{0,1} RBracket
                // element = Coma literal
                // 数组字面量

                let arr = self.parse_array_literal()?;
                return Ok(LiteralWithSpan {
                    kind: LiteralKind::Array(arr),
                    span: tok.span,
                });
                // return Ok(LiteralKind::Array(arr));
            }
            tk::LBrace => {
                // 结构体字面量
                let lbrace = self.consume(tk::LBrace)?.clone();

                self.eat_zeor_or_multy(tk::NewLine)?; // 可有可无的换行符.

                let mut fields: Vec<AsignmentStmt> = vec![];

                let mut count = 0;
                while !self.is_at_end() {
                    // 上一个字段结束
                    if count > 0 {
                        let k = self.peek().kind.clone();
                        if k.kind_is(&tk::Comma) {
                            self.consume(tk::Comma)?;
                        } else if k.kind_is(&tk::NewLine) {
                            self.consume(tk::NewLine)?;
                        }
                    }

                    _ = self.eat_zeor_or_multy(tk::NewLine)?; // 可有可无的换行符.

                    let tok = self.peek();
                    match tok.kind.clone() {
                        tk::RBrace => {
                            // 结构体结束

                            break;
                        }

                        tk::Identifier(_) => {
                            // 解析结构体字段

                            let name_tok = self.consume(tk::Identifier("".into()))?.clone();

                            if let tk::Identifier(name) = name_tok.kind.clone() {
                                self.consume(tk::Asign)?;

                                let value = self.parse_literal()?;
                                let arr = self.parse_array_literal()?;
                        

                                fields.push(AsignmentStmt {
                                    field_name: name.clone(),
                                    value,
                                    field_name_span: name_tok.span,
                                });
                             
                            }
                        }
                        _ => {
                            dp(format!("parse_literal error: unkonow token {:?}", tok));
                            todo!();
                        }
                    }

                    count += 1;
                }

                let rbrace = self.consume(tk::RBrace)?;

                return Ok(LiteralWithSpan {
                    kind: LiteralKind::Struct(fields),
                    span: Span {
                        start: lbrace.span.start.clone(),
                        end: rbrace.span.end.clone(),
                    },
                });
            }

            tk::Identifier(_name) => {
                let next = self.peek_next(1).kind.clone();
                match next {
                    tk::LParen => {
                        // 解析 enum literal

                        let enum_literal = self.parse_enum_literal()?;

                        return Ok(LiteralWithSpan {
                            kind: enum_literal,
                            span: self.peek().span.clone(),
                        });
                    }
                    _ => {
                        println!("{:?}", self.peek());
                        todo!();
                    }
                }
            }
            _ => {
                dp(format!("parse_literal error: unkonow token {:?}", tok));
                todo!();

                #[allow(unreachable_code)]
                return Err(ParserError::new(
                    self.file_path.clone(),
                    format!("parse_literal error: unkonow token {:?}", tok),
                    tok.span,
                ));
            }
        }
    }

    /// name = "hello"
    fn parse_asignment(&mut self) -> Result<StmtKind, ParserError> {
        // identifier asignment literal
        // 解析赋值语句

        let name_tok = self.consume(tk::Identifier("".into()))?.clone(); // identifier

        if let tk::Identifier(name) = name_tok.kind {
            // dp(format!("parse_asignment(&mut self)"));

            self.consume(tk::Asign)?; // asignment

            let value = self.parse_literal()?; // literal

            // dp(format!("parse_asignment(&mut self) value: {:?}", value));

            self.consume_stmt_end_token()?;

            return Ok(StmtKind::Asignment(AsignmentStmt {
                field_name: name,
                value,
                field_name_span: name_tok.span,
            }));
        } else {
            dp(format!(
                "parse_asignment error: unkonow token {:?}",
                self.peek().clone()
            ));
             
            return Err(ParserError::new(
                self.file_path.clone(),
                format!("parse_asignment error: unkonow token {:?}", self.peek()),
                self.peek().span.clone(),
            ));
        };
    }

    fn parse_default_value(&mut self) -> Result<Option<LiteralKind>, ParserError> {
        // 解析默认值
        let tok = self.peek();
        match tok.kind.clone() {
            tk::Default => {
                self.consume(tk::Default)?;

                let value = self.parse_literal()?;

                return Ok(Some(value.kind));
            }
            _ => {
                return Ok(None);
            }
        }
    }

    /// name : string
    fn parse_struct_field_def(&mut self) -> Result<StructFieldDefStmt, ParserError> {
        // struct_field_def = identifier Colon type_sign default_value{0,1}
        // default_value = default literal

        // 解析字段定义
        let name_tok = self.consume(tk::Identifier("".into()))?.clone();
        if let tk::Identifier(name) = name_tok.kind.clone() {
            self.consume(tk::Colon)?;

            let field_type = self.parse_type_sign()?;

            // 默认值
            let default_value = self.parse_default_value()?;

            // self.consume_stmt_end_token()?;

            return Ok(StructFieldDefStmt {
                field_name: name,
                _type: field_type,
                default: default_value,
                field_name_span: name_tok.span,
            });
        } else {
            dp(format!(
                "parse_field_def error: unkonow token {:?}",
                self.peek().clone()
            ));
            // todo!();
            return Err(ParserError::new(
                self.file_path.clone(),
                format!("parse_field_def error: unkonow token {:?}", self.peek()),
                self.peek().span.clone(),
            ));
        }
    }

    /// 解析使用 struct name { } 这种方式定义的结构体.
    fn parse_struct_def(&mut self) -> Result<StmtKind, ParserError> {
        // 解析结构体定义

        self.consume(tk::Struct)?;

        let name_tok = self.consume(tk::Identifier("".into()))?.clone();

        if let tk::Identifier(name) = name_tok.kind.clone() {
            self.consume(tk::LBrace)?;

            let mut fields: Vec<StructFieldDefStmt> = vec![];
            let mut count = 0;

            while !self.is_at_end() {
                if count > 0 {
                    let k = self.peek().kind.clone();
                    match k {
                        tk::Comma => {
                            self.consume(tk::Comma)?;
                        }
                        tk::NewLine => {
                            self.consume(tk::NewLine)?;
                        }

                        _ => {
                            break;
                        }
                    }
                }

                _ = self.eat_zeor_or_multy(tk::NewLine)?;

                if let tk::RBrace = self.peek().kind {
                    break;
                }

                let field = self.parse_struct_field_def()?;
                fields.push(field);
                count += 1;
            }

            self.consume(tk::RBrace)?;

            return Ok(StmtKind::StructDefStmt(StructDef {
                struct_name: name,
                fields,
                name_span: name_tok.span,
            }));
        } else {
            dp(format!(
                "parse_struct_def error: unkonow token {:?}",
                self.peek().clone()
            ));
            todo!();
            // return Err(ParserError::new(
            //     format!("parse_struct_def error: unkonow token {:?}", self.peek()),
            //     Some(self.peek().clone()),
            // ));
        };
    }

    fn parse_union_fields(&mut self) -> Result<Vec<LiteralKind>, ParserError> {
        let mut literals: Vec<LiteralKind> = vec![];

        let mut count = 0;

        loop {
            // fields = first{0,1} union_field{1,}
            // first = pipe{0,1} literal
            // union_field = pipe literal

            if count == 0 {
                // first = pipe{0,1} literal

                _ = self.consume(tk::Pipe); // pipe{0,1} 第一个 pipe 符号可有可无.

                let literal = self.parse_literal()?; // literal
                literals.push(literal.kind);
            } else {
                // union_field = pipe literal

                _ = self.eat_zeor_or_multy(tk::NewLine)?; // NewLine{0,}

                if self.peek().kind.clone().kind_is(&tk::Pipe) {
                    self.consume(tk::Pipe)?; // pipe
                    let literal = self.parse_literal()?; // literal
                    literals.push(literal.kind);
                } else {
                    break;
                }
            }

            count += 1;
        }

        return Ok(literals);
    }

    fn parse_use(&mut self) -> Result<StmtKind, ParserError> {
        // 解析 use 语句
        let _use_span = self.consume(tk::Use)?.span.clone();
        let url_tok = self.consume(tk::String("".into()))?.clone();

        if let tk::String(url) = url_tok.kind {
            self.consume_stmt_end_token()?;
            return Ok(StmtKind::Use(UseStmt {
                url: url,
                keyword_span: _use_span,
                url_span: url_tok.span,
            }));
        } else {
            return Err(ParserError::new(
                self.file_path.clone(),
                format!(
                    "parse_statement error: need {:?}, but found token {:?}",
                    tk::String("".into()),
                    self.peek()
                ),
                self.peek().span.clone(),
            ));
        }
    }

    fn parse_enum_def(&mut self) -> Result<StmtKind, ParserError> {
        // enum identifier LBrace newline{0,} enum_field{0,} RBrace
        // enum_field = newline{0,} identifier LParent typedef RParent newline

        // dp(format!("parse_enum_def(&mut self)"));

        self.consume(tk::Enum)?; // enum

        let enum_name_tok = self.consume(tk::Identifier("".into()))?; // identifier

        if let tk::Identifier(enum_name) = enum_name_tok.kind.clone() {
            self.consume(tk::LBrace)?; // LBrace

            self.eat_zeor_or_multy(tk::NewLine)?; // newline{0,}

            let mut fields: Vec<EnumField> = vec![];

            {
                // enum_field{0,}

                while !self.is_at_end() {
                    if let tk::RBrace = self.peek().kind.clone() {
                        break;
                    }

                    // let field_name_tok = self.consume(tk::Identifier("".into()))?;

                    let asdf = self.parse_enum_field()?;
                    fields.push(asdf);

                    // let field_name_tok = self.consume(tk::Identifier("".into())).unwrap();
                    // if let tk::Identifier(field_name) = field_name_tok.kind.clone() {
                    //     self.consume(tk::LParen)?;

                    //     let ty = self.parse_type_sign()?;

                    //     self.consume(tk::RParen)?;

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

            self.consume(tk::RBrace)?;

            let enum_type = EnumDef { enum_name, fields };
            return Ok(StmtKind::EnumDef(enum_type));
        } else {
            panic!("这是逻辑上不可能出现的错误.")
            // return Err(ParserError {
            //     message: format!("这是逻辑上不可能出现的错误"),
            //     token: Some(enum_name_tok.clone()),
            // });
        }
    }

    fn parse_union_def(&mut self) -> Result<StmtKind, ParserError> {
        // union LParent typesign RParent identifier Assignment union_field{1,}
        // union_field = pipe{1} literal

        // 解析联合体定义
        self.consume(tk::Union)?; // union

        // typesign
        let base_type: CbmlType = if let tk::LParen = self.peek().kind {
            // 解析联合体的基本类型
            self.consume(tk::LParen)?; // LParent
            let base_type = self.parse_type_sign()?; // typesign
            self.consume(tk::RParen)?; // RParent

            base_type
        } else {
            return Err(ParserError::new(
                self.file_path.clone(),
                format!(
                    "parse_statement error: need {:?}, but found token {:?}",
                    tk::LParen,
                    self.peek()
                ),
                self.peek().span.clone(),
            ));
        };

        // identifier
        let union_name: String = if let tk::Identifier(union_name) =
            self.consume(tk::Identifier("".into()))?.kind.clone()
        {
            union_name
        } else {
            return Err(ParserError::new(
                self.file_path.clone(),
                format!(
                    "parse_statement error: need {:?}, but found token {:?}",
                    tk::Identifier("".into()),
                    self.peek()
                ),
                self.peek().span.clone(),
            ));
        };

        self.consume(tk::Asign)?; // Assignment

        let alowd_values = self.parse_union_fields()?; // union_field{1,}

        // self.consume_stmt_end_token()?;

        return Ok(StmtKind::UnionDef(UnionDef {
            union_name,
            base_type,
            allowed_values: alowd_values,
        }));
    }

    fn parse_enum_field(&mut self) -> Result<EnumField, ParserError> {
        // enum_field =   identifier LParent typedef RParent newline

        // let field_name_tok = self.consume(tk::Identifier("".into()))?; // identifier
        let field_name_tok = self.consume(tk::Identifier("".into()))?.clone();

        if let tk::Identifier(field_name) = field_name_tok.kind.clone() {
            self.consume(tk::LParen)?; // LParent

            let ty = self.parse_type_sign()?; // typedef

            self.consume(tk::RParen)?;

            self.consume(tk::NewLine)?;

            let field = EnumField {
                field_name,
                _type: ty,
                field_name_span: field_name_tok.span,
            };

            // dp(format!("{:?}", field));

            return Ok(field);
        } else {
            panic!("这是逻辑上不可能出现的错误.")
        }
    }

    fn parse_enum_literal(&mut self) -> Result<LiteralKind, ParserError> {
        // enum_literal = identifier LParent literal RParent

        // LParent
        if let tk::Identifier(name) = self.consume(tk::Identifier("".into()))?.kind.clone() {
            self.consume(tk::LParen)?; // LParent

            let lit = self.parse_literal()?; // literal

            self.consume(tk::RParen)?; // RParent

            return Ok(LiteralKind::EnumFieldLiteral {
                field_name: name,
                literal: lit.kind.into(),
            });
        } else {
            panic!("这是逻辑上不可能出现的错误.");
        }
    }
}

impl<'a> CbmlParser<'a> {
    /// 检查是否到达 Token 列表的末尾
    fn is_at_end(&self) -> bool {
        self.current_position >= self.tokens.len() || self.peek().clone().kind.kind_is(&tk::EOF)
    }

    fn eat_zeor_or_multy(&mut self, kind: tk) -> Result<Vec<Token>, ParserError> {
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
        let re = self.tokens.get(self.current_position);
        match re {
            Some(_x) => _x,
            None => &self.eof,
        }
    }

    /// 消费一个期望的 Token，如果当前 Token 不匹配则返回错误
    fn consume(&mut self, kind: tk) -> Result<&Token, ParserError> {
        if self.check(&kind) {
            let tok = &self.tokens[self.current_position];

            self.current_position += 1;

            return Ok(tok);
        } else {
            Err(ParserError::new(
                self.file_path.clone(),
                format!(
                    "Expected token: TokenKind::{:?}, but found: TokenKind::{:?}",
                    kind,
                    self.peek().kind
                ),
                self.peek().span.clone(),
            ))
        }
    }

    // fn one_of(&mut self, kinds: &[tk]) -> Result<&Token, ParserError> {
    //     for kind in kinds {
    //         if self.check(kind) {
    //             return self.consume(kind.clone());
    //         }
    //     }

    //     Err(ParserError::new(
    //         format!("Expected one of {:?}, but found: {:?}", kinds, self.peek()),
    //         Some(self.peek().clone()),
    //     ))
    // }

    /// 检查当前 Token 是否与期望的 Token 匹配
    fn check(&self, kind: &tk) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.tokens[self.current_position].kind.kind_is(kind)
    }

    /// 查看还为解析的 Token,
    /// offset: 偏移量, 0 表示查看当前 Token;
    fn peek_next(&self, offset: usize) -> &Token {
        let re = self.tokens.get(self.current_position + offset);
        match re {
            Some(_x) => _x,
            None => &self.eof,
        }
    }

    /// 语句结尾符
    fn consume_stmt_end_token(&mut self) -> Result<Token, ParserError> {
        let tok = self.peek().clone();
        // dp(format!("consume_stmt_end_token: {:?}", tok));
        match &tok.kind {
            tk::NewLine => {
                self.consume(tk::NewLine)?;
                return Ok(tok);
            }
            tk::EOF => {
                return Ok(tok);
            }

            _ => {
                return Err(ParserError::new(
                    self.file_path.clone(),
                    format!("need: {:?}, but found: {:?}", tk::NewLine, self.peek()),
                    self.peek().span.clone(),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::lexer::tokenizer;

    #[test]
    fn test_parser() {
        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/1.cmml");
        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/1.typedef.cbml");

        asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/2_arr.cbml");
    }

    fn asdfasdfsdf(path: &str) {
        use std::fs::read_to_string;
        let code = read_to_string(path).unwrap();

        let tokens = tokenizer(path, &code).unwrap();

        // dp(format!("tokens: {:?}", tokens));

        let mut parser = CbmlParser::new(path.to_string(), &tokens);
        let re = parser.parse();
        match re {
            Ok(statements) => {
                statements.iter().for_each(|s| {
                    dp(format!("statement: {:?}", s));
                });
            }
            Err(e) => {
                e.iter().for_each(|s| {
                    dp(format!("message: {:?}", s));
                });
            }
        }
    }
}

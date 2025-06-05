use super::CbmlError;
use super::ast::stmt::*;


use crate::{dp, ToCbmlValue};
use crate::lexer::token::TokenKind as tk;
use crate::lexer::token::*;

#[derive(Debug, Clone, Eq, PartialOrd, Ord, PartialEq, Hash, Copy)]
pub struct NodeId {
    id: u64,
}

impl NodeId {
    pub fn to_u64(&self) -> u64 {
        self.id
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// cbml 解析器
pub struct CbmlParser<'a> {
    file_path: String,
    tokens: &'a [Token],
    current_position: usize,

    /// end of file
    eof: Token,

    /// 临时缓存的自增 id,
    /// 如果要生成 node_id, 请使用 self.gen_node_id()
    node_id: NodeId,
}

pub struct ParserResult {
    pub ast: Vec<Stmt>,
    pub errors: Vec<CbmlError>,
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
                TokenID(0),
            ),
            file_path,
            node_id: NodeId { id: 0 },
            // ast: vec![],
        }
    }

    /// 解析 Token 列表，直到结束并返回 AST
    pub fn parse(&mut self) -> ParserResult {
        let mut mark_pos = self.current_position; // 标记当前的位置.
        _ = mark_pos;

        let mut re: ParserResult = ParserResult {
            ast: Vec::new(),
            errors: Vec::new(),
        };

        while !self.is_at_end() {
            _ = self.eat_zeor_or_multy(tk::NewLine);

            mark_pos = self.current_position;
            let parse_result = self.parse_statement();

            match parse_result {
                Ok(s) => {
                    re.ast.push(s);
                    if self.current_position == mark_pos {
                        panic!("解析出了一个 stmt 却没有消耗任何 token.\n这个错误会导致无限循环...")
                    }
                }
                Err(e) => {
                    re.errors.push(e);

                    self.current_position += 1; // 移动到下一个 Token
                }
            }
            _ = self.eat_zeor_or_multy(tk::NewLine);
        }

        return re;
    }
}

impl<'a> CbmlParser<'a> {
    /// 解析单个语句，根据当前 Token 类型决定解析方式
    fn parse_statement(&mut self) -> Result<Stmt, CbmlError> {
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
                    let stmt = Stmt {
                        span: Span {
                            start: field_def.field_name_span.start.clone(),
                            end: field_def.end_span().end,
                        },
                        kind: StmtKind::FileFieldStmt(field_def),
                        node_id: self.gen_node_id(),
                    };

                    return Ok(stmt);
                } else if next_tok.kind.kind_is(&tk::LParen) {
                    // 解析 enum literal

                    let sdaf = self.parse_asignment()?;
                    return Ok(sdaf);
                } else {
                    return Err(CbmlError::new(
                        self.file_path.clone(),
                        format!(
                            "need {:?} or {:?}, but found {:?}",
                            tk::Asign.to_cbml_code(),
                            tk::Colon.to_cbml_code(),
                            next_tok.kind.to_cbml_code()
                        ),
                        next_tok.span.clone(),
                    ));
                }
            }
            tk::Use => self.parse_use(),
            // tk::Struct => return self.parse_struct_def(),
            // tk::Union => self.parse_union_def(),
            // tk::Enum => self.parse_enum_def(),
            tk::LineComment(_) => self.parse_line_comment(),
            tk::BlockComment(_) => self.parse_block_comment(),
            tk::DocComment(_) => self.parse_top_field_def(),

            x => {
                let skiped_tok = self.consume(x)?.clone(); // 跳过的 token.

                return Err(CbmlError::new(
                    self.file_path.clone(),
                    format!(
                        "parse_statement error: unkonow token {:?}",
                        skiped_tok.kind.to_cbml_code()
                    ),
                    self.peek().span.clone(),
                ));
            }
        }
    }

    fn parse_line_comment(&mut self) -> Result<Stmt, CbmlError> {
        let l = self.consume(tk::LineComment("".into()))?.clone();

        let TokenKind::LineComment(s) = l.kind else {
            return Err(CbmlError::new(
                self.file_path.clone(),
                "parse_line_comment error".into(),
                l.span,
            ));
        };

        let stmt = Stmt {
            span: l.span,
            kind: StmtKind::LineComment(s),
            node_id: self.gen_node_id(),
        };
        return Ok(stmt);
    }

    fn parse_block_comment(&mut self) -> Result<Stmt, CbmlError> {
        let l = self.consume(tk::BlockComment("".into()))?.clone();

        let TokenKind::BlockComment(s) = l.kind else {
            return Err(CbmlError::new(
                self.file_path.clone(),
                "parse_block_comment error".into(),
                l.span,
            ));
        };

        let stmt = Stmt {
            span: l.span,
            kind: StmtKind::BlockComment(s),
            node_id: self.gen_node_id(),
        };
        return Ok(stmt);
    }

    /// 类型标注
    fn parse_type_sign(&mut self) -> Result<TypeSignStmt, CbmlError> {
        // any | string | number | bool | identifier | Anonymous_optinal  | Anonymous_array | Anonymous_struct | Anonymous_union

        // 解析类型声明
        let tok = self.peek().clone();
        match tok.kind.clone() {
            tk::Any => {
                let _tok = self.consume(tk::Any)?;

                let type_sign = TypeSignStmt {
                    kind: TypeSignStmtKind::Any,
                    span: _tok.span.clone(),
                    node_id: self.gen_node_id(),
                };

                return Ok(type_sign);
            }
            tk::StringTy => {
                let _tok = self.consume(tk::StringTy)?;

                let type_sign = TypeSignStmt {
                    kind: TypeSignStmtKind::String,
                    span: _tok.span.clone(),
                    node_id: self.gen_node_id(),
                };
                return Ok(type_sign);
            }
            tk::NumberTy => {
                let numberty_tok = self.consume(tk::NumberTy)?;

                let type_sign = TypeSignStmt {
                    kind: TypeSignStmtKind::Number,
                    span: numberty_tok.span.clone(),
                    node_id: self.gen_node_id(),
                };
                return Ok(type_sign);
            }
            tk::BooleanTy => {
                let boolty_tok = self.consume(tk::BooleanTy)?;

                let type_sign = TypeSignStmt {
                    kind: TypeSignStmtKind::Boolean,
                    span: boolty_tok.span.clone(),
                    node_id: self.gen_node_id(),
                };
                return Ok(type_sign);
            }
            tk::Identifier(name) => {
                let iden_tok = self.consume(tk::Identifier("".into()))?;

                let type_sign = TypeSignStmt {
                    kind: TypeSignStmtKind::Custom(name),
                    span: iden_tok.span.clone(),
                    node_id: self.gen_node_id(),
                };
                return Ok(type_sign);
            }
            tk::QuestionMark => {
                // 可选类型

                let question_mark_tok = self.consume(tk::QuestionMark)?.clone();
                let inner_type = self.parse_type_sign()?;
                let end_span = inner_type.span.end.clone();

                let type_sign = TypeSignStmt {
                    kind: TypeSignStmtKind::Anonymous(super::ast::stmt::AnonymousTypeDefStmt {
                        kind: AnonymousTypeDefKind::Optional {
                            inner_type: Box::new(inner_type),
                        },
                        node_id: self.gen_node_id(),
                        span: Span {
                            start: question_mark_tok.span.start.clone(),
                            end: end_span.clone(),
                        },
                    }),
                    span: Span {
                        start: question_mark_tok.span.start,
                        end: end_span,
                    },
                    node_id: self.gen_node_id(),
                };

                return Ok(type_sign);
            }
            tk::LBracket => {
                // 匿名数组
                let l_tok = self.consume(tk::LBracket)?.clone();
                _ = self.eat_zeor_or_multy(tk::NewLine);

                let inner_type = self.parse_type_sign()?;

                _ = self.eat_zeor_or_multy(tk::NewLine);

                let r_tok = self.consume(tk::RBracket)?.clone();

                let type_sign = TypeSignStmt {
                    kind: TypeSignStmtKind::Anonymous(super::ast::stmt::AnonymousTypeDefStmt {
                        kind: AnonymousTypeDefKind::Array {
                            inner_type: Box::new(inner_type),
                        },
                        node_id: self.gen_node_id(),
                        span: Span {
                            start: l_tok.span.start.clone(),
                            end: r_tok.span.end.clone(),
                        },
                    }),
                    span: Span {
                        start: l_tok.span.start,
                        end: r_tok.span.end,
                    },
                    node_id: self.gen_node_id(),
                };
                return Ok(type_sign);
            }
            tk::LBrace => {
                // 解析匿名结构体.

                // 结构体类型
                let l_tok = self.consume(tk::LBrace)?.clone();

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

                let r_tok = self.consume(tk::RBrace)?.clone();

                let t = TypeSignStmtKind::Anonymous(super::ast::stmt::AnonymousTypeDefStmt {
                    kind: AnonymousTypeDefKind::Struct(fields),
                    node_id: self.gen_node_id(),
                    span: Span {
                        start: l_tok.span.start.clone(),
                        end: r_tok.span.end.clone(),
                    },
                });

                let type_sign = TypeSignStmt {
                    kind: t,
                    span: Span {
                        start: l_tok.span.start,
                        end: r_tok.span.end,
                    },
                    node_id: self.gen_node_id(),
                };
                return Ok(type_sign);
            }
            tk::Enum => {
                // 解析匿名 enum
                let _enum_key_word_tok: &Token = self.consume(tk::Enum)?;

                _ = self.eat_zeor_or_multy(tk::NewLine);
                let l_tok = self.consume(tk::LBrace)?.clone();
                _ = self.eat_zeor_or_multy(tk::NewLine);

                let mut fields: Vec<EnumFieldDef> = vec![];
                loop {
                    if let Ok(enum_field) = self.parse_enum_field() {
                        fields.push(enum_field);
                        _ = self.eat_zeor_or_multy(tk::NewLine);
                        continue;
                    } else {
                        break;
                    }
                }

                _ = self.eat_zeor_or_multy(tk::NewLine);
                let r_tok = self.consume(tk::RBrace)?.clone();
                _ = self.eat_zeor_or_multy(tk::NewLine);

                let kind = TypeSignStmtKind::Anonymous(AnonymousTypeDefStmt {
                    kind: AnonymousTypeDefKind::Enum { fields },
                    node_id: self.gen_node_id(),
                    span: Span {
                        start: l_tok.span.start.clone(),
                        end: r_tok.span.end.clone(),
                    },
                });

                let type_sign = TypeSignStmt {
                    kind,
                    span: Span {
                        start: l_tok.span.start,
                        end: r_tok.span.end,
                    },
                    node_id: self.gen_node_id(),
                };
                return Ok(type_sign);
            }
            x => {
                match x {
                    tk::Pipe | tk::Number(_) | tk::String(_) => {
                        // 解析匿名 union.
                        let alowd_values = self.parse_union_fields()?;

                        let asdf: Span = {
                            if let Some(first) = alowd_values.first() {
                                if let Some(last) = alowd_values.last() {
                                    Span {
                                        start: first.span.start.clone(),
                                        end: last.span.end.clone(),
                                    }
                                } else {
                                    self.peek().span.clone()
                                }
                            } else {
                                self.peek().span.clone()
                            }
                        };

                        let union_end = alowd_values
                            .last()
                            .map(|a| a.span.end.clone())
                            .unwrap_or(tok.span.end.clone());
                        let kind =
                            TypeSignStmtKind::Anonymous(super::ast::stmt::AnonymousTypeDefStmt {
                                kind: AnonymousTypeDefKind::Union {
                                    alowd_values: alowd_values
                                        .iter()
                                        .map(|x| x.to_cbml_value())
                                        .collect(),
                                },
                                node_id: self.gen_node_id(),
                                span: asdf,
                            });

                        let type_sign = TypeSignStmt {
                            kind,
                            span: Span {
                                start: tok.span.start.clone(),
                                end: union_end,
                            },
                            node_id: self.gen_node_id(),
                        };
                        return Ok(type_sign);
                    }

                    _ => {}
                };

                match self.peek_next(1).kind.clone() {
                    tk::Pipe => {
                        // 解析匿名 union.
                        let alowd_values = self.parse_union_fields()?;

                        let base_type: TypeSignStmtKind = TypeSignStmtKind::Any;

                        let union_end = alowd_values
                            .last()
                            .map(|a| a.span.end.clone())
                            .unwrap_or(tok.span.end.clone());

                        let type_sign = TypeSignStmt {
                            kind: base_type,
                            span: Span {
                                start: tok.span.start.clone(),
                                end: union_end,
                            },
                            node_id: self.gen_node_id(),
                        };
                        return Ok(type_sign);
                    }
                    _ => {}
                };

                return Err(CbmlError::new(
                    self.file_path.clone(),
                    format!(
                        "parse_type_sign error: unkonow token {:?}",
                        tok.kind.to_cbml_code()
                    ),
                    tok.span.clone(),
                ));
            }
        }
    }

    fn parse_array_literal(&mut self) -> Result<Literal, CbmlError> {
        /*
        array_literal = LBracket elements Coma{0,1} RBracket
        elements = Newline{0,} first_element tail_elements{0,}
        first_element = Newline{0,} literal
        tail_elements = splitor literal
        splitor = Newline{0,} Coma Newline{0,}
        LBracket = "["
        RBracket = "]"
        Coma = ","
        */

        let _lbracket = self.consume(tk::LBracket)?.clone(); // LBracket

        let mut elements: Vec<Literal> = Vec::new();

        //  空数组  [ ]
        if let tk::RBracket = self.peek().kind {
            let _rbracket = self.consume(tk::RBracket)?;

            return Ok(Literal {
                kind: LiteralKind::Array(elements),
                span: Span {
                    start: _lbracket.span.start,
                    end: _rbracket.span.end.clone(),
                },
            });
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
                        elements.push(literal);

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
        let _rbracket = self.consume(tk::RBracket)?; // RBracket

        return Ok(Literal {
            kind: LiteralKind::Array(elements),
            span: Span {
                start: _lbracket.span.start,
                end: _rbracket.span.end.clone(),
            },
        });
    }

    fn parse_struct_literal(&mut self) -> Result<Literal, CbmlError> {
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

                        fields.push(AsignmentStmt {
                            field_name: name.clone(),
                            value,
                            field_name_span: name_tok.span,
                        });
                    }
                }
                x => {
                    let sf = self.consume(x)?.clone();
                    return Err(CbmlError::err_unknow_token(self.file_path.clone(), sf));
                }
            }

            count += 1;
        }

        let rbrace = self.consume(tk::RBrace)?;

        return Ok(Literal {
            kind: LiteralKind::Struct(fields),
            span: Span {
                start: lbrace.span.start.clone(),
                end: rbrace.span.end.clone(),
            },
        });
    }
    fn parse_literal(&mut self) -> Result<Literal, CbmlError> {
        // 解析字面量
        let tok = self.peek().clone();
        match tok.kind {
            tk::String(s) => {
                let str = s.trim_start_matches("\"").trim_end_matches("\"");

                let a = LiteralKind::String(str.to_string());
                self.consume(tk::String("".into()))?;

                return Ok(Literal {
                    kind: a,
                    span: tok.span,
                });
            }
            tk::Number(n) => {
                self.consume(tk::Number(n.clone()))?;

                let a = LiteralKind::Number(n.clone());

                return Ok(Literal {
                    kind: a,
                    span: tok.span,
                });
            }
            tk::True => {
                self.consume(tk::True)?;

                return Ok(Literal {
                    kind: LiteralKind::Boolean(true),
                    span: tok.span,
                });
            }
            tk::False => {
                self.consume(tk::False)?;

                return Ok(Literal {
                    kind: LiteralKind::Boolean(false),
                    span: tok.span,
                });
            }
            tk::TkNone => {
                self.consume(tk::TkNone)?;

                return Ok(Literal {
                    kind: LiteralKind::LiteralNone,
                    span: tok.span,
                });
            }

            tk::Default => {
                let asdf = self.file_path.clone();
                let default_tok = self.consume(tk::Default)?;

                let e = CbmlError::err_default_keyword_not_allowed_in_literal(
                    asdf,
                    default_tok.span.clone(),
                );

                return Err(e);
            }
            tk::LBracket => {
                // 数组字面量

                // LBracket literal element{0,} coma{0,1} RBracket
                // element = Coma literal

                return self.parse_array_literal();
            }
            tk::LBrace => {
                // 结构体字面量

                return self.parse_struct_literal();
            }

            tk::Identifier(_name) => {
                let next_tok = self.peek_next(1).clone();

                let next = next_tok.kind.clone();
                match next {
                    tk::LParen => {
                        // 解析 enum literal

                        let enum_literal = self.parse_enum_literal()?;

                        return Ok(Literal {
                            kind: enum_literal,
                            span: self.peek().span.clone(),
                        });
                    }
                    _ => {
                        return Err(CbmlError::err_unknow_token(
                            self.file_path.clone(),
                            next_tok,
                        ));
                    }
                }
            }
            _ => {
                return Err(CbmlError::err_unknow_token(self.file_path.clone(), tok));
            }
        }
    }

    /// name = "hello"
    fn parse_asignment(&mut self) -> Result<Stmt, CbmlError> {
        // identifier asignment literal
        // 解析赋值语句

        let name_tok = self.consume(tk::Identifier("".into()))?.clone(); // identifier

        if let tk::Identifier(name) = name_tok.kind {
            self.consume(tk::Asign)?; // asignment

            let value = self.parse_literal()?; // literal

            let stmt = Stmt {
                span: Span {
                    start: name_tok.span.start.clone(),
                    end: value.span.end.clone(),
                },
                kind: StmtKind::Asignment(AsignmentStmt {
                    field_name: name,
                    value,
                    field_name_span: name_tok.span.clone(),
                }),
                node_id: self.gen_node_id(),
            };

            return Ok(stmt);
        } else {
            dp(format!(
                "parse_asignment error: unkonow token {:?}",
                self.peek().clone()
            ));

            return Err(CbmlError::new(
                self.file_path.clone(),
                format!(
                    "parse_asignment error: unkonow token {:?}",
                    self.peek().kind.to_cbml_code()
                ),
                self.peek().span.clone(),
            ));
        };
    }

    fn parse_default_value(&mut self) -> Result<Option<Literal>, CbmlError> {
        // 解析默认值
        let tok = self.peek();
        match tok.kind.clone() {
            tk::Default => {
                self.consume(tk::Default)?;

                let value = self.parse_literal()?;

                return Ok(Some(value));
            }
            _ => {
                return Ok(None);
            }
        }
    }

    fn parse_top_field_def(&mut self) -> Result<Stmt, CbmlError> {
        let doc = self.parse_document()?;

        // 目前只支持给 定义的字段添加文档.
        let mut field_def = match self.parse_struct_field_def() {
            Ok(v) => v,
            Err(_) => {
                let e = CbmlError {
                    error_code: 0000,
                    file_path: self.file_path.clone(),
                    msg: format!("文登注释不能在这里使用."),
                    span: doc.span,
                    note: Some(format!("文登注释只能在 .def.cbml 文件中的字段上使用.")),
                    help: None,
                };
                return Err(e);
            }
        };

        field_def.doc = Some(doc);

        let stmt = Stmt {
            span: Span {
                start: field_def.field_name_span.start.clone(),
                end: field_def.end_span().end,
            },
            kind: StmtKind::FileFieldStmt(field_def),
            node_id: self.gen_node_id(),
        };

        return Ok(stmt);
    }

    /// name : string
    fn parse_struct_field_def(&mut self) -> Result<StructFieldDefStmt, CbmlError> {
        // struct_field_def = document{0,1} identifier Colon type_sign default_value{0,1}
        // default_value = default literal

        let doc = {
            // document{0,1}
            if let Ok(d) = self.parse_document() {
                Some(d)
            } else {
                None
            }
        };

        // 解析字段定义
        let name_tok = self.consume(tk::Identifier("".into()))?.clone();
        let tk::Identifier(name) = name_tok.kind.clone() else {
            return Err(CbmlError::new(
                self.file_path.clone(),
                format!(
                    "parse_field_def error: unkonow token {:?}",
                    self.peek().kind.to_cbml_code()
                ),
                self.peek().span.clone(),
            ));
        };

        _ = self.consume(tk::Colon)?;

        let field_type = self.parse_type_sign()?;
        let type_sign = TypeSignStmt {
            node_id: self.gen_node_id(),
            kind: field_type.kind,
            span: Span {
                start: name_tok.span.start.clone(),
                end: field_type.span.end,
            },
        };

        // 默认值
        let default_value = self.parse_default_value()?;

        return Ok(StructFieldDefStmt {
            field_name: name,
            _type: type_sign,
            default: default_value,
            field_name_span: name_tok.span,
            doc: doc,
            node_id: self.gen_node_id(),
        });

        // if let tk::Identifier(name) = name_tok.kind.clone() {
        //     self.consume(tk::Colon)?;

        //     let field_type = self.parse_type_sign()?;
        //     let type_sign = TypeSignStmt {
        //         node_id: self.gen_node_id(),
        //         kind: field_type.kind,
        //         span: Span {
        //             start: name_tok.span.start.clone(),
        //             end: field_type.span.end,
        //         },
        //     };

        //     // 默认值
        //     let default_value = self.parse_default_value()?;

        //     return Ok(StructFieldDefStmt {
        //         field_name: name,
        //         _type: type_sign,
        //         default: default_value,
        //         field_name_span: name_tok.span,
        //         doc: doc,
        //         node_id: self.gen_node_id(),
        //     });
        // } else {
        //     dp(format!(
        //         "parse_field_def error: unkonow token {:?}",
        //         self.peek().clone()
        //     ));
        //     // todo!();
        //     return Err(ParserError::new(
        //         self.file_path.clone(),
        //         format!(
        //             "parse_field_def error: unkonow token {:?}",
        //             self.peek().kind.to_cbml_code()
        //         ),
        //         self.peek().span.clone(),
        //     ));
        // }
    }

    fn parse_document(&mut self) -> Result<DocumentStmt, CbmlError> {
        let mut duc_line = String::new();
        let mut first_line_span = None;
        let mut last_line_span = None;

        while !self.is_at_end() {
            _ = self.eat_zeor_or_multy(tk::NewLine);
            let Ok(doc_tok) = self.consume(tk::DocComment("".into())) else {
                break;
            };
            let tk::DocComment(s) = doc_tok.kind.clone() else {
                break;
            };

            let sadf = s.trim_start_matches("///");

            duc_line.push_str(sadf);

            if first_line_span.is_none() {
                first_line_span = Some(doc_tok.span.clone());
            }

            last_line_span = Some(doc_tok.span.clone());
        }

        if duc_line.len() > 0 {
            let s = duc_line;

            let first_span = first_line_span.unwrap();
            let last_span = last_line_span.unwrap_or(first_span.clone());

            return Ok(DocumentStmt {
                document: s,
                span: Span {
                    start: first_span.start,
                    end: last_span.end,
                },
            });
        } else {
            let e = CbmlError::new(
                self.file_path.clone(),
                format!(
                    "parse_document error: unkonow token {:?}",
                    self.peek().kind.to_cbml_code()
                ),
                self.peek().span.clone(),
            );
            return Err(e);
        }
    }

    /// 解析使用 struct name { } 这种方式定义的结构体.

    #[cfg(debug_assertions)]
    #[allow(dead_code)]
    fn parse_struct_def(&mut self) -> Result<Stmt, CbmlError> {
        // 解析结构体定义

        let doc = match self.parse_document() {
            Ok(d) => Some(d),
            Err(_) => None,
        };

        let key_word_struct = self.consume(tk::Struct)?.clone();

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

            let rbrace = self.consume(tk::RBrace)?;

            let stmt = Stmt {
                span: Span {
                    start: key_word_struct.span.start.clone(),
                    end: rbrace.span.end.clone(),
                },
                kind: StmtKind::StructDefStmt(StructDef {
                    struct_name: name,
                    fields,
                    name_span: name_tok.span,
                    doc,
                }),
                node_id: self.gen_node_id(),
            };

            return Ok(stmt);
        } else {
            #[cfg(debug_assertions)]
            {
                panic!("在逻辑上时不可能出现的错误.");
            };

            #[allow(unreachable_code)]
            return Err(CbmlError {
                error_code: 0000,
                file_path: self.file_path.clone(),
                msg: format!("parse_struct_def error: unkonow token {:?}", name_tok.kind),
                span: name_tok.span.clone(),
                note: None,
                help: None,
            });
        };
    }

    fn parse_union_fields(&mut self) -> Result<Vec<Literal>, CbmlError> {
        let mut literals: Vec<Literal> = vec![];

        let mut count = 0;

        loop {
            // fields = first{0,1} union_field{1,}
            // first = pipe{0,1} literal
            // union_field = pipe literal

            if count == 0 {
                // first = pipe{0,1} literal

                _ = self.consume(tk::Pipe); // pipe{0,1} 第一个 pipe 符号可有可无.

                let literal = self.parse_literal()?; // literal

                literals.push(literal)
            } else {
                // union_field = pipe literal

                _ = self.eat_zeor_or_multy(tk::NewLine)?; // NewLine{0,}

                if self.peek().kind.clone().kind_is(&tk::Pipe) {
                    self.consume(tk::Pipe)?; // pipe
                    let literal = self.parse_literal()?; // literal
                    literals.push(literal);
                } else {
                    break;
                }
            }

            count += 1;
        }

        return Ok(literals);
    }

    fn parse_use(&mut self) -> Result<Stmt, CbmlError> {
        // 解析 use 语句
        let _use_span = self.consume(tk::Use)?.span.clone();
        let url_tok = self.consume(tk::String("".into()))?.clone();

        if let tk::String(url) = url_tok.kind {
            self.consume_stmt_end_token()?;

            let stmt = Stmt {
                span: Span {
                    start: _use_span.start.clone(),
                    end: url_tok.span.end.clone(),
                },
                kind: StmtKind::Use(UseStmt {
                    url,
                    keyword_span: _use_span,
                    url_span: url_tok.span,
                }),
                node_id: self.gen_node_id(),
            };
            return Ok(stmt);
        } else {
            return Err(CbmlError::new(
                self.file_path.clone(),
                format!(
                    "parse_statement error: need {:?}, but found token {:?}",
                    tk::String("".into()),
                    self.peek().kind.to_cbml_code()
                ),
                self.peek().span.clone(),
            ));
        }
    }

    #[cfg(debug_assertions)]
    #[allow(dead_code)]
    fn parse_enum_def(&mut self) -> Result<Stmt, CbmlError> {
        // enum identifier LBrace newline{0,} enum_field{0,} RBrace
        // enum_field = newline{0,} identifier LParent typedef RParent newline

        // dp(format!("parse_enum_def(&mut self)"));

        // let doc = self.parse_document()?;
        let doc = match self.parse_document() {
            Ok(d) => Some(d),
            Err(_) => None,
        };

        let key_word_enum = self.consume(tk::Enum)?.clone(); // enum

        let enum_name_tok = self.consume(tk::Identifier("".into()))?.clone(); // identifier

        if let tk::Identifier(enum_name) = enum_name_tok.kind {
            self.consume(tk::LBrace)?; // LBrace

            self.eat_zeor_or_multy(tk::NewLine)?; // newline{0,}

            let mut fields: Vec<EnumFieldDef> = vec![];

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

            let rbrace = self.consume(tk::RBrace)?;

            let stmt = Stmt {
                span: Span {
                    start: key_word_enum.span.start,
                    end: rbrace.span.end.clone(),
                },
                kind: StmtKind::EnumDef(EnumDef {
                    enum_name,
                    fields,
                    doc,
                    name_span: enum_name_tok.span,
                }),
                node_id: self.gen_node_id(),
            };

            return Ok(stmt);
        } else {
            panic!("这是逻辑上不可能出现的错误.")
            // return Err(ParserError {
            //     message: format!("这是逻辑上不可能出现的错误"),
            //     token: Some(enum_name_tok.clone()),
            // });
        }
    }

    #[cfg(debug_assertions)]
    #[allow(dead_code)]
    fn parse_union_def(&mut self) -> Result<Stmt, CbmlError> {
        // union LParent typesign RParent identifier Assignment union_field{1,}
        // union_field = pipe{1} literal

        let doc = match self.parse_document() {
            Ok(d) => Some(d),
            Err(_) => None,
        };
        // 解析联合体定义
        self.consume(tk::Union)?; // union

        // typesign
        let base_type: TypeSignStmtKind = if let tk::LParen = self.peek().kind {
            // 解析联合体的基本类型
            self.consume(tk::LParen)?; // LParent
            let base_type = self.parse_type_sign()?; // typesign
            self.consume(tk::RParen)?; // RParent

            base_type.kind
        } else {
            return Err(CbmlError::new(
                self.file_path.clone(),
                format!(
                    "parse_statement error: need {:?}, but found token {:?}",
                    tk::LParen.to_cbml_code(),
                    self.peek().kind.to_cbml_code()
                ),
                self.peek().span.clone(),
            ));
        };

        let name_tok = self.consume(tk::Identifier("".into()))?.clone();
        // identifier
        let union_name: String = if let tk::Identifier(union_name) = name_tok.kind {
            union_name
        } else {
            return Err(CbmlError::new(
                self.file_path.clone(),
                format!(
                    "parse_statement error: need {:?}, but found token {:?}",
                    tk::Identifier("".into()),
                    self.peek().kind.to_cbml_code()
                ),
                self.peek().span.clone(),
            ));
        };

        self.consume(tk::Asign)?; // Assignment

        let alowd_values = self.parse_union_fields()?; // union_field{1,}

        let stmt = Stmt {
            span: Span {
                start: name_tok.span.start.clone(),
                end: self.peek().span.end.clone(),
            },
            kind: StmtKind::TypeDef(TypeDefStmt::UnionDef(UnionDef {
                union_name,
                base_type,
                allowed_values: alowd_values,
                doc,
                name_span: name_tok.span,
            })),
            node_id: self.gen_node_id(),
        };

        return Ok(stmt);
    }

    fn parse_enum_field(&mut self) -> Result<EnumFieldDef, CbmlError> {
        // enum_field =   identifier LParent typedef RParent newline

        let field_name_tok = self.consume(tk::Identifier("".into()))?.clone();

        if let tk::Identifier(field_name) = field_name_tok.kind.clone() {
            _ = self.eat_zeor_or_multy(tk::NewLine);
            self.consume(tk::LParen)?; // LParent
            _ = self.eat_zeor_or_multy(tk::NewLine);

            let ty = self.parse_type_sign()?; // typedef
            _ = self.eat_zeor_or_multy(tk::NewLine);

            self.consume(tk::RParen)?;
            self.consume(tk::NewLine)?; // ends.

            let field = EnumFieldDef {
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

    fn parse_enum_literal(&mut self) -> Result<LiteralKind, CbmlError> {
        // enum_literal = identifier LParent literal RParent

        let name_tok = self.consume(tk::Identifier("".into()))?.clone();
        // LParent
        // if let tk::Identifier(name) = self.consume(tk::Identifier("".into()))?.kind.clone() {
        if let tk::Identifier(name) = name_tok.kind {
            let _ = self.consume(tk::LParen)?.clone(); // LParent

            let lit = self.parse_literal()?; // literal

            let _ = self.consume(tk::RParen)?.clone(); // RParent

            return Ok(LiteralKind::EnumFieldLiteral {
                field_name: name,
                literal: lit.into(),
                span: name_tok.span, // span: Span {
                                     //     start: l_tok.span.start,
                                     //     end: r_tok.span.end,
                                     // },
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

    fn eat_zeor_or_multy(&mut self, kind: tk) -> Result<Vec<Token>, CbmlError> {
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
    fn consume(&mut self, kind: tk) -> Result<&Token, CbmlError> {
        if self.check(&kind) {
            let tok = &self.tokens[self.current_position];

            self.current_position += 1;

            return Ok(tok);
        } else {
            Err(CbmlError::new(
                self.file_path.clone(),
                format!(
                    "Expected token: {:?}, but found: {:?}",
                    kind.to_cbml_code(),
                    self.peek().kind.to_cbml_code()
                ),
                self.peek().span.clone(),
            ))
        }
    }

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
    fn consume_stmt_end_token(&mut self) -> Result<Token, CbmlError> {
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
                return Err(CbmlError::new(
                    self.file_path.clone(),
                    format!(
                        "need: {:?}, but found: {:?}",
                        tk::NewLine.to_cbml_code(),
                        self.peek().kind.to_cbml_code()
                    ),
                    self.peek().span.clone(),
                ));
            }
        }
    }

    fn gen_node_id(&mut self) -> NodeId {
        self.node_id.id += 1;
        return self.node_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::lexer::tokenize;

    #[test]
    fn test_parser() {
        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/1.cmml");
        // asdfasdfsdf("/Users/chenbao/Documents/GitHub/cbml/examples/1.typedef.cbml");

        test_parse_array();
    }

    fn test_parse_array() {
        let code = r##"

arr = [1,2,3,4,5 ]
 
arr_any = [1, true , "true", [1,2,3], {name = "string"}]

        
        "##;

        let tokens = tokenize("path", &code).tokens;

        // dp(format!("tokens: {:?}", tokens));

        let mut parser = CbmlParser::new("path".to_string(), &tokens);
        let re = parser.parse();
        assert_eq!(re.errors.is_empty(), true);
    }
}

use std::default;

use crate::{
    cbml_project::types::FieldDef,
    lexer::token::{Span, Token},
};

use super::{
    StmtKind,
    ast::stmt::{AsignmentStmt, Literal, UseStmt},
};

#[derive(Debug, Clone, PartialEq)]
pub struct ParserError {
    pub file_path: String,
    pub msg: String,
    pub span: Span,
    pub note: Option<String>,
    pub help: Option<String>,

    /// error code 可以用来给 language server 自动修复和自动补全.
    /// 如果不知道填那个 error 错的, 就填 0000.
    /// 0000 表示未知错误.
    pub error_code: u32,
}

impl ParserError {
    pub fn new(file_path: String, message: String, span: Span) -> Self {
        Self {
            file_path: file_path,
            msg: message,
            span,
            note: None,
            help: None,
            error_code: 0000,
        }
    }

    pub fn lookup<'a>(&self, source_code: &'a str) -> String {
        // 返回 (行号, 列号, 该行文本)
        // let line_idx = self.code_location.start.character_index;

        let charts: Vec<char> = source_code.chars().collect();

        let line_start = if self.span.start.character_index == 0 {
            0
        } else {
            self.span.start.character_index
        };

        // let line_end = self.code_location.end.character_index;
        let line_end = if self.span.end.character_index == 0 {
            0
        } else {
            self.span.end.character_index
        };
        // let sadf = charts.get(line_start..).unwrap();
        // for x in sadf {}

        let line_text = charts.get(line_start..line_end).unwrap_or(&[]);

        // let line_text = &source_code[line_start..line_end];
        let mut re = String::new();

        for x in line_text {
            re.push(*x);
        }

        return re;

        // (line_idx + 1, col + 1, line_text)
    }

    pub fn report_error(&self, source_code: &str) {
        let (line, col, line_text) = (
            self.span.start.line,
            self.span.start.column,
            self.lookup(source_code),
        );

        let message = &self.msg;

        println!("error: {}", message);
        println!("  --> {}:{}:{}", self.file_path, line + 1, col);
        println!("    |");
        println!("{:>3} | {}", line, line_text);
        println!("    | {:>width$}^", "", width = col as usize);

        if let Some(s) = &self.help {
            println!("  help: {}", s);
        };
    }
}

impl default::Default for ParserError {
    fn default() -> Self {
        Self {
            error_code: 0000,
            file_path: Default::default(),
            msg: Default::default(),
            span: Span {
                start: crate::lexer::token::Position {
                    line: 0,
                    column: 0,
                    character_index: 0,
                },
                end: crate::lexer::token::Position {
                    line: 0,
                    column: 0,
                    character_index: 0,
                },
            },
            note: Default::default(),
            help: Default::default(),
        }
    }
}

impl ParserError {
    /// 0000
    pub fn err_unknow_error(file_path: String, span: Span) -> Self {
        Self {
            file_path,
            msg: format!("unknow error"),
            span,
            note: None,
            help: None,
            error_code: 000,
        }
    }

    /// 0001
    pub fn err_cannot_open_file(
        source_code_file_path: String,
        target_file_path: &str,
        span: Span,
        err: std::io::Error,
    ) -> Self {
        Self {
            file_path: source_code_file_path,
            msg: format!("cannot open file: {:?}\n{}", target_file_path, err),
            span,
            note: None,
            help: None,
            error_code: 0001,
        }
    }

    /// 0002
    pub fn err_cannot_find_type(file_path: String, span: Span, type_name: &str) -> ParserError {
        Self {
            error_code: (0002),
            file_path,
            msg: format!("connot find type `{}` ", type_name),
            span,
            note: None,
            help: None,
        }
    }

    /// 0003
    pub fn err_unknow_field(file_path: String, span: Span, field_name: &str) -> Self {
        Self {
            error_code: (0003),
            file_path,
            msg: format!("unknow field `{}` ", field_name),
            span,
            note: None,
            help: None,
        }
    }

    /// 0004
    pub fn err_mismatched_types(
        file_path: String,
        span: Span,
        expected: &str,

        found: &str,
    ) -> Self {
        Self {
            error_code: (0004),
            file_path,
            msg: format!(
                "mismatched types, expected `{}` found  `{}` ",
                expected, found
            ),
            span,
            note: None,
            help: None,
        }
    }

    /// 0005
    pub fn err_union_duplicated_item(file_path: String, span: Span, item: &str) -> Self {
        // Self::new(file_path, format!("union duplicated item: {}", item), span)
        Self {
            error_code: (0005),
            file_path,
            msg: format!("union duplicated item: {}", item),
            span,
            note: None,
            help: None,
        }
    }

    /// 0006
    pub fn err_use_can_only_def_onece(file_path: String, span: Span) -> Self {
        // Self::new(file_path, format!("use can only def onece"), span)
        Self {
            error_code: (0006),
            file_path,
            msg: format!("use can only def onece"),
            span,
            note: None,
            help: None,
        }
    }

    /// 0007
    pub fn err_stmt_not_allow_in_current_scope(
        file_path: String,
        span: Span,
        stmt: &StmtKind,
    ) -> Self {
        // TypeCheckedResult::Error(format!("stmt not allow in current scope: {:?}", stmt))

        // Self::new(
        //     file_path,
        //     format!("stmt not allow in current scope: {:?}", stmt),
        //     span,
        // )
        Self {
            error_code: (0007),
            file_path,
            msg: format!("stmt not allow in current scope: {:?}", stmt),
            span,
            note: None,
            help: None,
        }
    }

    /// 0008
    pub fn err_field_alredy_exits(file_path: String, span: Span, field_name: &str) -> Self {
        // Self::new(
        //     file_path,
        //     format!("field `{}` alredy exit", field_name),
        //     span,
        // )

        Self {
            error_code: (0008),
            file_path,
            msg: format!("field `{}` alredy exit", field_name),
            span,
            note: None,
            help: None,
        }
    }

    /// 0009
    pub fn err_type_name_alredy_exits(file_path: String, span: Span, type_name: &str) -> Self {
        // Self::new(
        //     file_path,
        //     format!("type name `{}` alredy exit", type_name),
        //     span,
        // )

        Self {
            error_code: (0009),
            file_path,
            msg: format!("type name `{}` alredy exit", type_name),
            span,
            note: None,
            help: None,
        }
    }

    /// 0010
    pub fn err_filed_alredy_asignment(
        file_path: String,
        span: Span,
        asign: &AsignmentStmt,
    ) -> Self {
        // Self::new(
        //     file_path,
        //     format!("field `{}` alredy asignmented", asign.field_name),
        //     span,
        // )

        Self {
            error_code: (0010),
            file_path,
            msg: format!("field `{}` alredy asignmented", asign.field_name),
            span,
            note: None,
            help: None,
        }
    }

    /// 0011
    pub fn err_this_field_donot_have_default_value(
        file_path: String,
        literal_kind_defal_token_span: Span,
    ) -> Self {
        Self {
            error_code: (0011),
            file_path,
            msg: format!("this field donot have default value"),
            span: literal_kind_defal_token_span,
            note: None,
            help: None,
        }
    }

    /// 0012
    pub fn err_not_allow_in_union(file_path: String, literal: Literal) -> Self {
        let adf = match &literal.kind {
            super::ast::stmt::LiteralKind::EnumFieldLiteral { .. } => "enum",
            super::ast::stmt::LiteralKind::LiteralNone => "none",
            // super::ast::stmt::LiteralKind::Todo => "todo",
            // super::ast::stmt::LiteralKind::Default => "defaul",
            _ => "",
        };

        ParserError {
            error_code: 0012,
            file_path: file_path,
            msg: format!("{} 不能在 union 中使用.", adf),
            span: literal.span,
            note: Some(format!(
                "union 中可以使用的类型有 string number bool array struct"
            )),
            help: None,
        }
    }

    /// 0013
    pub fn err_default_keyword_not_allowed_in_literal(
        file_path: String,
        default_tok_span: Span,
    ) -> Self {
        ParserError {
            file_path,
            msg: format!("default 关键字只能用于字段申明时使用."),
            span: default_tok_span,
            note: Some(format!(
                "关于 default 关键字的用法, 可查看: [github.com](github.com)"
            )),
            help: None,
            error_code: (0013),
        }
    }

    /// 0014
    pub fn err_unknow_token(file_path: String, tok: Token) -> Self {
        ParserError {
            error_code: (0014),
            file_path,
            msg: format!("syntax error: unkonow token {:?}", tok.kind),
            span: tok.span,
            note: None,
            help: None,
        }
    }

    /// 0015
    pub fn err_has_fields_unasigned(
        file_path: String,
        unasigned_fields: Vec<&FieldDef>,
        span: Span,
    ) -> Self {
        let mut sdaf = String::new();

        for x in &unasigned_fields {
            sdaf.push_str(&x.name);
            sdaf.push_str(", ");
        }

        Self {
            error_code: (0015),
            file_path,
            msg: format!("还有 {} 个字段未赋值: {}", unasigned_fields.len(), sdaf),
            span: span,
            note: None,
            help: None,
        }
    }

    /// 0016
    pub fn err_use_imported_file_has_error(
        file_path: String,
        use_stmt: &UseStmt,
        def_file_errors_count: usize,
    ) -> Self {
        ParserError {
            error_code: 0016,
            file_path,
            msg: format!(
                "引用的 类型定义文件 中有 {} 个错误: \n{}",
                def_file_errors_count, &use_stmt.url
            ),
            span: use_stmt.keyword_span.clone(),
            note: None,
            help: None,
        }
    }

    /// 0017
    pub fn err_field_def_not_allow_in_here(file_path: String, span: Span) -> Self {
        ParserError {
            error_code: 0017,
            file_path,
            msg: format!("字段定义在这里是不允许的."),
            span,
            note: Some(format!("")),

            help: Some(format!("将字段定义移动道 typedef 文件中.")),
        }
    }
}

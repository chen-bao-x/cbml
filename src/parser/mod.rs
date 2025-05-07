use std::default;

pub use ast::stmt::StmtKind;
use ast::stmt::{AsignmentStmt, StructFieldDefStmt};
pub use cbml_parser::CbmlParser;

use crate::lexer::token::{Span, Token};

pub mod ast;
pub mod cbml_parser;

/// 解析 Token 列表并返回 AST
pub fn parse(file_path: String, source: &[Token]) -> Result<Vec<StmtKind>, Vec<ParserError>> {
    let mut parser = CbmlParser::new(file_path, source);

    return parser.parse();
}

#[derive(Debug, Clone)]
pub struct ParserError {
    pub file_path: String,
    pub msg: String,
    pub code_location: Span,
    pub note: Option<String>,
    pub help: Option<String>,
}

impl ParserError {
    pub fn new(file_path: String, message: String, span: Span) -> Self {
        Self {
            file_path: file_path,
            msg: message,
            code_location: span,
            note: None,
            help: None,
        }
    }

    pub fn lookup<'a>(&self, source_code: &'a str) -> &'a str {
        // 返回 (行号, 列号, 该行文本)
        // let line_idx = self.code_location.start.character_index;

        let line_start = self.code_location.start.character_index - 1;

        // let line_end = self.code_location.end.character_index;
        let line_end = source_code[line_start..]
            .find('\n')
            .map(|i| line_start + i)
            .unwrap_or(source_code.len());

        let line_text = &source_code[line_start..line_end];
        return line_text;

        // (line_idx + 1, col + 1, line_text)
    }

    pub fn report_error(&self, source_code: &str) {
        let (line, col, line_text) = (
            self.code_location.start.line,
            self.code_location.start.column,
            self.lookup(source_code),
        );

        let message = &self.msg;

        println!("error: {}", message);
        println!("  --> {}:{}:{}", self.file_path, line, col);
        println!("    |");
        println!("{:>3} | {}", line, line_text);
        println!("    | {:>width$}^", "", width = col - 1);

        if let Some(s) = &self.help {
            println!("  help: {}", s);
        };
    }
}

impl default::Default for ParserError {
    fn default() -> Self {
        Self {
            file_path: Default::default(),
            msg: Default::default(),
            code_location: Span {
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
    pub fn err_cannot_find_type(file_path: String, span: Span, type_name: &str) -> ParserError {
        Self::new(
            file_path,
            format!("connot find type `{}` ", type_name),
            span,
        )
    }

    pub fn err_unknow_field(file_path: String, span: Span, field_name: &str) -> Self {
        // TypeCheckedResult::Error(format!("unknow field `{}` ", field_name))

        Self::new(file_path, format!("unknow field `{}` ", field_name), span)
    }

    pub fn err_mismatched_types(
        file_path: String,
        span: Span,
        expected: &str,
        found: &str,
    ) -> Self {
        // TypeCheckedResult::Error(format!(
        //     "mismatched types, expected `{}` found  `{}` ",
        //     expected, found
        // ))

        Self::new(
            file_path,
            format!(
                "mismatched types, expected `{}` found  `{}` ",
                expected, found
            ),
            span,
        )
    }

    pub fn err_union_duplicated_item(file_path: String, span: Span, item: &str) -> Self {
        Self::new(file_path, format!("union duplicated item: {}", item), span)
    }

    pub fn err_use_can_only_def_onece(file_path: String, span: Span) -> Self {
        Self::new(file_path, format!("use can only def onece"), span)
    }

    pub fn err_stmt_not_allow_in_current_scope(
        file_path: String,
        span: Span,
        stmt: &StmtKind,
    ) -> Self {
        // TypeCheckedResult::Error(format!("stmt not allow in current scope: {:?}", stmt))

        Self::new(
            file_path,
            format!("stmt not allow in current scope: {:?}", stmt),
            span,
        )
    }

    pub fn err_field_alredy_exits(file_path: String, span: Span, field_name: &str) -> Self {
        Self::new(
            file_path,
            format!("field `{}` alredy exit", field_name),
            span,
        )
    }

    pub fn err_type_name_alredy_exits(file_path: String, span: Span, type_name: &str) -> Self {
        Self::new(
            file_path,
            format!("type name `{}` alredy exit", type_name),
            span,
        )
    }

    pub fn err_filed_alredy_asignment(
        file_path: String,
        span: Span,
        asign: &AsignmentStmt,
    ) -> Self {
        Self::new(
            file_path,
            format!("field `{}` alredy asignmented", asign.field_name),
            span,
        )
    }
}

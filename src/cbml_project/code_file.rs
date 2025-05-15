use std::{
    collections::{HashMap, HashSet},
    default, path,
};

use crate::{
    cbml_value::value::CbmlType,
    lexer::{token::Span, tokenizer},
    parser::{ast::stmt::Stmt, parser_error::ParserError},
};

use super::{FieldAsign, FieldDef, typedef_file::TypedefFile};

pub struct CodeFile {
    pub file_path: String,

    pub fields: Vec<FieldAsign>,

    pub use_url: Option<String>,
    pub typedef_file: Option<TypedefFile>,

    pub errors: Vec<ParserError>,

    last_token_span: Span,
}

impl CodeFile {
    pub fn new(file_path: String) -> Self {
        let mut f = Self {
            file_path: file_path.clone(),
            fields: Vec::new(),
            use_url: None,
            typedef_file: None,
            errors: Vec::new(),
            last_token_span: Span::empty(),
        };

        if (&file_path).ends_with(".def.cbml") {
            let e = ParserError {
                file_path,
                msg: format!("以 .def.cbml 的是类型定义文件."),
                code_location: Span::empty(),
                note: None,
                help: None,
            };

            f.errors.push(e);
        } else {
            f.parse(&file_path);
        };

        f.error_check();

        return f;
    }

    pub fn get_all_errors(&self) -> Vec<ParserError> {
        let mut re: Vec<ParserError> = vec![];

        re.extend_from_slice(&self.errors);

        let Some(def) = &self.typedef_file else {
            return re;
        };

        re.extend_from_slice(&def.errors);

        return re;
    }

    fn parse(&mut self, path: &str) {
        use crate::parser::cbml_parser::CbmlParser;
        use std::fs::read_to_string;

        let code = read_to_string(path).unwrap();

        let lexer_result = tokenizer(path, &code).map_err(|e| {
            println!("{:?}", e);
            return e;
        });

        let tokens = match lexer_result {
            Ok(t) => t,
            Err(e) => {
                self.errors.push(e);
                return ();
            }
        };

        if let Some(tok) = tokens.last() {
            self.last_token_span = tok.span.clone();
        }

        // dp(format!("tokens: {:?}", tokens));

        let mut parser = CbmlParser::new(path.to_string(), &tokens);
        let parser_result = parser.parse();

        match parser_result {
            Ok(ast) => {
                self.parse_ast(ast);
            }
            Err(mut errs) => {
                self.errors.append(&mut errs);
            }
        };
    }

    fn parse_ast(&mut self, ast: Vec<Stmt>) {
        for stmt in ast {
            self.parse_one_stmt(stmt);
        }
    }

    fn parse_one_stmt(&mut self, s: Stmt) {
        match s.kind {
            crate::parser::StmtKind::Use(use_stmt) => self.parse_use(&use_stmt),
            crate::parser::StmtKind::Asignment(asignment_stmt) => {
                self.parse_asignment(asignment_stmt)
            }
            crate::parser::StmtKind::FileFieldStmt(struct_field_def_stmt) => {
                self.parse_struct_field_def(&struct_field_def_stmt);
            }
            crate::parser::StmtKind::TypeAliasStmt(_) => {
                todo!()
            }
            crate::parser::StmtKind::StructDefStmt(struct_def) => {
                self.parse_struct_def(&struct_def)
            }
            crate::parser::StmtKind::EnumDef(enum_def) => self.parse_enum_def(&enum_def),
            // crate::parser::StmtKind::UnionDef(union_def) => self.parse_union_def(union_def),
            crate::parser::StmtKind::TypeDef(type_def_stmt) => self.parse_type_def(&type_def_stmt),
            crate::parser::StmtKind::LineComment(_) => {}
            crate::parser::StmtKind::BlockComment(_) => {}
            crate::parser::StmtKind::DocComment(_) => {}
            crate::parser::StmtKind::EmptyLine => {}
        };
    }

    fn parse_use(&mut self, use_stmt: &crate::parser::ast::stmt::UseStmt) {
        let Ok(_) = self.check_use(use_stmt) else {
            return ();
        };

        self.use_url = Some(use_stmt.get_use_url());
        let def_file = TypedefFile::new(use_stmt.get_use_url());

        // 错误检查.
        {
            if !def_file.errors.is_empty() {
                let e = ParserError {
                    file_path: self.file_path.clone(),
                    msg: format!(
                        "引用的 类型定义文件 中有 {} 个错误: \n{}",
                        def_file.errors.len(),
                        &use_stmt.url
                    ),
                    code_location: use_stmt.keyword_span.clone(),
                    note: None,
                    help: None,
                };
                self.errors.push(e);
            }
        }

        self.typedef_file = Some(def_file);
    }

    fn parse_asignment(&mut self, asignment_stmt: crate::parser::ast::stmt::AsignmentStmt) {
        let value = FieldAsign {
            name: asignment_stmt.field_name,
            value: asignment_stmt.value,
            span: asignment_stmt.field_name_span,
        };

        self.fields.push(value);
    }

    /// returns: (field_name, type)
    fn parse_struct_field_def(
        &mut self,
        struct_field_def_stmt: &crate::parser::ast::stmt::StructFieldDefStmt,
    ) {
        let e = ParserError {
            file_path: self.file_path.clone(),
            msg: format!("字段定义在这里是不允许的."),
            code_location: Span {
                start: struct_field_def_stmt.field_name_span.start.clone(),
                end: struct_field_def_stmt.end_span().end,
            },
            note: Some(format!("")),

            help: Some(format!("将字段定义移动道 typedef 文件中.")),
        };

        self.errors.push(e);
    }

    fn parse_struct_def(&mut self, struct_def: &crate::parser::ast::stmt::StructDef) {
        let e = ParserError {
            file_path: self.file_path.clone(),
            msg: format!("字段定义在这里是不允许的."),
            code_location: Span {
                start: struct_def.name_span.start.clone(),
                end: struct_def.end_span().end,
            },
            note: Some(format!("")),

            help: Some(format!("将字段定义移动道 typedef 文件中.")),
        };
        self.errors.push(e);
    }

    fn parse_enum_def(&mut self, enum_def: &crate::parser::ast::stmt::EnumDef) {
        let e = ParserError {
            file_path: self.file_path.clone(),
            msg: format!("类型定义在这里是不允许的."),
            code_location: enum_def.name_span.clone(),
            note: Some(format!("")),

            help: Some(format!("将字段定义移动道 typedef 文件中.")),
        };

        self.errors.push(e);
    }

    fn parse_type_def(&mut self, type_def_stmt: &crate::parser::ast::stmt::TypeDefStmt) {
        let e = ParserError {
            file_path: self.file_path.clone(),
            msg: format!("类型定义在这里是不允许的."),
            code_location: type_def_stmt.get_span(),
            note: Some(format!("")),

            help: Some(format!("将字段定义移动道 typedef 文件中.")),
        };

        self.errors.push(e);
    }
}

// error check
impl CodeFile {
    fn check_use(&mut self, use_stmt: &crate::parser::ast::stmt::UseStmt) -> Result<(), ()> {
        // 在 use 语句之前不能有 赋值语句.
        {
            if !self.fields.is_empty() {
                let e = ParserError {
                    file_path: self.file_path.clone(),
                    msg: format!("`use` 只能在文件的最开头."),
                    code_location: use_stmt.keyword_span.clone(),
                    note: None,
                    help: Some(format!("尝试将 `use` 移动到第一行")),
                };
                self.errors.push(e);
                return Err(());
            }
        };

        // use 语句只能使用一次.
        {
            if self.use_url.is_some() || self.typedef_file.is_some() {
                let e = ParserError::err_use_can_only_def_onece(
                    self.file_path.clone(),
                    use_stmt.url_span.clone(),
                );
                self.errors.push(e);
                return Err(());
            }
        };
        return Ok(());
    }

    fn error_check(&mut self) {
        self.check_duplicated_file_field_name();
        self.check_unasigned_field();
    }
    // 类型检查, 检查定义的字段的类型跟赋值的类型是否相同.

    // 字段重复检查, 一个字段只需要赋值一次.
    fn check_duplicated_file_field_name(&mut self) {
        let mut seen: HashSet<&String> = HashSet::new();
        let mut duplicates: Vec<&FieldAsign> = Vec::new();

        for x in &self.fields {
            if !seen.insert(&x.name) {
                duplicates.push(&x);
            }
        }

        let errors: Vec<ParserError> = duplicates
            .iter()
            .map(|x| {
                ParserError::err_field_alredy_exits(self.file_path.clone(), x.span.clone(), &x.name)
            })
            .collect();

        self.errors.extend(errors);
    }

    // 缺失字段检查, 检查定义了却没有赋值的 top level 字段,
    fn check_unasigned_field(&mut self) {
        let file_path = self.file_path.clone();

        let sadf = self.get_unasigned_field();
        if sadf.is_empty() {
            return;
        } else {
            sadf.iter().for_each(|x| {
                println!("{:?}", x);
            });
        }

        let e = ParserError {
            file_path: file_path,
            msg: format!("还有 {} 个字段未赋值.", sadf.len()),
            code_location: self.last_token_span.clone(),
            note: None,
            help: None,
        };
        self.errors.push(e);
    }

    pub fn get_unasigned_field(&mut self) -> Vec<&FieldDef> {
        let Some(def_file) = &self.typedef_file else {
            return Vec::new();
        };

        let mut unasigned_fields: Vec<&FieldDef> = Vec::new();

        let mut asigned_fields = self.fields.iter().map(|x| &x.name).collect::<HashSet<_>>();

        // 找出定义了却没有赋值的 top level 字段,
        {
            for x in &def_file.get_all_top_fields() {
                let remoed = asigned_fields.remove(&x.name);
                if !remoed {
                    unasigned_fields.push(x);
                }
            }
        }

        return unasigned_fields;
    }

    // 缺失字段检查, 检查 struct 中定义了却没有赋值的字段.
}

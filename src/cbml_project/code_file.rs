use super::{
    typedef_file::TypedefFile,
    types::{FieldAsign, FieldDef, TypeInfo},
};
use crate::{
    cbml_value::value::{CbmlType, CbmlTypeKind, CbmlValue, ToCbmlValue},
    formater::ToCbmlCode,
    lexer::{
        token::{Position, Span},
        tokenizer,
    },
    parser::{
        CbmlParser,
        ast::stmt::{Literal, LiteralKind, Stmt},
        parser_error::ParserError,
    },
    typecheck::types_for_check::ScopeID,
};
use std::{
    collections::{HashMap, HashSet},
    default, path,
};

#[derive(Debug, Clone)]
pub struct CodeFile {
    pub file_path: String,

    pub fields: Vec<FieldAsign>,

    pub use_url: Option<String>,
    pub typedef_file: Option<TypedefFile>,

    pub errors: Vec<ParserError>,

    last_line_span: Span,
}

impl CodeFile {
    pub fn new(file_path: String) -> Self {
        let mut f = Self {
            file_path: file_path.clone(),
            fields: Vec::new(),
            use_url: None,
            typedef_file: None,
            errors: Vec::new(),
            last_line_span: Span::empty(),
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
            f.parse_file(&file_path);
        };

        f.error_check();

        return f;
    }

    pub fn new_from(file_path: String, code: &str) -> Self {
        let mut f = Self {
            file_path: file_path.clone(),
            fields: Vec::new(),
            use_url: None,
            typedef_file: None,
            errors: Vec::new(),
            last_line_span: Span::empty(),
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
            f.parse_code(code);
        };

        f.error_check();

        return f;
    }

    pub fn get_all_errors(&self) -> Vec<ParserError> {
        let mut re: Vec<ParserError> = vec![];

        re.extend_from_slice(&self.errors);

        // let Some(def) = &self.typedef_file else {
        //     return re;
        // };

        // re.extend_from_slice(&def.errors);

        return re;
    }

    fn parse_file(&mut self, path: &str) {
        use crate::parser::cbml_parser::CbmlParser;
        use std::fs::read_to_string;

        let code = read_to_string(path).unwrap();
        self.parse_code(&code);
        // let lexer_result = tokenizer(path, &code);

        // let tokens = match lexer_result {
        //     Ok(t) => t,
        //     Err(e) => {
        //         self.errors.push(e);
        //         return ();
        //     }
        // };

        // if let Some(tok) = tokens.last() {
        //     self.last_token_span = tok.span.clone();
        // }

        // // dp(format!("tokens: {:?}", tokens));

        // let mut parser = CbmlParser::new(path.to_string(), &tokens);
        // let parser_result = parser.parse();

        // match parser_result {
        //     Ok(ast) => {
        //         self.parse_ast(ast);
        //     }
        //     Err(mut errs) => {
        //         self.errors.append(&mut errs);
        //     }
        // };
    }

    fn parse_code(&mut self, code: &str) {
        {
            let mut last_line_index: u32 = 0;
            for _ in code.lines() {
                last_line_index += 1;
            }

            self.last_line_span = Span {
                start: Position {
                    line: last_line_index,
                    column: 0,
                    character_index: code.chars().count(),
                },
                end: Position {
                    line: last_line_index,
                    column: 0,
                    character_index: code.chars().count(),
                },
            };
        }

        let lexer_result = tokenizer(&self.file_path, &code);

        let tokens = match lexer_result {
            Ok(t) => t,
            Err(e) => {
                self.errors.push(e);
                return ();
            }
        };

        // dp(format!("tokens: {:?}", tokens));

        let mut parser = CbmlParser::new(self.file_path.clone(), &tokens);
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
        if let Some(def_file) = &self.typedef_file {
            if !def_file.errors.is_empty() {
                return;
            }
        };

        self.check_duplicated_file_field_name();
        self.check_unasigned_field();
        self.check_type();

        // self.check_extra_field_asign();
    }

    // 类型检查, 检查赋值的类型跟定义的字段的类型是否相同.
    fn check_type(&mut self) {
        for x in &self.fields {
            let re = self.check_one_field_type(x);
            match re {
                Ok(_) => {}
                Err(e) => {
                    self.errors.push(e);
                }
            }
        }
    }

    fn is_same_type(&self, need_type: &CbmlType, found: &LiteralKind) -> bool {
        if let LiteralKind::Default = found {
            return true;
        }

        match need_type.kind.clone() {
            CbmlTypeKind::String => match found {
                LiteralKind::String { .. } => true,
                LiteralKind::Default => true,
                _ => false,
            },
            CbmlTypeKind::Number => match found {
                LiteralKind::Number(_) => true,
                LiteralKind::Default => true,
                _ => false,
            },
            CbmlTypeKind::Bool => match found {
                LiteralKind::Boolean(_) => true,
                LiteralKind::Default => true,
                _ => false,
            },
            CbmlTypeKind::Any => true,
            CbmlTypeKind::Array { inner_type, .. } => {
                //
                match found {
                    LiteralKind::Array(literals) => {
                        return literals.iter().all(|x| self.is_same_type(&inner_type, x));
                    }
                    LiteralKind::Default => true,
                    _ => false,
                }
            }
            CbmlTypeKind::Struct { mut fields } => {
                //
                {
                    fields.sort_by(|x, y| {
                        let x_name = &x.0;
                        let y_name = &y.0;
                        return x_name.cmp(&y_name);
                    });
                }

                match found {
                    LiteralKind::Struct(asignment_stmts) => {
                        if asignment_stmts.len() != fields.len() {
                            // 结构体字面量数量不同,
                            // 还有这些 field 需要填写,
                            // 这些 field 没有定义.
                            // TODO:

                            return false;
                        }

                        // let mut asignment_stmts = asignment_stmts.clone();

                        let mut key_value_pairs: Vec<(String, LiteralKind)> = Vec::new();
                        {
                            for x in asignment_stmts {
                                key_value_pairs.push((x.field_name.clone(), x.value.kind.clone()));
                            }
                        }

                        key_value_pairs.sort_by(|x, y| {
                            let x_name = &x.0;
                            let y_name = &y.0;
                            return x_name.cmp(&y_name);
                        });

                        let did_it_same = fields.iter().zip(key_value_pairs).all(|(x, y)| {
                            let x_name = &x.0;
                            let y_name = &y.0;
                            x_name == y_name && self.is_same_type(&x.1, &y.1)
                        });

                        return did_it_same;
                    }
                    LiteralKind::Todo => {
                        // 不检查 todo.

                        return true;
                    }
                    LiteralKind::Default => todo!("自定义 struct 类型的默认值暂时还未支持"),

                    _ => false,
                }
            }
            CbmlTypeKind::Optional {
                inner_type,
                // span: _span,
            } => {
                return match found {
                    LiteralKind::LiteralNone => true,
                    _ => self.is_same_type(&inner_type, found),
                };
            }
            CbmlTypeKind::Union { allowed_values } => {
                allowed_values.contains(&found.to_cbml_value())
            }
            CbmlTypeKind::Enum { fields } => match found {
                LiteralKind::EnumFieldLiteral {
                    field_name,
                    literal,
                    ..
                } => {
                    // 检查 EnumFieldLiteral 的名字是否包含在 CbmlTypeKind::Enum fields 中.

                    for x in fields {
                        if &x.0 == field_name {
                            return self.is_same_type(&x.1, literal);
                        }
                    }

                    return false;
                }
                _ => false,
            },
            // CbmlTypeKind::Custom { name } => {
            //     // 1. get raw type from name
            //     let Some(custom_type) = self.custom_to_raw(&name) else {
            //         return false;
            //     };

            //     return self.is_same_type(&custom_type.clone(), literal);
            // }
        }
    }

    fn check_one_field_type(&self, field: &FieldAsign) -> Result<(), ParserError> {
        let Some(type_info) = self.get_field_defined_type(field) else {
            // 这个赋值的字段并未定义过.
            let e = ParserError::err_unknow_field(
                self.file_path.clone(),
                field.span.clone(),
                &field.name,
            );

            return Err(e);
        };

        // todo: 一次检查一个字段, 如果这个字段的类型是 结构体, 则逐一检查每一个字段,

        if self.is_same_type(&type_info.ty, &field.value.kind) {
            return Ok(());
        } else {
            let e = ParserError::err_mismatched_types(
                self.file_path.clone(),
                field.span.clone(),
                // &type_info.ty.to_cbml_code(0),
                &type_info.name,
                &field.value.kind.to_cbml_code(0),
            );

            return Err(e);
        }
    }

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
        }

        let safd = sadf.iter().fold(String::new(), |mut x, y| {
            x.push_str(&format!(" \"{}\" ", &y.name));
            x
        });
        let e = ParserError {
            file_path: file_path,
            msg: format!("还有 {} 个字段未赋值: {}", sadf.len(), safd),
            code_location: self.last_line_span.clone(),
            note: None,
            help: None,
        };
        self.errors.push(e);
    }

    /// 检查那些 赋值了却并未定义的字段.
    fn check_extra_field_asign(&mut self) {
        let Some(def_file) = &self.typedef_file else {
            return;
        };

        for x in &self.fields {
            let sadf = def_file.get_field_def_by_name(&x.name);
            match sadf {
                Some(f) => {
                    continue;
                }
                None => {
                    // 这个赋值的字段并未定义过.
                    let e = ParserError::err_unknow_field(
                        self.file_path.clone(),
                        x.span.clone(),
                        &x.name,
                    );
                    self.errors.push(e);
                }
            }
        }
    }
    // 缺失字段检查, 检查 struct 中定义了却没有赋值的字段.
    fn check_struct_field(&mut self) {}
}

impl CodeFile {
    /// 获取字段的定义.
    pub fn get_field_def(&self, field_name: &String) -> Option<&FieldDef> {
        let Some(def_file) = &self.typedef_file else {
            return None;
        };

        let asdf = def_file.fields.iter().find(|x| &x.name == field_name);

        return asdf;
    }

    /// 获取字段定义的类型.
    /// todo: 如何知道这个 field 是 top level field 还是某个结构体的字段?
    pub fn get_field_defined_type(&self, field: &FieldAsign) -> Option<&TypeInfo> {
        let Some(def_file) = &self.typedef_file else {
            return None;
        };

        let Some(field_def) = self.get_field_def(&field.name) else {
            return None;
        };

        return def_file.types.get(&field_def.type_sign);
    }

    /// 获取定义了却并未赋值的 top level field.
    pub fn get_unasigned_field(&mut self) -> Vec<&FieldDef> {
        let Some(def_file) = &self.typedef_file else {
            return Vec::new();
        };

        let mut unasigned_fields: Vec<&FieldDef> = Vec::new();

        let mut asigned_fields = self.fields.iter().map(|x| &x.name).collect::<HashSet<_>>();

        // 找出定义了却没有赋值的 top level 字段,
        {
            for x in def_file.get_all_top_fields() {
                let remoed = asigned_fields.remove(&x.name);
                if !remoed {
                    unasigned_fields.push(x);
                }
            }
        }

        return unasigned_fields;
    }
}

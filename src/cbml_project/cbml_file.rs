use super::def_cbml_file::DefCbmlFile;
use super::types::FieldAsign;
use super::types::FieldDef;
use super::types::ScopeID;
use super::types::TypeInfo;

use crate::ToCbml;
use crate::ToCbmlValue;
use crate::cbml_data::cbml_type::CbmlType;
use crate::cbml_data::cbml_value::CbmlValue;
use crate::lexer::token::*;
use crate::lexer::tokenize;
use crate::parser::CbmlParser;
use crate::parser::ast::stmt::*;
use crate::parser::parser_error::CbmlError;
use std::collections::HashMap;
use std::collections::HashSet;

/// 对一个 .cbml 文件的抽象.
#[derive(Debug, Clone)]
pub struct CbmlFile {
    /// 这个 .cbml 文件的位置.
    pub file_path: String,

    pub typedef_file: Option<DefCbmlFile>,

    /// top fields and child fields.
    pub fields: Vec<FieldAsign>,

    /// 检查到的错误.
    pub errors: Vec<CbmlError>,

    /// fn check_unasigned_field(&mut self) 会用到这个属性.
    last_line_span: Span,

    field_id: usize,
    /// 解析 ast 时记录正在解析的语句所在的 scope.
    _current_scope: Vec<ScopeID>,
}

impl CbmlFile {
    ///
    pub fn new(file_path: String) -> Self {
        let mut f = Self {
            file_path: file_path.clone(),
            fields: Vec::new(),
            // use_url: None,
            typedef_file: None,
            errors: Vec::new(),
            last_line_span: Span::empty(),
            field_id: 0,
            _current_scope: Vec::new(),
        };

        if (&file_path).ends_with(".def.cbml") {
            let e = CbmlError {
                error_code: 0000,
                file_path,
                msg: format!("以 .def.cbml 的是类型定义文件."),
                span: Span::empty(),
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

    /// 如果没有 file_path, 则使用空字符串: String::new().
    pub fn new_from(file_path: String, code: &str) -> Self {
        let mut f = Self {
            file_path: file_path.clone(),
            fields: Vec::new(),
            // use_url: None,
            typedef_file: None,
            errors: Vec::new(),
            last_line_span: Span::empty(),
            field_id: 0,
            _current_scope: Vec::new(),
        };

        if (&file_path).ends_with(".def.cbml") {
            let e = CbmlError {
                error_code: 0000,
                file_path,
                msg: format!("以 .def.cbml 的是类型定义文件."),
                span: Span::empty(),
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

    pub fn get_all_errors(&self) -> Vec<CbmlError> {
        let mut re: Vec<CbmlError> = vec![];

        re.extend_from_slice(&self.errors);

        return re;
    }

    /// 获取字段的定义.
    pub fn get_field_def(&self, field_name: &String, scope: ScopeID) -> Option<&FieldDef> {
        let Some(def_file) = &self.typedef_file else {
            return None;
        };

        let asdf = def_file.fields_map.get(&(field_name.clone(), scope));

        return asdf;
    }

    /// 获取字段定义的类型.
    pub fn get_field_defined_type(&self, field: &FieldAsign) -> Option<&TypeInfo> {
        let Some(field_def) = self.get_field_def(&field.name, field.scope.clone()) else {
            return None;
        };

        return Some(&field_def.type_);
    }

    /// 获取定义了却并未赋值的 top level field.
    pub fn get_unasigned_fields(&self) -> Vec<&FieldDef> {
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

    /// goto_difinition 的时候会用到.
    pub fn get_field_def_by_location(&self, line: u32, colunm: u32) -> Vec<&FieldDef> {
        let mut matchd_field_asign: Vec<&FieldAsign> = Vec::new();

        for x in &self.fields {
            if x.span.is_contain(line, colunm) {
                matchd_field_asign.push(x);
            }
        }

        let mut re: Vec<&FieldDef> = Vec::new();
        for x in matchd_field_asign {
            if let Some(def) = self.get_field_def(&x.name, x.scope.clone()) {
                re.push(def);
            };
        }

        return re;
    }

    fn kind_to_value(&self, f: FieldAsign) -> CbmlValue {
        match f.value.kind {
            LiteralKind::String(s) => CbmlValue::String(s),
            LiteralKind::Number(n) => CbmlValue::Number(n),
            LiteralKind::Boolean(b) => CbmlValue::Boolean(b),
            LiteralKind::Array(literals) => {
                CbmlValue::Array(literals.iter().map(|x| x.to_cbml_value()).collect())
            }
            LiteralKind::Struct(asignment_stmts) => {
                let mut fields: HashMap<String, CbmlValue> = HashMap::new();

                for x in asignment_stmts {
                    fields.insert(x.field_name.clone(), x.value.to_cbml_value());
                }

                return CbmlValue::Struct(fields);
            }
            LiteralKind::LiteralNone => CbmlValue::None,
            LiteralKind::EnumFieldLiteral {
                field_name,
                literal,
                ..
            } => CbmlValue::EnumField(field_name, Box::new(literal.to_cbml_value())),
        }
    }
}

impl CbmlFile {
    fn parse_file(&mut self, path: &str) {
        // use crate::parser::cbml_parser::CbmlParser;
        use std::fs::read_to_string;

        let code = read_to_string(path).unwrap();
        self.parse_code(&code);
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

        let lexer_result = tokenize(&self.file_path, &code);
        let tokens = lexer_result.tokens;
        // let tokens = match lexer_result {
        //     Ok(t) => t,
        //     Err(e) => {
        //         self.errors.push(e);
        //         return ();
        //     }
        // };

        // dp(format!("tokens: {:?}", tokens));

        let mut parser = CbmlParser::new(self.file_path.clone(), &tokens);
        let parser_result = parser.parse();

        if !parser_result.errors.is_empty() {
            self.errors.extend_from_slice(&parser_result.errors);
        }

        self.parse_ast(parser_result.ast);
        // match parser_result {
        //     Ok(ast) => {
        //         self.parse_ast(ast);
        //     }
        //     Err(mut errs) => {
        //         self.errors.append(&mut errs);
        //     }
        // };
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

        // self.use_url = Some(use_stmt.get_use_url());

        // 检测这个文件是否能打开.
        {
            let asdf = std::fs::File::open(use_stmt.get_use_url());
            match asdf {
                Ok(f) => _ = f.try_clone(),
                Err(e) => {
                    let e = CbmlError {
                        error_code: 0000,
                        file_path: use_stmt.get_use_url(),
                        msg: format!("{}", e),
                        span: Span::empty(),
                        note: None,
                        help: None,
                    };
                    self.errors.push(e);
                }
            }
        }

        let def_file = DefCbmlFile::new(use_stmt.get_use_url());

        // 错误检查.
        {
            if !def_file.errors.is_empty() {
                let e = CbmlError::err_use_imported_file_has_error(
                    self.file_path.clone(),
                    use_stmt,
                    def_file.errors.len(),
                );
                self.errors.push(e);
            }
        }

        self.typedef_file = Some(def_file);
    }

    fn parse_asignment(&mut self, asignment_stmt: crate::parser::ast::stmt::AsignmentStmt) {
        let value = FieldAsign {
            name: asignment_stmt.field_name.clone(),
            value: asignment_stmt.value.clone(),
            span: asignment_stmt.field_name_span,
            id: self.new_field_id(),
            scope: self.get_current_scope_id(),
        };

        self.fields.push(value);

        self.into_scope(ScopeID::new(asignment_stmt.field_name.clone()));
        self.parse_chile_fields(
            &asignment_stmt.value.kind,
            asignment_stmt.field_name.clone(),
        );
        self.outgoing_scope();
    }

    fn parse_chile_fields(&mut self, kind: &LiteralKind, field_name: String) {
        match kind {
            // LiteralKind::Array(literals) => todo!(),
            LiteralKind::Struct(asignment_stmts) => {
                for x in asignment_stmts {
                    self.parse_asignment(x.clone());
                }
            }
            LiteralKind::EnumFieldLiteral {
                field_name: enum_field_name,
                literal,
                ..
            } => {
                self.into_scope(ScopeID::new(enum_field_name.clone()));
                self.parse_chile_fields(&literal.kind, enum_field_name.clone());
                self.outgoing_scope();
            }
            LiteralKind::Array(literals) => {
                for x in literals {
                    self.parse_chile_fields(&x.kind, field_name.clone());
                }
            }

            _ => {}
        };
    }

    /// returns: (field_name, type)
    fn parse_struct_field_def(
        &mut self,
        struct_field_def_stmt: &crate::parser::ast::stmt::StructFieldDefStmt,
    ) {
        let e = CbmlError::err_field_def_not_allow_in_here(
            self.file_path.clone(),
            Span {
                start: struct_field_def_stmt.field_name_span.start.clone(),
                end: struct_field_def_stmt.end_span().end,
            },
        );
        self.errors.push(e);

        // let e = ParserError {
        //     error_code: 0000,
        //     file_path: self.file_path.clone(),
        //     msg: format!("字段定义在这里是不允许的."),
        //     span: Span {
        //         start: struct_field_def_stmt.field_name_span.start.clone(),
        //         end: struct_field_def_stmt.end_span().end,
        //     },
        //     note: Some(format!("")),

        //     help: Some(format!("将字段定义移动道 typedef 文件中.")),
        // };

        // self.errors.push(e);
    }

    fn parse_struct_def(&mut self, struct_def: &crate::parser::ast::stmt::StructDef) {
        let e = CbmlError {
            error_code: 0000,
            file_path: self.file_path.clone(),
            msg: format!("字段定义在这里是不允许的."),
            span: Span {
                start: struct_def.name_span.start.clone(),
                end: struct_def.end_span().end,
            },
            note: Some(format!("")),

            help: Some(format!("将字段定义移动道 typedef 文件中.")),
        };
        self.errors.push(e);
    }

    fn parse_enum_def(&mut self, enum_def: &crate::parser::ast::stmt::EnumDef) {
        let e = CbmlError {
            error_code: 0000,
            file_path: self.file_path.clone(),
            msg: format!("类型定义在这里是不允许的."),
            span: enum_def.name_span.clone(),
            note: Some(format!("")),

            help: Some(format!("将字段定义移动道 typedef 文件中.")),
        };

        self.errors.push(e);
    }

    fn parse_type_def(&mut self, type_def_stmt: &crate::parser::ast::stmt::TypeDefStmt) {
        let e = CbmlError {
            error_code: 0000,
            file_path: self.file_path.clone(),
            msg: format!("类型定义在这里是不允许的."),
            span: type_def_stmt.get_span(),
            note: Some(format!("")),

            help: Some(format!("将字段定义移动道 typedef 文件中.")),
        };

        self.errors.push(e);
    }

    fn new_field_id(&mut self) -> usize {
        self.field_id += 1;
        return self.field_id;
    }

    fn get_current_scope_id(&self) -> ScopeID {
        let mut re = String::new();

        for x in &self._current_scope {
            re.push_str("::");
            re.push_str(&x.0);
        }

        return ScopeID::new(re);
    }

    fn into_scope(&mut self, scope_id: ScopeID) {
        self._current_scope.push(scope_id);
    }

    fn outgoing_scope(&mut self) {
        let _ = self._current_scope.pop();
    }
}

// error check
impl CbmlFile {
    fn check_use(&mut self, use_stmt: &crate::parser::ast::stmt::UseStmt) -> Result<(), ()> {
        // 在 use 语句之前不能有 赋值语句.
        {
            if !self.fields.is_empty() {
                let e = CbmlError {
                    error_code: 0000,
                    file_path: self.file_path.clone(),
                    msg: format!("`use` 只能在文件的最开头."),
                    span: use_stmt.keyword_span.clone(),
                    note: None,
                    help: Some(format!("尝试将 `use` 移动到第一行")),
                };
                self.errors.push(e);
                return Err(());
            }
        };

        // use 语句只能使用一次.
        {
            // if self.use_url.is_some() || self.typedef_file.is_some() {
            if self.typedef_file.is_some() {
                let e = CbmlError::err_use_can_only_def_onece(
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
        self.check_duplicated_field_name();
        if let Some(def_file) = &self.typedef_file {
            if !def_file.errors.is_empty() {
                return;
            }
        } else {
            return;
        };

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

    #[allow(dead_code)]
    fn type_check_for_array(
        &mut self,
        inner_type: Box<CbmlType>,
        found: &LiteralKind,
        span: Span,
    ) -> Result<(), CbmlError> {
        //
        match found {
            LiteralKind::Array(literals) => {
                literals.iter().all(|x| self.is_same_type(&inner_type, x));

                for x in literals {
                    if !self.is_same_type(&inner_type, x) {};
                }

                Ok(())
            }
            // LiteralKind::Default => Ok(()),
            _ => {
                let e = CbmlError::err_mismatched_types(
                    self.file_path.clone(),
                    span,
                    &CbmlType::Array {
                        inner_type: inner_type,
                    }
                    // &CbmlType {
                    //     kind: CbmlTypeKind::Array {
                    //         inner_type: inner_type,
                    //     },
                    // }
                    .to_cbml(0),
                    &found.to_cbml(0),
                );
                Err(e)
            }
        }
    }

    // fn is_same_type(&self, need_type: &CbmlType, found: &LiteralKind) -> bool {
    fn is_same_type(&self, need_type: &CbmlType, found: &Literal) -> bool {
        let kind = &found.kind;
        // if let LiteralKind::Default = kind {
        //     return true;
        // }

        match need_type.clone() {
            CbmlType::String => match kind {
                LiteralKind::String { .. } => true,
                // LiteralKind::Default => true,
                _ => false,
            },
            CbmlType::Number => match kind {
                LiteralKind::Number(_) => true,
                // LiteralKind::Default => true,
                _ => false,
            },
            CbmlType::Bool => match kind {
                LiteralKind::Boolean(_) => true,
                // LiteralKind::Default => true,
                _ => false,
            },
            CbmlType::Any => true,
            CbmlType::Array { inner_type } => {
                // return self.type_check_for_array(inner_type, found);
                //
                match kind {
                    LiteralKind::Array(literals) => {
                        if literals.is_empty() {
                            return true;
                        }
                        return literals.iter().all(|x| self.is_same_type(&inner_type, x));
                    }
                    // LiteralKind::Default => true,
                    _ => false,
                }
            }
            CbmlType::Struct { mut fields } => {
                //
                {
                    fields.sort_by(|x, y| {
                        let x_name = &x.0;
                        let y_name = &y.0;
                        return x_name.cmp(&y_name);
                    });
                }

                match kind {
                    LiteralKind::Struct(asignment_stmts) => {
                        if asignment_stmts.len() != fields.len() {
                            // 结构体字面量数量不同,
                            // 还有这些 field 需要填写,
                            // 这些 field 没有定义.
                            // TODO:

                            return false;
                        }

                        // let mut asignment_stmts = asignment_stmts.clone();

                        let mut key_value_pairs: Vec<(String, Literal)> = Vec::new();
                        {
                            for x in asignment_stmts {
                                key_value_pairs.push((x.field_name.clone(), x.value.clone()));
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
                    // LiteralKind::Todo => {
                    //     // 不检查 todo.

                    //     return true;
                    // }
                    // LiteralKind::Default => todo!("自定义 struct 类型的默认值暂时还未支持"),
                    _ => false,
                }
            }
            CbmlType::Optional {
                inner_type,
                // span: _span,
            } => {
                return match kind {
                    LiteralKind::LiteralNone => true,
                    _ => self.is_same_type(&inner_type, found),
                };
            }
            CbmlType::Union { allowed_values } => allowed_values.contains(&found.to_cbml_value()),
            CbmlType::Enum { fields } => match kind {
                LiteralKind::EnumFieldLiteral {
                    field_name,
                    literal,
                    ..
                } => {
                    // 检查 EnumFieldLiteral 的名字是否包含在 CbmlType::Enum fields 中.

                    for x in fields {
                        if &x.0 == field_name {
                            return self.is_same_type(&x.1, literal);
                        }
                    }

                    return false;
                }
                _ => false,
            },
        }
    }

    fn check_one_field_type(&self, field: &FieldAsign) -> Result<(), CbmlError> {
        let Some(type_info) = self.get_field_defined_type(field) else {
            // 这个赋值的字段并未定义过.
            let e = CbmlError::err_unknow_field(
                self.file_path.clone(),
                field.span.clone(),
                // &field.name,
                &format!("name: {}, scope: {}", field.name, field.scope.0),
            );

            return Err(e);
        };

        // todo: 一次检查一个字段, 如果这个字段的类型是 结构体, 则逐一检查每一个字段,

        // note: 做类型检查时, 需要先确定 top level field 的类型, 里面的字段类型才能被确定.

        // 匹配 top level field 的类型.
        if self.is_same_type(&type_info.ty, &field.value) {
            return Ok(());
        } else {
            // 如果是结构体, 匹配里面每一个字段的类型, 找到出错的那个字段.

            let e = CbmlError::err_mismatched_types(
                self.file_path.clone(),
                field.value.span.clone(),
                &type_info.ty.to_cbml(0),
                &field.value.kind.to_cbml(0),
            );

            return Err(e);
        }
    }

    // 字段重复检查, 一个字段只需要赋值一次.
    fn check_duplicated_field_name(&mut self) {
        let mut seen: HashSet<(String, ScopeID)> = HashSet::new();
        let mut duplicates: Vec<&FieldAsign> = Vec::new();

        for x in &self.fields {
            if !seen.insert((x.name.clone(), x.scope.clone())) {
                duplicates.push(&x);
            }
        }

        let errors: Vec<CbmlError> = duplicates
            .iter()
            .map(|x| {
                // ParserError::err_field_alredy_exits(self.file_path.clone(), x.span.clone(), &x.name)
                CbmlError::err_field_alredy_exits(
                    self.file_path.clone(),
                    x.span.clone(),
                    &format!("name: {}, scope: {}", x.name, x.scope.0),
                )
            })
            .collect();

        self.errors.extend(errors);
    }

    // 缺失字段检查, 检查定义了却没有赋值的 top level 字段,
    fn check_unasigned_field(&mut self) {
        let file_path = self.file_path.clone();

        let span = self.last_line_span.clone();
        let unasigned_field = self.get_unasigned_fields();
        if unasigned_field.is_empty() {
            return;
        }

        let e = CbmlError::err_has_fields_unasigned(file_path, unasigned_field, span);
        self.errors.push(e);
    }

    /// 检查那些 赋值了却并未定义的字段.
    #[allow(dead_code)]
    fn check_extra_field_asign(&mut self) {
        let Some(def_file) = &self.typedef_file else {
            return;
        };

        for x in &self.fields {
            let sadf = def_file.get_field_def_by_name(x.name.clone(), x.scope.clone());
            match sadf {
                Some(_f) => {
                    continue;
                }
                None => {
                    // 这个赋值的字段并未定义过.
                    let e = CbmlError::err_unknow_field(
                        self.file_path.clone(),
                        x.span.clone(),
                        &x.name,
                    );
                    self.errors.push(e);
                }
            }
        }
    }
    /// 缺失字段检查, 检查 struct 中定义了却没有赋值的字段.
    #[allow(dead_code)]
    fn check_struct_field(&mut self) {}
}

impl ToCbml for CbmlFile {
    fn to_cbml(&self, deepth: usize) -> String {
        let mut re = String::new();

        for x in &self.fields {
            re.push_str(&x.to_cbml(deepth));
            re.push_str("\n");
        }

        return re;
    }
}

impl ToCbmlValue for CbmlFile {
    fn to_cbml_value(&self) -> CbmlValue {
        let mut root: HashMap<String, CbmlValue> = HashMap::new();

        let root_id = ScopeID::new(String::new());
        let top_fields: Vec<&FieldAsign> =
            self.fields.iter().filter(|x| x.scope == root_id).collect();

        for x in top_fields {
            let re = self.kind_to_value(x.clone());
            root.insert(x.name.clone(), re);
        }

        return CbmlValue::Struct(root);
    }
}

/// 将 .cbml 转换为对应编程语言的数据字面量.
impl CbmlFile {
    pub fn generate_rust_data(&self) -> String {
        todo!()
    }
}

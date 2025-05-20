use super::types::*;
use crate::cbml_data::cbml_type::{CbmlType, CbmlTypeKind};
use crate::cbml_data::cbml_value::*;
use crate::lexer::token::Span;
use crate::lexer::tokenize;
use crate::parser::CbmlParser;
use crate::parser::ast::stmt::*;
use crate::parser::parser_error::ParserError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TypedefFile {
    pub file_path: String,

    pub fields_map: HashMap<(String, ScopeID), FieldDef>,

    pub errors: Vec<ParserError>,

    /// 解析 ast 时记录正在解析的语句所在的 scope.
    _current_scope: Vec<ScopeID>,

    // count: usize,
    _type_id: usize,
}

impl TypedefFile {
    pub fn new(file_path: String) -> Self {
        let mut f = Self {
            file_path: file_path.clone(),

            errors: Vec::new(),
            _current_scope: Vec::new(),
            // count: 0,
            fields_map: HashMap::new(),
            _type_id: 0,
        };

        if file_path.ends_with(".def.cbml") {
            f.parse_file(&file_path);
        } else {
            let e = ParserError {
                file_path,
                msg: format!("类型定义文件的文件名需要以 .def.cbml 结尾."),
                span: Span::empty(),
                note: None,
                help: None,
                error_code: 0000,
            };

            f.errors.push(e);
        }

        return f;
    }

    pub fn new_from(file_path: String, code: &str) -> Self {
        let mut f = Self {
            file_path: file_path.clone(),
            // types: HashMap::new(),
            // fields: Vec::new(),
            errors: Vec::new(),
            _current_scope: Vec::new(),
            // count: 0,
            fields_map: HashMap::new(),
            _type_id: 0,
        };

        if file_path.ends_with(".def.cbml") {
            f.parse_code(code);
        } else {
            let e = ParserError {
                error_code: 0000,
                file_path,
                msg: format!("类型定义文件的文件名需要以 .def.cbml 结尾."),
                span: Span::empty(),
                note: None,
                help: None,
            };

            f.errors.push(e);
        }

        return f;
    }

    pub fn get_field_def_by_name(&self, name: String, scope: ScopeID) -> Option<&FieldDef> {
        let key = (name, scope);
        self.fields_map.get(&key)
    }

    /// goto_difinition 的时候会用到.
    pub fn get_field_def_by_location(&self, line: u32, colunm: u32) -> Vec<&FieldDef> {
        let mut matchd_field_asign: Vec<&FieldDef> = Vec::new();

        for (_, x) in &self.fields_map {
            if x.span.is_contain(line, colunm) {
                matchd_field_asign.push(x);
            }
        }

        // let mut re: Vec<&FieldDef> = Vec::new();
        // for x in matchd_field_asign {
        //     if let Some(def) = self.get_field_def(&x.name, x.scope.clone()) {
        //         re.push(def);
        //     };
        // }

        return matchd_field_asign;
    }

    pub fn get_all_top_fields(&self) -> Vec<&FieldDef> {
        let top_scope = ScopeID::new(String::new());
        self.fields_map
            .iter()
            .filter(|x| x.1.scope == top_scope)
            .map(|x| x.1)
            .collect()
    }
}
impl TypedefFile {
    fn parse_file(&mut self, path: &str) {
        use std::fs::read_to_string;

        match read_to_string(path) {
            Ok(code) => {
                self.parse_code(&code);
            }
            Err(e) => {
                let e = ParserError {
                    error_code: 0000,
                    file_path: path.to_string(),
                    msg: format!("{:?}", e),
                    span: Span::empty(),
                    note: None,
                    help: None,
                };
                self.errors.push(e);
            }
        };
    }

    fn parse_code(&mut self, code: &str) {
        let path = &self.file_path;

        let lexer_result = tokenize(path, &code);

        self.errors.extend(lexer_result.errors);
        let tokens = lexer_result.tokens;

        let mut parser = CbmlParser::new(path.to_string(), &tokens);
        let parser_result = parser.parse();

        if !parser_result.errors.is_empty() {
            self.errors.extend_from_slice(&parser_result.errors);
        }

        self.parse_ast(parser_result.ast);
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

    fn parse_ast(&mut self, ast: Vec<Stmt>) {
        // 先分成两部分, top level field def 和 type def.

        let mut top_fields_def: Vec<Stmt> = Vec::new();
        let mut type_def: Vec<Stmt> = Vec::new();

        for s in ast {
            match s.kind {
                crate::parser::StmtKind::FileFieldStmt(_) => top_fields_def.push(s),
                // crate::parser::StmtKind::TypeDef(_) => type_def.push(s),
                _ => type_def.push(s),
            }

            // self.parse_one_stmt(s);
        }

        // 先解析类型定义
        for x in type_def {
            self.parse_one_stmt(x);
        }

        // 在解析字段定义.
        for x in top_fields_def {
            self.parse_one_stmt(x);
        }
    }

    fn parse_one_stmt(&mut self, s: Stmt) {
        match s.kind {
            crate::parser::StmtKind::Use(use_stmt) => self.parse_use(use_stmt),

            crate::parser::StmtKind::Asignment(asignment_stmt) => {
                self.parse_asignment(asignment_stmt)
            }

            crate::parser::StmtKind::FileFieldStmt(struct_field_def_stmt) => {
                self.parse_struct_field_def(struct_field_def_stmt);
            }
            crate::parser::StmtKind::TypeAliasStmt(_) => {}

            crate::parser::StmtKind::StructDefStmt(struct_def) => self.parse_struct_def(struct_def),

            crate::parser::StmtKind::EnumDef(enum_def) => self.parse_enum_def(enum_def),

            // crate::parser::StmtKind::UnionDef(union_def) => self.parse_union_def(union_def),
            crate::parser::StmtKind::TypeDef(type_def_stmt) => self.parse_type_def(type_def_stmt),
            crate::parser::StmtKind::LineComment(_) => {}
            crate::parser::StmtKind::BlockComment(_) => {}
            crate::parser::StmtKind::DocComment(_) => {}
            crate::parser::StmtKind::EmptyLine => {}
        };
    }

    fn parse_use(&mut self, use_stmt: UseStmt) {
        let e = ParserError {
            error_code: 0000,
            file_path: self.file_path.clone(),
            msg: format!("不能类型定义文件中使用 use 语句."),
            span: use_stmt.keyword_span,
            note: None,
            help: None,
        };
        self.errors.push(e);
    }

    fn parse_asignment(&mut self, a: AsignmentStmt) {
        let e = ParserError {
            error_code: 0000,
            file_path: self.file_path.clone(),
            msg: format!("不能在类型定义文件中给用字段赋值."),
            span: a.field_name_span,
            note: None,
            help: None,
        };
        self.errors.push(e);
    }

    fn parse_struct_field_def(
        &mut self,
        struct_field_def_stmt: crate::parser::ast::stmt::StructFieldDefStmt,
    ) -> (String, CbmlType) {
        let span = struct_field_def_stmt.get_span();
        let type_sign_span = struct_field_def_stmt._type.span.clone();

        let ty: CbmlType = self.parse_type_sign_stmt(
            struct_field_def_stmt._type,
            &struct_field_def_stmt.field_name,
        );

        let info = TypeInfo {
            ty: ty.clone(),
            span: type_sign_span,
            type_id: self.gen_type_id(),
        };

        //  struct_field_def_stmt.doc.unwrap().document;

        let filed_def = FieldDef {
            name: struct_field_def_stmt.field_name.clone(),
            default_value: struct_field_def_stmt.default.clone(),
            span: span,
            scope: self.get_current_scope_id(),
            type_: info,
            doc: struct_field_def_stmt.doc.map(|x| x.document),
        };

        let key = (filed_def.name.clone(), self.get_current_scope_id());
        self.fields_map.insert(key, filed_def);

        return (struct_field_def_stmt.field_name.clone(), ty);
    }

    fn parse_struct_def(&mut self, struct_def: crate::parser::ast::stmt::StructDef) {
        // struct_def

        let mut adsfsadf: Vec<(String, CbmlType)> = Vec::new();

        self.into_scope(ScopeID::new(struct_def.struct_name.clone()));
        for x in struct_def.fields {
            let (field_name, field_type) = self.parse_struct_field_def(x);
            adsfsadf.push((field_name, field_type));
        }
        self.outgoing_scope();

        // let struct_type = CbmlType {
        //     kind: CbmlTypeKind::Struct { fields: adsfsadf },
        // };
        // let asdf = self.gen_type_id();
        // self.insert_type(
        //     struct_def.struct_name.clone(),
        //     TypeInfo {
        //         name: struct_def.struct_name.clone(),
        //         ty: struct_type,
        //         span: struct_def.name_span.clone(),
        //         type_id: asdf,
        //     },
        // );
    }

    fn gen_type_id(&mut self) -> usize {
        self._type_id += 1;
        self._type_id
    }

    fn parse_enum_def(&mut self, enum_def: EnumDef) {
        let mut adsfsadf: Vec<(String, CbmlType)> = Vec::new();

        self.into_scope(ScopeID::new(enum_def.enum_name.clone()));
        for x in enum_def.fields {
            let ty = self.parse_type_sign_stmt(x._type, &x.field_name);
            adsfsadf.push((x.field_name.clone(), ty));
        }
        self.outgoing_scope();

        // let enum_type = CbmlType {
        //     kind: CbmlTypeKind::Enum { fields: adsfsadf },
        // };

        // let sadf = self.gen_type_id();
        // self.insert_type(
        //     enum_def.enum_name.clone(),
        //     TypeInfo {
        //         name: enum_def.enum_name.clone(),
        //         ty: enum_type,
        //         span: enum_def.name_span.clone(),
        //         type_id: sadf,
        //     },
        // );
    }

    fn parse_enum_field_def(&mut self, enum_field_def: EnumFieldDef) -> (String, CbmlType) {
        let ty = self.parse_type_sign_stmt(
            enum_field_def._type,
            &enum_field_def.field_name,
            // enum_field_def.field_name_span.clone(),
        );

        let info = TypeInfo {
            // name: type_name.clone(),
            ty: ty.clone(),
            span: enum_field_def.field_name_span.clone(),
            type_id: self.gen_type_id(),
        };

        let field_def = FieldDef {
            name: enum_field_def.field_name.clone(),
            default_value: None,
            span: enum_field_def.field_name_span.clone(),
            scope: self.get_current_scope_id(),
            type_: info,
            doc: None,
        };

        // self.fields.push(d.clone());
        let key = (field_def.name.clone(), self.get_current_scope_id());
        self.fields_map.insert(key, field_def);

        return (enum_field_def.field_name.clone(), ty);
    }

    fn parse_union_def(&mut self, union_def: UnionDef) {
        // let union_name = union_def.union_name.clone();
        // let union_span = union_def.name_span.clone();

        let mut alowd_values: Vec<CbmlValue> = Vec::new();
        for x in &union_def.allowed_values {
            alowd_values.push(x.to_cbml_value());
        }

        // let union_type = CbmlType {
        //     kind: CbmlTypeKind::Union {
        //         allowed_values: alowd_values,
        //     },
        // };

        // let sadf = self.gen_type_id();
        // self.insert_type(
        //     union_name.clone(),
        //     TypeInfo {
        //         name: union_name.clone(),
        //         ty: union_type.clone(),
        //         span: union_span,
        //         type_id: sadf,
        //     },
        // );
    }

    // return: (type_name, CbmlTYpe)
    fn parse_type_sign_stmt(
        &mut self,
        // sign: TypeSignStmtKind,
        sign: TypeSignStmt,
        field_name: &str,
        // span: Span,
        // ) -> (String, CbmlType) {
    ) -> CbmlType {
        // let _ = span;
        let span = sign.span;

        match sign.kind {
            crate::parser::ast::stmt::TypeSignStmtKind::String => CbmlType {
                kind: CbmlTypeKind::String,
            },
            crate::parser::ast::stmt::TypeSignStmtKind::Number => CbmlType {
                kind: CbmlTypeKind::Number,
            },
            crate::parser::ast::stmt::TypeSignStmtKind::Boolean => CbmlType {
                kind: CbmlTypeKind::Bool,
            },
            crate::parser::ast::stmt::TypeSignStmtKind::Any => CbmlType {
                kind: CbmlTypeKind::Any,
            },

            crate::parser::ast::stmt::TypeSignStmtKind::Anonymous(anonymous_type_def_stmt) => {
                let a = self.parse_anonymous_type_def_stmt(anonymous_type_def_stmt, field_name);
                return a;
            }
            crate::parser::ast::stmt::TypeSignStmtKind::Custom(_custom_type_namee) => {
                //
                let e = ParserError {
                    error_code: 0000,
                    file_path: self.file_path.clone(),
                    msg: format!("unkonw type: {}", _custom_type_namee),
                    span,
                    note: None,
                    help: None,
                };
                self.errors.push(e);

                return CbmlType {
                    kind: CbmlTypeKind::Any,
                };
            }
        }
    }

    // return: (type_name, CbmlType)
    fn parse_anonymous_type_def_stmt(
        &mut self,
        anonymous_type_def_stmt: AnonymousTypeDefStmt,
        field_name: &str,
    ) -> CbmlType {
        // let anony_span = anonymous_type_def_stmt.span.clone();

        match anonymous_type_def_stmt.kind {
            crate::parser::ast::stmt::AnonymousTypeDefKind::Array { inner_type } => {
                let ty = self.parse_type_sign_stmt(
                    *inner_type,
                    // &format!("{}{}", field_name, "array_inner"),
                    field_name,
                    // anony_span.clone(),
                );

                let array_type = CbmlType {
                    kind: CbmlTypeKind::Array {
                        inner_type: ty.clone().into(),
                    },
                };

                return array_type;
            }
            crate::parser::ast::stmt::AnonymousTypeDefKind::Enum { fields } => {
                let mut fieasdfasflds: Vec<(String, CbmlType)> = Vec::new();

                self.into_scope(ScopeID::new(field_name.to_string()));
                for x in fields {
                    let sadf = self.parse_enum_field_def(x);
                    fieasdfasflds.push(sadf);
                }
                self.outgoing_scope();

                return CbmlType {
                    kind: CbmlTypeKind::Enum {
                        fields: fieasdfasflds,
                    },
                };
            }
            crate::parser::ast::stmt::AnonymousTypeDefKind::Struct(struct_field_def_stmts) => {
                let mut adsfsadf: Vec<(String, CbmlType)> = Vec::new();

                self.into_scope(ScopeID::new(field_name.to_string()));
                for x in struct_field_def_stmts {
                    let ty = self.parse_type_sign_stmt(x._type.clone(), &x.field_name);
                    adsfsadf.push((x.field_name.clone(), ty));

                    // 存储字段.
                    _ = self.parse_struct_field_def(x);
                }
                self.outgoing_scope();

                let struct_type = CbmlType {
                    kind: CbmlTypeKind::Struct { fields: adsfsadf },
                };

                return struct_type;
            }
            crate::parser::ast::stmt::AnonymousTypeDefKind::Union { alowd_values } => {
                // let anony_union_name = self.auto_gen_type_name(field_name);
                let _anony_union_name: String = String::new();

                let union_type = CbmlType {
                    kind: CbmlTypeKind::Union {
                        allowed_values: alowd_values.clone(),
                    },
                };

                return union_type;
            }
            crate::parser::ast::stmt::AnonymousTypeDefKind::Optional { inner_type } => {
                let ty = self.parse_type_sign_stmt(
                    *inner_type,
                    &format!("{}_{}", field_name, "optional"),
                    // anony_span.clone(),
                );

                let optional_type = CbmlType {
                    kind: CbmlTypeKind::Optional {
                        inner_type: ty.clone().into(),
                    },
                };

                return optional_type;
            }
        }
    }

    fn parse_type_def(&mut self, type_def_stmt: TypeDefStmt) {
        use crate::parser::ast::stmt::TypeDefStmt;

        match type_def_stmt {
            TypeDefStmt::StructDefStmt(struct_def) => self.parse_struct_def(struct_def),
            TypeDefStmt::EnumDef(enum_def) => self.parse_enum_def(enum_def),
            TypeDefStmt::UnionDef(union_def) => self.parse_union_def(union_def),
        }
    }
}

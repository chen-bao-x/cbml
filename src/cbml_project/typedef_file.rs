use std::clone;
use std::collections::HashMap;
use std::hash::Hash;

use crate::lexer::tokenizer;
use crate::typecheck::types_for_check::ScopeID;

use crate::cbml_value::value::CbmlType;
use crate::cbml_value::value::CbmlTypeKind;
use crate::cbml_value::value::CbmlValue;
use crate::cbml_value::value::ToCbmlValue;
use crate::lexer::token::Span;
use crate::parser::CbmlParser;
use crate::parser::ast::stmt::AnonymousTypeDefStmt;
use crate::parser::ast::stmt::AsignmentStmt;
use crate::parser::ast::stmt::Literal;
use crate::parser::ast::stmt::Stmt;
use crate::parser::ast::stmt::TypeSignStmt;
use crate::parser::ast::stmt::TypeSignStmtKind;
use crate::parser::ast::stmt::UseStmt;
use crate::parser::parser_error::ParserError;

use super::types::FieldDef;
use super::types::TypeInfo;
#[derive(Debug, Clone)]
pub struct TypedefFile {
    pub file_path: String,
    /// key: type_name
    pub types: HashMap<String, TypeInfo>,

    ///
    pub fields: Vec<FieldDef>,

    pub errors: Vec<ParserError>,

    fields_map: HashMap<String, usize>,

    /// 解析 ast 时记录正在解析的语句所在的 scope.
    current_scope: Vec<ScopeID>,

    count: usize,
}

impl TypedefFile {
    pub fn new(file_path: String) -> Self {
        let mut f = Self {
            file_path: file_path.clone(),
            types: HashMap::new(),
            fields: Vec::new(),
            errors: Vec::new(),
            current_scope: Vec::new(),
            count: 0,
            fields_map: HashMap::new(),
        };

        if file_path.ends_with(".def.cbml") {
            f.parse_file(&file_path);
        } else {
            let e = ParserError {
                file_path,
                msg: format!("类型定义文件的文件名需要以 .def.cbml 结尾."),
                code_location: Span::empty(),
                note: None,
                help: None,
            };

            f.errors.push(e);
        }

        return f;
    }

    pub fn new_from(file_path: String, code: &str) -> Self {
        let mut f = Self {
            file_path: file_path.clone(),
            types: HashMap::new(),
            fields: Vec::new(),
            errors: Vec::new(),
            current_scope: Vec::new(),
            count: 0,
            fields_map: HashMap::new(),
        };

        if file_path.ends_with(".def.cbml") {
            f.parse_code(code);
        } else {
            let e = ParserError {
                file_path,
                msg: format!("类型定义文件的文件名需要以 .def.cbml 结尾."),
                code_location: Span::empty(),
                note: None,
                help: None,
            };

            f.errors.push(e);
        }

        return f;
    }

    pub fn get_field_def_by_name(&self, name: &str) -> Option<&FieldDef> {
        let Some(sdf) = self.fields_map.get(name) else {
            return None;
        };

        self.fields.get(*sdf)
    }

    fn parse_file(&mut self, path: &str) {
        use crate::parser::cbml_parser::CbmlParser;
        use std::fs::read_to_string;

        let code = read_to_string(path).unwrap();
        self.parse_code(&code);
    }

    fn parse_code(&mut self, code: &str) {
        let path = &self.file_path;

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
        }
    }

    fn insert_type(&mut self, name: String, ty: TypeInfo) {
        match &ty.ty.kind {
            CbmlTypeKind::String
            | CbmlTypeKind::Number
            | CbmlTypeKind::Bool
            | CbmlTypeKind::Any => {}
            _ => {
                let asdf = self.types.insert(name.clone(), ty);
                match asdf {
                    Some(v) => {
                        dbg!(v);
                        dbg!(self.types.get(&name));
                    }
                    None => {}
                };
            }
        }
    }

    fn get_current_scope_id(&self) -> ScopeID {
        let mut re = String::new();

        re.push_str(&self.file_path);

        for x in &self.current_scope {
            re.push_str("::");
            re.push_str(&x.0);
        }

        return ScopeID::new(re);
    }

    fn into_scope(&mut self, scope_id: ScopeID) {
        self.current_scope.push(scope_id);
    }

    fn outgoing_scope(&mut self) {
        let _ = self.current_scope.pop();
    }

    /// 匿名类型自动生成类型名字.
    ///  name: { }
    /// 自动生成的名字就是: "path/to/deffiel.def.cbml::anonymous_type_for_name"
    /// field.scope + anonymous_type_for_ + field.name
    fn gen_anonymous_type_name(current_scope: &ScopeID, field_name: &str) -> String {
        let mut re = String::new();

        re.push_str(&current_scope.0);
        re.push_str("::");

        re.push_str("anonymous_type_for_");
        re.push_str(field_name);

        return re;
    }

    fn auto_gen_type_name(&mut self, field_name: &str) -> String {
        self.count += 1;
        format!("{}", self.count)
        // Self::gen_anonymous_type_name(&self.get_current_scope_id(), field_name)
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
        match &s.kind {
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

    fn parse_use(&mut self, use_stmt: &UseStmt) {
        let e = ParserError {
            file_path: self.file_path.clone(),
            msg: format!("不能类型定义文件中使用 use 语句."),
            code_location: use_stmt.keyword_span.clone(),
            note: None,
            help: None,
        };
        self.errors.push(e);
    }

    fn parse_asignment(&mut self, a: &AsignmentStmt) {
        let e = ParserError {
            file_path: self.file_path.clone(),
            msg: format!("不能在类型定义文件中给用字段赋值."),
            code_location: a.field_name_span.clone(),
            note: None,
            help: None,
        };
        self.errors.push(e);
    }

    fn parse_struct_field_def(
        &mut self,
        struct_field_def_stmt: &crate::parser::ast::stmt::StructFieldDefStmt,
    ) -> (String, CbmlType) {
        let (type_name, ty) = self.parse_type_sign_stmt(
            &struct_field_def_stmt._type.kind,
            &struct_field_def_stmt.field_name,
            Span {
                start: struct_field_def_stmt.field_name_span.start.clone(),
                end: struct_field_def_stmt.end_span().end,
            },
        );

        // struct_field_def_stmt._type;
        let filed_def = FieldDef {
            name: struct_field_def_stmt.field_name.clone(),
            type_sign: type_name,
            default_value: struct_field_def_stmt.default.clone(),
            span: Span {
                start: struct_field_def_stmt.field_name_span.start.clone(),
                end: struct_field_def_stmt.end_span().end,
            },
            scope: self.get_current_scope_id(),
        };

        self.fields.push(filed_def);
        // cache field in index map.
        {
            let last_index: usize = if self.fields.len() == 0 {
                0
            } else {
                self.fields.len() - 1
            };

            self.fields_map
                .insert(struct_field_def_stmt.field_name.clone(), last_index);
        }

        return (struct_field_def_stmt.field_name.clone(), ty);
    }

    fn parse_struct_def(&mut self, struct_def: &crate::parser::ast::stmt::StructDef) {
        // struct_def

        let mut adsfsadf: Vec<(String, CbmlType)> = Vec::new();

        self.into_scope(ScopeID::new(struct_def.struct_name.clone()));
        for x in &struct_def.fields {
            let (field_name, field_type) = self.parse_struct_field_def(x);
            adsfsadf.push((field_name, field_type));
        }
        self.outgoing_scope();

        let struct_type = CbmlType {
            kind: CbmlTypeKind::Struct { fields: adsfsadf },
        };

        self.insert_type(
            struct_def.struct_name.clone(),
            TypeInfo {
                name: struct_def.struct_name.clone(),
                ty: struct_type,
                span: struct_def.name_span.clone(),
            },
        );
    }

    fn parse_enum_def(&mut self, enum_def: &crate::parser::ast::stmt::EnumDef) {
        let mut adsfsadf: Vec<(String, CbmlType)> = Vec::new();

        self.into_scope(ScopeID::new(enum_def.enum_name.clone()));
        for x in &enum_def.fields {
            let (_, ty) =
                self.parse_type_sign_stmt(&x._type.kind, &x.field_name, enum_def.name_span.clone());
            adsfsadf.push((x.field_name.clone(), ty));
        }
        self.outgoing_scope();

        let enum_type = CbmlType {
            kind: CbmlTypeKind::Enum { fields: adsfsadf },
        };

        self.insert_type(
            enum_def.enum_name.clone(),
            TypeInfo {
                name: enum_def.enum_name.clone(),
                ty: enum_type,
                span: enum_def.name_span.clone(),
            },
        );
    }

    fn parse_enum_field_def(&mut self, enum_field_def: &crate::parser::ast::stmt::EnumFieldDef) {
        let (type_name, ty) = self.parse_type_sign_stmt(
            &enum_field_def._type.kind,
            &enum_field_def.field_name,
            enum_field_def.field_name_span.clone(),
        );

        let d = FieldDef {
            name: enum_field_def.field_name.clone(),
            type_sign: type_name,
            default_value: None,
            span: enum_field_def.field_name_span.clone(),
            scope: self.get_current_scope_id(),
        };

        self.fields.push(d);
    }

    fn parse_union_def(&mut self, union_def: &crate::parser::ast::stmt::UnionDef) {
        let union_name = union_def.union_name.clone();
        let union_span = union_def.name_span.clone();

        let mut alowd_values: Vec<CbmlValue> = Vec::new();
        for x in &union_def.allowed_values {
            alowd_values.push(x.to_cbml_value());
        }

        let union_type = CbmlType {
            kind: CbmlTypeKind::Union {
                allowed_values: alowd_values,
            },
        };

        self.insert_type(
            union_name.clone(),
            TypeInfo {
                name: union_name.clone(),
                ty: union_type.clone(),
                span: union_span,
            },
        );
    }

    // return: (type_name, CbmlTYpe)
    fn parse_type_sign_stmt(
        &mut self,
        sign: &TypeSignStmtKind,
        field_name: &str,
        span: Span,
    ) -> (String, CbmlType) {
        match sign {
            crate::parser::ast::stmt::TypeSignStmtKind::String => (
                "string".to_string(),
                CbmlType {
                    kind: CbmlTypeKind::String,
                },
            ),
            crate::parser::ast::stmt::TypeSignStmtKind::Number => (
                "number".to_string(),
                CbmlType {
                    kind: CbmlTypeKind::Number,
                },
            ),
            crate::parser::ast::stmt::TypeSignStmtKind::Boolean => (
                "bool".to_string(),
                CbmlType {
                    kind: CbmlTypeKind::Bool,
                },
            ),
            crate::parser::ast::stmt::TypeSignStmtKind::Any => (
                "any".to_string(),
                CbmlType {
                    kind: CbmlTypeKind::Any,
                },
            ),
            // crate::parser::ast::stmt::TypeSignStmtKind::Array { inner_type } => {
            //     todo!();
            // }
            // crate::parser::ast::stmt::TypeSignStmtKind::Struct(struct_field_def_stmts) => todo!(),
            // crate::parser::ast::stmt::TypeSignStmtKind::Optional { inner_type } => todo!(),
            crate::parser::ast::stmt::TypeSignStmtKind::Anonymous(anonymous_type_def_stmt) => {
                self.parse_anonymous_type_def_stmt(anonymous_type_def_stmt, field_name)
            }
            crate::parser::ast::stmt::TypeSignStmtKind::Custom(custom_type_name) => {
                let sadf = self.types.get(custom_type_name);
                match sadf {
                    Some(ty_info) => return (custom_type_name.to_string(), ty_info.ty.clone()),
                    None => {
                        let e = ParserError {
                            file_path: self.file_path.clone(),
                            msg: format!("connot find type {}", custom_type_name),
                            note: None,
                            help: None,
                            code_location: span,
                        };

                        self.errors.push(e);

                        return (
                            custom_type_name.to_string(),
                            CbmlType {
                                kind: CbmlTypeKind::Any,
                            },
                        );
                    }
                }

                // sadf.unwrap().ty
                // return (custom_type_name.to_string(), sadf.unwrap().ty.clone());
                // todo!();
            }
        }
    }

    // return: (type_name, CbmlType)
    fn parse_anonymous_type_def_stmt(
        &mut self,
        anonymous_type_def_stmt: &crate::parser::ast::stmt::AnonymousTypeDefStmt,
        field_name: &str,
    ) -> (String, CbmlType) {
        let anony_span = anonymous_type_def_stmt.span.clone();

        match &anonymous_type_def_stmt.kind {
            crate::parser::ast::stmt::AnonymousTypeDefKind::Array { inner_type } => {
                let (_, ty) = self.parse_type_sign_stmt(
                    inner_type.as_ref(),
                    &format!("{}{}", field_name, "array_inner"),
                    anony_span.clone(),
                );

                let array_type = CbmlType {
                    kind: CbmlTypeKind::Array {
                        inner_type: ty.clone().into(),
                    },
                };

                let array_name = self.auto_gen_type_name(field_name);
                let ty_info = TypeInfo {
                    name: array_name.clone(),
                    ty: array_type.clone(),
                    span: anony_span,
                };

                self.insert_type(array_name.clone(), ty_info);
                return (array_name, array_type);
            }
            crate::parser::ast::stmt::AnonymousTypeDefKind::Enum { fields } => {
                todo!()
            }
            crate::parser::ast::stmt::AnonymousTypeDefKind::Struct(struct_field_def_stmts) => {
                let mut adsfsadf: Vec<(String, CbmlType)> = Vec::new();

                self.into_scope(ScopeID::new(field_name.to_string()));
                for x in struct_field_def_stmts {
                    let (_, ty) =
                        self.parse_type_sign_stmt(&x._type.kind, &x.field_name, anony_span.clone());
                    adsfsadf.push((x.field_name.clone(), ty));
                }
                self.outgoing_scope();

                let struct_type = CbmlType {
                    kind: CbmlTypeKind::Struct { fields: adsfsadf },
                };

                let anony_struct_name = self.auto_gen_type_name(field_name);
                self.insert_type(
                    anony_struct_name.clone(),
                    TypeInfo {
                        name: anony_struct_name.clone(),
                        ty: struct_type.clone(),
                        span: anony_span,
                    },
                );

                return (anony_struct_name, struct_type);
            }
            crate::parser::ast::stmt::AnonymousTypeDefKind::Union { alowd_values } => {
                let anony_union_name = self.auto_gen_type_name(field_name);

                let union_type = CbmlType {
                    kind: CbmlTypeKind::Union {
                        allowed_values: alowd_values.clone(),
                    },
                };

                self.insert_type(
                    anony_union_name.clone(),
                    TypeInfo {
                        name: anony_union_name.clone(),
                        ty: union_type.clone(),
                        span: anony_span,
                    },
                );

                return (anony_union_name, union_type);
            }
            crate::parser::ast::stmt::AnonymousTypeDefKind::Optional { inner_type } => {
                let anony_optional_name = self.auto_gen_type_name(field_name);
                let (_, ty) = self.parse_type_sign_stmt(
                    inner_type.as_ref(),
                    &format!("{}_{}", field_name, "optional"),
                    anony_span.clone(),
                );

                let optional_type = CbmlType {
                    kind: CbmlTypeKind::Optional {
                        inner_type: ty.clone().into(),
                    },
                };

                self.insert_type(
                    anony_optional_name.clone(),
                    TypeInfo {
                        name: anony_optional_name.clone(),
                        ty: optional_type.clone(),
                        span: anony_span,
                    },
                );

                return (anony_optional_name, optional_type);
            }
        }
    }

    fn parse_type_def(&mut self, type_def_stmt: &crate::parser::ast::stmt::TypeDefStmt) {
        use crate::parser::ast::stmt::TypeDefStmt;

        match type_def_stmt {
            TypeDefStmt::StructDefStmt(struct_def) => self.parse_struct_def(struct_def),
            TypeDefStmt::EnumDef(enum_def) => self.parse_enum_def(enum_def),
            TypeDefStmt::UnionDef(union_def) => self.parse_union_def(union_def),
        }
    }

    pub fn get_all_top_fields(&self) -> Vec<&FieldDef> {
        let top_scope = ScopeID::new(self.file_path.clone());
        self.fields
            .iter()
            .filter(|x| x.scope == top_scope)
            .collect()
    }
}

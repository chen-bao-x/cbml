// 感觉学些越乱,
// 重新实现得了.

pub mod types_for_check;

use types_for_check::{DefinedField, DefinedType, ScopeID, TypeSign};

use crate::cbml_value::ToCbmlType;
use crate::cbml_value::value::{CbmlType, CbmlTypeKind, ToCbmlValue};
use crate::formater::ToCbmlCode;
use crate::lexer::token::Span;
use crate::lexer::token::Token;
use crate::lexer::tokenizer;
use crate::parser::StmtKind;
use crate::parser::ast::stmt::LiteralKind;
use crate::parser::ast::stmt::Stmt;
use crate::parser::ast::stmt::StructFieldDefStmt;
use crate::parser::ast::stmt::TypeDefStmt;
use crate::parser::ast::stmt::TypeSignStmtKind;
use crate::parser::ast::stmt::{AsignmentStmt, EnumFieldDef};
use crate::parser::parser_error::ParserError;
use std::clone;
use std::collections::HashMap;

/// 检查 cbml 文件
// pub fn typecheck(file_path: String, ast: &Vec<StmtKind>) -> Vec<ParserError> {
pub fn typecheck(file_path: String, ast: &Vec<Stmt>) -> Vec<ParserError> {
    let mut type_checker = TypeChecker::new(file_path);

    return type_checker.typecheck(ast);
}

/// 检查 cbml 文件
// pub fn typecheck_for_def(file_path: String, ast: &Vec<StmtKind>) -> Vec<ParserError> {
pub fn typecheck_for_def(file_path: String, ast: &Vec<Stmt>) -> Vec<ParserError> {
    let mut type_checker = TypeChecker::new(file_path);

    type_checker.state = State::InTypedef;
    let re = type_checker.typecheck(&ast);
    type_checker.state = State::InFile;

    return re;
}

#[derive(Debug, Clone)]
/// 类型检查
pub struct TypeChecker {
    /// use "" 语句中引用的类型定义文件.
    pub use_path: Option<String>,

    pub data_file: DataFile,
    // pub type_def_file: TypeDefFile,
    pub type_def_file: Option<types_for_check::TypeDefFile>,

    /// 还没有解析前, self.pro == None
    // pub pro: types_for_check::Project,

    // pub defined_fields: HashMap<String, CbmlType>,
    /// cbml file path.
    pub file_path: String,

    /// 是否已经加载了 类型定义文件并将 自定定义 和 类型定义添加到了  defined_fields  custom_types 中.
    // pub is_def_file_loaded: bool,
    pub is_def_file_loaded: IsDefFileLoaded,

    /// 正在解析 cbml 文件, 还是在解析 类型定义文件.
    pub state: State,

    /// 解析 ast 时记录正在解析的语句所在的 scope.
    current_scope: Vec<ScopeID>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IsDefFileLoaded {
    /// 加载并解析了, 没有 语法错误 语义错误 类型错误 等等的错误.
    ParsedOk,
    /// 加载并解析了, 要错误.
    ParsedHasError(Vec<ParserError>),
    /// 还没有加载或者 cbml 文件并没有使用 use 语句来加载类型定义文件.
    /// 不使用 use 语句来加载来行定义文件也是允许的.
    Unload,
}

impl IsDefFileLoaded {
    pub fn is_ok(&self) -> bool {
        match self {
            Self::ParsedOk => true,
            _ => false,
        }
    }
    pub fn is_loaded(&self) -> bool {
        match self {
            IsDefFileLoaded::Unload => false,
            _ => true,
        }
    }

    pub fn has_error(&self) -> bool {
        match self {
            Self::ParsedHasError(_) => true,
            _ => false,
        }
    }

    pub fn get_errors(&self) -> Option<&Vec<ParserError>> {
        let Self::ParsedHasError(error) = self else {
            return None;
        };

        return Some(error);
    }
}

impl TypeChecker {
    pub fn new(file_path: String) -> Self {
        let type_def_file = if file_path.ends_with(".def.cbml") {
            Some(types_for_check::TypeDefFile::new(file_path.clone()))
        } else {
            None
        };

        let code_file = if file_path.ends_with(".def.cbml") {
            Some(types_for_check::CodeFile::new(file_path.clone()))
        } else {
            None
        };

        TypeChecker {
            is_def_file_loaded: IsDefFileLoaded::Unload,
            state: State::InFile,
            use_path: None,
            file_path: file_path.clone(),
            data_file: DataFile::new(),
            type_def_file,
            current_scope: Vec::new(),
        }
    }

    pub fn typecheck(&mut self, ast: &Vec<Stmt>) -> Vec<ParserError> {
        let mut re: Vec<ParserError> = vec![];

        for s in ast {
            let asdf = self.check_one_stmt(s);
            if let Some(arr) = asdf {
                for x in arr {
                    re.push(x);
                }
            }
        }

        return re;
    }

    /// 检查类型的名称是否重复.
    pub fn check_duplicated_type_name(
        &self,
        file_path: String,
        span: Span,
        name: &str,
    ) -> Option<ParserError> {
        let Some(def_file) = &self.type_def_file else {
            return None;
        };

        let re = def_file.types.get(name);
        return match re {
            Some(_a) => Some(ParserError::new(
                file_path,
                format!("类型 `{}` 已经存在: at: ", name,),
                span,
            )),
            None => None,
        };
    }

    /// 检查重复的 file level field.
    pub fn check_duplicated_file_field_name(
        &self,
        file_path: String,
        name: &str,
        span: Span,
        scope_id: ScopeID,
    ) -> Option<ParserError> {
        let Some(def_file) = &self.type_def_file else {
            return None;
        };

        let re = def_file.fields.get(&(name.to_string(), scope_id));

        return match re {
            Some(_a) => Some(ParserError::new(
                file_path,
                format!("field `{}` 已经存在: at: ", name,),
                span,
            )),
            None => None,
        };
    }

    /// 是否是自定义类型, 比如使用 struct enum union 等关键字定义的类型.
    pub fn is_named_type(&self, name: &str) -> bool {
        // let re = self.type_def_file.named_types.get(name);

        let Some(def_file) = &self.type_def_file else {
            return false;
        };

        let re = def_file.types.get(name);

        return match re {
            Some(_a) => true,
            None => false,
        };

        // 如果使用了 Custom 类型, 这个类型是否存在.
        // {
        //     if let CbmlType::Custom(ref name) = field.ty {
        //         let re = self.custom_types.get(name);
        //         match re {
        //             Some(_) => {}
        //             None => {
        //                 return TypeCheckedResult::Error(format!("connot find type `{}` ", name));
        //             }
        //         }
        //     }
        // }
    }

    // fn did_allow_in_state(&mut self, stmt: &StmtKind) -> Option<ParserError> {
    fn did_allow_in_state(&mut self, stmt: &Stmt) -> Option<ParserError> {
        // config_file = useStmt{0,1} b{0,}
        // b = linecomment | blockComment | asignment
        //

        // typedef file
        // typedef_file = FileFieldDef | TypeAlias | StructDef | EnumDef | UnionDef | LineComment | BlockComment | DocComment

        match self.state {
            State::InFile => match stmt.kind {
                StmtKind::Asignment(_)
                | StmtKind::Use(_)
                | StmtKind::LineComment(_)
                | StmtKind::BlockComment(_) => None,
                _ => Some(ParserError::new(
                    self.file_path.clone(),
                    format!("stmt not allow in current scope: {:?}", stmt),
                    stmt.kind.get_span(),
                )),
            },
            State::InTypedef => match &stmt.kind {
                StmtKind::Asignment(_) | StmtKind::Use(_) => Some(ParserError {
                    file_path: self.file_path.clone(),
                    msg: format!("stmt not allow in current scope: {:?}", stmt),
                    code_location: stmt.kind.get_span(),
                    note: None,
                    help: None,
                }),
                _ => None,
            },
        }
    }

    // pub fn check_one_stmt(&mut self, stmt: &StmtKind) -> Option<Vec<ParserError>> {
    pub fn check_one_stmt(&mut self, stmt: &Stmt) -> Option<Vec<ParserError>> {
        let mut result: Vec<ParserError> = vec![];

        let re = self.did_allow_in_state(&stmt);
        if let Some(e) = re {
            result.push(e);
        }

        match &stmt.kind {
            StmtKind::Use(_url) => {
                let use_path = _url.get_use_url();
                self.use_path = Some(use_path.clone());

                if self.state == State::InTypedef {
                    let e = ParserError {
                        file_path: self.file_path.clone(),
                        msg: format!(""),
                        code_location: _url.keyword_span.clone(),
                        note: None,
                        help: None,
                    };
                    return Some(vec![e]);
                }

                // error: 在 use 语句之前不能有 赋值语句.
                {
                    if !self.data_file.asignments.is_empty() {
                        let e = ParserError {
                            file_path: self.file_path.clone(),
                            msg: format!("`use` 只能在文件的最开头."),
                            code_location: _url.keyword_span.clone(),
                            note: None,
                            help: Some(format!("尝试将 `use` 移动到第一行")),
                        };
                        result.push(e);
                    }
                };

                // error: 重复的 use 语句, use 语句只能使用一次.
                {
                    if self.is_def_file_loaded.is_ok() {
                        let e = ParserError::err_use_can_only_def_onece(
                            self.file_path.clone(),
                            _url.url_span.clone(),
                        );
                        result.push(e);
                    } else {
                        self.is_def_file_loaded = IsDefFileLoaded::ParsedOk;
                    }
                };

                // 读取 类型定义文件.
                {
                    // TODO:
                    // 如果是文件 url 则读取文件
                    // 如果是网络 url 则下载这个文件.
                    let re = std::fs::read_to_string(&use_path.clone());

                    match re {
                        Ok(code) => {
                            // println!("{code}");
                            let re = self.read_type_def_file(&use_path, &code);

                            if let Some(mut err) = re {
                                let asadsfdf = ParserError {
                                    file_path: self.file_path.clone(),
                                    msg: format!(
                                        "引用的 类型定义文件 中有 {} 个错误: \n{}",
                                        err.len(),
                                        &_url.url
                                    ),
                                    code_location: _url.keyword_span.clone(),
                                    note: None,
                                    help: None,
                                };

                                err.push(asadsfdf);

                                return Some(err);
                            }
                        }
                        Err(e) => {
                            let err = ParserError::err_cannot_open_file(
                                self.file_path.clone(),
                                &_url.url,
                                _url.url_span.clone(),
                                e,
                            );

                            return Some(vec![err]);
                        }
                    };
                }
            }

            StmtKind::FileFieldStmt(field_def) => {
                // struct_field_def_stmt.field_name;
                // struct_field_def_stmt.default;
                // struct_field_def_stmt.ty; // 如果使用了 Custom 类型, 这个类型是否存在.

                // 名称不能重复
                {
                    let re = self.check_duplicated_file_field_name(
                        self.file_path.clone(),
                        &field_def.field_name,
                        field_def.field_name_span.clone(), // struct_field_def_stmt.span,
                        self.get_current_scope_id(),
                    );
                    if let Some(e) = re {
                        result.push(e);
                        // return Some(vec![e]);
                    }
                }

                // 如果使用了 Custom 类型, 这个类型是否存在.
                if let TypeSignStmtKind::Custom(name) = &field_def._type.kind {
                    // if self.is_def_file_loaded.is_loaded() {
                    if self.is_def_file_loaded.is_ok() {
                        if !self.is_named_type(name) {
                            let e = ParserError::new(
                                self.file_path.clone(),
                                format!("connot find type {}", name,),
                                field_def.field_name_span.clone(),
                            );
                            result.push(e);
                            // return Some(vec![e]);
                        }
                    }
                }

                // if let Some(default_value) = &field_def.default {
                //     let defnied_type = field_def._type.kind.to_cbml_type();

                //     if !self.is_same_type(&defnied_type, &default_value.kind) {
                //         // 类型错误, 需要 {} found {}

                //         let e = ParserError::err_mismatched_types(
                //             self.file_path.clone(),
                //             field_def.field_name_span.clone(),
                //             &field_def._type.kind.to_cbml_code(0),
                //             &default_value.kind.to_cbml_code(0),
                //         );
                //         result.push(e);
                //         // return Some(vec![e]);
                //     }
                // }

                {
                    let k = field_def.field_name.clone();
                    // let t = field_def._type.clone().to_cbml_type();

                    if let Err(_) =
                        self.push_field_def(k, field_def.clone(), self.get_current_scope_id())
                    {
                        let e = ParserError::err_field_alredy_exits(
                            self.file_path.clone(),
                            field_def.field_name_span.clone(),
                            &field_def.field_name,
                        );
                        result.push(e);
                        // return Some(vec![e]);
                    };
                }
            }
            StmtKind::TypeAliasStmt(_) => {
                todo!();

                // // 如果使用了 Custom 类型, 这个类型是否存在.
                // if self.push_type_def(s.name.clone(), s.ty.clone()) {
                //     let e = ParserError::err_type_name_alredy_exits(
                //         self.file_path.clone(),
                //         s.name_span.clone(),
                //         &s.name,
                //     );
                //     result.push(e);
                //     // return Some(vec![e]);
                // }
            }
            StmtKind::StructDefStmt(struct_def) => {
                let re = self.check_duplicated_type_name(
                    self.file_path.clone(),
                    struct_def.name_span.clone(),
                    &struct_def.struct_name,
                );
                if let Some(e) = re {
                    result.push(e);
                }

                {
                    // fields 里面是否有重名的.
                    let mut field_names: HashMap<&String, &String> = HashMap::new();

                    for field in struct_def.fields.iter() {
                        let re = field_names.insert(&field.field_name, &field.field_name); // fields 里面是否有重名的.
                        match re {
                            None => {}
                            Some(s) => {
                                let e = ParserError {
                                    file_path: self.file_path.clone(),
                                    msg: format!("属性名称重复: {}", s),
                                    code_location: field.field_name_span.clone(),
                                    note: None,
                                    help: None,
                                };
                                result.push(e);
                            }
                        };

                        // 如果使用了 Custom 类型, 这个类型是否存在.
                        // {
                        //     if let TypeSignStmtKind::Custom(ref name) = field._type.kind {
                        //         if let Some(def_file) = &self.type_def_file {
                        //             let re = &self.type_def_file.unwrap().types.get(name);
                        //             match re {
                        //                 Some(_) => { /* 有这个类型 */ }
                        //                 None => {
                        //                     let e = ParserError::err_cannot_find_type(
                        //                         self.file_path.clone(),
                        //                         field.field_name_span.clone(),
                        //                         name,
                        //                     );
                        //                     result.push(e);
                        //                 }
                        //             }
                        //         };
                        //     }
                        // }
                    }
                }

                {
                    let struct_name = struct_def.struct_name.clone();

                    // let end: Position = {
                    //     if let Some(last) = struct_def.fields.last() {
                    //         last.end_span().end
                    //     } else {
                    //         struct_def.name_span.end.clone()
                    //     }
                    // };

                    // let fields = struct_def
                    //     .fields
                    //     .iter()
                    //     .map(|x| (x.field_name.clone(), x._type.to_cbml_type()))
                    //     .collect();
                    // let ty = CbmlType {
                    //     kind: CbmlTypeKind::Struct { fields: fields },
                    //     // name: Some(struct_name.clone()),
                    // };

                    // if let Err(_) = self.push_type_def(
                    //     struct_name,
                    //     ty,
                    //     struct_def.name_span.clone(),
                    //     self.get_current_scope_id(),
                    // ) {
                    //     let e = ParserError::err_type_name_alredy_exits(
                    //         self.file_path.clone(),
                    //         struct_def.name_span.clone(),
                    //         &struct_def.struct_name,
                    //     );
                    //     result.push(e);
                    // };
                }
            }
            StmtKind::EnumDef(enum_def) => {
                // fields 里面是否有重名的.
                // 如果使用了 Custom 类型, 这个类型是否存在.

                let enum_name = enum_def.enum_name.clone();

                {
                    // fields 里面是否有重名的.
                    let mut field_names: HashMap<&String, &String> = HashMap::new();

                    if let Some(def_file) = &self.type_def_file {
                        for field in enum_def.fields.iter() {
                            let key = (field.field_name.clone(), self.get_current_scope_id());
                            let new_scope = ScopeID::new(enum_name.clone());

                            self.into_scope(new_scope);

                            let re = self.push_enum_field_def(
                                enum_name.clone(),
                                field,
                                self.get_current_scope_id(),
                            );

                            let re = field_names.insert(&field.field_name, &field.field_name); // fields 里面是否有重名的.
                            match re {
                                None => {}
                                Some(s) => {
                                    let e = ParserError {
                                        file_path: self.file_path.clone(),
                                        msg: format!("属性名称重复: {}", s),
                                        code_location: field.field_name_span.clone(),
                                        note: None,
                                        help: None,
                                    };
                                    result.push(e);
                                }
                            };

                            // 如果使用了 Custom 类型, 这个类型是否存在.
                            // {
                            //     if let TypeSignStmtKind::Custom(ref name) = field._type.kind {
                            //         if let Some(def_file) = self.type_def_file {
                            //             let re = self.type_def_file.unwrap().types.get(name);
                            //             match re {
                            //                 Some(_) => { /* 有这个类型 */ }
                            //                 None => {
                            //                     let e = ParserError::err_cannot_find_type(
                            //                         self.file_path.clone(),
                            //                         field.field_name_span.clone(),
                            //                         name,
                            //                     );
                            //                     result.push(e);
                            //                 }
                            //             }
                            //         };
                            //     }
                            // }
                        }
                    };
                }

                // every thing is ok.
                {
                    let enum_name = enum_def.enum_name.clone();

                    // let fields = enum_def
                    //     .fields
                    //     .iter()
                    //     .map(|x| (x.field_name.clone(), x._type.to_cbml_type()))
                    //     .collect();

                    // let kind = CbmlTypeKind::Enum { fields: fields };

                    // let ty = CbmlType {
                    //     kind: CbmlTypeKind::Enum { fields: fields },
                    //     // name: Some(enum_name.clone()),
                    // };

                    // if let Err(_) = self.push_type_def(
                    //     enum_name,
                    //     ty,
                    //     enum_def.name_span.clone(),
                    //     self.get_current_scope_id(),
                    // ) {
                    //     let e = ParserError::err_type_name_alredy_exits(
                    //         self.file_path.clone(),
                    //         enum_def.name_span.clone(),
                    //         &enum_def.enum_name,
                    //     );
                    //     result.push(e);
                    // };
                }
            }
            // StmtKind::TypeDef(union_def) => {
            //     // 如果使用了 Custom 类型, 这个类型是否存在.
            //     // alowd_values 是否有重复的.

            //     let re = self.check_duplicated_type_name(
            //         self.file_path.clone(),
            //         stmt.span.clone(),
            //         union_def.get_name(),
            //     );
            //     if let Some(e) = re {
            //         result.push(e);
            //     }
            //     // 检查 base_type 是 Custom 时, 这个 Custom 的类型是否存在.
            //     // if let TypeSignStmtKind::Custom(name) = &union_def.base_type {
            //     //     if !self.is_named_type(name) {
            //     //         // return ParserError::err_cannot_find_type(name);

            //     //         let e = ParserError::err_cannot_find_type(
            //     //             self.file_path.clone(),
            //     //             union_def.name_span.clone(),
            //     //             name,
            //     //         );
            //     //         result.push(e);
            //     //     }
            //     // }

            //     // alowd_values 是否有重复的.
            //     {
            //         let _allowed_values: Vec<LiteralKind> = {
            //             let mut arr: Vec<LiteralKind> = vec![];
            //             for x in &union_def.allowed_values {
            //                 arr.push(x.kind.clone());
            //             }

            //             arr
            //         };

            //         let mut arr: Vec<&LiteralKind> = vec![];

            //         for x in &union_def.allowed_values {
            //             if arr.contains(&&x.kind) {
            //                 // 有重复的项

            //                 let e = ParserError::err_union_duplicated_item(
            //                     self.file_path.clone(),
            //                     x.span.clone(),
            //                     &x.kind.to_cbml_code(0),
            //                 );
            //                 result.push(e);
            //             } else {
            //                 arr.push(&x.kind);
            //             }
            //         }
            //     }

            //     {
            //         let union_name = union_def.union_name.clone();

            //         let alowd_values = union_def
            //             .allowed_values
            //             .iter()
            //             .map(|x| x.to_cbml_value())
            //             .collect();

            //         let ty = CbmlType {
            //             kind: CbmlTypeKind::Union {
            //                 allowed_values: alowd_values,
            //             },
            //             // name: Some(union_name.clone()),
            //         };

            //         let sadf = self.push_type_def(
            //             union_name,
            //             ty,
            //             union_def.name_span.clone(),
            //             self.get_current_scope_id(),
            //         );
            //         if let Err(_) = sadf {
            //             let e = ParserError::err_type_name_alredy_exits(
            //                 self.file_path.clone(),
            //                 union_def.name_span.clone(),
            //                 &union_def.union_name,
            //             );

            //             result.push(e);
            //         };
            //     }
            // }
            StmtKind::Asignment(asign) => {
                // 检查 field_name 在 typedef 文件中是否存在.
                // value 字面量类型推导.
                // 检查 field_name 在 typedef 文件中定义的类型.
                // 检查 value 是否符合 field_name 在 typedef 文件中定义的类型.

                // self.custom_types.contains_key(k)

                if let Some(def_file) = &self.type_def_file {
                    let re = def_file
                        .fields
                        .get(&(asign.field_name.clone(), self.get_current_scope_id()));

                    let re = re.map(|x| x.clone());

                    // 赋值的 字段 需要在 def_file 中定义过.
                    match re {
                        None => {
                            // 使用了 use 语句, 而这个字段并未在 def_file 中定义过, 则报错.
                            if self.is_def_file_loaded.is_ok() {
                                let e = ParserError::err_unknow_field(
                                    self.file_path.clone(),
                                    asign.field_name_span.clone(),
                                    &asign.field_name,
                                );

                                result.push(e);
                            }
                        }
                        Some(f) => {
                            let saf = def_file.types.get(&f._type.type_name);

                            match saf {
                                Some(_type) => {
                                    let ty = _type.clone();

                                    // 赋值的类型需要与定义的类型相同.
                                    if !self.is_same_type(&ty._type.clone(), &asign.value.kind) {
                                        let e = ParserError::err_mismatched_types(
                                            self.file_path.clone(),
                                            asign.field_name_span.clone(),
                                            &f._type.type_name.clone(),
                                            // &field_def._type.kind.to_cbml_code(0),
                                            &asign.value.kind.to_cbml_code(0),
                                        );
                                        result.push(e);
                                    };
                                }
                                None => {}
                            };
                        }
                    }
                }

                if self.push_field_asign(asign.clone()) {
                    let e = ParserError::err_filed_alredy_asignment(
                        self.file_path.clone(),
                        asign.field_name_span.clone(),
                        &asign,
                    );
                    result.push(e);
                };
            }
            StmtKind::LineComment(_) => {}
            StmtKind::BlockComment(_) => {}
            StmtKind::DocComment(_) => {}
            StmtKind::EmptyLine => todo!(),
            StmtKind::TypeDef(type_def_stmt) => match type_def_stmt {
                TypeDefStmt::StructDefStmt(_) => todo!(),
                TypeDefStmt::EnumDef(_) => todo!(),
                TypeDefStmt::UnionDef(_) => todo!(),
            },
        }

        if result.is_empty() {
            return None;
        } else {
            return Some(result);
        }
    }

    // pub fn custom_to_raw(&self, type_name: &String) -> Option<&CbmlType> {
    //     self.type_def_file.named_types.get(type_name)
    // }

    pub fn read_type_def_file(
        &mut self,
        def_file_path: &str,
        code: &str,
    ) -> Option<Vec<ParserError>> {
        use crate::parser::cbml_parser::CbmlParser;

        // 类型定义文件的文件后缀需要是 .def.cbml
        if !def_file_path.ends_with(".def.cbml") {
            let e = ParserError {
                file_path: self.file_path.clone(),
                msg: format!("类型定义文件的文件名需要以 .def.cbml 结尾"),
                code_location: Span::empty(),
                note: None,
                help: None,
            };

            return Some(vec![e]);
        }

        // error check.
        if def_file_path == self.file_path {
            let e = ParserError {
                file_path: self.file_path.clone(),
                msg: format!("不能类型定义文件中使用 use 语句."),
                code_location: Span::empty(),
                note: None,
                help: None,
            };

            return Some(vec![e]);
        }

        let re = tokenizer(def_file_path, &code);
        let tokens: Vec<Token> = match re {
            Ok(a) => a,
            Err(e) => {
                self.is_def_file_loaded = IsDefFileLoaded::ParsedHasError(vec![e.clone()]);

                return Some(vec![e]);
            }
        };

        let mut parser = CbmlParser::new(def_file_path.to_string(), &tokens);
        let re = parser.parse();

        // self.type_def_file.tokens = tokens;

        match re {
            Ok(ast) => {
                self.state = State::InTypedef;
                let type_checked_result = self.typecheck(&ast);
                self.state = State::InFile;

                // self.type_def_file.ast = ast;

                if type_checked_result.is_empty() {
                    // dp("没有检查出类型错误.");
                    self.is_def_file_loaded = IsDefFileLoaded::ParsedOk;
                    return None;
                } else {
                    self.is_def_file_loaded =
                        IsDefFileLoaded::ParsedHasError(type_checked_result.clone());
                    return Some(type_checked_result);
                }
            }
            Err(e) => {
                self.is_def_file_loaded = IsDefFileLoaded::ParsedHasError(e.clone());
                return Some(e);
            }
        }
    }

    /// 检查字面量的类型是否符合类型定义文件的要求.
    pub fn is_same_type(&mut self, need_type: &CbmlType, literal: &LiteralKind) -> bool {
        if let LiteralKind::Default = literal {
            return true;
        }

        match need_type.kind.clone() {
            CbmlTypeKind::String => match literal {
                LiteralKind::String { .. } => true,
                LiteralKind::Default => true,
                _ => false,
            },
            CbmlTypeKind::Number => match literal {
                LiteralKind::Number(_) => true,
                LiteralKind::Default => true,
                _ => false,
            },
            CbmlTypeKind::Bool => match literal {
                LiteralKind::Boolean(_) => true,
                LiteralKind::Default => true,
                _ => false,
            },
            CbmlTypeKind::Any => true,
            CbmlTypeKind::Array { inner_type, .. } => match literal {
                LiteralKind::Array(literals) => {
                    return literals.iter().all(|x| self.is_same_type(&inner_type, x));
                }
                LiteralKind::Default => true,
                _ => false,
            },
            CbmlTypeKind::Struct { mut fields } => {
                //
                {
                    fields.sort_by(|x, y| {
                        let x_name = &x.0;
                        let y_name = &y.0;
                        return x_name.cmp(&y_name);
                    });
                }

                match literal {
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
                return match literal {
                    LiteralKind::LiteralNone => true,
                    _ => self.is_same_type(&inner_type, literal),
                };
            }
            CbmlTypeKind::Union { allowed_values } => {
                allowed_values.contains(&literal.to_cbml_value())
            }
            CbmlTypeKind::Enum { fields } => match literal {
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
}

impl TypeChecker {
    /// 如果添加成功, 则返回 true.
    fn push_enum_field_def(
        &mut self,
        enum_name: String,
        field_def: &EnumFieldDef,
        scope: ScopeID,
    ) -> Result<(), ()> {
        let type_name: String = match &field_def._type.kind {
            TypeSignStmtKind::String => "string".into(),
            TypeSignStmtKind::Number => "number".into(),
            TypeSignStmtKind::Boolean => "bool".into(),
            TypeSignStmtKind::Any => "any".into(),

            // 如果是匿名类型
            // TypeSignStmtKind::Array { inner_type } => {
            //     let auto_generated_type_name =
            //         gen_anonymous_type_name(&self.get_current_scope_id(), &enum_name);
            //     self.push_type_def(
            //         auto_generated_type_name.clone(),
            //         CbmlType {
            //             kind: CbmlTypeKind::Array {
            //                 inner_type: inner_type.to_cbml_type().into(),
            //             },
            //         },
            //         field_def.field_name_span.clone(),
            //         self.get_current_scope_id(),
            //     );

            //     auto_generated_type_name
            // }
            // TypeSignStmtKind::Struct(struct_field_def_stmts) => {
            //     let mut fields: Vec<(String, CbmlType)> = vec![];

            //     for x in struct_field_def_stmts {
            //         let field_def = x._type.to_cbml_type();
            //         fields.push((x.field_name.clone(), field_def));

            //         self.push_field_def(
            //             x.field_name.clone(),
            //             x.clone(),
            //             self.get_current_scope_id(),
            //         );
            //     }

            //     let auto_generated_type_name =
            //         gen_anonymous_type_name(&self.get_current_scope_id(), &enum_name);
            //     self.push_type_def(
            //         auto_generated_type_name.clone(),
            //         CbmlType {
            //             kind: CbmlTypeKind::Struct { fields: fields },
            //         },
            //         field_def.field_name_span.clone(),
            //         self.get_current_scope_id(),
            //     );

            //     auto_generated_type_name
            // }

            // TypeSignStmtKind::Optional { inner_type } => {
            //     let auto_generated_type_name =
            //         gen_anonymous_type_name(&self.get_current_scope_id(), &enum_name);
            //     self.push_type_def(
            //         auto_generated_type_name.clone(),
            //         CbmlType {
            //             kind: CbmlTypeKind::Optional {
            //                 inner_type: inner_type.to_cbml_type().into(),
            //             },
            //         },
            //         field_def.field_name_span.clone(),
            //         self.get_current_scope_id(),
            //     );

            //     auto_generated_type_name
            // }
            TypeSignStmtKind::Anonymous(anonymous_type_def_stmt) => {
                // let field_def = anonymous_type_def_stmt.to_cbml_type();

                let auto_generated_type_name =
                    gen_anonymous_type_name(&self.get_current_scope_id(), &enum_name);

                // self.push_type_def(
                //     auto_generated_type_name.clone(),
                //     field_def,
                //     anonymous_type_def_stmt.span.clone(),
                //     self.get_current_scope_id(),
                // );

                auto_generated_type_name
            }
            TypeSignStmtKind::Custom(custom_name) => custom_name.clone(),
        };

        let defed_field = types_for_check::DefinedField {
            name: enum_name.clone(),
            _type: TypeSign::from_asdf(type_name, field_def.field_name_span.clone()),
            span: Span::empty(),
            scope: scope,
        };

        let Some(type_def_file) = &mut self.type_def_file else {
            return Err(());
        };

        let re = type_def_file
            .fields
            .get(&(enum_name.clone(), defed_field.scope.clone()));

        match re {
            Some(_) => {
                // name 已经存在
                return Err(());
            }
            None => {
                let _ = type_def_file
                    .fields
                    .insert((enum_name, defed_field.scope.clone()), defed_field);

                return Ok(());
            }
        }
    }

    /// 如果 name 添加成功, 则会返回 true. 已经有同名的 field 在这个 scope 内, 则返回 false.
    fn push_field_def(
        &mut self,
        struct_name: String,
        ty: StructFieldDefStmt,
        scope: ScopeID,
    ) -> Result<(), ()> {
        // println!("push_filed_def: {:?}", struct_name);

        let type_name: String = match ty._type.kind {
            TypeSignStmtKind::String => "string".into(),
            TypeSignStmtKind::Number => "number".into(),
            TypeSignStmtKind::Boolean => "bool".into(),
            TypeSignStmtKind::Any => "any".into(),

            // 如果是匿名类型
            // TypeSignStmtKind::Array { inner_type } => {
            //     let auto_generated_type_name =
            //         gen_anonymous_type_name(&self.get_current_scope_id(), &struct_name);
            //     let re = self.push_type_def(
            //         auto_generated_type_name.clone(),
            //         CbmlType {
            //             kind: CbmlTypeKind::Array {
            //                 inner_type: inner_type.to_cbml_type().into(),
            //             },
            //         },
            //         ty.field_name_span.clone(),
            //         self.get_current_scope_id(),
            //     );

            //     auto_generated_type_name
            // }
            // TypeSignStmtKind::Struct(struct_field_def_stmts) => {
            //     let mut fields: Vec<(String, CbmlType)> = vec![];

            //     for x in struct_field_def_stmts {
            //         let ty = x._type.to_cbml_type();
            //         fields.push((x.field_name.clone(), ty));

            //         let re = self.push_field_def(x.field_name.clone(), x, scope.clone());
            //     }

            //     let auto_generated_type_name =
            //         gen_anonymous_type_name(&self.get_current_scope_id(), &struct_name);
            //     let re = self.push_type_def(
            //         auto_generated_type_name.clone(),
            //         CbmlType {
            //             kind: CbmlTypeKind::Struct { fields: fields },
            //         },
            //         ty.field_name_span.clone(),
            //         self.get_current_scope_id(),
            //     );

            //     auto_generated_type_name
            // }

            // TypeSignStmtKind::Optional { inner_type } => {
            //     let auto_generated_type_name =
            //         gen_anonymous_type_name(&self.get_current_scope_id(), &struct_name);
            //     let re = self.push_type_def(
            //         auto_generated_type_name.clone(),
            //         CbmlType {
            //             kind: CbmlTypeKind::Optional {
            //                 inner_type: inner_type.to_cbml_type().into(),
            //             },
            //         },
            //         ty.field_name_span.clone(),
            //         self.get_current_scope_id(),
            //     );

            //     auto_generated_type_name
            // }
            TypeSignStmtKind::Anonymous(anonymous_type_def_stmt) => {
                // let ty = anonymous_type_def_stmt.to_cbml_type();

                let auto_generated_type_name =
                    gen_anonymous_type_name(&self.get_current_scope_id(), &struct_name);

                // let re = self.push_type_def(
                //     auto_generated_type_name.clone(),
                //     ty,
                //     anonymous_type_def_stmt.span,
                //     self.get_current_scope_id(),
                // );

                auto_generated_type_name
            }
            TypeSignStmtKind::Custom(custom_name) => custom_name.clone(),
        };

        let defed_field = types_for_check::DefinedField {
            name: struct_name.clone(),
            _type: TypeSign::from_asdf(type_name, ty.field_name_span.clone()),
            span: Span::empty(),
            scope: scope,
        };

        let Some(type_def_file) = &mut self.type_def_file else {
            return Err(());
        };

        let re = type_def_file
            .fields
            .get(&(struct_name.clone(), defed_field.scope.clone()));

        return match re {
            Some(_) => {
                // name 已经存在
                Err(())
            }
            None => {
                type_def_file
                    .fields
                    .insert((struct_name, defed_field.scope.clone()), defed_field);

                Ok(())
            }
        };
    }

    /// 如果 name 已经存在, 则会返回 true.
    fn push_field_asign(&mut self, asign: AsignmentStmt) -> bool {
        let re = self
            .data_file
            .asignments
            .insert(asign.field_name.clone(), asign);

        match re {
            Some(_) => {
                // name 已经存在
                true
            }
            None => false,
        }
    }

    /// 如果 name 已经存在, 则会返回 true.
    // fn push_type_def(&mut self, type_name: String, ty: TypeSignStmt) -> bool {
    fn push_type_def(
        &mut self,
        type_name: String,
        ty: CbmlType,
        span: Span,
        scope: ScopeID,
    ) -> Result<(), ()> {
        let Some(def_file) = &mut self.type_def_file else {
            return Err(());
        };
        let defined_type = DefinedType {
            type_name: type_name.clone(),
            _type: ty,
            span: span,
            scope: scope,
        };

        let re = def_file.types.get(&type_name);

        return match re {
            Some(_) => {
                // name 已经存在
                Err(())
            }
            None => {
                def_file.types.insert(type_name, defined_type);
                Ok(())
            }
        };
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    /// .cbml
    InFile,
    /// .typedef.cbml
    InTypedef,
}

/// .def.cbml file.
#[derive(Debug, Clone)]
pub struct TypeDefFile {
    /// 自定义的 file level field.
    pub defined_fields: HashMap<String, StructFieldDefStmt>,

    /// 匿名类型, person: {name:string,age:number} person 的类型就是一个匿名结构体类型.
    /// anonymous_types key 的生成规则: 1_anonymous_type_for_person,
    /// 匿名类型以数字 1 开头是因为 自定义类型 的名称不能以 数字 开头.
    /// 一个 typedef 文件中的 field 不能重名, 所以最后面都上 field name 可以了防止重名.
    pub anonymous_types: HashMap<String, CbmlType>,

    /// 自定义的类型, 例如: struct, enum, union, type alias, named array,
    pub named_types: HashMap<String, CbmlType>,

    pub ast: Vec<Stmt>,
    pub tokens: Vec<Token>,
}

impl TypeDefFile {
    fn new() -> Self {
        Self {
            named_types: HashMap::new(),
            anonymous_types: HashMap::new(),
            defined_fields: HashMap::new(),
            ast: Vec::new(),
            tokens: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.named_types.is_empty()
            && self.anonymous_types.is_empty()
            && self.defined_fields.is_empty()
            && self.ast.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct DataFile {
    /// use "path/to/name.def.cbml" 语句中引用的类型定义文件.
    pub use_path: Option<String>,

    /// field assignment
    /// a = 123 这样的赋值语句.
    pub asignments: HashMap<String, AsignmentStmt>,

    // pub defined_fields: HashMap<String, CbmlType>,
    /// .cbml file path.
    pub file_path: String,

    pub ast: Vec<Stmt>,
}

struct Asignment {
    name: String,
}

impl DataFile {
    fn new() -> Self {
        Self {
            use_path: None,
            asignments: HashMap::new(),
            file_path: String::new(),
            ast: vec![],
        }
    }
    pub fn is_empty(&self) -> bool {
        self.use_path == None
            && self.asignments.is_empty()
            && self.file_path == String::new()
            && self.ast.is_empty()
    }
}

// struct TypeCheckedResult;

// struct TypeCheckError(String);

// #[derive(Debug)]
// pub enum TypeCheckedResult {
//     Ok,
//     Warning,
//     Error(String),
// }

// 类型推导

// fn type_inference() {}

// /// rust-analyzer types
// pub struct TokenStaticData {
//     pub documentation: Option<Documentation>,
//     pub hover: Option<HoverResult>,
//     pub definition: Option<FileRange>,
//     pub references: Vec<ReferenceData>,
//     pub moniker: Option<MonikerResult>,
//     pub display_name: Option<String>,
//     pub signature: Option<String>,
//     pub kind: SymbolInformationKind,
// }

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

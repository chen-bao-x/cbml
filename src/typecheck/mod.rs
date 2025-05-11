use crate::ToCbmlCode;
use crate::lexer::token::Position;
use crate::lexer::token::Span;
use crate::lexer::tokenizer;
use crate::parser::ParserError;
use crate::parser::StmtKind;
use crate::parser::ast::stmt::AsignmentStmt;
use crate::parser::ast::stmt::LiteralKind;
use crate::parser::ast::stmt::StructFieldDefStmt;
use crate::parser::ast::stmt::TypeSignStmt;
use crate::parser::ast::stmt::TypeSignStmtKind;
use std::collections::HashMap;

// 为什么失败、在哪失败、甚至有时候还告诉你怎么修！
// 🎯 核心原则：错误信息不仅是反馈，更是教学工具！
//
// 错误信息 = 编译器和开发者之间的「对话」。
// 一个好编译器不是说“你错了”，而是说：“嘿，我猜你可能是想这样？”
//
// 6. 颜色！颜色！颜色！（重要的说三遍）🌈
//
// 用 ANSI 颜色高亮：
// 	•	红色：error
// 	•	黄色：warning
// 	•	青色：help
// 	•	绿色：路径、类型提示
//
// Rust CLI 本身就是超漂亮的终端艺术品，别忘了这一块！
//
// 7. 提供自动修复建议 / LSP 支持（进阶）
// 	•	支持 JSON 输出
// 	•	提供“fix-it hints”（可以被 IDE 自动修复）
// 	•	支持 LSP 插件（语法树 + diagnostic 提示）
//
// 这就能让你的编译器配合编辑器时实现“悬停提示 + 快捷修复”！
//
// *    名称重复
// •	错误位置
// •	期望类型 vs 实际类型
// •	推测失败原因

/// 检查 cbml 文件
pub fn typecheck(file_path: String, ast: &Vec<StmtKind>) -> Vec<ParserError> {
    let mut type_checker = TypeChecker::new(file_path);

    return type_checker.typecheck(ast);
}

/// 检查 cbml 文件
pub fn typecheck_for_def(file_path: String, ast: &Vec<StmtKind>) -> Vec<ParserError> {
    let mut type_checker = TypeChecker::new(file_path);

    type_checker.state = State::InTypedef;
    let re = type_checker.typecheck(&ast);
    type_checker.state = State::InFile;

    return re;
}
#[derive(Debug, Clone)]
pub enum State {
    /// .cbml
    InFile,
    /// .typedef.cbml
    InTypedef,
}

#[derive(Debug, Clone)]
/// 类型检查
pub struct TypeChecker {
    /// use "" 语句中引用的类型定义文件.
    pub use_path: Option<String>,

    /// 自定义的类型, 例如: struct, enum, union, type alias, named array,
    // pub custom_types: HashMap<String, TypeSignStmtKind>,
    pub custom_types: HashMap<String, TypeSignStmt>,

    /// 自定义的 file level field.
    pub defined_fields: HashMap<String, StructFieldDefStmt>,

    /// cbml file path.
    pub file_path: String,

    /// field assignment
    /// a = 123 这样的赋值语句.
    pub asignments: HashMap<String, AsignmentStmt>,

    /// 是否已经加载了 类型定义文件并将 自定定义 和 类型定义添加到了  defined_fields  custom_types 中.
    // pub is_def_file_loaded: bool,
    pub is_def_file_loaded: IsDefFileLoaded,

    /// 正在解析 cbml 文件, 还是在解析 类型定义文件.
    pub state: State,
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
    /// 如果 name 已经存在, 则会返回 true.
    // fn push_field_def(&mut self, name: String, ty: CbmlType) -> bool {
    fn push_field_def(&mut self, name: String, ty: StructFieldDefStmt) -> bool {
        let re = self.defined_fields.insert(name, ty);

        return match re {
            Some(_) => {
                // name 已经存在
                true
            }
            None => false,
        };
    }

    /// 如果 name 已经存在, 则会返回 true.
    fn push_field_asign(&mut self, asign: AsignmentStmt) -> bool {
        let re = self.asignments.insert(asign.field_name.clone(), asign);

        match re {
            Some(_) => {
                // name 已经存在
                true
            }
            None => false,
        }
    }

    /// 如果 name 已经存在, 则会返回 true.
    fn push_type_def(&mut self, type_name: String, ty: TypeSignStmt) -> bool {
        let re = self.custom_types.insert(type_name, ty);
        match re {
            Some(_) => {
                // name 已经存在
                true
            }
            None => false,
        }
    }
}
impl TypeChecker {
    pub fn new(file_path: String) -> Self {
        TypeChecker {
            custom_types: HashMap::new(),
            is_def_file_loaded: IsDefFileLoaded::Unload,
            state: State::InFile,
            defined_fields: HashMap::new(),
            asignments: HashMap::new(),
            use_path: None,
            file_path: file_path,
            // symbol_table: SymbolTable::new(),
        }
    }

    pub fn typecheck(&mut self, ast: &Vec<StmtKind>) -> Vec<ParserError> {
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
        let re = self.custom_types.get(name);
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
    ) -> Option<ParserError> {
        let re = self.defined_fields.get(name);
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
        let re = self.custom_types.get(name);
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

    fn did_allow_in_state(&mut self, stmt: &StmtKind) -> Option<ParserError> {
        // config_file = useStmt{0,1} b{0,}
        // b = linecomment | blockComment | asignment
        //

        // typedef file
        // typedef_file = FileFieldDef | TypeAlias | StructDef | EnumDef | UnionDef | LineComment | BlockComment | DocComment

        match self.state {
            State::InFile => match stmt {
                StmtKind::Asignment(_)
                | StmtKind::Use(_)
                | StmtKind::LineComment(_)
                | StmtKind::BlockComment(_) => None,
                _ => Some(ParserError::new(
                    self.file_path.clone(),
                    format!("stmt not allow in current scope: {:?}", stmt),
                    stmt.get_span(),
                )),
            },
            State::InTypedef => match stmt {
                StmtKind::Asignment(_) | StmtKind::Use(_) => Some(ParserError {
                    file_path: self.file_path.clone(),
                    msg: format!("stmt not allow in current scope: {:?}", stmt),
                    code_location: stmt.get_span(),
                    note: None,
                    help: None,
                }),
                _ => None,
            },
        }
    }

    pub fn check_one_stmt(&mut self, stmt: &StmtKind) -> Option<Vec<ParserError>> {
        let mut result: Vec<ParserError> = vec![];

        let re = self.did_allow_in_state(&stmt);
        if let Some(e) = re {
            result.push(e);
        }

        match stmt {
            StmtKind::FileFieldStmt(field_def) => {
                // struct_field_def_stmt.field_name;
                // struct_field_def_stmt.default;
                // struct_field_def_stmt.ty; // 如果使用了 Custom 类型, 这个类型是否存在.

                // 名称是否重复
                let re = self.check_duplicated_file_field_name(
                    self.file_path.clone(),
                    &field_def.field_name,
                    field_def.field_name_span.clone(), // struct_field_def_stmt.span,
                );
                if let Some(e) = re {
                    result.push(e);
                    // return Some(vec![e]);
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

                if let Some(default_value) = &field_def.default {
                    if !self.is_same_type(&field_def._type.kind, &default_value.kind) {
                        // 类型错误, 需要 {} found {}

                        let e = ParserError::err_mismatched_types(
                            self.file_path.clone(),
                            field_def.field_name_span.clone(),
                            &field_def._type.kind.to_cbml_code(0),
                            &default_value.kind.to_cbml_code(0),
                        );
                        result.push(e);
                        // return Some(vec![e]);
                    }
                }

                {
                    let k = field_def.field_name.clone();
                    let _ = field_def._type.clone();

                    if self.push_field_def(k, field_def.clone()) {
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
            StmtKind::TypeAliasStmt(s) => {
                // 如果使用了 Custom 类型, 这个类型是否存在.
                if self.push_type_def(s.name.clone(), s.ty.clone()) {
                    let e = ParserError::err_type_name_alredy_exits(
                        self.file_path.clone(),
                        s.name_span.clone(),
                        &s.name,
                    );
                    result.push(e);
                    // return Some(vec![e]);
                }
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
                        {
                            if let TypeSignStmtKind::Custom(ref name) = field._type.kind {
                                let re = self.custom_types.get(name);
                                match re {
                                    Some(_) => {}
                                    None => {
                                        let e = ParserError::err_cannot_find_type(
                                            self.file_path.clone(),
                                            field.field_name_span.clone(),
                                            name,
                                        );
                                        result.push(e);
                                    }
                                }
                            }
                        }
                    }
                }

                {
                    let k = struct_def.struct_name.clone();

                    let end: Position = {
                        if let Some(last) = struct_def.fields.last() {
                            last.end_span().end
                        } else {
                            struct_def.name_span.end.clone()
                        }
                    };

                    let type_sign = TypeSignStmt {
                        kind: TypeSignStmtKind::Struct(struct_def.fields.clone()),
                        span: Span {
                            start: struct_def.name_span.start.clone(),
                            end: end,
                        },
                    };

                    // self.custom_types.insert(k, v);
                    if self.push_type_def(k, type_sign) {
                        let e = ParserError::err_type_name_alredy_exits(
                            self.file_path.clone(),
                            struct_def.name_span.clone(),
                            &struct_def.struct_name,
                        );
                        result.push(e);
                    };
                }
            }
            StmtKind::EnumDef(enum_def) => {
                // enum_def.enum_name;
                // enum_def.fields;

                // fields 里面是否有重名的.
                // 如果使用了 Custom 类型, 这个类型是否存在.

                {
                    // fields 里面是否有重名的.
                    let mut field_names: HashMap<&String, &String> = HashMap::new();

                    for field in enum_def.fields.iter() {
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
                        {
                            if let TypeSignStmtKind::Custom(ref name) = field._type {
                                let re = self.custom_types.get(name);
                                match re {
                                    Some(_) => {}
                                    None => {
                                        let e = ParserError::err_cannot_find_type(
                                            self.file_path.clone(),
                                            field.field_name_span.clone(),
                                            name,
                                        );

                                        result.push(e);
                                    }
                                }
                            }
                        }
                    }
                }

                {
                    let k = enum_def.enum_name.clone();

                    let type_sign = TypeSignStmt {
                        kind: TypeSignStmtKind::Enum {
                            enum_name: enum_def.enum_name.clone(),
                            fields: enum_def.fields.clone(),
                        },

                        span: Span {
                            start: enum_def.name_span.start.clone(),
                            end: enum_def.name_span.end.clone(),
                        },
                    };

                    // let v = TypeSignStmtKind::Enum {
                    //     enum_name: enum_def.enum_name.clone(),
                    //     fields: enum_def.fields.clone(),
                    // };

                    // self.custom_types.insert(k, v);

                    // self.custom_types.insert(k, v);
                    if self.push_type_def(k, type_sign) {
                        let e = ParserError::err_type_name_alredy_exits(
                            self.file_path.clone(),
                            enum_def.name_span.clone(),
                            &enum_def.enum_name,
                        );
                        result.push(e);
                    };
                }
            }
            StmtKind::UnionDef(union_def) => {
                // union_def.union_name;
                // union_def.base_type;
                // union_def.alowd_values;
                // 如果使用了 Custom 类型, 这个类型是否存在.
                // alowd_values 是否有重复的.

                let re = self.check_duplicated_type_name(
                    self.file_path.clone(),
                    union_def.name_span.clone(),
                    &union_def.union_name,
                );
                if let Some(e) = re {
                    result.push(e);
                }
                // 检查 base_type 是 Custom 时, 这个 Custom 的类型是否存在.
                if let TypeSignStmtKind::Custom(name) = &union_def.base_type {
                    if !self.is_named_type(name) {
                        // return ParserError::err_cannot_find_type(name);

                        let e = ParserError::err_cannot_find_type(
                            self.file_path.clone(),
                            union_def.name_span.clone(),
                            name,
                        );
                        result.push(e);
                    }
                }

                // 检查 alowd_values 的类型是否符合 base_type
                for x in &union_def.allowed_values {
                    if !self.is_same_type(&union_def.base_type, &x.kind) {
                        let e = ParserError::err_mismatched_types(
                            self.file_path.clone(),
                            x.span.clone(),
                            &union_def.base_type.to_cbml_code(0),
                            &format!("{}", &x.kind.to_cbml_code(0)),
                        );
                        result.push(e);
                    }
                }

                // alowd_values 是否有重复的.
                {
                    let _allowed_values: Vec<LiteralKind> = {
                        let mut arr: Vec<LiteralKind> = vec![];
                        for x in &union_def.allowed_values {
                            arr.push(x.kind.clone());
                        }

                        arr
                    };

                    let mut arr: Vec<&LiteralKind> = vec![];
                    // let mut arr: Vec<&LiteralKind> = allowed_values.iter().collect();

                    // for x in &allowed_values {
                    for x in &union_def.allowed_values {
                        if arr.contains(&&x.kind) {
                            // 有重复的项

                            let e = ParserError::err_union_duplicated_item(
                                self.file_path.clone(),
                                x.span.clone(),
                                &x.kind.to_cbml_code(0),
                            );
                            result.push(e);
                        } else {
                            arr.push(&x.kind);
                        }
                    }
                }

                {
                    let k = union_def.union_name.clone();

                    let type_sign = TypeSignStmt {
                        kind: TypeSignStmtKind::Union {
                            base_type: union_def.base_type.clone().into(),
                            alowd_values: union_def.allowed_values.clone(),
                        },
                        span: Span {
                            start: union_def.name_span.start.clone(),
                            end: union_def.name_span.end.clone(),
                        },
                    };

                    // let v = TypeSignStmtKind::Union {
                    //     base_type: union_def.base_type.clone().into(),
                    //     alowd_values: union_def.allowed_values.clone(),
                    // };

                    // self.custom_types.insert(k, v);

                    if self.push_type_def(k, type_sign) {
                        let e = ParserError::err_type_name_alredy_exits(
                            self.file_path.clone(),
                            union_def.name_span.clone(),
                            &union_def.union_name,
                        );

                        result.push(e);
                    };
                }
            }
            StmtKind::Use(_url) => {
                let use_path = _url.get_converted_string();
                self.use_path = Some(use_path.clone());

                // error: 在 use 语句之前有 赋值语句.
                {
                    if !self.asignments.is_empty() {
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
            StmtKind::Asignment(asign) => {
                // 检查 field_name 在 typedef 文件中是否存在.
                // value 字面量类型推导.
                // 检查 field_name 在 typedef 文件中定义的类型.
                // 检查 value 是否符合 field_name 在 typedef 文件中定义的类型.

                // self.custom_types.contains_key(k)

                // 检查 field_name 在 typedef 文件中是否存在.
                match self.defined_fields.get(&asign.field_name) {
                    Some(ty) => {
                        // 检查 value 是否符合 field_name 在 typedef 文件中定义的类型.
                        let field_def = ty.clone();
                        let ty = field_def._type;

                        if !self.is_same_type(&ty.kind, &asign.value.kind) {
                            let e = ParserError::err_mismatched_types(
                                self.file_path.clone(),
                                asign.field_name_span.clone(),
                                &ty.kind.to_cbml_code(0),
                                &asign.value.kind.to_cbml_code(0),
                            );
                            result.push(e);
                        };

                        // 如果 literal 时 `default` 的话,
                        // 检查定义 field 的时候是否设置了默认值.
                        if &asign.value.kind == &LiteralKind::Default {
                            //
                            if let Some(default_value) = field_def.default {
                                // field 定义了默认值.

                                let need_type = &ty.kind;

                                let kind = default_value.kind;
                                if !self.is_same_type(need_type, &kind) {
                                    let e = ParserError::err_mismatched_types(
                                        self.file_path.clone(),
                                        asign.field_name_span.clone(),
                                        &ty.kind.to_cbml_code(0),
                                        &asign.value.kind.to_cbml_code(0),
                                    );
                                    result.push(e);
                                };
                            } else {
                                // field 并没有没定义默认值, 所以不能使用 default 来赋值.

                                let e = ParserError::err_this_field_donot_have_default_value(
                                    self.file_path.clone(),
                                    asign.value.span.clone(),
                                );

                                result.push(e);
                            }

                            // if let Some(sadf) = self.defined_fields.get(&asign.field_name) {
                            //     // field 定义了默认值.
                            //     {};

                            //     let need_type = sadf._type.clone();
                            //     let kind = asign.value.kind.clone();

                            //     if !self.is_same_type(&need_type, &kind) {
                            //         let e = ParserError::err_mismatched_types(
                            //             self.file_path.clone(),
                            //             asign.field_name_span.clone(),
                            //             &ty.to_cbml_code(),
                            //             &asign.value.kind.to_cbml_code(),
                            //         );
                            //         result.push(e);
                            //     };
                            // } else {
                            //     // field 并没有没定义默认值, 所以不能使用 default 来赋值.

                            //     let e = ParserError::err_this_field_donot_have_default_value(
                            //         self.file_path.clone(),
                            //         asign.value.span.clone(),
                            //     );

                            //     result.push(e);
                            // }
                        }
                    }
                    None => {
                        // if self.is_def_file_loaded.is_loaded() {

                        if self.is_def_file_loaded.is_ok() {
                            let e = ParserError::err_unknow_field(
                                self.file_path.clone(),
                                asign.field_name_span.clone(),
                                &asign.field_name,
                            );

                            result.push(e);
                        }
                    }
                };

                // self.push_field_asign(asign.clone());

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
        }
        if result.is_empty() {
            return None;
        } else {
            return Some(result);
        }
    }

    pub fn custom_to_raw(&self, need_type: &TypeSignStmtKind) -> TypeSignStmtKind {
        let mut re = need_type.clone();

        while let TypeSignStmtKind::Custom(name) = &re {
            match self.custom_types.get(name) {
                Some(ty) => re = ty.kind.clone(),
                None => break,
            };
        }

        return re;
    }

    pub fn read_type_def_file(
        &mut self,
        def_file_path: &str,
        code: &str,
    ) -> Option<Vec<ParserError>> {
        use crate::parser::cbml_parser::CbmlParser;

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

        if def_file_path == self.file_path {
            let e = ParserError {
                file_path: self.file_path.clone(),
                msg: format!("类型定义文件中不能使用 use 语句."),
                code_location: Span::empty(),
                note: None,
                help: None,
            };

            return Some(vec![e]);
        }

        // let tokens = tokenizer(def_file_path, &code);
        let re = tokenizer(def_file_path, &code);
        let tokens = match re {
            Ok(a) => a,
            Err(e) => {
                self.is_def_file_loaded = IsDefFileLoaded::ParsedHasError(vec![e.clone()]);

                return Some(vec![e]);
            }
        };

        // dp(format!("tokens: {:?}", tokens));

        let mut parser = CbmlParser::new(def_file_path.to_string(), &tokens);
        let re = parser.parse();

        match re {
            Ok(ast) => {
                self.state = State::InTypedef;
                let re = self.typecheck(&ast);
                self.state = State::InFile;

                if re.is_empty() {
                    // dp("没有检查出类型错误.");
                    return None;
                } else {
                    // has errors.
                    // re.iter().for_each(|x| {
                    //     dp(format!("{:?}", x));
                    // });
                    // panic!();
                    self.is_def_file_loaded = IsDefFileLoaded::ParsedHasError(re.clone());
                    return Some(re);
                }
            }
            Err(e) => {
                self.is_def_file_loaded = IsDefFileLoaded::ParsedHasError(e.clone());
                return Some(e);
            }
        }
    }

    /// 检查字面量的类型是否符合类型定义文件的要求.
    pub fn is_same_type(&mut self, need_type: &TypeSignStmtKind, literal: &LiteralKind) -> bool {
        if let LiteralKind::Default = literal {
            return true;
        }

        match need_type {
            TypeSignStmtKind::String => match literal {
                LiteralKind::String { .. } => true,
                LiteralKind::Default => true,
                _ => false,
            },
            TypeSignStmtKind::Number => match literal {
                LiteralKind::Number(_) => true,
                LiteralKind::Default => true,
                _ => false,
            },
            TypeSignStmtKind::Boolean => match literal {
                LiteralKind::Boolean(_) => true,
                _ => false,
            },
            TypeSignStmtKind::Any => true,
            TypeSignStmtKind::Array { inner_type, .. } => match literal {
                LiteralKind::Array(literals) => {
                    return literals.iter().all(|x| self.is_same_type(inner_type, x));
                }
                _ => false,
            },
            TypeSignStmtKind::Struct(struct_field_def_stmts) => {
                let mut struct_field_def_stmts = struct_field_def_stmts.clone();
                struct_field_def_stmts.sort_by(|x, y| x.field_name.cmp(&y.field_name));

                match literal {
                    LiteralKind::Struct(asignment_stmts) => {
                        if asignment_stmts.len() != struct_field_def_stmts.len() {
                            // 结构体字面量数量不同,
                            // 还有这些 field 需要填写,
                            // 这些 field 没有定义.
                            // TODO:

                            return false;
                        }

                        let mut asignment_stmts = asignment_stmts.clone();

                        asignment_stmts.sort_by(|x, y| x.field_name.cmp(&y.field_name));

                        let afsdf = struct_field_def_stmts.iter().zip(asignment_stmts).all(|x| {
                            let a = x.0;
                            let b = x.1;

                            a.field_name == b.field_name
                                && self.is_same_type(&a._type.kind, &b.value.kind)
                        });

                        return afsdf;
                    }
                    LiteralKind::Todo => {
                        // 不检查 todo.

                        return true;
                    }
                    LiteralKind::Default => todo!("自定义 struct 类型的默认值暂时还未支持"),

                    _ => false,
                }
            }
            TypeSignStmtKind::Union {
                base_type,
                alowd_values,
            } => {
                let arr: Vec<LiteralKind> = {
                    let mut a = vec![];
                    for x in alowd_values {
                        a.push(x.kind.clone());
                    }

                    a
                };

                return arr.contains(literal) && self.is_same_type(base_type, literal);
            }
            TypeSignStmtKind::Optional {
                inner_type,
                // span: _span,
            } => {
                return match literal {
                    LiteralKind::LiteralNone => true,
                    _ => self.is_same_type(inner_type, literal),
                };
            }
            TypeSignStmtKind::Enum {
                enum_name: _enum_name,
                fields,
            } => {
                //
                match literal {
                    LiteralKind::EnumFieldLiteral {
                        field_name: enum_field_literal_name,
                        literal: lit,
                        span: _,
                    } => {
                        let re = fields.iter().any(|x| {
                            &x.field_name == enum_field_literal_name
                                && self.is_same_type(&x._type, lit)
                        });

                        return re;
                    }
                    _ => false,
                }
            }
            TypeSignStmtKind::Custom(type_custom_name) => {
                let re = self.custom_types.get(type_custom_name);

                return match re {
                    Some(t) => {
                        let need = t.kind.clone();
                        return self.is_same_type(&need, literal);
                    }
                    None => false,
                };
            }
        }
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

// trait IsSameType {
//     fn is_same_type(&self, other: &Self) -> bool;
// }

// trait ToCbmltype {
//     fn to_cbmltype(&self) -> CbmlType;
// }

// impl ToCbmltype for CbmlType {
//     fn to_cbmltype(&self) -> CbmlType {
//         return self.clone();
//     }
// }

// impl ToCbmltype for UnionDef {
//     fn to_cbmltype(&self) -> CbmlType {
//         CbmlType::Union {
//             base_type: self.base_type.clone().into(),
//             alowd_values: self.allowed_values.clone(),
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use crate::parser::ast::stmt::{AsignmentStmt, EnumField, Literal, StructFieldDefStmt};

//     use super::*;

//     #[test]
//     fn test_is_same_type_string() {
//         let mut s = TypeChecker::new("".into());

//         assert!(s.is_same_type(
//             &CbmlType::String,
//             &Literal::String {
//                 val: "".into(),
//                 span: Span::empty()
//             }
//         ));

//         assert!(!s.is_same_type(&CbmlType::String, &Literal::Number(1_f64)));
//         assert!(!s.is_same_type(&CbmlType::String, &Literal::Boolean(true)));
//         assert!(!s.is_same_type(&CbmlType::String, &Literal::Array(vec![])));
//     }

//     #[test]
//     fn test_is_same_type_number() {
//         let mut s = TypeChecker::new("".into());
//         assert!(s.is_same_type(&CbmlType::Number, &Literal::Number(1_f64)));
//         assert!(!s.is_same_type(&CbmlType::Number, &Literal::Boolean(true)));
//     }

//     #[test]
//     fn test_is_same_type_array() {
//         let array_a = CbmlType::Array {
//             inner_type: Box::new(CbmlType::String),
//         };

//         let array_b = Literal::Array(vec![
//             &Literal::String {
//                 val: "".into(),
//                 span: Span::empty(),
//             },
//             &Literal::String {
//                 val: "".into(),
//                 span: Span::empty(),
//             },
//         ]);

//         let array_c = Literal::Array(vec![Literal::Number(1_f64), Literal::Number(1_f64)]);

//         let mut s = TypeChecker::new("".into());

//         assert!(s.is_same_type(&array_a, &array_b));

//         assert!(!s.is_same_type(&array_a, &array_c));
//     }

//     #[test]
//     fn test_is_same_type_struct() {
//         let struct_a = CbmlType::Struct(vec![
//             StructFieldDefStmt {
//                 field_name: "field1".to_string(),
//                 _type: CbmlType::String,
//                 default: None,
//                 field_name_span: Span::empty(),
//             },
//             StructFieldDefStmt {
//                 field_name: "field2".to_string(),
//                 _type: CbmlType::Number,
//                 default: None,
//                 field_name_span: Span::empty(),
//             },
//         ]);

//         let struct_b = Literal::Struct(vec![
//             AsignmentStmt {
//                 field_name: "field1".to_string(),
//                 value: Literal::String {
//                     val: "".into(),
//                     span: Span::empty(),
//                 },
//                 field_name_span: Span::empty(),
//             },
//             AsignmentStmt {
//                 field_name: "field2".to_string(),
//                 value: Literal::Number(99.into()),
//                 field_name_span: Span::empty(),
//             },
//         ]);

//         let struct_c = Literal::Struct(vec![
//             AsignmentStmt {
//                 field_name: "field1_sadf".to_string(),
//                 value: Literal::String {
//                     val: "".into(),
//                     span: Span::empty(),
//                 },
//                 field_name_span: Span::empty(),
//             },
//             AsignmentStmt {
//                 field_name: "field2".to_string(),
//                 value: Literal::String {
//                     val: "".into(),
//                     span: Span::empty(),
//                 },
//                 field_name_span: Span::empty(),
//             },
//         ]);

//         let mut s = TypeChecker::new("".into());

//         assert!(s.is_same_type(&struct_a, &struct_b));

//         assert!(!s.is_same_type(&struct_a, &struct_c));
//     }

//     #[test]
//     fn test_is_same_type_union() {
//         let union_a = CbmlType::Union {
//             base_type: Box::new(CbmlType::String),
//             alowd_values: vec![
//                 Literal::String {
//                     val: "value1".into(),
//                     span: Span::empty(),
//                 },
//                 Literal::String {
//                     val: "value1".into(),
//                     span: Span::empty(),
//                 },
//             ],
//         };

//         let union_b = Literal::String {
//             val: "value1".into(),
//             span: Span::empty(),
//         };

//         let union_c = Literal::String {
//             val: "value19999".into(),
//             span: Span::empty(),
//         };

//         let mut s = TypeChecker::new("".into());

//         assert!(s.is_same_type(&union_a, &union_b));

//         assert!(!s.is_same_type(&union_a, &union_c));
//     }

//     #[test]
//     fn test_is_same_type_enum() {
//         let enum_a = CbmlType::Enum {
//             enum_name: "enum1".to_string(),
//             fields: vec![
//                 EnumField {
//                     field_name: "field1".to_string(),
//                     _type: CbmlType::String,
//                 },
//                 EnumField {
//                     field_name: "field2".to_string(),
//                     _type: CbmlType::Number,
//                 },
//             ],
//         };

//         let enum_b = Literal::EnumFieldLiteral {
//             field_name: "field1".into(),
//             literal: Literal::String("()".into()).into(),
//         };

//         let enum_c = Literal::EnumFieldLiteral {
//             field_name: "field1".into(),
//             literal: Literal::LiteralNone.into(),
//         };

//         let mut s = TypeChecker::new("".into());

//         assert!(s.is_same_type(&enum_a, &enum_b));

//         assert!(!s.is_same_type(&enum_a, &enum_c));
//     }

//     #[test]
//     fn test_is_same_type_optional() {
//         let optional_a = CbmlType::Optional {
//             inner_type: Box::new(CbmlType::String),
//         };

//         let optional_b = Literal::String("()".into());

//         let optional_c = Literal::Number(100_f64);
//         let optional_d = Literal::LiteralNone;

//         let mut s = TypeChecker::new("".into());

//         assert!(s.is_same_type(&optional_a, &optional_b));
//         assert!(s.is_same_type(&optional_a, &optional_d));

//         assert!(!s.is_same_type(&optional_a, &optional_c));
//     }
// }

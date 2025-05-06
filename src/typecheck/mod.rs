use crate::dp;
use crate::lexer::token::Span;
use crate::lexer::tokenizer;
use crate::parser::ParserError;
use crate::parser::StmtKind;
use crate::parser::ast::stmt::AsignmentStmt;
use crate::parser::ast::stmt::CbmlType;
use crate::parser::ast::stmt::LiteralKind;
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

enum State {
    /// .cbml
    InFile,
    /// .typedef.cbml
    InTypedef,
}

/// 类型检查
struct TypeChecker {
    type_def_file_path: Option<String>,

    /// 自定义的类型, 例如: struct, enum, union, type alias, named array,
    custom_types: HashMap<String, CbmlType>,

    /// 自定义的 file level field.
    fields: HashMap<String, CbmlType>,

    file_path: String,
    /// field assignment
    asignments: HashMap<String, AsignmentStmt>,

    is_typedefed: bool,

    state: State,
}

enum PushedResult {
    Ok,
    AlreadyExits,
}

impl PushedResult {
    fn is_ok(&self) -> bool {
        match self {
            PushedResult::Ok => true,
            _ => false,
        }
    }
}

impl TypeChecker {
    /// 如果 name 已经存在, 则会返回 true.
    fn push_field_def(&mut self, name: String, ty: CbmlType) -> bool {
        let re = self.fields.insert(name, ty);
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
    fn push_type_def(&mut self, type_name: String, ty: CbmlType) -> bool {
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
    fn new(file_path: String) -> Self {
        TypeChecker {
            custom_types: HashMap::new(),
            is_typedefed: false,
            state: State::InFile,
            fields: HashMap::new(),
            asignments: HashMap::new(),
            type_def_file_path: None,
            file_path: file_path,
        }
    }

    fn typecheck(&mut self, ast: &Vec<StmtKind>) -> Vec<ParserError> {
        let mut re: Vec<ParserError> = vec![];
        for s in ast {
            let asdf = self.check_one(s);
            if let Some(a) = asdf {
                re.push(a);
            }
        }

        return re;
    }

    /// 检查类型的名称是否重复.
    fn check_duplicated_type_name(
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
    fn check_duplicated_file_field_name(
        &self,
        file_path: String,
        name: &str,
        span: Span,
    ) -> Option<ParserError> {
        let re = self.fields.get(name);
        return match re {
            Some(_a) => Some(ParserError::new(
                file_path,
                format!("field `{}` 已经存在: at: ", name,),
                span,
            )),
            None => None,
        };
    }

    fn is_named_type(&self, name: &str) -> bool {
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

    fn check_one(&mut self, stmt: &StmtKind) -> Option<ParserError> {
        let re = self.did_allow_in_state(&stmt);
        if re.is_some() {
            return re;
        };

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

                if re.is_some() {
                    return re;
                }

                // 如果使用了 Custom 类型, 这个类型是否存在.
                if let CbmlType::Custom(name) = &field_def._type {
                    if !self.is_named_type(name) {
                        return Some(ParserError::new(
                            self.file_path.clone(),
                            format!("connot find type {}", name,),
                            field_def.field_name_span.clone(),
                        ));
                    }
                }

                if let Some(default_value) = &field_def.default {
                    if !self.is_same_type(&field_def._type, default_value) {
                        // 类型错误, 需要 {} found {}

                        return ParserError::err_mismatched_types(
                            self.file_path.clone(),
                            field_def.field_name_span.clone(),
                            &field_def._type.to_cbml_code(),
                            &default_value.to_cbml_code(),
                        )
                        .into();
                    }
                }

                {
                    let k = field_def.field_name.clone();
                    let v = field_def._type.clone();

                    if self.push_field_def(k, v) {
                        return ParserError::err_field_alredy_exits(
                            self.file_path.clone(),
                            field_def.field_name_span.clone(),
                            &field_def.field_name,
                        )
                        .into();
                    };
                }

                return None;
            }

            StmtKind::TypeAliasStmt(s) => {
                // 如果使用了 Custom 类型, 这个类型是否存在.
                if self.push_type_def(s.name.clone(), s.ty.clone()) {
                    return Some(ParserError::err_type_name_alredy_exits(
                        self.file_path.clone(),
                        s.name_span.clone(),
                        &s.name,
                    ));
                }

                return None;
            }
            StmtKind::StructDefStmt(struct_def) => {
                let re = self.check_duplicated_type_name(
                    self.file_path.clone(),
                    struct_def.name_span.clone(),
                    &struct_def.struct_name,
                );
                if re.is_some() {
                    return re;
                }

                {
                    // fields 里面是否有重名的.
                    let mut field_names: HashMap<&String, &String> = HashMap::new();

                    for field in struct_def.fields.iter() {
                        let re = field_names.insert(&field.field_name, &field.field_name); // fields 里面是否有重名的.
                        match re {
                            None => {}
                            Some(s) => {
                                return Some(ParserError {
                                    file_path: self.file_path.clone(),
                                    msg: format!("属性名称重复: {}", s),
                                    code_location: field.field_name_span.clone(),
                                    note: None,
                                    help: None,
                                });
                            }
                        };

                        // 如果使用了 Custom 类型, 这个类型是否存在.
                        {
                            if let CbmlType::Custom(ref name) = field._type {
                                let re = self.custom_types.get(name);
                                match re {
                                    Some(_) => {}
                                    None => {
                                        return ParserError::err_cannot_find_type(
                                            self.file_path.clone(),
                                            field.field_name_span.clone(),
                                            name,
                                        )
                                        .into();
                                    }
                                }
                            }
                        }
                    }
                }

                {
                    let k = struct_def.struct_name.clone();
                    let v = CbmlType::Struct(struct_def.fields.clone());

                    // self.custom_types.insert(k, v);
                    if self.push_type_def(k, v) {
                        return ParserError::err_type_name_alredy_exits(
                            self.file_path.clone(),
                            struct_def.name_span.clone(),
                            &struct_def.struct_name,
                        )
                        .into();
                    };
                }

                return None;
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
                                return Some(ParserError {
                                    file_path: self.file_path.clone(),
                                    msg: format!("属性名称重复: {}", s),
                                    code_location: field.field_name_span.clone(),
                                    note: None,
                                    help: None,
                                });
                            }
                        };

                        // 如果使用了 Custom 类型, 这个类型是否存在.
                        {
                            if let CbmlType::Custom(ref name) = field._type {
                                let re = self.custom_types.get(name);
                                match re {
                                    Some(_) => {}
                                    None => {
                                        return Some(ParserError::err_cannot_find_type(
                                            self.file_path.clone(),
                                            todo!(),
                                            name,
                                        ))
                                        .into();
                                    }
                                }
                            }
                        }
                    }
                }

                {
                    let k = enum_def.enum_name.clone();
                    let v = CbmlType::Enum {
                        enum_name: enum_def.enum_name.clone(),
                        fields: enum_def.fields.clone(),
                    };

                    // self.custom_types.insert(k, v);

                    // self.custom_types.insert(k, v);
                    if self.push_type_def(k, v) {
                        return ParserError::err_type_name_alredy_exits(
                            self.file_path.clone(),
                            todo!(),
                            &enum_def.enum_name,
                        )
                        .into();
                    };
                }

                return None;
            }
            StmtKind::UnionDef(union_def) => {
                // union_def.union_name;
                // union_def.base_type;
                // union_def.alowd_values;
                // 如果使用了 Custom 类型, 这个类型是否存在.
                // alowd_values 是否有重复的.

                let re = self.check_duplicated_type_name(
                    self.file_path.clone(),
                    todo!(),
                    &union_def.union_name,
                );
                if re.is_some() {
                    return re;
                }

                // 检查 base_type 是 Custom 时, 这个 Custom 的类型是否存在.
                if let CbmlType::Custom(name) = &union_def.base_type {
                    if !self.is_named_type(name) {
                        // return ParserError::err_cannot_find_type(name);
                        return Some(ParserError::err_cannot_find_type(
                            self.file_path.clone(),
                            todo!(),
                            name,
                        ))
                        .into();
                    }
                }

                // 检查 alowd_values 的类型是否符合 base_type
                for x in &union_def.allowed_values {
                    if !self.is_same_type(&union_def.base_type, x) {
                        return ParserError::err_mismatched_types(
                            self.file_path.clone(),
                            todo!(),
                            &union_def.base_type.to_cbml_code(),
                            &x.to_cbml_code(),
                        )
                        .into();
                    }
                }

                // alowd_values 是否有重复的.
                {
                    let allowed_values: Vec<LiteralKind> = union_def.allowed_values.clone();
                    let mut arr: Vec<&LiteralKind> = vec![];

                    for x in &allowed_values {
                        if arr.contains(&x) {
                            // 有重复的项

                            return ParserError::err_union_duplicated_item(
                                self.file_path.clone(),
                                todo!(),
                                &x.to_cbml_code(),
                            )
                            .into();
                        } else {
                            arr.push(x);
                        }
                    }
                }

                {
                    let k = union_def.union_name.clone();

                    let v = CbmlType::Union {
                        base_type: union_def.base_type.clone().into(),
                        alowd_values: union_def.allowed_values.clone(),
                    };

                    // self.custom_types.insert(k, v);

                    if self.push_type_def(k, v) {
                        return ParserError::err_type_name_alredy_exits(
                            self.file_path.clone(),
                            todo!(),
                            &union_def.union_name,
                        )
                        .into();
                    };
                }

                return None;
            }

            StmtKind::Use(_url) => {
                if self.is_typedefed {
                    return ParserError::err_use_can_only_def_onece(
                        self.file_path.clone(),
                        _url.url_span.clone(),
                    )
                    .into();
                } else {
                    self.is_typedefed = true;
                }

                // 如果是文件 url 则读取文件
                // 如果是网络 url 则下载这个文件.
                let re = std::fs::read_to_string(&_url.url);
                match re {
                    Ok(code) => {
                        // println!("{code}");
                        self.read_typedef(&_url.url, &code);
                    }
                    Err(e) => {
                        eprintln!("error: {:?}", e);
                    }
                };

                return None;
            }
            StmtKind::Asignment(asign) => {
                // 检查 field_name 在 typedef 文件中是否存在.
                // value 字面量类型推导.
                // 检查 field_name 在 typedef 文件中定义的类型.
                // 检查 value 是否符合 field_name 在 typedef 文件中定义的类型.

                // self.custom_types.contains_key(k)

                // 检查 field_name 在 typedef 文件中是否存在.
                match self.fields.get(&asign.field_name) {
                    Some(ty) => {
                        // 检查 value 是否符合 field_name 在 typedef 文件中定义的类型.
                        let ty = ty.clone();
                        if !self.is_same_type(&ty, &asign.value.kind) {
                            return Some(ParserError::err_mismatched_types(
                                self.file_path.clone(),
                                asign.field_name_span.clone(),
                                &ty.to_cbml_code(),
                                &asign.value.kind.to_cbml_code(),
                            ));
                        };
                    }
                    None => {
                        return Some(ParserError::err_unknow_field(
                            self.file_path.clone(),
                            asign.field_name_span.clone(),
                            &asign.field_name,
                        ));
                    }
                };

                // self.push_field_asign(asign.clone());

                if self.push_field_asign(asign.clone()) {
                    return Some(ParserError::err_filed_alredy_asignment(
                        self.file_path.clone(),
                        asign.field_name_span.clone(),
                        &asign,
                    ));
                };

                return None;
            }
            StmtKind::LineComment(_) => None,
            StmtKind::BlockComment(_) => None,
            StmtKind::DocComment(_) => None,
        }
    }

    fn custom_to_raw(&self, need_type: &CbmlType) -> CbmlType {
        let mut re = need_type.clone();

        while let CbmlType::Custom(name) = need_type {
            match self.custom_types.get(name) {
                Some(ty) => re = ty.clone(),
                None => break,
            };
        }

        return re;
    }

    fn read_typedef(&mut self, file_path: &str, code: &str) {
        use crate::parser::cbml_parser::CbmlParser;

        let tokens = tokenizer(file_path, &code)
            .map_err(|e| {
                println!("{:?}", e);
                return e;
            })
            .unwrap();

        // dp(format!("tokens: {:?}", tokens));

        let mut parser = CbmlParser::new(file_path.to_string(), &tokens);
        let re = parser.parse();

        match re {
            Ok(ast) => {
                self.state = State::InTypedef;
                let re = self.typecheck(&ast);
                self.state = State::InFile;

                if re.is_empty() {
                    dp("没有检查出类型错误.");
                } else {
                    // has errors.
                    re.iter().for_each(|x| {
                        dp(format!("{:?}", x));
                    });
                    panic!();
                }
            }
            Err(e) => {
                e.iter().for_each(|s| {
                    dp(format!("message: {:?}", s));
                    // dp(format!("tok: {:?}", s.token));
                });

                panic!();
            }
        }
    }

    fn is_same_type(&mut self, need_type: &CbmlType, literal: &LiteralKind) -> bool {
        match need_type {
            CbmlType::String => match literal {
                LiteralKind::String { .. } => true,
                _ => false,
            },
            CbmlType::Number => match literal {
                LiteralKind::Number(_) => true,
                _ => false,
            },
            CbmlType::Boolean => match literal {
                LiteralKind::Boolean(_) => true,
                _ => false,
            },
            CbmlType::Any => true,
            CbmlType::Array { inner_type } => match literal {
                LiteralKind::Array(literals) => {
                    return literals.iter().all(|x| self.is_same_type(inner_type, x));
                }
                _ => false,
            },
            CbmlType::Struct(struct_field_def_stmts) => {
                let mut struct_field_def_stmts = struct_field_def_stmts.clone();
                struct_field_def_stmts.sort_by(|x, y| x.field_name.cmp(&y.field_name));

                match literal {
                    LiteralKind::Struct(asignment_stmts) => {
                        if asignment_stmts.len() != struct_field_def_stmts.len() {
                            return false;
                        }

                        let mut asignment_stmts = asignment_stmts.clone();

                        asignment_stmts.sort_by(|x, y| x.field_name.cmp(&y.field_name));

                        let afsdf = struct_field_def_stmts.iter().zip(asignment_stmts).all(|x| {
                            let a = x.0;
                            let b = x.1;

                            a.field_name == b.field_name
                                && self.is_same_type(&a._type, &b.value.kind)
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
                // return literal.to_cbml_type() == CbmlType::Struct(struct_field_def_stmts.clone());
            }
            CbmlType::Union {
                base_type,
                alowd_values,
            } => alowd_values.contains(literal) && self.is_same_type(base_type, literal),
            CbmlType::Optional { inner_type } => match literal {
                LiteralKind::LiteralNone => true,
                _ => self.is_same_type(inner_type, literal),
            },
            CbmlType::Enum {
                enum_name: _enum_name,
                fields,
            } => {
                //
                match literal {
                    LiteralKind::EnumFieldLiteral {
                        field_name: enum_field_literal_name,
                        literal: lit,
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
            CbmlType::Custom(type_custom_name) => {
                let re = self.custom_types.get(type_custom_name);

                return match re {
                    Some(t) => {
                        let need = t.clone();
                        return self.is_same_type(&need, literal);
                    }
                    None => false,
                };
            }
        }
    }
}

struct TypeCheckedResult;

struct TypeCheckError(String);

// #[derive(Debug)]
// pub enum TypeCheckedResult {
//     Ok,
//     Warning,
//     Error(String),
// }

impl ParserError {}

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

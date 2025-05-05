use std::collections::{HashMap, HashSet};

use crate::parser::{
    Stmt,
    ast::stmt::{CbmlType, EnumField, Literal, UnionDef},
};
// *    名称重复
// •	错误位置
// •	期望类型 vs 实际类型
// •	推测失败原因
pub fn typecheck(ast: Vec<Stmt>) -> Vec<TypeCheckedResult> {
    let mut type_checker = TypeChecker::new();

    return type_checker.typecheck(ast);
}

/// 类型检查
struct TypeChecker {
    /// 自定义的类型, 例如: struct, enum, union, type alias, named array,
    custom_types: HashMap<String, CbmlType>,
}

impl TypeChecker {
    fn new() -> Self {
        TypeChecker {
            custom_types: HashMap::new(),
        }
    }
    fn typecheck(&mut self, ast: Vec<Stmt>) -> Vec<TypeCheckedResult> {
        let mut re: Vec<TypeCheckedResult> = vec![];
        for s in ast {
            let asdf = self.check_one(s);
            if !asdf.is_ok() {
                re.push(asdf);
            }
        }

        return re;
    }

    /// 检查类型的名称是否重复.
    fn check_duplicated_name(&self, name: &str) -> TypeCheckedResult {
        let re = self.custom_types.get(name);
        return match re {
            Some(_a) => TypeCheckedResult::Error(format!("类型 `{}` 已经存在: at: ", name,)),
            None => TypeCheckedResult::Ok,
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

    fn check_one(&mut self, stmt: Stmt) -> TypeCheckedResult {
        match stmt {
            Stmt::FileFieldStmt(ref struct_field_def_stmt) => {
                // struct_field_def_stmt.field_name;
                // struct_field_def_stmt.default;
                // struct_field_def_stmt.ty; // 如果使用了 Custom 类型, 这个类型是否存在.

                // 名称是否重复
                let re = self.check_duplicated_name(&struct_field_def_stmt.field_name);
                if !re.is_ok() {
                    return re;
                }

                // 如果使用了 Custom 类型, 这个类型是否存在.
                if let CbmlType::Custom(name) = &struct_field_def_stmt.ty {
                    if !self.is_named_type(name) {
                        return TypeCheckedResult::err_cannot_find_type(name);
                    }
                }

                if let Some(default_value) = &struct_field_def_stmt.default {
                    if !self.is_same_type(&struct_field_def_stmt.ty, default_value) {
                        // 类型错误, 需要 {} found {}

                        return TypeCheckedResult::err_mismatched_types(
                            &struct_field_def_stmt.ty.to_cbml_code(),
                            &default_value.to_cbml_type().to_cbml_code(),
                        );
                    }
                }

                {
                    let k = struct_field_def_stmt.field_name.clone();
                    let v = struct_field_def_stmt.ty.clone();

                    self.custom_types.insert(k, v);
                }
                return TypeCheckedResult::Ok;
            }
            Stmt::TypeAliasStmt(name, cbml_type) => {
                // 如果使用了 Custom 类型, 这个类型是否存在.

                todo!();
            }
            Stmt::StructDefStmt(struct_def) => {
                let re = self.check_duplicated_name(&struct_def.struct_name);
                if !re.is_ok() {
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
                                return TypeCheckedResult::Error(format!("属性名称重复: {}", s));
                            }
                        };

                        // 如果使用了 Custom 类型, 这个类型是否存在.
                        {
                            if let CbmlType::Custom(ref name) = field.ty {
                                let re = self.custom_types.get(name);
                                match re {
                                    Some(_) => {}
                                    None => {
                                        return TypeCheckedResult::err_cannot_find_type(name);
                                        // return TypeCheckedResult::Error(format!(
                                        //     "connot find type `{}` ",
                                        //     name
                                        // ));
                                    }
                                }
                            }
                        }
                    }
                }

                {
                    let k = struct_def.struct_name.clone();
                    let v = CbmlType::Struct(struct_def.fields);

                    self.custom_types.insert(k, v);
                }

                return TypeCheckedResult::Ok;
            }
            Stmt::EnumDef(ref enum_def) => {
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
                                return TypeCheckedResult::Error(format!("属性名称重复: {}", s));
                            }
                        };

                        // 如果使用了 Custom 类型, 这个类型是否存在.
                        {
                            if let CbmlType::Custom(ref name) = field.ty {
                                let re = self.custom_types.get(name);
                                match re {
                                    Some(_) => {}
                                    None => {
                                        return TypeCheckedResult::Error(format!(
                                            "connot find type `{}` ",
                                            name
                                        ));
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

                    self.custom_types.insert(k, v);
                }

                return TypeCheckedResult::Ok;
            }
            Stmt::UnionDef(union_def) => {
                // union_def.union_name;
                // union_def.base_type;
                // union_def.alowd_values;
                // 如果使用了 Custom 类型, 这个类型是否存在.
                // alowd_values 是否有重复的.

                let re = self.check_duplicated_name(&union_def.union_name);
                if !re.is_ok() {
                    return re;
                }

                // 检查 base_type 是 Custom 时, 这个 Custom 的类型是否存在.
                if let CbmlType::Custom(name) = &union_def.base_type {
                    if !self.is_named_type(name) {
                        return TypeCheckedResult::err_cannot_find_type(name);
                    }
                }

                // 检查 alowd_values 的类型是否符合 base_type
                for x in &union_def.allowed_values {
                    if !self.is_same_type(&union_def.base_type, x) {
                        return TypeCheckedResult::err_mismatched_types(
                            &union_def.base_type.to_cbml_code(),
                            &x.to_cbml_type().to_cbml_code(),
                        );
                    }
                }

                // alowd_values 是否有重复的.
                {
                    let allowed_values: Vec<Literal> = union_def.allowed_values.clone();
                    let mut arr: Vec<&Literal> = vec![];

                    for x in &allowed_values {
                        if arr.contains(&x) {
                            // 有重复的项

                            return TypeCheckedResult::err_union_duplicated_item(&x.to_cbml_code());
                        } else {
                            arr.push(x);
                        }
                    }
                }

                {
                    let k = union_def.union_name.clone();

                    let v = CbmlType::Union {
                        base_type: union_def.base_type.clone().into(),
                        alowd_values: union_def.allowed_values,
                    };

                    self.custom_types.insert(k, v);
                }

                return TypeCheckedResult::Ok;
            }

            Stmt::Use(url) => {
                // 如果是文件 url 则读取文件
                // 如果是网络 url 则下载这个文件.

                todo!();
            }
            Stmt::Asignment(asign) => {
                // 检查 field_name 在 typedef 文件中是否存在.
                // value 字面量类型推导.
                // 检查 field_name 在 typedef 文件中定义的类型.
                // 检查 value 是否符合 field_name 在 typedef 文件中定义的类型.

                // self.custom_types.contains_key(k)

                // 检查 field_name 在 typedef 文件中是否存在.
                match self.custom_types.get(&asign.field_name) {
                    Some(ty) => {
                        // 检查 value 是否符合 field_name 在 typedef 文件中定义的类型.
                        let ty = ty.clone();
                        if !self.is_same_type(&ty, &asign.value) {
                            return TypeCheckedResult::err_mismatched_types(
                                &ty.to_cbml_code(),
                                &ty.to_cbml_code(),
                            );
                        };
                    }
                    None => {
                        return TypeCheckedResult::err_cannot_find_type(&asign.field_name);
                    }
                };

                return TypeCheckedResult::Ok;
            }
            Stmt::LineComment(_) => TypeCheckedResult::Ok,
            Stmt::BlockComment(_) => TypeCheckedResult::Ok,
            Stmt::DocComment(_) => TypeCheckedResult::Ok,
        }
    }
}

#[derive(Debug)]
pub enum TypeCheckedResult {
    Ok,
    Warning(),
    Error(String),
}

impl TypeCheckedResult {
    fn is_ok(&self) -> bool {
        match self {
            TypeCheckedResult::Ok => true,
            _ => false,
        }
    }

    fn err_cannot_find_type(type_name: &str) -> Self {
        TypeCheckedResult::Error(format!("connot find type `{}` ", type_name))
    }

    fn err_mismatched_types(expected: &str, found: &str) -> Self {
        TypeCheckedResult::Error(format!(
            "mismatched types, expected `{}` found  `{}` ",
            expected, found
        ))
    }

    fn err_union_duplicated_item(item: &str) -> Self {
        TypeCheckedResult::Error(format!("union duplicated item: {}", item))
    }
}

/// 类型推导
fn type_inference() {}

trait IsSameType {
    fn is_same_type(&self, other: &Self) -> bool;
}

// impl IsSameType for CbmlType {
impl TypeChecker {
    fn is_same_type(&mut self, need_type: &CbmlType, literal: &Literal) -> bool {
        match need_type {
            CbmlType::String => match literal {
                Literal::String(_) => true,
                _ => false,
            },
            CbmlType::Number => match literal {
                Literal::Number(_) => true,
                _ => false,
            },
            CbmlType::Boolean => match literal {
                Literal::Boolean(_) => true,
                _ => false,
            },
            CbmlType::Any => todo!(),
            CbmlType::Array { inner_type } => match literal {
                Literal::Array(literals) => {
                    return literals.iter().all(|x| self.is_same_type(inner_type, x));
                }
                _ => false,
            },
            CbmlType::Struct(struct_field_def_stmts) => {
                literal.to_cbml_type() == CbmlType::Struct(struct_field_def_stmts.clone())
            }
            CbmlType::Union {
                base_type,
                alowd_values,
            } => alowd_values.contains(literal) && self.is_same_type(base_type, literal),
            CbmlType::Optional { inner_type } => match literal {
                Literal::LiteralNone => true,
                _ => self.is_same_type(inner_type, literal),
            },
            CbmlType::Enum {
                enum_name: _enum_name,
                fields,
            } => match literal {
                Literal::EnumField {
                    field_name,
                    literal: x,
                } => {
                    let enum_field = EnumField {
                        field_name: field_name.clone(),
                        ty: x.to_cbml_type(),
                    };

                    fields.contains(&enum_field)
                }
                _ => false,
            },
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
        // match (need_type, literal) {
        //     (CbmlType::String, CbmlType::String) => true,
        //     (CbmlType::String, CbmlType::Custom(s)) => {
        //         todo!();
        //     }
        //     (CbmlType::Number, CbmlType::Number) => true,
        //     (CbmlType::Boolean, CbmlType::Boolean) => true,
        //     (CbmlType::Any, CbmlType::Any) => true,
        //     (CbmlType::Array { inner_type: a }, CbmlType::Array { inner_type: b }) => {
        //         self.is_same_type(a, b)
        //     }
        //     (CbmlType::Struct(_a), CbmlType::Struct(_b)) => {
        //         let mut a = _a.clone();
        //         let mut b = _b.clone();

        //         a.sort_by(|x, y| x.field_name.cmp(&y.field_name));
        //         b.sort_by(|x, y| x.field_name.cmp(&y.field_name));

        //         a.len() == b.len() // 匿名结构体的属性数量相同
        //             && a.iter()
        //                 .zip(b.iter())
        //                 .all(|(a, b)| a.field_name == b.field_name && self.is_same_type(&a.ty, &b.ty))
        //     }
        //     (
        //         CbmlType::Union {
        //             base_type: base_type_a,
        //             alowd_values: alowd_values_a,
        //         },
        //         CbmlType::Union {
        //             base_type: base_type_b,
        //             alowd_values: alowd_values_b,
        //         },
        //     ) => self.is_same_type(base_type_a, base_type_b) && alowd_values_a == alowd_values_b,
        //     (CbmlType::Optional { inner_type: a }, CbmlType::Optional { inner_type: b }) => {
        //         self.is_same_type(a, b)
        //     }
        //     (
        //         CbmlType::Enum {
        //             field_name: field_name_a,
        //             fields: fields_a,
        //         },
        //         CbmlType::Enum {
        //             field_name: field_name_b,
        //             fields: fields_b,
        //         },
        //     ) => {
        //         // 名称相同
        //         // fields 数量相同
        //         // 相同名称的 field 的类型也相同.

        //         let same_enum_name = field_name_a == field_name_b;

        //         let mut fa = fields_a.clone();
        //         let mut fb = fields_b.clone();

        //         fa.sort_by(|x, y| x.field_name.cmp(&y.field_name));
        //         fb.sort_by(|x, y| x.field_name.cmp(&y.field_name));

        //         let same_len = fa.len() == fb.len(); // 匿名结构体的属性数量相同

        //         let all_field_same_name_and_same_type = fa.iter().zip(fb.iter()).all(|(a, b)| {
        //             let field_same_name: bool = a.field_name == b.field_name; // field name are same
        //             let field_same_type = self.is_same_type(&a.ty, &b.ty); // field type are same

        //             return field_same_name && field_same_type;
        //         });

        //         return same_enum_name && same_len && all_field_same_name_and_same_type;
        //     }
        //     (CbmlType::Custom(a), CbmlType::Custom(b)) => a == b,
        //     _ => false,
        // }
    }
}

trait ToCbmltype {
    fn to_cbmltype(&self) -> CbmlType;
}

impl ToCbmltype for CbmlType {
    fn to_cbmltype(&self) -> CbmlType {
        return self.clone();
    }
}

impl ToCbmltype for UnionDef {
    fn to_cbmltype(&self) -> CbmlType {
        CbmlType::Union {
            base_type: self.base_type.clone().into(),
            alowd_values: self.allowed_values.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::stmt::{AsignmentStmt, EnumField, Literal, StructFieldDefStmt};

    use super::*;

    #[test]
    fn test_is_same_type_string() {
        let mut s = TypeChecker::new();

        assert!(s.is_same_type(&CbmlType::String, &Literal::String("()".into())));

        assert!(!s.is_same_type(&CbmlType::String, &Literal::Number(1_f64)));
        assert!(!s.is_same_type(&CbmlType::String, &Literal::Boolean(true)));
        assert!(!s.is_same_type(&CbmlType::String, &Literal::Array(vec![])));
    }

    #[test]
    fn test_is_same_type_number() {
        let mut s = TypeChecker::new();
        assert!(s.is_same_type(&CbmlType::Number, &Literal::Number(1_f64)));
        assert!(!s.is_same_type(&CbmlType::Number, &Literal::Boolean(true)));
    }

    #[test]
    fn test_is_same_type_array() {
        let array_a = CbmlType::Array {
            inner_type: Box::new(CbmlType::String),
        };

        let array_b = Literal::Array(vec![
            Literal::String("()".into()),
            Literal::String("()".into()),
        ]);

        let array_c = Literal::Array(vec![Literal::Number(1_f64), Literal::Number(1_f64)]);

        let mut s = TypeChecker::new();

        assert!(s.is_same_type(&array_a, &array_b));

        assert!(!s.is_same_type(&array_a, &array_c));
    }

    #[test]
    fn test_is_same_type_struct() {
        let struct_a = CbmlType::Struct(vec![
            StructFieldDefStmt {
                field_name: "field1".to_string(),
                ty: CbmlType::String,
                default: None,
            },
            StructFieldDefStmt {
                field_name: "field2".to_string(),
                ty: CbmlType::Number,
                default: None,
            },
        ]);

        let struct_b = Literal::Struct(vec![
            AsignmentStmt {
                field_name: "field1".to_string(),
                value: Literal::String("()".into()),
            },
            AsignmentStmt {
                field_name: "field2".to_string(),
                value: Literal::Number(99.into()),
            },
        ]);

        let struct_c = Literal::Struct(vec![
            AsignmentStmt {
                field_name: "field1_sadf".to_string(),
                value: Literal::String("()".into()),
            },
            AsignmentStmt {
                field_name: "field2".to_string(),
                value: Literal::String("()".into()),
            },
        ]);

        let mut s = TypeChecker::new();

        assert!(s.is_same_type(&struct_a, &struct_b));

        assert!(!s.is_same_type(&struct_a, &struct_c));
    }

    #[test]
    fn test_is_same_type_union() {
        let union_a = CbmlType::Union {
            base_type: Box::new(CbmlType::String),
            alowd_values: vec![
                Literal::String("value1".to_string()),
                Literal::String("value2".to_string()),
            ],
        };

        let union_b = Literal::String("value1".into());

        let union_c = Literal::String("value99999".into());

        let mut s = TypeChecker::new();

        assert!(s.is_same_type(&union_a, &union_b));

        assert!(!s.is_same_type(&union_a, &union_c));
    }

    #[test]
    fn test_is_same_type_enum() {
        let enum_a = CbmlType::Enum {
            enum_name: "enum1".to_string(),
            fields: vec![
                EnumField {
                    field_name: "field1".to_string(),
                    ty: CbmlType::String,
                },
                EnumField {
                    field_name: "field2".to_string(),
                    ty: CbmlType::Number,
                },
            ],
        };

        let enum_b = Literal::EnumField {
            field_name: "field1".into(),
            literal: Literal::String("()".into()).into(),
        };

        let enum_c = Literal::EnumField {
            field_name: "field1".into(),
            literal: Literal::LiteralNone.into(),
        };

        let mut s = TypeChecker::new();

        assert!(s.is_same_type(&enum_a, &enum_b));

        assert!(!s.is_same_type(&enum_a, &enum_c));
    }

    #[test]
    fn test_is_same_type_optional() {
        let optional_a = CbmlType::Optional {
            inner_type: Box::new(CbmlType::String),
        };

        let optional_b = Literal::String("()".into());

        let optional_c = Literal::Number(100_f64);
        let optional_d = Literal::LiteralNone;

        let mut s = TypeChecker::new();

        assert!(s.is_same_type(&optional_a, &optional_b));
        assert!(s.is_same_type(&optional_a, &optional_d));

        assert!(!s.is_same_type(&optional_a, &optional_c));
    }
}

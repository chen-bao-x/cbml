use crate::cbml_project::types::FieldDef;
use crate::cbml_value::value::*;
use crate::parser::ast::stmt::*;

pub trait ToCbmlCode {
    fn to_cbml_code(&self, deepth: usize) -> String;
}

impl ToCbmlCode for Vec<Stmt> {
    fn to_cbml_code(&self, deepth: usize) -> String {
        let mut re = String::new();
        for x in self {
            // re.push_str("\n");
            re.push_str(&x.to_cbml_code(deepth));
            re.push_str("\n");
        }

        return re;
    }
}

impl ToCbmlCode for Stmt {
    fn to_cbml_code(&self, deepth: usize) -> String {
        self.kind.to_cbml_code(deepth)
    }
}

impl ToCbmlCode for Vec<StmtKind> {
    fn to_cbml_code(&self, deepth: usize) -> String {
        let mut re = String::new();

        for x in self {
            match &x {
                StmtKind::LineComment(_) => {
                    re.push_str("\n");
                }
                StmtKind::DocComment(_) => {
                    re.push_str("\n");
                }
                _ => {
                    // top level field 间隔一行更好看.
                    // if deepth == 0 {
                    //     re.push_str("\n");
                    // }
                }
            };

            re.push_str(&x.to_cbml_code(deepth));

            match &x {
                StmtKind::LineComment(_) => {}
                StmtKind::DocComment(_) => {}
                _ => {
                    // top level field 间隔一行更好看.
                    if deepth == 0 {
                        re.push_str("\n");
                    }
                }
            };
        }

        return re;
    }
}
impl ToCbmlCode for StmtKind {
    fn to_cbml_code(&self, deepth: usize) -> String {
        match self {
            StmtKind::Use(use_stmt) => use_stmt.to_cbml_code(deepth),
            StmtKind::Asignment(asignment_stmt) => asignment_stmt.to_cbml_code(deepth),
            StmtKind::FileFieldStmt(struct_field_def_stmt) => {
                struct_field_def_stmt.to_cbml_code(deepth)
            }
            StmtKind::TypeAliasStmt(type_alias_stmt) => type_alias_stmt.to_cbml_code(deepth),
            StmtKind::StructDefStmt(struct_def) => struct_def.to_cbml_code(deepth),
            StmtKind::EnumDef(enum_def) => enum_def.to_cbml_code(deepth),
            // StmtKind::UnionDef(union_def) => union_def.to_cbml_code(deepth),
            StmtKind::LineComment(s) => format!("{}", s),
            StmtKind::BlockComment(s) => format!("{}", s),
            StmtKind::DocComment(s) => format!("{}", s.document),
            StmtKind::EmptyLine => "\n".to_string(),
            StmtKind::TypeDef(type_def_stmt) => match type_def_stmt {
                // TypeDefStmt::TypeAliasStmt(type_alias_stmt) => type_alias_stmt.to_cbml_code(deepth),
                TypeDefStmt::StructDefStmt(struct_def) => struct_def.to_cbml_code(deepth),
                TypeDefStmt::EnumDef(enum_def) => enum_def.to_cbml_code(deepth),
                TypeDefStmt::UnionDef(union_def) => union_def.to_cbml_code(deepth),
            },
        }
    }
}

impl ToCbmlCode for UseStmt {
    fn to_cbml_code(&self, deepth: usize) -> String {
        _ = deepth;

        if self.url.starts_with("\"") && self.url.ends_with("\"") {
            return format!("use {}", self.url);
        } else {
            format!("use \"{}\"", self.url)
        }
    }
}

impl ToCbmlCode for TypeAliasStmt {
    fn to_cbml_code(&self, deepth: usize) -> String {
        let mut re = String::new();
        re.push_str(&format!("type {} = ", self.name));
        re.push_str(&self.ty.to_cbml_code(deepth));
        re.push_str("\n");

        return re;
    }
}
impl ToCbmlCode for AsignmentStmt {
    fn to_cbml_code(&self, deepth: usize) -> String {
        let mut re = String::new();

        re.push_str(&format!(
            "{}{} = {}",
            "    ".repeat(deepth),
            self.field_name,
            self.value.kind.to_cbml_code(deepth)
        ));

        return re;
    }
}

impl ToCbmlCode for Vec<StructFieldDefStmt> {
    fn to_cbml_code(&self, deepth: usize) -> String {
        let mut re = String::new();

        for x in self {
            re.push_str(&x.to_cbml_code(deepth));
            re.push('\n');
        }

        return re;
    }
}
impl ToCbmlCode for StructFieldDefStmt {
    fn to_cbml_code(&self, deepth: usize) -> String {
        let mut re = String::new();

        if let Some(doc) = &self.doc {
            doc.document.lines().for_each(|x| {
                re.push_str(&format!("///{}\n", x));
            });
        }
        re.push_str(&"    ".repeat(deepth));
        re.push_str(&self.field_name);
        re.push_str(": ");
        re.push_str(&self._type.kind.to_cbml_code(deepth));

        if let Some(default) = &self.default {
            re.push_str(" default ");
            re.push_str(&default.kind.to_cbml_code(deepth));
        }

        return re;
    }
}

impl ToCbmlCode for EnumFieldDef {
    fn to_cbml_code(&self, deepth: usize) -> String {
        let mut re = String::new();
        re.push_str(&"    ".repeat(deepth));
        re.push_str(&self.field_name);
        re.push_str("(");
        re.push_str(&self._type.to_cbml_code(deepth));
        re.push_str(") ");

        return re;
    }
}

impl ToCbmlCode for Literal {
    fn to_cbml_code(&self, deepth: usize) -> String {
        self.kind.to_cbml_code(deepth)
    }
}

impl ToCbmlCode for LiteralKind {
    fn to_cbml_code(&self, deepth: usize) -> String {
        match self {
            LiteralKind::String(s) => {
                let mut re = String::new();
                re.push_str(&format!("{}", s));
                return re;
            }
            LiteralKind::Number(n) => {
                let mut re = String::new();
                re.push_str(&format!("{}", n));
                return re;
            }
            LiteralKind::Boolean(b) => {
                let mut re = String::new();
                re.push_str(&format!("{}", b));
                return re;
            }
            LiteralKind::Array(literals) => {
                let mut re = String::new();
                re.push_str("[");

                let mut count = 0;
                for l in literals {
                    let a = if count < literals.len() - 1 { ", " } else { "" };
                    re.push_str(&format!("{}{}", l.to_cbml_code(deepth), a));
                    count += 1;
                }

                re.push_str("]");

                // 换行风格.
                {
                    if re.contains("\n") {
                        re.clear();

                        re.push_str("[\n");

                        for l in literals {
                            re.push_str(&format!(
                                "{}{},\n",
                                "    ".repeat(deepth + 1),
                                l.to_cbml_code(deepth + 1)
                            ));
                        }

                        re.push_str("]");
                    }
                }
                return re;
            }
            LiteralKind::Struct(asignment_stmts) => {
                let mut re = String::new();
                re.push_str("{");

                {
                    let newline_style: String = asignment_stmts
                        .iter()
                        .map(|x| x.to_cbml_code(deepth + 1)) // 每一个 stmt 转换为代码.
                        .fold("\n".to_string(), |mut a, b| {
                            a.push_str(&b);

                            a.push('\n'); // 添加分隔符
                            a
                        });

                    // {
                    //     if newline_style.len() < 100 && deepth == 0 {
                    //         let mut count = 0;
                    //         let comma_style: String = asignment_stmts
                    //             .iter()
                    //             .map(|x| x.to_cbml_code(0)) // 每一个 stmt 转换为代码.
                    //             .fold(" ".to_string(), |mut a, b| {
                    //                 a.push_str(&b);

                    //                 // 避免添加最后一个逗号.
                    //                 if count < asignment_stmts.len() - 1 {
                    //                     a.push_str(", "); // 添加分隔符
                    //                 }
                    //                 count += 1;
                    //                 // a.push(','); // 添加分隔符
                    //                 a
                    //             });

                    //         re.push_str(&comma_style);
                    //         re.push_str(" }");
                    //     } else {
                    //         re.push_str(&newline_style);
                    //         re.push_str(&"    ".repeat(deepth));
                    //         re.push_str("}");
                    //     }
                    // }
                    re.push_str(&newline_style);
                    re.push_str(&"    ".repeat(deepth));
                    re.push_str("}");
                }

                return re;
            }

            LiteralKind::EnumFieldLiteral {
                field_name: _field_name,
                literal: _literal,
                span: _,
            } => {
                let mut re = String::new();
                re.push_str(_field_name);
                re.push('(');

                re.push_str(&_literal.to_cbml_code(deepth));

                re.push(')');
                return re;
            }
            LiteralKind::LiteralNone => {
                let mut re = String::new();
                re.push_str("none");
                return re;
            }
            LiteralKind::Todo => {
                let mut re = String::new();
                re.push_str("todo");
                return re;
            }
            LiteralKind::Default => {
                let mut re = String::new();
                re.push_str("default");
                return re;
            }
        }
    }
}

impl ToCbmlCode for TypeSignStmt {
    fn to_cbml_code(&self, deepth: usize) -> String {
        let mut re = String::new();
        re.push_str(&self.kind.to_cbml_code(deepth));
        re.push_str("\n");

        return re;
    }
}

impl ToCbmlCode for TypeSignStmtKind {
    fn to_cbml_code(&self, deepth: usize) -> String {
        match self {
            TypeSignStmtKind::String => format!("string"),
            TypeSignStmtKind::Number => format!("number"),
            TypeSignStmtKind::Boolean => format!("bool"),
            TypeSignStmtKind::Any => format!("any"),

            TypeSignStmtKind::Custom(name) => name.clone(),
            TypeSignStmtKind::Anonymous(anonymous_type_def_stmt) => {
                match &anonymous_type_def_stmt.kind {
                    AnonymousTypeDefKind::Enum { fields } => {
                        let mut str = String::new();
                        str.push_str(&format!("enum {{",));
                        for field in fields {
                            str.push_str(&format!(
                                "{}( {} )\n ",
                                field.field_name,
                                field._type.to_cbml_code(deepth)
                            ));
                        }
                        str.push_str(r"}");
                        return str;
                    }
                    AnonymousTypeDefKind::Struct(struct_field_def_stmts) => {
                        let mut re = String::new();
                        re.push_str("{\n");

                        re.push_str(&struct_field_def_stmts.to_cbml_code(deepth + 1));

                        re.push_str(&"    ".repeat(deepth));

                        re.push_str("}");
                        return re;
                    }
                    AnonymousTypeDefKind::Union {
                        // base_type,
                        alowd_values,
                    } => {
                        let mut str = String::new();
                        let mut counter = 0;

                        alowd_values.iter().for_each(|x| {
                            counter += 1;
                            if counter < alowd_values.len() {
                                str.push_str(&format!("{} | ", x.to_cbml_code(deepth)));
                            } else {
                                str.push_str(&format!("{} ", x.to_cbml_code(deepth)));
                            }
                        });

                        return str;
                    }
                    AnonymousTypeDefKind::Array { inner_type } => {
                        format!("[{}]", inner_type.to_cbml_code(deepth + 1))
                    }
                    AnonymousTypeDefKind::Optional { inner_type } => {
                        format!("?{}", inner_type.to_cbml_code(deepth))
                    }
                }
            }
        }
    }
}

impl ToCbmlCode for StructDef {
    fn to_cbml_code(&self, deepth: usize) -> String {
        let mut re = String::new();

        re.push_str(&format!("struct {} {{\n", self.struct_name));

        re.push_str(&self.fields.to_cbml_code(deepth + 1));

        re.push_str("}");

        return re;
    }
}

impl ToCbmlCode for EnumDef {
    fn to_cbml_code(&self, deepth: usize) -> String {
        let mut re = String::new();

        re.push_str(&format!("enum {} ", self.enum_name));
        re.push_str("{\n");
        for field in &self.fields {
            let a = field.to_cbml_code(deepth + 1);
            re.push_str(&a);
            re.push_str("\n");
        }
        re.push_str("}");
        return re;
    }
}

impl ToCbmlCode for UnionDef {
    fn to_cbml_code(&self, deepth: usize) -> String {
        let mut re = String::new();
        re.push_str(&format!("union {} = ", self.union_name));

        re.push_str("(");
        re.push_str(&self.base_type.to_cbml_code(deepth));
        re.push_str(")");
        re.push_str(" = ");

        for x in &self.allowed_values {
            re.push_str(&x.kind.to_cbml_code(deepth));
            re.push_str(" | ");
        }

        re.push_str("\n");
        return re;
    }
}

// CbmlValue

impl ToCbmlCode for CbmlType {
    fn to_cbml_code(&self, deepth: usize) -> String {
        self.kind.to_cbml_code(deepth)
    }
}

impl ToCbmlCode for CbmlValue {
    fn to_cbml_code(&self, deepth: usize) -> String {
        match self {
            CbmlValue::None => format!("none"),
            CbmlValue::String(s) => format!("{}", s),
            CbmlValue::Number(n) => format!("{}", n),
            CbmlValue::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
            CbmlValue::Array(cbml_values) => {
                let mut re = String::new();
                re.push_str("[");

                for l in cbml_values {
                    re.push_str(&format!("{}, ", l.to_cbml_code(deepth)));
                }

                re.push_str("]");

                if re.contains("\n") {
                    re.clear();

                    re.push_str("[\n");

                    for l in cbml_values {
                        re.push_str(&format!(
                            "{}{},\n",
                            "    ".repeat(deepth + 1),
                            l.to_cbml_code(deepth + 1)
                        ));
                    }

                    re.push_str("]");
                }
                return re;
            }
            CbmlValue::Struct(hash_map) => {
                let mut re = String::new();
                re.push_str("{");

                {
                    let newline_style: String = hash_map
                        .iter()
                        .map(|x| x.1.to_cbml_code(deepth + 1)) // 每一个 stmt 转换为代码.
                        .fold("\n".to_string(), |mut a, b| {
                            a.push_str(&b);

                            a.push('\n'); // 添加分隔符
                            a
                        });

                    re.push_str(&newline_style);
                    re.push_str(&"    ".repeat(deepth));
                    re.push_str("}");
                }

                return re;
            }
            CbmlValue::EnumField(name, cbml_value) => {
                let mut re = String::new();
                re.push_str(&format!("{} (", name));
                re.push_str(&cbml_value.to_cbml_code(0));
                re.push_str(")\n");
                return re;
            }
        }
    }
}

impl ToCbmlCode for CbmlTypeKind {
    fn to_cbml_code(&self, deepth: usize) -> String {
        match self {
            CbmlTypeKind::String => format!("string"),
            CbmlTypeKind::Number => format!("number"),
            CbmlTypeKind::Bool => format!("bool"),
            CbmlTypeKind::Any => format!("any"),
            CbmlTypeKind::Array { inner_type } => {
                format!("[{}]", inner_type.to_cbml_code(deepth))
            }
            CbmlTypeKind::Union { allowed_values } => {
                let mut re = String::new();

                for x in allowed_values {
                    re.push_str(&format!("{} | ", x.to_cbml_code(deepth)));
                }

                return re;
            }
            CbmlTypeKind::Optional { inner_type } => {
                format!("?{}", inner_type.to_cbml_code(deepth))
            }
            CbmlTypeKind::Struct { fields } => {
                let mut re = String::new();
                re.push_str("{\n");

                for x in fields {
                    re.push_str(&format!("{}: {}\n", x.0, x.1.to_cbml_code(deepth)));
                }

                re.push_str("}\n");

                return re;
            }
            CbmlTypeKind::Enum { fields } => {
                let mut re = String::new();
                for x in fields {
                    re.push_str(&format!("{}( {} )\n ", x.0, x.1.to_cbml_code(deepth)));
                }
                return re;
            } // CbmlTypeKind::Custom { name } => name.to_string(),
        }
    }
}

impl ToCbmlCode for FieldDef {
    fn to_cbml_code(&self, deepth: usize) -> String {
        let mut re = String::new();

        re.push_str(&"    ".repeat(deepth));
        re.push_str(&self.name);
        re.push_str(": ");
        re.push_str(&self.type_.ty.to_cbml_code(0));

        if let Some(default) = &self.default_value {
            re.push_str(" default ");
            re.push_str(&default.kind.to_cbml_code(deepth));
        }

        return re;
    }
}

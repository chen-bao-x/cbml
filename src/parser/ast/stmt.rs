use std::{convert::identity, io::repeat, process::Command};

use crate::{ToCbmlCode, indent, lexer::token::Span};

/// StructFieldDef | TypeAlias | StructDef | EnumDef | UnionDef | LineComment | BlockComment | DocComment
// struct TypedefFile {
//     val: Vec<>,
// }

#[derive(Debug, Clone)]
pub enum StmtKind {
    Use(UseStmt), // use "path/to/file"

    /// name = "hello"  identifier asignment literal
    Asignment(AsignmentStmt),

    /// 在文件中定义一个属性.
    FileFieldStmt(StructFieldDefStmt), // name : type; 文件的 field,

    TypeAliasStmt(TypeAliasStmt), // type name = type

    StructDefStmt(StructDef),
    EnumDef(EnumDef), // enum Haha { ssh(string), git( {url: string, branch: string} ) }
    UnionDef(UnionDef), // 具名 union

    LineComment(String),
    BlockComment(String),
    DocComment(String),
}

impl StmtKind {
    pub fn get_span(&self) -> Span {
        match self.clone() {
            StmtKind::Use(use_stmt) => use_stmt.keyword_span,
            StmtKind::Asignment(asignment_stmt) => asignment_stmt.field_name_span,
            StmtKind::FileFieldStmt(struct_field_def_stmt) => struct_field_def_stmt.field_name_span,
            StmtKind::TypeAliasStmt(a) => a.name_span,
            StmtKind::StructDefStmt(struct_def) => struct_def.name_span,
            StmtKind::EnumDef(enum_def) => enum_def.name_span,
            StmtKind::UnionDef(union_def) => union_def.name_span,
            StmtKind::LineComment(_) => todo!(),
            StmtKind::BlockComment(_) => todo!(),
            StmtKind::DocComment(_) => todo!(),
        }
    }
}

impl ToCbmlCode for Vec<StmtKind> {
    fn to_cbml_code(&self, deepth: usize) -> String {
        let mut re = String::new();

        for x in self {
            // top level field 间隔一行更好看.
            if deepth == 0 {
                re.push_str("\n");
            }
            re.push_str(&x.to_cbml_code(deepth));
            re.push('\n');
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
            StmtKind::UnionDef(union_def) => union_def.to_cbml_code(deepth),
            StmtKind::LineComment(s) => format!("// {}", s),
            StmtKind::BlockComment(s) => format!("/* {} */", s),
            StmtKind::DocComment(s) => format!("/// {}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UseStmt {
    pub url: String,
    pub keyword_span: Span,
    pub url_span: Span,
}

impl ToCbmlCode for UseStmt {
    fn to_cbml_code(&self, deepth: usize) -> String {
        if self.url.starts_with("\"") && self.url.ends_with("\"") {
            return format!("use {}", self.url);
        } else {
            format!("use \"{}\"", self.url)
        }
    }
}

#[derive(Debug, Clone)]

pub struct TypeAliasStmt {
    pub name: String,
    pub ty: TypeSignStmt,
    pub doc: Option<DocumentStmt>,

    pub name_span: Span,
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

/// 赋值语句,
/// name = "hello"
#[derive(Debug, Clone, PartialEq)]
pub struct AsignmentStmt {
    pub field_name: String,
    // pub value: LiteralKind,
    pub value: Literal,
    pub field_name_span: Span,
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

/// 属性类型申明
/// name: string
/// name: string default "hello"
#[derive(Debug, Clone, PartialEq)]
pub struct StructFieldDefStmt {
    pub field_name: String,
    // pub _type: TypeSignStmtKind,
    pub _type: TypeSignStmt,
    // pub default: Option<LiteralKind>,
    pub default: Option<Literal>,

    pub doc: Option<DocumentStmt>,

    pub field_name_span: Span,
}

impl StructFieldDefStmt {
    pub fn end_span(&self) -> Span {
        if let Some(v) = &self.default {
            return v.span.clone();
        } else {
            return self._type.span.clone();
        }
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

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentStmt {
    pub document: String,
    pub span: Span,
}

/// 枚举属性申明
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EnumFieldDef {
    pub field_name: String,
    pub _type: TypeSignStmtKind,

    pub field_name_span: Span,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    pub kind: LiteralKind,
    pub span: Span,
}

/// 字面量
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralKind {
    String(String),
    // String {
    //     val: String,
    //     span: Span,
    // },
    Number(f64),
    Boolean(bool),
    Array(Vec<LiteralKind>),    // [1,2,2]
    Struct(Vec<AsignmentStmt>), // 结构体字面量暂时先不做.

    /// enum field literal
    EnumFieldLiteral {
        field_name: String,
        literal: Box<LiteralKind>,
        span: Span,
    },

    // Optional,
    LiteralNone, // none
    Todo,
    Default,
}

/// 为 匿名 union 推导类型.
impl LiteralKind {
    fn is_same_kind(&self, other: &LiteralKind) -> bool {
        use LiteralKind::*;

        match (self, other) {
            (String { .. }, String { .. }) => true,
            (Number(_), Number(_)) => true,
            (Boolean(_), Boolean(_)) => true,
            (Array(_), Array(_)) => true,
            (Struct(_), Struct(_)) => true,
            (LiteralNone, LiteralNone) => true,
            (Todo, Todo) => true,
            (Default, Default) => true,
            // (Union(_), Union(_)) => true,
            _ => false,
        }
    }

    pub fn union_base_type(arr: &[LiteralKind]) -> TypeSignStmtKind {
        let re = LiteralKind::union_base_type_2(arr);
        return match re {
            TypeInference::Inferenced(cbml_type) => cbml_type,
            // TypeInference::UnInference => CbmlType::Any,
            TypeInference::InferenceUnkonw => TypeSignStmtKind::Any,
        };
    }

    fn union_base_type_2(arr: &[LiteralKind]) -> TypeInference {
        match arr.len() {
            0 => {
                return TypeInference::InferenceUnkonw;
            }
            1 => {
                return LiteralKind::from_vec_literal(arr);
            }
            _ => {
                if Self::all_same_kind(arr) {
                    return LiteralKind::from_vec_literal(arr);
                } else {
                    return TypeInference::Inferenced(TypeSignStmtKind::Any);
                }
            }
        }
    }
    fn all_same_kind(arr: &[LiteralKind]) -> bool {
        match arr.len() {
            0 => {
                panic!();
            }
            1 => {
                return true;
            }
            _ => {
                let first = arr[0].clone();
                for i in 1..arr.len() {
                    if !first.is_same_kind(&arr[i]) {
                        return false;
                    }
                }
                return true;
            }
        }
    }

    pub fn from_vec_literal(arr: &[LiteralKind]) -> TypeInference {
        let base: &LiteralKind = Self::skip_none(arr).unwrap_or(&LiteralKind::LiteralNone);

        return match base {
            LiteralKind::String { .. } => TypeInference::Inferenced(TypeSignStmtKind::String),
            LiteralKind::Number(_) => TypeInference::Inferenced(TypeSignStmtKind::Number),
            LiteralKind::Boolean(_) => TypeInference::Inferenced(TypeSignStmtKind::Boolean),
            LiteralKind::Array(literals) => {
                let inter_type = LiteralKind::union_base_type(&literals);

                return TypeInference::Inferenced(TypeSignStmtKind::Array {
                    inner_type: Box::new(inter_type),
                });
            }
            LiteralKind::Struct(fields) => {
                let asdf: Vec<StructFieldDefStmt> = fields
                    .iter()
                    .map(|x| {
                        let re = LiteralKind::from_vec_literal(&[x.value.clone().kind]);
                        let ty: TypeSignStmtKind = match re {
                            TypeInference::Inferenced(cbml_type) => cbml_type,
                            // TypeInference::UnInference => CbmlType::Any,
                            TypeInference::InferenceUnkonw => TypeSignStmtKind::Any,
                        };

                        let type_sign = TypeSignStmt {
                            kind: ty,
                            span: Span {
                                start: x.field_name_span.start.clone(),
                                end: x.value.span.end.clone(),
                            },
                        };

                        return StructFieldDefStmt {
                            field_name: x.field_name.clone(),
                            _type: type_sign,
                            default: None,
                            field_name_span: x.field_name_span.clone(),
                            doc: None,
                        };
                    })
                    .collect();

                return TypeInference::Inferenced(TypeSignStmtKind::Struct(asdf));
            }
            LiteralKind::LiteralNone => TypeInference::InferenceUnkonw,
            LiteralKind::Todo => todo!(),
            LiteralKind::Default => todo!(),
            // Literal::Union(literals) => {
            //     return Literal::union_base_type_2(literals);
            // }
            LiteralKind::EnumFieldLiteral {
                field_name: _field_name,
                literal: _lit,
                span: _,
            } => {
                // let re = Literal::from_vec_literal(&[*literal.clone()]);

                // let ty: CbmlType = match re {
                //     TypeInference::Inferenced(cbml_type) => cbml_type,
                //     TypeInference::UnInference => CbmlType::Any,
                //     TypeInference::InferenceUnkonw => CbmlType::Any,
                // };

                // return TypeInference::Inferenced(CbmlType::Enum {
                //     field_name: field_name.clone(),
                //     field_type: ty.into(),
                // });

                todo!();
            }
        };
    }

    fn skip_none(arr: &[LiteralKind]) -> Option<&LiteralKind> {
        let len = arr.len();
        let mut count = 0;

        while count < len {
            count += 1;

            if let Some(l) = arr.get(count) {
                match l {
                    LiteralKind::LiteralNone | LiteralKind::Todo | LiteralKind::Default => {
                        continue;
                    }
                    _ => return Some(l),
                }
            } else {
                break;
            }
        }

        return None;
    }
}

impl ToCbmlCode for LiteralKind {
    fn to_cbml_code(&self, deepth: usize) -> String {
        match self {
            LiteralKind::String(s) => {
                let mut re = String::new();
                re.push_str(&format!("\"{}\"", s));
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
                for l in literals {
                    re.push_str(&format!("{}, ", l.to_cbml_code(deepth)));
                }
                re.push_str("]");
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

                    if newline_style.len() < 100 {
                        let mut count = 0;
                        let comma_style: String = asignment_stmts
                            .iter()
                            .map(|x| x.to_cbml_code(0)) // 每一个 stmt 转换为代码.
                            .fold(" ".to_string(), |mut a, b| {
                                a.push_str(&b);

                                // 避免添加最后一个逗号.
                                if count < asignment_stmts.len() - 1 {
                                    a.push_str(", "); // 添加分隔符
                                }
                                count += 1;
                                // a.push(','); // 添加分隔符
                                a
                            });

                        re.push_str(&comma_style);
                        re.push_str(" }");
                    } else {
                        re.push_str(&newline_style);
                        re.push_str(&"    ".repeat(deepth));
                        re.push_str("}");
                    }
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

#[derive(Debug, Clone, PartialEq)]
pub struct TypeSignStmt {
    pub kind: TypeSignStmtKind,
    pub span: Span,
}

impl ToCbmlCode for TypeSignStmt {
    fn to_cbml_code(&self, deepth: usize) -> String {
        let mut re = String::new();
        re.push_str(&self.kind.to_cbml_code(deepth));
        re.push_str("\n");

        return re;
    }
}

// TypeSignStmtKind
/// 自带的几个基础类型
/// struct enum union 支持 匿名类型.
// #[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TypeSignStmtKind {
    String,  // string
    Number,  // number
    Boolean, // bool
    Any,     // any

    /// 匿名数组类型
    /// [Type]
    Array {
        inner_type: Box<TypeSignStmtKind>,
    },

    /// 匿名结构体
    Struct(Vec<StructFieldDefStmt>),

    /// 匿名 union
    Union {
        base_type: Box<TypeSignStmtKind>,
        // alowd_values: Vec<LiteralKind>, // 1 | 2 | 3
        alowd_values: Vec<Literal>, // 1 | 2 | 3
    }, // 匿名联合类型

    Optional {
        inner_type: Box<TypeSignStmtKind>,
        // span: Span,
    }, // ?string /number ?bool ?[string] ?[number] ?[bool] ?{name: string}

    /// 匿名 enum
    Enum {
        enum_name: String,
        // field_type: Box<CbmlType>,
        fields: Vec<EnumFieldDef>,
    },

    /// 用户自定义的且设置了名字的类型.
    Custom(String), // 自定义类型 struct name, union(string) name, type name,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CbmlType {
    String,
    Number,
    Bool,
    Any,

    Array { inner_type: Box<CbmlType> },
    Struct { fields: Vec<(String, CbmlType)> },
    Union { allowed_values: Vec<LiteralKind> },
    Optional { inner_type: Box<CbmlType> },
    Enum { fields: Vec<(String, CbmlType)> },
}

impl ToCbmlCode for TypeSignStmtKind {
    fn to_cbml_code(&self, deepth: usize) -> String {
        match self {
            TypeSignStmtKind::String => format!("string"),
            TypeSignStmtKind::Number => format!("number"),
            TypeSignStmtKind::Boolean => format!("bool"),
            TypeSignStmtKind::Any => format!("any"),
            TypeSignStmtKind::Array { inner_type, .. } => {
                format!("[{}]", inner_type.to_cbml_code(deepth + 1))
            }
            TypeSignStmtKind::Struct(struct_field_def_stmts) => {
                let mut re = String::new();
                re.push_str("{\n");

                re.push_str(&struct_field_def_stmts.to_cbml_code(deepth + 1));

                re.push_str(&"    ".repeat(deepth));

                re.push_str("}");
                return re;
            }
            TypeSignStmtKind::Union {
                base_type: _base_type,
                alowd_values,
            } => {
                let mut str = String::new();
                let mut counter = 0;

                alowd_values.iter().for_each(|x| {
                    counter += 1;
                    if counter < alowd_values.len() {
                        str.push_str(&format!("{} | ", x.kind.to_cbml_code(deepth)));
                    } else {
                        str.push_str(&format!("{} ", x.kind.to_cbml_code(deepth)));
                    }
                });

                return str;
            }
            TypeSignStmtKind::Optional { inner_type } => {
                format!("?{}", inner_type.to_cbml_code(deepth))
            }
            TypeSignStmtKind::Enum {
                enum_name: field_name,
                fields,
            } => {
                let mut str = String::new();
                str.push_str(&format!("enum {} {{", field_name));
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
            TypeSignStmtKind::Custom(name) => name.clone(),
        }
    }
}
impl TypeSignStmtKind {
    pub fn is_custom_ty(&self) -> bool {
        match self {
            TypeSignStmtKind::Custom(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypeInference {
    Inferenced(TypeSignStmtKind), // 推导出来的类型.
    // UnInference,          // 还没推导
    InferenceUnkonw, // 推导了, 没推导出来.
}

/// 具名 struct
#[derive(Debug, Clone)]
pub struct StructDef {
    pub struct_name: String,

    // fields: HashMap<String, CbmlType>, // 字段名不能重复, 所以用 HashMap.
    pub fields: Vec<StructFieldDefStmt>, // 字段名不能重复, 所以用 HashMap., // 字段名不能重复, 所以用 HashMap.

    pub name_span: Span,

    pub doc: Option<DocumentStmt>,
}
impl ToCbmlCode for StructDef {
    fn to_cbml_code(&self, deepth: usize) -> String {
        let _ = deepth;

        let mut re = String::new();

        re.push_str(&format!("struct {} {{\n", self.struct_name));

        re.push_str(&self.fields.to_cbml_code(deepth + 1));

        re.push_str("}");

        return re;
    }
}

/// 具名 enum
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub enum_name: String,

    pub fields: Vec<EnumFieldDef>,

    pub doc: Option<DocumentStmt>,

    pub name_span: Span,
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

/// 具名 union
#[derive(Debug, Clone)]
pub struct UnionDef {
    pub union_name: String,
    pub base_type: TypeSignStmtKind,
    // pub allowed_values: Vec<LiteralKind>, // 1 | 2 | 3
    pub allowed_values: Vec<Literal>, // 1 | 2 | 3
    pub doc: Option<DocumentStmt>,
    pub name_span: Span,
}

impl UnionDef {
    #[allow(dead_code)]
    pub fn duplicate_check(&self) -> Vec<LiteralKind> {
        let mut re: Vec<&LiteralKind> = Vec::new();
        let mut duplicated: Vec<LiteralKind> = Vec::new();

        for v in &self.allowed_values {
            if re.contains(&&v.kind) {
                duplicated.push(v.kind.clone());
            } else {
                re.push(&v.kind)
            }
        }
        return duplicated;
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

// code_file
//      file_path
//      source code
//      tokens
//      statments

// project leverl symbol_table

// typedef_file
//      self_file

// cbml_file
//      self_file
//      Option<typedef_file>

//      symbol_table

// #[test]
// fn adsfdsaf() {
//     let s = "let a = 'abcd'";
//     let v = &s[8..=13];
//     println!("{:?}", v);

//     let kind = TokenKind::String("".into());
//     let span = Span {
//         start: Position {
//             line: 0,
//             column: 0,
//             character_index: 8,
//         },
//         end: Position {
//             line: 0,
//             column: 0,
//             character_index: 13,
//         },
//     };
//     let tok = Token::new(kind, span);

//     // tok.span.lookup(s);
//     println!("{:?}", tok.span.lookup(s));
// }

// struct CodeFile<'a> {
//     file_path: &'a String,
//     source_code: String,
//     tokens_: Vec<Box<HToken>>,
// }

// impl<'a> CodeFile<'a> {
//     fn dsaf(&mut self) {

//         // let a = self.source_code[0..=9];
//     }
// }

// struct HToken {
//     kind: TokenKind,
//     span: Span,
// }

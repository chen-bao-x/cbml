use crate::lexer::token::{Span, Token};

struct File {
    val: Vec<AsignmentStmt>,
}

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
            StmtKind::FileFieldStmt(struct_field_def_stmt) => todo!(),
            StmtKind::TypeAliasStmt(_) => todo!(),
            StmtKind::StructDefStmt(struct_def) => todo!(),
            StmtKind::EnumDef(enum_def) => todo!(),
            StmtKind::UnionDef(union_def) => todo!(),
            StmtKind::LineComment(_) => todo!(),
            StmtKind::BlockComment(_) => todo!(),
            StmtKind::DocComment(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UseStmt {
    pub url: String,
    pub keyword_span: Span,
    pub url_span: Span,
}

#[derive(Debug, Clone)]

pub struct TypeAliasStmt {
    pub name: String,
    pub ty: CbmlType,
    pub doc: Option<DocumentStmt>,

    pub name_span: Span,
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

/// 属性类型申明
/// name: string
/// name: string default "hello"
#[derive(Debug, Clone, PartialEq)]
pub struct StructFieldDefStmt {
    pub field_name: String,
    pub _type: CbmlType,
    pub default: Option<LiteralKind>,

    pub doc: Option<DocumentStmt>,

    pub field_name_span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentStmt {
    pub document: String,
    pub span: Span,
}

/// 枚举属性申明
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EnumField {
    pub field_name: String,
    pub _type: CbmlType,

    pub field_name_span: Span,
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

    //  union 字面量? 娜 union(string) 的字面量是 string
    // Union(Vec<Literal>),

    // Optional,
    // Any,
    /// enum field literal
    EnumFieldLiteral {
        field_name: String,
        literal: Box<LiteralKind>,
    },
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

    pub fn union_base_type(arr: &[LiteralKind]) -> CbmlType {
        let re = LiteralKind::union_base_type_2(arr);
        return match re {
            TypeInference::Inferenced(cbml_type) => cbml_type,
            // TypeInference::UnInference => CbmlType::Any,
            TypeInference::InferenceUnkonw => CbmlType::Any,
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
                    return TypeInference::Inferenced(CbmlType::Any);
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
            LiteralKind::String { .. } => TypeInference::Inferenced(CbmlType::String),
            LiteralKind::Number(_) => TypeInference::Inferenced(CbmlType::Number),
            LiteralKind::Boolean(_) => TypeInference::Inferenced(CbmlType::Boolean),
            LiteralKind::Array(literals) => {
                let inter_type = LiteralKind::union_base_type(&literals);

                return TypeInference::Inferenced(CbmlType::Array {
                    inner_type: Box::new(inter_type),
                });
            }
            LiteralKind::Struct(fields) => {
                let asdf: Vec<StructFieldDefStmt> = fields
                    .iter()
                    .map(|x| {
                        let re = LiteralKind::from_vec_literal(&[x.value.clone().kind]);
                        let ty: CbmlType = match re {
                            TypeInference::Inferenced(cbml_type) => cbml_type,
                            // TypeInference::UnInference => CbmlType::Any,
                            TypeInference::InferenceUnkonw => CbmlType::Any,
                        };

                        return StructFieldDefStmt {
                            field_name: x.field_name.clone(),
                            _type: ty,
                            default: None,
                            field_name_span: x.field_name_span.clone(),
                            doc: None,
                        };
                    })
                    .collect();

                return TypeInference::Inferenced(CbmlType::Struct(asdf));
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

impl LiteralKind {
    pub fn to_cbml_code(&self) -> String {
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
                    re.push_str(&format!("{}, ", l.to_cbml_code()));
                }
                re.push_str("]");
                return re;
            }
            LiteralKind::Struct(asignment_stmts) => {
                let mut re = String::new();
                re.push_str("{");
                for a in asignment_stmts {
                    re.push_str(&format!(
                        "{}: {}, ",
                        a.field_name,
                        a.value.kind.to_cbml_code()
                    ));
                }
                re.push_str("}");
                return re;
            }

            LiteralKind::EnumFieldLiteral {
                field_name: _field_name,
                literal: _literal,
            } => {
                let mut re = String::new();
                re.push_str(_field_name);
                re.push('(');

                re.push_str(&_literal.to_cbml_code());

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

/// 自带的几个基础类型
/// struct enum union 支持 匿名类型.
// #[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum CbmlType {
    String,  // string
    Number,  // number
    Boolean, // bool
    Any,     // any

    /// 匿名数组类型
    /// [Type]
    Array {
        inner_type: Box<CbmlType>,
    },

    /// 匿名结构体
    Struct(Vec<StructFieldDefStmt>),

    /// 匿名 union
    Union {
        base_type: Box<CbmlType>,
        // alowd_values: Vec<LiteralKind>, // 1 | 2 | 3
        alowd_values: Vec<Literal>, // 1 | 2 | 3
    }, // 匿名联合类型

    Optional {
        inner_type: Box<CbmlType>,
    }, // ?string /number ?bool ?[string] ?[number] ?[bool] ?{name: string}

    /// 匿名 enum
    Enum {
        enum_name: String,
        // field_type: Box<CbmlType>,
        fields: Vec<EnumField>,
    },

    /// 用户自定义的且设置了名字的类型.
    Custom(String), // 自定义类型 struct name, union(string) name, type name,
}

impl CbmlType {
    pub fn to_cbml_code(&self) -> String {
        match self {
            CbmlType::String => format!("string"),
            CbmlType::Number => format!("number"),
            CbmlType::Boolean => format!("bool"),
            CbmlType::Any => format!("any"),
            CbmlType::Array { inner_type } => format!("[{}]", inner_type.to_cbml_code()),
            CbmlType::Struct(struct_field_def_stmts) => {
                let mut str = String::new();
                str.push_str("{");
                for s in struct_field_def_stmts {
                    str.push_str(&format!("{}: {}, ", s.field_name, s._type.to_cbml_code()));
                }
                str.push_str("}");
                return str;
            }
            CbmlType::Union {
                base_type: _base_type,
                alowd_values,
            } => {
                let mut str = String::new();
                let mut counter = 0;

                alowd_values.iter().for_each(|x| {
                    counter += 1;
                    if counter < alowd_values.len() {
                        str.push_str(&format!("{} | ", x.kind.to_cbml_code()));
                    } else {
                        str.push_str(&format!("{} ", x.kind.to_cbml_code()));
                    }
                });

                return str;
            }
            CbmlType::Optional { inner_type } => {
                format!("?{}", inner_type.to_cbml_code())
            }
            CbmlType::Enum {
                enum_name: field_name,
                fields,
            } => {
                let mut str = String::new();
                str.push_str(&format!("enum {} {{", field_name));
                for field in fields {
                    str.push_str(&format!(
                        "{}: {}, ",
                        field.field_name,
                        field._type.to_cbml_code()
                    ));
                }
                str.push_str("}");
                return str;
            }
            CbmlType::Custom(name) => name.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypeInference {
    Inferenced(CbmlType), // 推导出来的类型.
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

/// 具名 enum
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub enum_name: String,

    pub fields: Vec<EnumField>,

    pub doc: Option<DocumentStmt>,

    pub name_span: Span,
}

/// 具名 union
#[derive(Debug, Clone)]
pub struct UnionDef {
    pub union_name: String,
    pub base_type: CbmlType,
    // pub allowed_values: Vec<LiteralKind>, // 1 | 2 | 3
    pub allowed_values: Vec<Literal>, // 1 | 2 | 3
    pub doc: Option<DocumentStmt>,
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

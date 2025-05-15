use std::collections::HashMap;

use crate::formater::ToCbmlCode;

pub trait ToCbmlValue {
    fn to_cbml_value(&self) -> CbmlValue;
}

#[derive(Debug, Clone, PartialEq)]
pub enum CbmlValue {
    String(String),
    Number(f64),
    Boolean(bool),

    /// LiteralNone
    None,

    /// [1,2,3]
    Array(Vec<CbmlValue>),

    /// { name = "hello", age = 99 }
    Struct(HashMap<String, CbmlValue>),

    /// ssh({git = "", branch = "main"})
    EnumField(String, Box<CbmlValue>),
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

#[derive(Debug, Clone)]
pub struct CbmlType {
    /// String, Number, Bool, Any, 这几种内置类型是不需要定义名字的.
    pub kind: CbmlTypeKind,
}

impl PartialEq for CbmlType {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}
impl ToCbmlCode for CbmlType {
    fn to_cbml_code(&self, deepth: usize) -> String {
        self.kind.to_cbml_code(deepth)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CbmlTypeKind {
    String,
    Number,
    Bool,
    Any,

    Array {
        inner_type: Box<CbmlType>,
    },
    Union {
        allowed_values: Vec<CbmlValue>,
    },
    Optional {
        inner_type: Box<CbmlType>,
    },

    /// 匿名结构体 Vec<(Name, Type)>
    Struct {
        fields: Vec<(String, CbmlType)>,
    },

    /// 匿名枚举 Vec<(Name, Type)>
    Enum {
        fields: Vec<(String, CbmlType)>,
    },
}

// impl PartialEq for CbmlTypeKind {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (
//                 Self::Array {
//                     inner_type: l_inner_type,
//                 },
//                 Self::Array {
//                     inner_type: r_inner_type,
//                 },
//             ) => l_inner_type == r_inner_type,
//             (Self::Struct { fields: l_fields }, Self::Struct { fields: r_fields }) => {
//                 l_fields == r_fields
//             }
//             (
//                 Self::Union {
//                     allowed_values: l_allowed_values,
//                 },
//                 Self::Union {
//                     allowed_values: r_allowed_values,
//                 },
//             ) => l_allowed_values == r_allowed_values,
//             (
//                 Self::Optional {
//                     inner_type: l_inner_type,
//                 },
//                 Self::Optional {
//                     inner_type: r_inner_type,
//                 },
//             ) => l_inner_type == r_inner_type,
//             (Self::Enum { fields: l_fields }, Self::Enum { fields: r_fields }) => {
//                 l_fields == r_fields
//             }
//             (Self::Any, _) => true,
//             _ => core::mem::discriminant(self) == core::mem::discriminant(other),
//         }
//     }
// }

impl ToCbmlCode for CbmlTypeKind {
    fn to_cbml_code(&self, deepth: usize) -> String {
        match self {
            CbmlTypeKind::String => format!("string"),
            CbmlTypeKind::Number => format!("number"),
            CbmlTypeKind::Bool => format!("bool"),
            CbmlTypeKind::Any => format!("any"),
            CbmlTypeKind::Array { inner_type } => inner_type.to_cbml_code(deepth),
            CbmlTypeKind::Union { allowed_values } => {
                let mut re = String::new();

                for x in allowed_values {
                    re.push_str(&format!("{} | ", x.to_cbml_code(deepth)));
                }

                return re;
            }
            CbmlTypeKind::Optional { inner_type } => inner_type.to_cbml_code(deepth),
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

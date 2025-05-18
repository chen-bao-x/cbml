use std::collections::HashMap;

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

    /// ssh("value")
    EnumField(String, Box<CbmlValue>),
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

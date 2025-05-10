use std::collections::HashMap;

use crate::ToCbmlCode;

#[allow(dead_code)]
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
            CbmlValue::String(s) => format!("\"{}\"", s),
            CbmlValue::Number(n) => format!("{}", n),
            CbmlValue::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
            CbmlValue::Array(cbml_values) => {
                let mut re = String::new();
                re.push_str("[");

                let mut count = 0;
                for v in cbml_values {
                    if count >= cbml_values.len() - 1 {
                        // 最后一个逗号不用打印

                        re.push_str(&format!("{}", v.to_cbml_code(0)));
                    } else {
                        re.push_str(&format!("{}, ", v.to_cbml_code(0)));
                    }
                    count += 1;
                }
                re.push_str("]");
                return re;
            }
            CbmlValue::Struct(hash_map) => {
                let mut re = String::new();
                re.push_str("{");

                let mut count = 0;
                for (k, v) in hash_map {
                    if count >= hash_map.len() - 1 {
                        re.push_str(&format!("{}: {} ", k, v.to_cbml_code(0)));
                    } else {
                        re.push_str(&format!("{}: {}, ", k, v.to_cbml_code(0)));
                    }

                    count += 1;
                }
                re.push_str("}");
                return re;
            }
            CbmlValue::EnumField(name, cbml_value) => {
                let mut re = String::new();
                re.push_str(&format!("{}(", name));
                re.push_str(&cbml_value.to_cbml_code(0));
                re.push_str(")");
                return re;
            }
        }
    }
}

impl CbmlValue {
    pub fn kind_is(&self, kind: CbmlValueKind) -> bool {
        match self {
            CbmlValue::String(_) => kind == CbmlValueKind::String,
            CbmlValue::Number(_) => kind == CbmlValueKind::Number,
            CbmlValue::Boolean(_) => kind == CbmlValueKind::Boolean,
            CbmlValue::None => kind == CbmlValueKind::None,
            CbmlValue::Array(_) => kind == CbmlValueKind::Array,
            CbmlValue::Struct(_) => kind == CbmlValueKind::Struct,
            CbmlValue::EnumField(_, _) => kind == CbmlValueKind::EnumField,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum CbmlValueKind {
    String,
    Number,
    Boolean,
    None,
    Array,
    Struct,
    EnumField,
}

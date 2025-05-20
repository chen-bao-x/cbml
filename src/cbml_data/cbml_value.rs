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

impl CbmlValue {
    pub fn to_json_string() {}
    pub fn to_json() {}
}

use std::collections::HashMap;

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

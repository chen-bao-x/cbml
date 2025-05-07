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

impl CbmlValue {
    fn to_cbml_code(&self) -> String {
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

                        re.push_str(&format!("{}", v.to_cbml_code()));
                    } else {
                        re.push_str(&format!("{}, ", v.to_cbml_code()));
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
                        re.push_str(&format!("{}: {} ", k, v.to_cbml_code()));
                    } else {
                        re.push_str(&format!("{}: {}, ", k, v.to_cbml_code()));
                    }

                    count += 1;
                }
                re.push_str("}");
                return re;
            }
            CbmlValue::EnumField(name, cbml_value) => {
                let mut re = String::new();
                re.push_str(&format!("{}(", name));
                re.push_str(&cbml_value.to_cbml_code());
                re.push_str(")");
                return re;
            }
        }
    }

    fn kind_is(&self, kind: CbmlValueKind) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_cbml_code_none() {
        let value = CbmlValue::None;
        println!("{}", value.to_cbml_code());
        assert_eq!(value.to_cbml_code(), "none");
    }

    #[test]
    fn test_to_cbml_code_string() {
        let value = CbmlValue::String("hello".to_string());
        println!("{}", value.to_cbml_code());
        assert_eq!(value.to_cbml_code(), "\"hello\"");
    }

    #[test]
    fn test_to_cbml_code_number() {
        let value = CbmlValue::Number(42.0);
        println!("{}", value.to_cbml_code());
        assert_eq!(value.to_cbml_code(), "42");
    }

    #[test]
    fn test_to_cbml_code_boolean_true() {
        let value = CbmlValue::Boolean(true);
        println!("{}", value.to_cbml_code());
        assert_eq!(value.to_cbml_code(), "true");
    }

    #[test]
    fn test_to_cbml_code_boolean_false() {
        let value = CbmlValue::Boolean(false);
        println!("{}", value.to_cbml_code());
        assert_eq!(value.to_cbml_code(), "false");
    }

    #[test]
    fn test_to_cbml_code_array() {
        let value = CbmlValue::Array(vec![
            CbmlValue::Number(1.0),
            CbmlValue::Number(2.0),
            CbmlValue::Number(3.0),
        ]);
        println!("{}", value.to_cbml_code());
        assert_eq!(value.to_cbml_code(), "[1, 2, 3]");
    }

    #[test]
    fn test_to_cbml_code_struct() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), CbmlValue::String("Alice".to_string()));
        map.insert("age".to_string(), CbmlValue::Number(30.0));
        let value = CbmlValue::Struct(map);
        let result = value.to_cbml_code();
        println!("{}", value.to_cbml_code());
        assert!(result == "{name: \"Alice\", age: 30 }" || result == "{age: 30, name: \"Alice\" }");
    }

    #[test]
    fn test_to_cbml_code_enum_field() {
        let value = CbmlValue::EnumField(
            "Option".to_string(),
            Box::new(CbmlValue::String("SomeValue".to_string())),
        );
        println!("{}", value.to_cbml_code());
        assert_eq!(value.to_cbml_code(), "Option(\"SomeValue\")");
    }
}

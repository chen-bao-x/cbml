use crate::{AndThenTo, ToCbml};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum CbmlValue {
    String(String),
    Number(f64),
    Boolean(bool),

    /// Literal `none`
    None,

    /// [1,2,3]
    Array(Vec<CbmlValue>),

    /// person("张三")
    EnumField(String, Box<CbmlValue>),

    /// { name = "hello", age = 99 }
    Struct(HashMap<String, CbmlValue>),
}

struct CbmlFile {
    
}

impl ToCbml for CbmlValue {
    fn to_cbml(&self, deepth: usize) -> String {
        match self {
            CbmlValue::None => format!("none"),
            CbmlValue::String(s) => format!("\"{}\"", s),
            CbmlValue::Number(n) => format!("{}", n),
            CbmlValue::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
            CbmlValue::Array(cbml_values) => {
                let mut re = String::new();
                re.push_str("[");

                for l in cbml_values {
                    re.push_str(&format!("{}, ", l.to_cbml(deepth)));
                }

                re.push_str("]");

                if re.contains("\n") {
                    re.clear();

                    re.push_str("[\n");

                    for l in cbml_values {
                        re.push_str(&format!(
                            "{}{},\n",
                            "    ".repeat(deepth + 1),
                            l.to_cbml(deepth + 1)
                        ));
                    }

                    re.push_str("]");
                }
                return re;
            }
            CbmlValue::Struct(hash_map) => {
                let mut re = String::new();
                re.push_str("{\n");

                for (name, val) in hash_map {
                    re.push_str(&format!(
                        "{}{} = {}\n",
                        "    ".repeat(deepth + 1),
                        name,
                        val.to_cbml(deepth + 1)
                    ));
                }
                re.push_str(&"    ".repeat(deepth));
                re.push_str("}");

                return re;
            }
            CbmlValue::EnumField(name, cbml_value) => {
                let mut re = String::new();

                re.push_str(&format!("{} (", name));
                re.push_str(&cbml_value.to_cbml(0));
                re.push_str(")\n");
                return re;
            }
        }
    }
}

impl CbmlValue {
    ///
    ///
    /// 空字符串 "" 会被视为 debug look up.
    ///
    pub fn key_path<const N: usize>(&self, key_chain: [&str; N]) -> Option<&CbmlValue> {
        let mut val: Option<&CbmlValue> = Some(self);

        for key in key_chain {
            // debug look up.
            {
                if key == "" {
                    match &val {
                        Some(v) => println!("{}", v.to_cbml(0)),
                        None => {
                            println!("look up: error unknown field.",)
                        }
                    }
                    continue;
                }
            }

            // val = val.cbml_struct().map(|x| &x[key]);
            val = val
                .cbml_struct()
                .map(|x| x.get(key).expect(&format!("key: {}", key)));
        }
        return val;
    }
}

impl<'a> AndThenTo<'a> for Option<&'a CbmlValue> {
    fn look_up(&self) -> &Self {
        println!("{:?}", self);
        self
    }
    /// aaaaaaa
    fn cbml_str(&self) -> Option<&'a str> {
        self.and_then(|x| x.cbml_str())
    }
    fn cbml_number(&self) -> Option<f64> {
        self.and_then(|x| x.cbml_number())
    }

    fn cbml_bool(&self) -> Option<bool> {
        self.and_then(|x| x.cbml_bool())
    }

    fn cbml_none(&self) -> Option<CbmlNoneValue> {
        self.and_then(|x| x.cbml_none())
    }

    fn cbml_array(&self) -> Option<&'a Vec<CbmlValue>> {
        self.and_then(|x| x.cbml_array())
    }

    fn cbml_struct(&self) -> Option<&'a HashMap<String, CbmlValue>> {
        self.and_then(|x| x.cbml_struct())
    }

    fn cbml_enum_field(&self) -> Option<(String, Box<CbmlValue>)> {
        self.and_then(|x| x.cbml_enum_field())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CbmlNoneValue();

impl CbmlValue {
    pub fn look_up(&self) -> &Self {
        println!("{:?}", self);
        self
    }

    pub fn cbml_number(&self) -> Option<f64> {
        let CbmlValue::Number(n) = self else {
            return None;
        };

        return Some(*n);
    }

    pub fn cbml_str(&self) -> Option<&str> {
        let CbmlValue::String(s) = self else {
            return None;
        };

        return Some(s);
    }
    pub fn cbml_bool(&self) -> Option<bool> {
        let CbmlValue::Boolean(b) = self else {
            return None;
        };

        return Some(*b);
    }

    pub fn cbml_none(&self) -> Option<CbmlNoneValue> {
        let CbmlValue::None = self else {
            return None;
        };

        return Some(CbmlNoneValue {});
    }

    pub fn cbml_array(&self) -> Option<&Vec<CbmlValue>> {
        let CbmlValue::Array(vec) = self else {
            return None;
        };
        return Some(vec);
    }

    pub fn cbml_struct(&self) -> Option<&HashMap<String, CbmlValue>> {
        let CbmlValue::Struct(hash_map) = self else {
            return None;
        };

        return Some(hash_map);
    }

    // pub fn cbml_enum_field(&self) -> Result<(String, Box<CbmlValue>), ErrMsg> {
    pub fn cbml_enum_field(&self) -> Option<(String, Box<CbmlValue>)> {
        let CbmlValue::EnumField(name, cbml_value) = self else {
            return None;
        };

        return Some((name.clone(), cbml_value.clone()));
    }
}

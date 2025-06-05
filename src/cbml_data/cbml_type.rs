use crate::ToCbml;

use super::cbml_value::CbmlValue;


#[derive(Debug, Clone, PartialEq)]
pub enum CbmlType {
    /// 这个类型是一个特殊的类型, 用来表示字符串.    
    /// "hello world"  
    /// 支持直接换行:  
    /// "hello  
    /// world"  
    ///
    /// 支持转义字符:  
    /// "hello\nworld"  
    ///
    /// 支持转义双引号:  
    /// "hello, \"name\""  
    ///
    /// 支持转义单引号:  
    /// "hello, \'name\'"  
    ///
    /// 支持转义反斜杠:  
    /// "hello, \\name\\"  
    ///
    /// 支持 unicode:  
    /// "hello, \u{4f60}\u{597d}"  
    String,

    /// 这个类型是一个特殊的类型, 用来表示数字.  
    /// 123, -123, 0.123, -0.123, 0xfff  0110101010  
    Number,

    /// 这个类型是一个特殊的类型, 用来表示布尔值.  
    /// true or false  
    Bool,

    /// 这个类型是一个特殊的类型, 用来表示不做类型约束类型.  
    Any,

    /// 这个类型是一个特殊的类型, 用来表示空值.  
    /// [number] -> [1,2,3]  
    /// [string] -> ["hello", "world"]  
    /// [bool] -> [true, false]  
    /// [any] -> [1, "hello", true, [1,2,3], {name = "hello"}]  
    ///
    Array { inner_type: Box<CbmlType> },

    /// 这个类型是一个特殊的类型, 用来表示联合类型.  
    /// 1 | 2 | 3 | "hello" | false | [1,2,3] | {name = "hello"}  
    /// union 的每个选项只能是一个 值(CbmlValue).  
    /// 每个选项不能相同:  
    /// 1 | 1 | 2  这样的事不允许的.  
    Union { allowed_values: Vec<CbmlValue> },

    /// 这个类型是一个特殊的类型, 用来表示可选类型.  
    /// ?string -> none  
    /// ?string -> "str"  
    /// ?number -> none  
    /// ?number -> 2  
    /// ?bool -> none  
    /// ?bool -> true  
    /// ?[string] -> none  
    /// ?[string] -> ["a", "b"]  
    /// ?[number] -> none  
    /// ?[number] -> [1, 2, 3]  
    Optional { inner_type: Box<CbmlType> },

    /// 这个类型是一个特殊的类型, 用来表示联合类型.  
    /// 定义某个字段为 结构体类型:  
    /// ```cbml
    /// person: {
    ///    name: string
    ///    age: number
    /// }
    /// 赋值:  
    /// ```cbml
    /// person = {
    ///    name = "hello"
    ///    age = 18
    /// }
    /// ```
    Struct { fields: Vec<(String, CbmlType)> },

    /// 这个类型是一个特殊的类型, 用来表示结构体类型.  
    /// 定义某个字段为 枚举类型:  
    /// ```cbml
    /// who: enum {
    ///     张三({name = "zhangsan", age = 18}),
    ///     李四({name = "lisi", age = 20}),
    ///     王五({name = "wangwu", age = 22}),
    /// }
    /// ```
    /// 赋值:  
    /// ```cbml
    /// who = 张三({name = "zhangsan", age = 18})
    /// ```
    Enum { fields: Vec<(String, CbmlType)> },
}

impl ToCbml for CbmlType {
    fn to_cbml(&self, deepth: usize) -> String {
        match self {
            CbmlType::String => format!("string"),
            CbmlType::Number => format!("number"),
            CbmlType::Bool => format!("bool"),
            CbmlType::Any => format!("any"),
            CbmlType::Array { inner_type } => {
                format!("[{}]", inner_type.to_cbml(deepth))
            }
            CbmlType::Union { allowed_values } => {
                let mut re = String::new();

                let mut count = 0;
                for x in allowed_values {
                    re.push_str(&format!("{} ", x.to_cbml(deepth)));

                    if count < allowed_values.len() - 1 {
                        re.push_str("| ");
                    }

                    count += 1;
                }

                return re;
            }
            CbmlType::Optional { inner_type } => {
                format!("?{}", inner_type.to_cbml(deepth))
            }
            CbmlType::Struct { fields } => {
                let mut re = String::new();
                re.push_str("{\n");

                for x in fields {
                    re.push_str(&format!(
                        "{}{}: {}\n",
                        "    ".repeat(deepth + 1),
                        x.0,
                        x.1.to_cbml(deepth + 1)
                    ));
                }
                re.push_str(&"    ".repeat(deepth));
                re.push_str("}");

                return re;
            }
            CbmlType::Enum { fields } => {
                let mut re = String::new();
                re.push_str("enum {\n");

                for x in fields {
                    let field_name = &x.0;
                    let field_type = &x.1;

                    re.push_str(&format!(
                        "{}{}({})\n",
                        "    ".repeat(deepth + 1),
                        field_name,
                        field_type.to_cbml(deepth + 1)
                    ));
                }
                re.push_str(&"    ".repeat(deepth));
                re.push_str("}");
                return re;
            }
        }
    }
}

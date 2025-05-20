use super::cbml_value::CbmlValue;

pub trait ToCbmlType {
    fn to_cbml_type(&self) -> CbmlType;
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
    /// 这个类型是一个特殊的类型, 用来表示字符串.
    /// "hello world"
    /// 支持直接换行:
    /// "hello
    /// world"
    /// 支持转义字符:
    /// "hello\nworld"
    /// 支持转义双引号:
    /// "hello, \"name\""
    /// 支持转义单引号:
    /// "hello, \'name\'"
    /// 支持转义反斜杠:
    /// "hello, \\name\\"
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
    /// person: {
    ///    name: string
    ///    age: number
    /// }
    /// 赋值:
    /// person = {
    ///    name = "hello"
    ///    age = 18
    /// }
    Struct { fields: Vec<(String, CbmlType)> },

    /// 这个类型是一个特殊的类型, 用来表示结构体类型.
    /// 定义某个字段为 枚举类型:
    /// who: enum {
    ///     张三({name = "zhangsan", age = 18}),
    ///     李四({name = "lisi", age = 20}),
    ///     王五({name = "wangwu", age = 22}),
    /// }
    /// 赋值:
    /// who = 张三({name = "zhangsan", age = 18})
    Enum { fields: Vec<(String, CbmlType)> },
}
// created by author at 20250520 11:28.
// 
// this is on line 1.
// this is on line 2.
// this is on line 3.

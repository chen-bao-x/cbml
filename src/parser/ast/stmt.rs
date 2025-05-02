use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Stmt {
    Use(String), // use "path/to/file"
    /// name = "hello"  identifier asignment literal
    Asignment {
        field_name: String,
        value: Literal,
    },
    FieldDef(FieldDefinition),   // name : type
    TypeAlias(String, CbmlType), // type name = type
    StructDef(StructTy),

    // {
    //     name: String,
    //     fields: Vec<FieldDefinition>, // 字段名不能重复, 所以用 HashMap.
    // },
    UnionDef {
        base_type: Box<CbmlType>,
        alowd_values: Vec<Literal>, // 1 | 2 | 3
    },
}

/// name: string
/// name: string default "hello"
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    pub field_name: String,
    pub ty: CbmlType,
    pub default: Option<Literal>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<Literal>), // [1,2,2]

    Struct(HashMap<String, Literal>), // 结构体字面量暂时先不做.
    None,                             // none
    Todo,
    Default,
}

/// 自带的几个基础类型
#[derive(Debug, Clone)]
pub enum CbmlType {
    String {
        default: Option<String>,
    }, // string
    Number {
        default: Option<f64>,
    }, // number
    Boolean {
        default: Option<bool>,
    }, // bool
    Array {
        inner_type: Box<CbmlType>,
        default: Option<Vec<Literal>>,
    }, // [Type], // 数组类型
    Struct(Vec<FieldDefinition>), // 结构体类型 HashMap<String, Literal>
    Union {
        ty: UnionTY,
        default: Option<Literal>,
    }, // 联合类型
    Any {
        default: Option<Literal>,
    }, // any
    Optional {
        ty: Box<CbmlType>,
        default: Option<Literal>,
    }, // ?string /number ?bool ?[string] ?[number] ?[bool] ?{name: string}
    Custom {
        type_name: String,
        default: Option<Literal>,
    }, // 自定义类型 struct name, union(string) name, type name,
}

#[derive(Debug, Clone)]
pub struct StructTy {
    pub name: String,

    // fields: HashMap<String, CbmlType>, // 字段名不能重复, 所以用 HashMap.
    pub fields: Vec<FieldDefinition>, // 字段名不能重复, 所以用 HashMap., // 字段名不能重复, 所以用 HashMap.
}

#[derive(Debug, Clone)]
struct UnionTY {
    name: String,
    base_type: Box<CbmlType>,
    alowd_values: Vec<Literal>, // 1 | 2 | 3
}

impl UnionTY {
    fn duplicate_check(&self) -> Vec<Literal> {
        let mut re: Vec<&Literal> = Vec::new();
        let mut duplicated: Vec<Literal> = Vec::new();
        for v in &self.alowd_values {
            if re.contains(&v) {
                duplicated.push(v.clone());
            } else {
                re.push(v)
            }
        }
        return duplicated;
    }
}

// #[derive(Debug, Clone)]
// struct Value {
//     value: Literal,
//     ty: CbmlType,
// }

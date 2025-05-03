#[derive(Debug, Clone)]
pub enum Stmt {
    Use(String), // use "path/to/file"
    /// name = "hello"  identifier asignment literal
    Asignment {
        field_name: String,
        value: Literal,
    },

    StructFieldDef(StructFieldDefinition), // name : type; 文件的 field,
    TypeAlias(String, CbmlType),           // type name = type
    StructDef(StructTy),
    EnumDef(EnumTy), // enum Haha { ssh(string), git( {url: string, branch: string} ) }
    UnionDef(UnionTy), // 具名 union

    LineComment(String),
    BlockComment(String),
    DocComment(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Asignment {
    pub field_name: String,
    pub value: Literal,
}

/// name: string
/// name: string default "hello"
#[derive(Debug, Clone)]
pub struct StructFieldDefinition {
    pub field_name: String,
    pub ty: CbmlType,
    pub default: Option<Literal>,
}

#[derive(Debug, Clone)]
pub struct EnumFieldDefinition {
    pub name: String,
    pub ty: CbmlType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<Literal>),    // [1,2,2]
    Struct(Vec<Asignment>), // 结构体字面量暂时先不做.
    Union(Vec<Literal>),
    Enum {
        field_name: String,
        literal: Box<Literal>,
    },
    None, // none
    Todo,
    Default,
}

/// 为 匿名 union 推导类型.
impl Literal {
    fn is_same_kind(&self, other: &Literal) -> bool {
        use Literal::*;

        match (self, other) {
            (String(_), String(_)) => true,
            (Number(_), Number(_)) => true,
            (Boolean(_), Boolean(_)) => true,
            (Array(_), Array(_)) => true,
            (Struct(_), Struct(_)) => true,
            (None, None) => true,
            (Todo, Todo) => true,
            (Default, Default) => true,
            (Union(_), Union(_)) => true,
            _ => false,
        }
    }

    pub fn union_base_type(arr: &[Literal]) -> CbmlType {
        let re = Literal::union_base_type_2(arr);
        return match re {
            TypeInference::Inferenced(cbml_type) => cbml_type,
            TypeInference::UnInference => CbmlType::Any,
            TypeInference::InferenceUnkonw => CbmlType::Any,
        };
    }

    fn union_base_type_2(arr: &[Literal]) -> TypeInference {
        match arr.len() {
            0 => {
                return TypeInference::InferenceUnkonw;
            }
            1 => {
                return Literal::from_vec_literal(arr);
            }
            _ => {
                if all_same_kind(arr) {
                    return Literal::from_vec_literal(arr);
                } else {
                    return TypeInference::Inferenced(CbmlType::Any);
                }
            }
        }

        fn all_same_kind(arr: &[Literal]) -> bool {
            match arr.len() {
                0 => {
                    panic!();
                }
                1 => {
                    return true;
                }
                _ => {
                    let first = arr[0].clone();
                    for i in 1..arr.len() {
                        if !first.is_same_kind(&arr[i]) {
                            return false;
                        }
                    }
                    return true;
                }
            }
        }
    }

    pub fn from_vec_literal(arr: &[Literal]) -> TypeInference {
        let base: &Literal = Self::skip_none(arr).unwrap_or(&Literal::None);

        return match base {
            Literal::String(_) => TypeInference::Inferenced(CbmlType::String),
            Literal::Number(_) => TypeInference::Inferenced(CbmlType::Number),
            Literal::Boolean(_) => TypeInference::Inferenced(CbmlType::Boolean),
            Literal::Array(literals) => {
                let inter_type = Literal::union_base_type(&literals);

                return TypeInference::Inferenced(CbmlType::Array {
                    inner_type: Box::new(inter_type),
                });
            }
            Literal::Struct(fields) => {
                let asdf: Vec<StructFieldDefinition> = fields
                    .iter()
                    .map(|x| {
                        let re = Literal::from_vec_literal(&[x.value.clone()]);
                        let ty: CbmlType = match re {
                            TypeInference::Inferenced(cbml_type) => cbml_type,
                            TypeInference::UnInference => CbmlType::Any,
                            TypeInference::InferenceUnkonw => CbmlType::Any,
                        };

                        return StructFieldDefinition {
                            field_name: x.field_name.clone(),
                            ty: ty,
                            default: None,
                        };
                    })
                    .collect();

                return TypeInference::Inferenced(CbmlType::Struct(asdf));
            }
            Literal::None => TypeInference::InferenceUnkonw,
            Literal::Todo => todo!(),
            Literal::Default => todo!(),
            Literal::Union(literals) => {
                return Literal::union_base_type_2(literals);
            }
            Literal::Enum {
                field_name,
                literal,
            } => {
                let re = Literal::from_vec_literal(&[*literal.clone()]);

                let ty: CbmlType = match re {
                    TypeInference::Inferenced(cbml_type) => cbml_type,
                    TypeInference::UnInference => CbmlType::Any,
                    TypeInference::InferenceUnkonw => CbmlType::Any,
                };

                return TypeInference::Inferenced(CbmlType::Enum {
                    field_name: field_name.clone(),
                    field_type: ty.into(),
                });
            }
        };
    }

    fn skip_none(arr: &[Literal]) -> Option<&Literal> {
        let mut len = arr.len();
        let mut count = 0;

        while count < len {
            count += 1;

            if let Some(l) = arr.get(count) {
                match l {
                    Literal::None | Literal::Todo | Literal::Default => {
                        continue;
                    }
                    _ => return Some(l),
                }
            } else {
                break;
            }
        }

        return None;
    }
}

/// 自带的几个基础类型
/// struct enum union 支持 匿名类型.
#[derive(Debug, Clone)]
pub enum CbmlType {
    String,  // string
    Number,  // number
    Boolean, // bool
    Array {
        inner_type: Box<CbmlType>,
    }, // [Type], // 数组类型
    Struct(Vec<StructFieldDefinition>), // 匿名结构体
    Union {
        base_type: Box<CbmlType>,
        alowd_values: Vec<Literal>, // 1 | 2 | 3
    }, // 联合类型
    Optional {
        ty: Box<CbmlType>,
    }, // ?string /number ?bool ?[string] ?[number] ?[bool] ?{name: string}
    Any,     // any
    Enum {
        field_name: String,
        field_type: Box<CbmlType>,
    },

    /// 用户自定义的且设置了名字的类型.
    Custom(String), // 自定义类型 struct name, union(string) name, type name,
}

#[derive(Debug, Clone)]
pub enum TypeInference {
    Inferenced(CbmlType), // 推导出来的类型.
    UnInference,          // 还没推导

    InferenceUnkonw, // 推导了, 没推导出来.
}

#[derive(Debug, Clone)]
pub struct StructTy {
    pub name: String,

    // fields: HashMap<String, CbmlType>, // 字段名不能重复, 所以用 HashMap.
    pub fields: Vec<StructFieldDefinition>, // 字段名不能重复, 所以用 HashMap., // 字段名不能重复, 所以用 HashMap.
}
#[derive(Debug, Clone)]
pub struct EnumTy {
    pub name: String,

    // fields: HashMap<String, CbmlType>, // 字段名不能重复, 所以用 HashMap.
    pub fields: Vec<EnumFieldDefinition>, // 字段名不能重复, 所以用 HashMap., // 字段名不能重复, 所以用 HashMap.
}

#[derive(Debug, Clone)]
pub struct UnionTy {
    pub name: String,
    pub base_type: CbmlType,
    pub alowd_values: Vec<Literal>, // 1 | 2 | 3
}

impl UnionTy {
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

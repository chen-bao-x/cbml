use crate::formater::ToCbmlCode;
use crate::parser::ast::stmt::{Literal, TypeSignStmt};
use crate::{cbml_value::value::CbmlType, lexer::token::Span};

use std::collections::HashMap;
// asin: a = 234234
// asign: b = { name = "" }
#[derive(Debug, Clone)]
struct AsignedField {
    name: String,

    /// 有名字的类型就是类型的名字,
    /// 如果是匿名类型, 则是自动生成的名字.
    _type: Literal,

    span: Span,

    /// 这个字段属于哪个 作用域.
    scope: Scope,
}
#[derive(Debug, Clone)]
enum AsignedLiteral {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<AsignedLiteral>), // [1,2,2]
    Struct(Vec<AsignedField>),  // 结构体字面量暂时先不做.

    /// enum field literal
    EnumFieldLiteral {
        field_name: String,
        literal: Box<AsignedField>,
        span: Span,
    },

    // Optional,
    LiteralNone, // none

    Default,
}

#[derive(Debug, Clone)]
pub struct DefinedField {
    pub name: String,

    /// 如果是匿名类型, 则自动生成名字.
    pub _type: TypeSign,

    pub span: Span,

    /// 这个字段属于哪个 作用域.
    pub scope: ScopeID,
}

// impl ToCbmlCode for DefinedField{
//     fn to_cbml_code(&self, deepth: usize) -> String {
//         format!("{}: {} = {}")
//     }
// }
// fields: 无论层级的字段定义,
// name: string
// perseon: {
//      name: String
//      age: number
// }

// 根据字段名找到对应的 类型, 以及定义这个类型的位置.

/// 类型签名, who: person 中 person 的部分.
/// 根据 类型签名 找到对应的 类型.
#[derive(Debug, Clone)]
pub struct TypeSign {
    /// 类型的名字,
    /// 如果是匿名类型, 则会自动生成名字.
    /// 根据 type_name 找到对应的具体类型.
    pub type_name: String,

    pub kind: TypeSignKind,

    pub span: Span,
}

impl TypeSign {
    pub fn from_asdf(type_name: String, span: Span) -> Self {
        let sadf = match type_name.as_str() {
            "string" => TypeSignKind::String,
            "number" => TypeSignKind::Number,
            "bool" => TypeSignKind::Boolean,
            "any" => TypeSignKind::Any,

            _ => TypeSignKind::Named(type_name.clone()),
        };

        return Self {
            type_name,
            kind: sadf,
            span,
        };
    }
}

impl TypeSign {
    pub fn new(type_sing_stmt: TypeSignStmt, type_name: String) {
        let sadfsdf = type_name.clone();

        let kind = match type_sing_stmt.kind {
            crate::parser::ast::stmt::TypeSignStmtKind::String => TypeSignKind::String,
            crate::parser::ast::stmt::TypeSignStmtKind::Number => TypeSignKind::Number,
            crate::parser::ast::stmt::TypeSignStmtKind::Boolean => TypeSignKind::Boolean,
            crate::parser::ast::stmt::TypeSignStmtKind::Any => TypeSignKind::Any,
            crate::parser::ast::stmt::TypeSignStmtKind::Custom(name) => TypeSignKind::Named(name),

            // crate::parser::ast::stmt::TypeSignStmtKind::Array { .. } => {
            //     TypeSignKind::Named(type_name)
            // }
            // crate::parser::ast::stmt::TypeSignStmtKind::Struct(_) => TypeSignKind::Named(type_name),
            // crate::parser::ast::stmt::TypeSignStmtKind::Optional { .. } => {
            //     TypeSignKind::Named(type_name)
            // }
            crate::parser::ast::stmt::TypeSignStmtKind::Anonymous(_) => {
                TypeSignKind::Named(type_name)
            }
        };

        Self {
            type_name: sadfsdf,
            kind,
            span: type_sing_stmt.span,
        };
    }
}

/// 根据 TypeID 找到对应的实际类型.
#[derive(Debug, Clone)]
pub struct TypeID {}

#[derive(Debug, Clone)]
pub enum TypeSignKind {
    String,
    Number,
    Boolean,
    Any,

    /// 匿名类型会被自动生成一个名字,
    Named(String),
}

/// 具名类型以及自动生成了名字的匿名类型.
#[derive(Debug, Clone)]
pub struct DefinedType {
    /// 类型名在当前 .def.cbml 中时唯一的. 不能重名.
    pub type_name: String,
    pub _type: CbmlType,

    /// 源代码中定义类型的位置.
    pub span: Span,
    pub scope: ScopeID,
}

#[derive(Debug, Clone)]
pub enum Literalsadfsadf {}

/// 每个 Scope 的 id 在当前文件中都需要是独一无二的.
#[derive(Debug, Clone)]
pub struct Scope {
    id: ScopeID,
    chil_scopes: Vec<Scope>,

    /// 类型定义
    /// 字段定义
    /// 字段赋值
    /// 当前作用域内的 所有符号.
    /// 不包含子作用域的符号.
    symbols: Vec<Symbol>,

    /// 如果是 root scope, 则没有 parent scope.
    parent_scope: Option<ScopeID>,
}

impl Scope {
    fn find_child_scope(&self, id: ScopeID) -> Option<Scope> {
        todo!();
        // return None;
    }
}

/// 所有有名字的东西,
/// 有名字的类型,
/// 自动生成了名字的 匿名类型.
/// 字段
#[derive(Debug, Clone)]
pub struct SymbolTable {}

/// 所有有名字的东西,
/// 有名字的类型,
/// 自动生成了名字的 匿名类型.
/// 字段
#[derive(Debug, Clone)]
pub struct Symbol {
    name: String,

    /// 这个 symbol 在源代码中的位置.
    // location: Location,
    kind: SymbolKind,
    // /// 这个符号所在的 作用域.
    // scope: ScopeID,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct ScopeID(pub String);

impl ScopeID {
    pub fn new(scope: String) -> Self {
        Self(scope)
    }
}

#[derive(Debug, Clone)]
enum SymbolKind {
    /// 被复制的字段.
    AsignedField,

    /// 字段定义.
    DefinedField,

    /// 类型定义.
    DefinedType,
}
#[derive(Debug, Clone)]
pub struct CodeFile {
    file_path: String,
    fields: Vec<AsignedField>,

    /// 使用 use 语句引入的 类型定义文件.
    requred_typedef_file: Option<TypeDefFile>,
}
impl CodeFile {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            fields: Vec::new(),
            requred_typedef_file: None,
        }
    }
}

impl CodeFile {
    fn find_defnition(&self, field: AsignedField) -> Option<&DefinedField> {
        // 相同作用域
        // 相同名称

        let Some(file) = &self.requred_typedef_file else {
            return None;
        };

        return file.fields.get(&(field.name, field.scope.id));
    }

    fn find_type_definetion(&self, type_sign: TypeSign) -> Option<&DefinedType> {
        // type_sign.type_name
        let Some(file) = &self.requred_typedef_file else {
            return None;
        };

        return file.types.get(&type_sign.type_name);
    }
}

#[derive(Debug, Clone)]
pub struct TypeDefFile {
    pub file_path: String,

    /// top leverl fields and enum struct fields
    pub fields: std::collections::HashMap<(String, ScopeID), DefinedField>,

    /// 在这种文件中定义的类型.
    pub types: std::collections::HashMap<String, DefinedType>,
}

impl TypeDefFile {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            fields: HashMap::new(),
            types: HashMap::new(),
        }
    }

    pub fn top_fields(&self) -> Vec<&DefinedField> {
        let a = ScopeID(self.file_path.clone());

        let mut re: Vec<&DefinedField> = Vec::new();

        for x in &self.fields {
            if x.0.1 == a {
                re.push(x.1);
            }
        }

        return re;
    }
}

// #[derive(Debug, Clone)]
// pub enum Project {
//     code_file(CodeFile),
//     type_def_file(TypeDefFile),
// }

// impl Project {
//     fn new(path: String) -> Self {
//         todo!()
//     }
// }

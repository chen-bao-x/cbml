use std::collections::HashMap;

use crate::cbml_data::cbml_value::{CbmlValue, ToCbmlValue};
use crate::lexer::token::Span;
use crate::parser::cbml_parser::NodeId;

#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
    pub node_id: NodeId,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    Use(UseStmt), // use "path/to/file"

    /// name = "hello"  identifier asignment literal
    Asignment(AsignmentStmt),

    /// 在文件中定义一个属性.
    FileFieldStmt(StructFieldDefStmt), // name : type; 文件的 field,

    TypeAliasStmt(TypeAliasStmt), // type name = type

    StructDefStmt(StructDef),
    EnumDef(EnumDef), // enum Haha { ssh(string), git( {url: string, branch: string} ) }
    // UnionDef(UnionDef), // 具名 union
    /// 定义了一个有名字的类型
    TypeDef(TypeDefStmt),

    LineComment(String),
    BlockComment(String),
    DocComment(DocumentStmt),

    /// 空行,
    EmptyLine,
}

impl StmtKind {
    pub fn get_span(&self) -> Span {
        match self.clone() {
            StmtKind::Use(use_stmt) => use_stmt.keyword_span,
            StmtKind::Asignment(asignment_stmt) => asignment_stmt.field_name_span,
            StmtKind::FileFieldStmt(struct_field_def_stmt) => struct_field_def_stmt.field_name_span,
            StmtKind::TypeAliasStmt(a) => a.name_span,
            StmtKind::StructDefStmt(struct_def) => struct_def.name_span,
            StmtKind::EnumDef(enum_def) => enum_def.name_span,
            // StmtKind::UnionDef(union_def) => union_def.name_span,
            StmtKind::LineComment(_) => todo!(),
            StmtKind::BlockComment(_) => todo!(),
            StmtKind::DocComment(d) => d.span,
            StmtKind::EmptyLine => todo!(),
            StmtKind::TypeDef(type_def_stmt) => match type_def_stmt {
                // TypeDefStmt::TypeAliasStmt(type_alias_stmt) => type_alias_stmt.name_span,
                TypeDefStmt::StructDefStmt(struct_def) => struct_def.name_span,
                TypeDefStmt::EnumDef(enum_def) => enum_def.name_span,
                TypeDefStmt::UnionDef(union_def) => union_def.name_span,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypeDefStmt {
    StructDefStmt(StructDef),
    EnumDef(EnumDef), // enum Haha { ssh(string), git( {url: string, branch: string} ) }
    UnionDef(UnionDef), // 具名 union
}

impl TypeDefStmt {
    pub fn get_span(&self) -> Span {
        match self {
            TypeDefStmt::StructDefStmt(struct_def) => struct_def.name_span.clone(),
            TypeDefStmt::EnumDef(enum_def) => enum_def.name_span.clone(),
            TypeDefStmt::UnionDef(union_def) => union_def.name_span.clone(),
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            TypeDefStmt::StructDefStmt(struct_def) => &struct_def.struct_name,
            TypeDefStmt::EnumDef(enum_def) => &enum_def.enum_name,
            TypeDefStmt::UnionDef(union_def) => &union_def.union_name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UseStmt {
    /// use "path/to/file"  包含开头和结尾的 双引号, self.url 中保存的是未转译的原始代码.
    pub url: String,
    pub keyword_span: Span,
    pub url_span: Span,
}
impl UseStmt {
    pub fn get_use_url(&self) -> String {
        // 处理空字符串的情况""
        if self.url.len() < 2 {
            return self.url.clone();
        }

        let len = self.url.len();
        let start = 1; // 去掉开头的 双引号
        let end = len - 1; // 去掉结尾的 双引号

        return self.url[start..end].to_string();
    }
}

#[derive(Debug, Clone)]

pub struct TypeAliasStmt {
    pub name: String,
    pub ty: TypeSignStmt,
    pub doc: Option<DocumentStmt>,

    pub name_span: Span,
}

/// 赋值语句,
/// name = "hello"
#[derive(Debug, Clone, PartialEq)]
pub struct AsignmentStmt {
    pub field_name: String,
    // pub value: LiteralKind,
    pub value: Literal,
    pub field_name_span: Span,
}

/// 属性类型申明
/// name: string
/// name: string default "hello"
#[derive(Debug, Clone, PartialEq)]
pub struct StructFieldDefStmt {
    pub field_name: String,

    pub _type: TypeSignStmt,

    pub default: Option<Literal>,

    pub doc: Option<DocumentStmt>,

    pub field_name_span: Span,

    pub node_id: NodeId,
}

impl StructFieldDefStmt {
    pub fn end_span(&self) -> Span {
        if let Some(v) = &self.default {
            return v.span.clone();
        } else {
            return self._type.span.clone();
        }
    }

    pub fn get_span(&self) -> Span {
        Span {
            start: self.field_name_span.start.clone(),
            end: self.end_span().end,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentStmt {
    pub document: String,
    pub span: Span,
}

/// 枚举属性申明
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EnumFieldDef {
    pub field_name: String,
    // pub _type: TypeSignStmtKind,
    pub _type: TypeSignStmt,

    pub field_name_span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    pub kind: LiteralKind,
    pub span: Span,
}

/// 字面量
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralKind {
    String(String),
    // String {
    //     val: String,
    //     span: Span,
    // },
    Number(f64),
    Boolean(bool),

    Array(Vec<Literal>),        // [1,2,2]
    Struct(Vec<AsignmentStmt>), // 结构体字面量暂时先不做.
    /// enum field literal
    EnumFieldLiteral {
        field_name: String,
        literal: Box<Literal>,
        span: Span,
    },

    // Optional,
    LiteralNone, // none

                 // /// 这个可能会留下隐患, 暂时先不支持 todo 功能.
                 // Todo,
                 // Default,
}

impl ToCbmlValue for Literal {
    fn to_cbml_value(&self) -> CbmlValue {
        self.kind.to_cbml_value()
    }
}

impl ToCbmlValue for LiteralKind {
    fn to_cbml_value(&self) -> CbmlValue {
        match self.clone() {
            LiteralKind::String(s) => CbmlValue::String(s),
            LiteralKind::Number(n) => CbmlValue::Number(n),
            LiteralKind::Boolean(b) => CbmlValue::Boolean(b),
            LiteralKind::Array(literals) => {
                CbmlValue::Array(literals.iter().map(|x| x.to_cbml_value()).collect())
            }
            LiteralKind::Struct(asignment_stmts) => {
                let mut fields: HashMap<String, CbmlValue> = HashMap::new();

                for x in asignment_stmts {
                    fields.insert(x.field_name.clone(), x.value.to_cbml_value());
                }

                return CbmlValue::Struct(fields);
            }
            LiteralKind::LiteralNone => CbmlValue::None,
            LiteralKind::EnumFieldLiteral {
                field_name,
                literal,
                ..
            } => CbmlValue::EnumField(field_name, Box::new(literal.to_cbml_value())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeSignStmt {
    pub kind: TypeSignStmtKind,
    pub span: Span,

    /// 可能会定义匿名类型, 所以需要 node_id
    pub node_id: NodeId,
}

// TypeSignStmtKind
/// 自带的几个基础类型
/// struct enum union 支持 匿名类型.
// #[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TypeSignStmtKind {
    String,  // string
    Number,  // number
    Boolean, // bool
    Any,     // any

    /// 匿名类型
    Anonymous(AnonymousTypeDefStmt),

    /// 用户自定义的且设置了名字的类型.
    Custom(String), // 自定义类型 struct name, union(string) name, type name,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnonymousTypeDefStmt {
    pub kind: AnonymousTypeDefKind,
    pub node_id: NodeId,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnonymousTypeDefKind {
    /// 匿名数组类型
    /// [Type]
    // Array { inner_type: Box<TypeSignStmtKind> },
    Array { inner_type: Box<TypeSignStmt> },

    /// 匿名 enum
    Enum {
        // field_type: Box<CbmlType>,
        fields: Vec<EnumFieldDef>,
    },
    /// 匿名结构体
    Struct(Vec<StructFieldDefStmt>),
    /// 匿名 union
    Union {
        alowd_values: Vec<CbmlValue>, // 1 | 2 | 3 | "asdf" | false | [1,2,3]
    }, // 匿名联合类型

    Optional {
        inner_type: Box<TypeSignStmt>,
        // span: Span,
    }, // ?string /number ?bool ?[string] ?[number] ?[bool] ?{name: string}
}

impl TypeSignStmtKind {
    pub fn is_custom_ty(&self) -> bool {
        match self {
            TypeSignStmtKind::Custom(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypeInference {
    Inferenced(TypeSignStmtKind), // 推导出来的类型.
    // UnInference,          // 还没推导
    InferenceUnkonw, // 推导了, 没推导出来.
}

/// 具名 struct
#[derive(Debug, Clone)]
pub struct StructDef {
    pub doc: Option<DocumentStmt>,

    pub struct_name: String,

    // fields: HashMap<String, CbmlType>, // 字段名不能重复, 所以用 HashMap.
    pub fields: Vec<StructFieldDefStmt>, // 字段名不能重复, 所以用 HashMap., // 字段名不能重复, 所以用 HashMap.

    pub name_span: Span,
}

impl StructDef {
    pub fn end_span(&self) -> Span {
        let Some(last) = self.fields.last() else {
            return self.name_span.clone();
        };

        return Span {
            start: self.name_span.start.clone(),
            end: last.field_name_span.end.clone(),
        };
    }
}

/// 具名 enum
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub doc: Option<DocumentStmt>,

    pub enum_name: String,

    pub fields: Vec<EnumFieldDef>,

    pub name_span: Span,
}

/// 具名 union
#[derive(Debug, Clone)]
pub struct UnionDef {
    pub union_name: String,
    pub base_type: TypeSignStmtKind,
    // pub allowed_values: Vec<LiteralKind>, // 1 | 2 | 3
    pub allowed_values: Vec<Literal>, // 1 | 2 | 3
    pub doc: Option<DocumentStmt>,
    pub name_span: Span,
}

impl UnionDef {
    #[allow(dead_code)]
    pub fn duplicate_check(&self) -> Vec<LiteralKind> {
        let mut re: Vec<&LiteralKind> = Vec::new();
        let mut duplicated: Vec<LiteralKind> = Vec::new();

        for v in &self.allowed_values {
            if re.contains(&&v.kind) {
                duplicated.push(v.kind.clone());
            } else {
                re.push(&v.kind)
            }
        }
        return duplicated;
    }
}

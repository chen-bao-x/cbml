use crate::{
    cbml_value::value::CbmlType, lexer::token::Span, parser::ast::stmt::Literal,
    typecheck::types_for_check::ScopeID,
};

#[derive(Debug, Clone)]
pub struct FieldAsign {
    pub name: String,
    pub value: Literal,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub type_sign: String,
    pub default_value: Option<Literal>,
    pub span: Span,
    pub scope: ScopeID,
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    /// 匿名类型也需要自动生成类型名字,
    /// 内置类型是有名字的.
    pub name: String,

    pub ty: CbmlType,
    pub span: Span,
}

use crate::cbml_value::value::CbmlType;
use crate::lexer::token::Span;
use crate::parser::ast::stmt::Literal;

#[derive(Debug, Clone)]
pub struct FieldAsign {
    pub name: String,
    pub value: Literal,
    pub span: Span,
    pub id: usize,
    pub scope: ScopeID,
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,

    pub type_: TypeInfo,
    pub default_value: Option<Literal>,
    pub span: Span,
    pub scope: ScopeID,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    /// 匿名类型也需要自动生成类型名字,
    /// 内置类型是有名字的.
    // pub name: String,
    pub ty: CbmlType,
    pub span: Span,

    pub(crate) type_id: usize,
}
impl TypeInfo {
    pub fn get_type_id(&self) -> usize {
        match &self.ty.kind {
            crate::cbml_value::value::CbmlTypeKind::String => 0,
            crate::cbml_value::value::CbmlTypeKind::Number => 1,
            crate::cbml_value::value::CbmlTypeKind::Bool => 2,
            crate::cbml_value::value::CbmlTypeKind::Any => 3,
            _ => self.type_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct ScopeID(pub String);

impl ScopeID {
    pub fn new(scope: String) -> Self {
        Self(scope)
    }
}

use crate::cbml_data::cbml_type::*;
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

impl FieldAsign {
    pub fn child_scope(&self) -> ScopeID {
        // let asdf = vec![self.scope.clone(), ScopeID::new(self.name.clone())];

        let mut re = String::new();
        re.push_str(&self.scope.0);
        re.push_str("::");
        re.push_str(&self.name);

        return ScopeID::new(re);
    }
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

impl FieldDef {
    pub fn child_scope(&self) -> ScopeID {
        // let asdf = vec![self.scope.clone(), ScopeID::new(self.name.clone())];

        let mut re = String::new();
        re.push_str(&self.scope.0);
        re.push_str("::");
        re.push_str(&self.name);

        return ScopeID::new(re);
    }
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
            CbmlTypeKind::String => 0,
            CbmlTypeKind::Number => 1,
            CbmlTypeKind::Bool => 2,
            CbmlTypeKind::Any => 3,
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

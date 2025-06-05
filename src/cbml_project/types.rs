///! 区别于 AST 的类型定义.
/// 这里的类型主要用于 错误检查 代码生成 符号跳转, 查找符号之间的关系等.
use crate::cbml_data::cbml_type::*;

use crate::ToCbml;
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
    pub scope_id: ScopeID,
    pub doc: Option<String>,
}

impl FieldDef {
    pub fn child_scope(&self) -> ScopeID {
        // let asdf = vec![self.scope.clone(), ScopeID::new(self.name.clone())];

        let mut re = String::new();
        re.push_str(&self.scope_id.0);
        re.push_str("::");
        re.push_str(&self.name);

        return ScopeID::new(re);
    }
}

impl ToCbml for FieldAsign {
    fn to_cbml(&self, deepth: usize) -> String {
        let mut re = String::new();

        re.push_str(&"    ".repeat(deepth));
        re.push_str(&self.name);
        re.push_str("= ");
        re.push_str(&self.value.to_cbml(deepth));

        return re;
    }
}
impl ToCbml for FieldDef {
    fn to_cbml(&self, deepth: usize) -> String {
        let mut re = String::new();

        if let Some(doc) = &self.doc {
            re.push_str(&"    ".repeat(deepth));
            re.push_str(&format!("{}", doc));
        }

        re.push_str(&"    ".repeat(deepth));
        re.push_str(&self.name);
        re.push_str(": ");
        re.push_str(&self.type_.ty.to_cbml(0));

        if let Some(default) = &self.default_value {
            re.push_str(" default ");
            re.push_str(&default.kind.to_cbml(deepth));
        }

        return re;
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
        match &self.ty {
            CbmlType::String => 0,
            CbmlType::Number => 1,
            CbmlType::Bool => 2,
            CbmlType::Any => 3,
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

    pub fn empty() -> Self {
        Self::new(String::new())
    }
}

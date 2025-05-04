use crate::parser::Stmt;

// pub fn typecheck(ast: Vec<Stmt>) {

// }


enum TypeCheckedResult {
    Ast(Vec<Stmt>),
    Warning(),
}
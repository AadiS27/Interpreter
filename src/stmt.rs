use crate::expr::Expr;


pub enum Stmt {
    Expression { expression: Expr },
    Print { expression: Expr },
    Var { name: String, initializer: Option<Expr> },
    Block(Vec<Stmt>),
}

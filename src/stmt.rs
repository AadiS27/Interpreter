use crate::expr::Expr;


pub enum Stmt {
    Expression { expression: Expr },
    Print { expression: Expr },
    Var { name: String, initializer: Option<Expr> },
    Block(Vec<Stmt>),
    If { condition: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}

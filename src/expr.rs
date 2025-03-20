// Generated Rust AST for Expr
use crate::token::Token;
use std::any::Any;


pub trait ExprVisitor {
    fn visit_binary(&self, expr: &Binary) -> String;
    fn visit_grouping(&self, expr: &Grouping) -> String;
    fn visit_literal(&self, expr: &Literal) -> String;
    fn visit_unary(&self, expr: &Unary) -> String;
     fn visit_variable(&self, expr: &Variable) -> String;

}
// pub trait Expr {
//     fn accept<T>(&self, visitor: & ExprVisitor<T>) -> T;
// }
pub enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    Variable(Variable),
    Assign(String, Box<Expr>), // Represents variable assignment
    If { condition: Box<Expr>, then_branch: Box<Expr>, else_branch: Option<Box<Expr>> },
    Logical { left: Box<Expr>, operator: Token, right: Box<Expr> },
}
    


pub struct Logical {
    pub left: Box< Expr>,
    pub operator: Token,
    pub right: Box< Expr>,
}

pub struct Binary {
    pub left: Box< Expr>,
    pub operator: Token,
    pub right: Box< Expr>,
}

impl Binary {
    pub fn new(left: Box< Expr>, operator: Token, right: Box< Expr>) -> Self {
        Binary {
            left: left,
            operator: operator,
            right: right,
        }
    }
}


pub struct Grouping {
    pub expression: Box< Expr>,
}

impl Grouping {
    pub fn new(expression: Box< Expr>) -> Self {
        Grouping {
            expression: expression,
        }
    }
}



use std::sync::Arc;
use std::fmt::Debug;
use crate::token::TokenLiteral;  // Import TokenLiteral

#[derive(Clone)]
pub struct Literal {
    pub value: Arc<dyn Any + Send + Sync>,
}

impl Literal {
    pub fn new(value: TokenLiteral) -> Self {
        Literal {
            value: Arc::new(value),  // Store TokenLiteral directly
        }
    }

   
}

// Debug implementation for easier printing
impl Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Literal(...)")
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: Token,
}

impl Variable {
    pub fn new(name: Token) -> Self {
        Variable {
            name: name,
        }
    }
}





pub struct Unary {
    pub operator: Token,
    pub right: Box< Expr>,
}

impl Unary {
    pub fn new(operator: Token, right: Box< Expr>) -> Self {
        Unary {
            operator: operator,
            right: right,
        }
    }
}

impl Expr {
    pub fn accept(&self, visitor: &dyn ExprVisitor) -> String {
        match self {
            Expr::Binary(b) => visitor.visit_binary(b),
            Expr::Grouping(g) => visitor.visit_grouping(g),
            Expr::Literal(l) => visitor.visit_literal(l),
            Expr::Unary(u) => visitor.visit_unary(u),
            Expr::Variable(v) => visitor.visit_variable(v),
            Expr::Assign(name, expr) => format!("Assign({}, ...)", name),
            Expr::If { condition, then_branch, else_branch } => {
                format!("If {{ {}, {}, {} }}", condition.accept(visitor), then_branch.accept(visitor), else_branch.as_ref().map_or("None".to_string(), |e| e.accept(visitor)))
            }
            Expr::Logical { left, operator, right } => {
                format!("Logical {{ {}, {:?}, {} }}", left.accept(visitor), operator, right.accept(visitor))
            }
        }
    }
}
// Generated Rust AST for Expr
use crate::token::Token;
use std::any::Any;


pub trait ExprVisitor {
    fn visit_binary(&self, expr: &Binary) -> String;
    fn visit_grouping(&self, expr: &Grouping) -> String;
    fn visit_literal(&self, expr: &Literal) -> String;
    fn visit_unary(&self, expr: &Unary) -> String;
}
// pub trait Expr {
//     fn accept<T>(&self, visitor: & ExprVisitor<T>) -> T;
// }
pub enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    
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

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.value.downcast_ref::<T>()
    }
}

// Debug implementation for easier printing
impl Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Literal(...)")
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
        }
    }
}
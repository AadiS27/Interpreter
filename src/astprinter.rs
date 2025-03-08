use crate::expr::{Expr, ExprVisitor, Binary, Grouping, Literal, Unary};
use crate::token::{Token, TokenType, TokenLiteral};


pub struct AstPrinter;

impl AstPrinter {

    pub fn new() -> Self {
        AstPrinter
    }
    pub fn print(&self, expr: &Expr) -> String {
        expr.accept(self) // Now it returns the AST as a string
    }

    fn parenthesize(&self, name: &str, expressions: &[&Expr]) -> String {
        let mut result = String::from("(");
        result.push_str(name);
        for expr in expressions {
            result.push(' ');
            result.push_str(&expr.accept(self)); // Now it works
        }
        result.push(')');
        result
    }
}

impl ExprVisitor for AstPrinter {
    fn visit_binary(&self, expr: &Binary) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&expr.left, &expr.right])
    }

    fn visit_grouping(&self, expr: &Grouping) -> String {
        self.parenthesize("group", &[&expr.expression])
    }

    fn visit_literal(&self, expr: &Literal) -> String {
        if let Some(value) = expr.value.downcast_ref::<String>() {
            return value.clone();
        }
        "nil".to_string()
    }

    fn visit_unary(&self, expr: &Unary) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&expr.right])
    }
}


fn main() {
    let expr = Expr::Binary(Binary {
        left: Box::new(Expr::Unary(Unary {
            operator: Token {
                lexeme: "-".to_string(),
                token_type: TokenType::MINUS,
                literal: TokenLiteral::Null,
            },
            right: Box::new(Expr::Literal(Literal {
                value: Box::new("123".to_string()),
            })),
        })),
        operator: Token {
            lexeme: "*".to_string(),
            token_type: TokenType::STAR,
            literal: TokenLiteral::Null,
        },
        right: Box::new(Expr::Grouping(Grouping {
            expression: Box::new(Expr::Literal(Literal {
                value: Box::new("45.67".to_string()),
            })),
        })),
    });

    let printer = AstPrinter;
    println!("{}", printer.print(&expr));
}
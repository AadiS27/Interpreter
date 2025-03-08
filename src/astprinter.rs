use crate::expr::{Expr, ExprVisitor, Binary, Grouping, Literal, Unary};
use crate::token::{Token, TokenType};

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&self, expr: Expr) -> String {
        Expr::accept(self,&expr)
    }

    fn parenthesize(&self, name: &str, expressions: &[&dyn Expr]) -> String {
        let mut result = String::from("(");
        result.push_str(name);
        for expr in expressions {
            result.push(' ');
            result.push_str(&expr.accept(self));
        }
        result.push(')');
        result
    }
}

impl ExprVisitor<String> for AstPrinter {
    fn visit_binary(&self, expr: &Binary) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&*expr.left, &*expr.right])
    }

    fn visit_grouping(&self, expr: &Grouping) -> String {
        self.parenthesize("group", &[&*expr.expression])
    }

    fn visit_literal(&self, expr: &Literal) -> String {
        match expr.value.downcast_ref::<String>() {
            Some(s) => s.clone(),
            None => "nil".to_string(),
        }
    }

    fn visit_unary(&self, expr: &Unary) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&*expr.right])
    }
}
fn main() {
    let expr = Binary {
        left: Box::new(Unary {
            operator: Token { lexeme: "-".to_string(), token_type: TokenType::Minus, literal: None },
            right: Box::new(Literal { value: Box::new("123".to_string()) }),
        }),
        operator: Token { lexeme: "*".to_string(), token_type: TokenType::Star, literal: None },
        right: Box::new(Grouping {
            expression: Box::new(Literal { value: Box::new("45.67".to_string()) }),
        }),
    };

    let printer = AstPrinter;
    println!("{}", printer.print(&expr));
}

use crate::expr::{Binary, Expr, ExprVisitor, Grouping, Literal, Unary, Variable};
use crate::token::TokenLiteral;

pub struct AstPrinter;

impl AstPrinter {
   
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
        if let Some(token_literal) = expr.value.downcast_ref::<TokenLiteral>() {
            match token_literal {
                TokenLiteral::Number(n) => return n.to_string(),
                TokenLiteral::String(s) => return format!("\"{}\"", s),
                TokenLiteral::Boolean(b) => return b.to_string(),
                TokenLiteral::Identifier(id) => return id.clone(),
                TokenLiteral::Null => return "nil".to_string(),
            }
        }

        if let Some(b) = expr.value.downcast_ref::<bool>() {
            return b.to_string();
        }
        if let Some(n) = expr.value.downcast_ref::<f64>() {
            return n.to_string();
        }
        if let Some(s) = expr.value.downcast_ref::<String>() {
            return format!("\"{}\"", s);
        }

        "nil".to_string()
    }
    fn visit_unary(&self, expr: &Unary) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&expr.right])
    }
    fn visit_variable(&self, expr: &Variable) -> String {
        expr.name.lexeme.clone()
    }
}

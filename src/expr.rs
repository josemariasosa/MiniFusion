use std::fmt;

use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Eq,
    Gt,
    Lt,
    And,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Column(String),
    Literal(Value),
    Binary {
        left: Box<Expr>,
        op: Operator,
        right: Box<Expr>,
    },
}

pub fn col(name: &str) -> Expr {
    Expr::Column(name.to_string())
}

pub fn lit(value: Value) -> Expr {
    Expr::Literal(value)
}

impl Expr {
    pub fn binary(self, op: Operator, right: Expr) -> Expr {
        Expr::Binary {
            left: Box::new(self),
            op,
            right: Box::new(right),
        }
    }

    pub fn eq(self, right: Expr) -> Expr {
        self.binary(Operator::Eq, right)
    }

    pub fn gt(self, right: Expr) -> Expr {
        self.binary(Operator::Gt, right)
    }

    pub fn lt(self, right: Expr) -> Expr {
        self.binary(Operator::Lt, right)
    }

    pub fn and(self, right: Expr) -> Expr {
        self.binary(Operator::And, right)
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operator::And => write!(f, "AND"),
            Operator::Eq => write!(f, "=="),
            Operator::Gt => write!(f, ">"),
            Operator::Lt => write!(f, "<"),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Column(name) => write!(f, "{name}"),
            Expr::Literal(value) => write!(f, "{value}"),
            Expr::Binary { left, op, right } => {
                write!(f, "({left} {op} {right})")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test 1 — Column Expression
    #[test]
    fn column_expression() {
        assert_eq!(col("distance_km"), Expr::Column("distance_km".to_owned()))
    }

    // Test 2 — Literal Expression
    #[test]
    fn literal_expression() {
        assert_eq!(lit(Value::Int64(42)), Expr::Literal(Value::Int64(42)))
    }

    // Test 3 — Binary Expression
    #[test]
    fn binary_expression() {
        let expr = col("fare").gt(lit(Value::Int64(100)));

        // The root is a Binary expression.
        assert!(matches!(expr, Expr::Binary { .. }));

        let Expr::Binary { left, op, right } = expr else {
            panic!("Expected a Binary expression")
        };

        // The operator is Gt.
        assert!(matches!(op, Operator::Gt));

        // The left child is a column reference.
        assert!(matches!(*left, Expr::Column(_)));

        // The right child is a literal.
        assert!(matches!(*right, Expr::Literal(_)));
    }

    // Test 4 — Nested Expressions
    #[test]
    fn nested_expressions() {
        let expr = col("distance_km")
            .gt(lit(Value::Int64(5)))
            .and(col("fare").lt(lit(Value::Int64(200))));
        assert!(matches!(expr, Expr::Binary { .. }));

        let Expr::Binary { left, op, right } = expr else {
            panic!("Expected a Binary expression")
        };
        // Verify that the root operator is And, with two binary expressions as children.
        assert!(matches!(op, Operator::And));
        assert!(matches!(*left, Expr::Binary { .. }));
        assert!(matches!(*right, Expr::Binary { .. }));
    }

    // Test 5 — Expression Display
    #[test]
    fn expression_display() {
        let expr = col("distance_km")
            .gt(lit(Value::Int64(5)))
            .and(col("fare").lt(lit(Value::Int64(200))));

        assert_eq!(format!("{}", expr), "((distance_km > 5) AND (fare < 200))");
    }
}

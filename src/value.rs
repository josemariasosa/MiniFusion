use std::fmt;

use crate::schema::DataType;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int64(i64),
    Float64(f64),
    Utf8(String),
    Boolean(bool),
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int64(v) => write!(f, "{v}"),
            Value::Float64(v) => write!(f, "{v}"),
            Value::Utf8(v) => write!(f, "{v}"),
            Value::Boolean(v) => write!(f, "{v}"),
            Value::Null => write!(f, "NULL"),
        }
    }
}

impl Value {
    pub fn data_type(&self) -> Option<DataType> {
        match self {
            Value::Int64(_) => Some(DataType::Int64),
            Value::Float64(_) => Some(DataType::Float64),
            Value::Utf8(_) => Some(DataType::Utf8),
            Value::Boolean(_) => Some(DataType::Boolean),
            Value::Null => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::expr::Expr;

    use super::*;

    // 1. Two Value::Int64(42) values compare equal.
    #[test]
    fn int64_equality() {
        assert_eq!(Value::Int64(42), Value::Int64(42));
    }

    // 2. Value::Int64(42) and Value::Float64(42.0) are not equal.
    #[test]
    fn int64_float64_inequality() {
        assert_ne!(Value::Int64(42), Value::Float64(42.0));
    }

    // 3. Two Value::Utf8 values with identical text compare equal.
    #[test]
    fn utf8_equality() {
        assert_eq!(
            Value::Utf8("hello".to_string()),
            Value::Utf8("hello".to_string())
        );
    }

    // 4. Value::Null compares equal to itself using Rust's PartialEq.
    #[test]
    fn null_equality() {
        assert_eq!(Value::Null, Value::Null);
    }

    #[test]
    fn boolean_equality() {
        assert_eq!(Value::Boolean(true), Value::Boolean(true));
        assert_ne!(Value::Boolean(true), Value::Boolean(false));
    }

    #[test]
    fn literal_int64_type() {
        let Expr::Literal(value) = Expr::Literal(Value::Int64(4)) else {
            panic!()
        };
        assert_eq!(value.data_type().unwrap(), DataType::Int64);
    }

    #[test]
    fn literal_float64_type() {
        let Expr::Literal(value) = Expr::Literal(Value::Float64(5.0)) else {
            panic!()
        };
        assert_eq!(value.data_type().unwrap(), DataType::Float64);
    }

    #[test]
    fn literal_utf8_type() {
        let Expr::Literal(value) = Expr::Literal(Value::Utf8("Maria".to_owned())) else {
            panic!()
        };
        assert_eq!(value.data_type().unwrap(), DataType::Utf8);
    }

    #[test]
    fn literal_boolean_type() {
        let Expr::Literal(value) = Expr::Literal(Value::Boolean(false)) else {
            panic!()
        };
        assert_eq!(value.data_type().unwrap(), DataType::Boolean);
    }

    #[test]
    fn literal_null_has_no_type() {
        let Expr::Literal(value) = Expr::Literal(Value::Null) else {
            panic!()
        };
        assert!(value.data_type().is_none());
    }
}

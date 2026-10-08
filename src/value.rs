#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int64(i64),
    Float64(f64),
    Utf8(String),
    Boolean(bool),
    Null,
}

#[cfg(test)]
mod tests {
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
}

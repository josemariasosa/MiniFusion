use crate::schema::{DataType, Schema};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    UnknownColumn(String),
    UntypedNull,
    TypeMismatch,
    InvalidOperator,
}

pub fn resolve_column_type(name: &str, schema: &Schema) -> Result<DataType, ValidationError> {
    schema
        .fields
        .iter()
        .find(|f| f.name == name)
        .map(|f| f.data_type.clone())
        .ok_or_else(|| ValidationError::UnknownColumn(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trips_schema;

    #[test]
    fn resolve_existing_column() {
        let schema = trips_schema();
        let data_type = resolve_column_type("zone", &schema).unwrap();
        assert_eq!(data_type, DataType::Utf8);
    }

    #[test]
    fn reject_unknown_column() {
        let schema = trips_schema();
        assert!(resolve_column_type("unknown", &schema).is_err());
    }

    #[test]
    fn resolve_empty_schema() {
        let schema = Schema::new(vec![]);
        assert!(resolve_column_type("unknown", &schema).is_err());
    }
}

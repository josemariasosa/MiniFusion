use crate::{
    expr::{Expr, Operator},
    schema::{DataType, Field, Schema},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    UnknownColumn(String),
    UntypedNull,
    TypeMismatch,
    InvalidOperator,
    ExpectedBoolean,
}

pub fn resolve_column_type(name: &str, schema: &Schema) -> Result<DataType, ValidationError> {
    schema
        .fields
        .iter()
        .find(|f| f.name == name)
        .map(|f| f.data_type.clone())
        .ok_or_else(|| ValidationError::UnknownColumn(name.to_string()))
}

pub fn infer_expr_type(expr: &Expr, schema: &Schema) -> Result<DataType, ValidationError> {
    match expr {
        Expr::Column(name) => resolve_column_type(name, schema),

        Expr::Literal(value) => value
            .data_type()
            .ok_or_else(|| ValidationError::UntypedNull),

        Expr::Binary { left, op, right } => {
            let left_type = infer_expr_type(left, schema)?;
            let right_type = infer_expr_type(right, schema)?;
            infer_binary_type(op, &left_type, &right_type)
        }
    }
}

fn infer_binary_type(
    op: &Operator,
    left: &DataType,
    right: &DataType,
) -> Result<DataType, ValidationError> {
    if left != right {
        return Err(ValidationError::TypeMismatch);
    }

    match (left, op) {
        (DataType::Int64 | DataType::Float64, Operator::Eq | Operator::Gt | Operator::Lt) => {
            Ok(DataType::Boolean)
        }
        (DataType::Utf8, Operator::Eq) => Ok(DataType::Boolean),
        (DataType::Boolean, Operator::And | Operator::Eq) => Ok(DataType::Boolean),
        _ => Err(ValidationError::InvalidOperator),
    }
}

pub fn validate_filter(predicate: &Expr, schema: &Schema) -> Result<(), ValidationError> {
    let data_type = infer_expr_type(predicate, schema)?;

    if data_type == DataType::Boolean {
        Ok(())
    } else {
        Err(ValidationError::ExpectedBoolean)
    }
}

pub fn project_schema(
    columns: &[String],
    input_schema: &Schema,
) -> Result<Schema, ValidationError> {
    let new_fields: Vec<Field> = columns
        .iter()
        .map(|name| {
            input_schema
                .fields
                .iter()
                .find(|field| &field.name == name)
                .cloned()
                .ok_or_else(|| ValidationError::UnknownColumn(name.clone()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Schema::new(new_fields))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        expr::{col, lit},
        trips_schema,
        value::Value,
    };

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

    #[test]
    fn test_infer_expr_type() {
        let schema = trips_schema();
        assert_eq!(
            infer_expr_type(&col("fare"), &schema).unwrap(),
            DataType::Float64
        );
        assert_eq!(
            infer_expr_type(&lit(Value::Int64(10)), &schema).unwrap(),
            DataType::Int64
        );
    }

    #[test]
    fn infer_matrix_comparison() {
        let cases = [
            // Numeric
            (DataType::Int64, Operator::Eq),
            (DataType::Int64, Operator::Gt),
            (DataType::Int64, Operator::Lt),
            (DataType::Float64, Operator::Eq),
            (DataType::Float64, Operator::Gt),
            (DataType::Float64, Operator::Lt),
            // String
            (DataType::Utf8, Operator::Eq),
            // Boolean
            (DataType::Boolean, Operator::Eq),
            (DataType::Boolean, Operator::And),
        ];

        for (data_type, operator) in cases {
            let result = infer_binary_type(&operator, &data_type, &data_type);

            assert_eq!(
                result,
                Ok(DataType::Boolean),
                "Failed for {data_type:?} with {operator:?}"
            );
        }
    }

    #[test]
    fn reject_invalid_binary_types() {
        let cases = [
            (
                DataType::Utf8,
                Operator::And,
                DataType::Utf8,
                ValidationError::InvalidOperator,
            ),
            (
                DataType::Boolean,
                Operator::Gt,
                DataType::Boolean,
                ValidationError::InvalidOperator,
            ),
            // reject_mixed_numeric_types
            (
                DataType::Int64,
                Operator::Gt,
                DataType::Float64,
                ValidationError::TypeMismatch,
            ),
            // reject_string_greater_than_number
            (
                DataType::Utf8,
                Operator::Gt,
                DataType::Int64,
                ValidationError::TypeMismatch,
            ),
            // reject_numeric_and
            (
                DataType::Float64,
                Operator::And,
                DataType::Float64,
                ValidationError::InvalidOperator,
            ),
            (
                DataType::Int64,
                Operator::And,
                DataType::Int64,
                ValidationError::InvalidOperator,
            ),
        ];

        for (left, op, right, expected_error) in cases {
            let result = infer_binary_type(&op, &left, &right);

            assert_eq!(
                result,
                Err(expected_error),
                "Failed for {left:?} {op:?} {right:?}"
            );
        }
    }

    #[test]
    fn reject_untyped_null() {
        let schema = trips_schema();
        let expr = lit(Value::Null);

        let result = infer_expr_type(&expr, &schema);

        assert_eq!(result, Err(ValidationError::UntypedNull));
    }

    #[test]
    fn infer_nested_boolean_expression() {
        let schema = trips_schema();

        let expr = col("fare")
            .gt(lit(Value::Float64(100.0)))
            .and(col("zone").eq(lit(Value::Utf8("A".to_string()))));

        let result = infer_expr_type(&expr, &schema);

        assert_eq!(result, Ok(DataType::Boolean));
    }

    #[test]
    fn accept_boolean_filter() {
        let schema = trips_schema();
        let expr = col("fare")
            .gt(lit(Value::Float64(100.0)))
            .and(col("zone").eq(lit(Value::Utf8("A".to_string()))));

        assert!(validate_filter(&expr, &schema).is_ok());
    }

    #[test]
    fn reject_numeric_filter() {
        let schema = trips_schema();
        assert_eq!(
            validate_filter(&col("fare"), &schema),
            Err(ValidationError::ExpectedBoolean)
        );
    }

    #[test]
    fn reject_unknown_filter_column() {
        let schema = trips_schema();

        assert_eq!(
            validate_filter(&col("unknown"), &schema),
            Err(ValidationError::UnknownColumn("unknown".to_string()))
        );
    }

    #[test]
    fn reject_invalid_filter_expression() {
        let schema = trips_schema();

        let expr = col("fare").and(lit(Value::Utf8("yep".to_string())));

        assert_eq!(
            validate_filter(&expr, &schema),
            Err(ValidationError::TypeMismatch)
        );
    }

    #[test]
    fn project_existing_columns_preserves_order() {
        let schema = trips_schema();
        let projected =
            project_schema(&vec!["fare".to_owned(), "zone".to_owned()], &schema).unwrap();

        assert_eq!(schema.fields.len(), 4);
        assert_eq!(projected.fields.len(), 2);
        assert_eq!(
            (
                projected.fields[0].name.clone(),
                projected.fields[1].name.clone()
            ),
            ("fare".to_owned(), "zone".to_owned())
        );
    }

    #[test]
    fn reject_unknown_projection_column() {
        let schema = trips_schema();
        let projected = project_schema(&vec!["unknown".to_owned()], &schema);

        assert_eq!(
            projected,
            Err(ValidationError::UnknownColumn("unknown".to_owned()))
        )
    }

    #[test]
    fn project_empty_column_list() {
        let schema = trips_schema();
        let projected = project_schema(&vec![], &schema).unwrap();

        assert_eq!(schema.fields.len(), 4);
        assert_eq!(projected.fields.len(), 0);
    }
}

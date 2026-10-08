Step 1 — Understand our first abstraction: Value
Before defining expressions or logical plans, we need to decide what a value means inside our query language.
Consider:
SELECT zone, fare
FROM trips
WHERE distance_km > 5;


There are three important elements here:
- distance_km is a reference to a column.
- 5 is a literal value.
- distance_km > 5 is an expression that will eventually evaluate to a Boolean.
That means we need a way to represent literal values. We can begin with a Rust enum.
Your first implementation task
Create a module at src/value.rs:
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int64(i64),
    Float64(f64),
    Utf8(String),
    Boolean(bool),
    Null,
}


And expose it from main.rs:
mod value;


Why an enum instead of using generics like Value<T>?
Because a single dataset can contain columns of different types, and expressions can combine them. We need a unified representation of scalar values.
The variant Null is included because missing values are fundamental to query engines, although we won't implement SQL's three-valued logic yet.
In real DataFusion, a similar role is played by ScalarValue, which has a much richer set of supported types.
Write your first tests
In value.rs, add a #[cfg(test)] module that verifies:
1. Two Value::Int64(42) values compare equal.
2. Value::Int64(42) and Value::Float64(42.0) are not equal.
3. Two Value::Utf8 values with identical text compare equal.
4. Value::Null compares equal to itself using Rust's PartialEq.
The fourth test is particularly interesting: Rust equality of two Value::Null variants does not mean SQL evaluates NULL = NULL to TRUE. We'll deal with that distinction when we implement expression evaluation.
Run:
cargo fmt
cargo test
cargo clippy --all-targets -- -D warnings


One design question before Step 2
Look at our Value definition:
Int64(i64),
Float64(f64),
Utf8(String),
Boolean(bool),
Null,


Why do you think we need both Value and (eventually) a separate DataType enum?
For example:
enum DataType {
    Int64,
    Float64,
    Utf8,
    Boolean,
}


What information could DataType describe that Value alone cannot?
Think about a table schema before any rows have been loaded.
This is our first conceptual exercise. We'll use your answer to move into the distinction between data, types, and schemas, which is foundational to logical planning in DataFusion. 

Apache DataFusion documentation



CP1 progress: Step 1 — Scalar values. No external dependencies required.
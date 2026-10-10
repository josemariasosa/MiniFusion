# CP1 — Step 05: Schema-Aware Validation

## Objective

Teach MiniFusion how to validate expressions and logical plans against a dataset schema.

Until now, our engine has been able to describe queries, but it has no idea whether they make sense.

For example:

```rust
let plan = LogicalPlan::scan("trips")
    .filter(col("unknown").gt(lit(Value::Int64(5))));
```

This is a perfectly valid Rust structure.

But is it a valid query?

We cannot know until we inspect the schema of `trips`.

In this checkpoint, we will introduce a small validation layer that can answer that question without reading any actual data.

**Our goal: transform an unchecked query description into a validated query plan.**

We will proceed incrementally, introducing one responsibility at a time.

---

## 1. Understanding the Problem

Consider our existing table schema:

```text
trips
--------------------------------
trip_id        Int64
zone           Utf8
distance_km    Float64
fare           Float64
```

And the following expressions:

```text
distance_km > 5
fare > 100
zone = "A"
unknown > 10
fare AND zone
```

Some are valid, others are not.

Our validation layer should eventually determine:

| Expression | Expected result |
|---|---|
| `distance_km > 5` | Boolean |
| `fare > 100` | Boolean |
| `zone = "A"` | Boolean |
| `unknown > 10` | Error: unknown column |
| `fare AND zone` | Error: incompatible types |

For now, we will support a deliberately small type system.

### Our initial rules

- `Int64` compared with `Int64` is valid.
- `Float64` compared with `Float64` is valid.
- `Utf8` compared with `Utf8` is valid for equality.
- `Boolean` compared with `Boolean` is valid for equality.
- `Gt` and `Lt` require numeric operands of the same type.
- `And` requires two Boolean operands.
- Comparisons and `And` produce `Boolean`.
- Mixed numeric types are not automatically converted.

That last restriction is important.

For this checkpoint, `Float64 > Int64` will be rejected, even though real query engines commonly support numeric coercion.

We'll introduce coercion as a separate problem later.

This means our earlier example:

```rust
col("distance_km").gt(lit(Value::Int64(5)))
```

must now use a `Float64` literal:

```rust
col("distance_km").gt(lit(Value::Float64(5.0)))
```

This is intentional.

We want to understand type checking before introducing implicit conversions.

---

## 2. Step A — Identifying Literal Types

We already have two related enums:

```rust
pub enum Value {
    Int64(i64),
    Float64(f64),
    Utf8(String),
    Boolean(bool),
    Null,
}
```

And:

```rust
pub enum DataType {
    Int64,
    Float64,
    Utf8,
    Boolean,
}
```

Our first task is to connect them.

### Exercise A1

In `src/value.rs`, implement:

```rust
impl Value {
    pub fn data_type(&self) -> Option<DataType> {
        todo!()
    }
}
```

Import `DataType` from your schema module.

Why `Option<DataType>`?

Because `Value::Null` has no concrete type by itself in our current model.

For example:

```rust
Value::Int64(42).data_type()
```

Should return:

```rust
Some(DataType::Int64)
```

But:

```rust
Value::Null.data_type()
```

Should return:

```rust
None
```

We will not implement typed nulls in this checkpoint.

### Tests

Write these tests:

- `literal_int64_type`
- `literal_float64_type`
- `literal_utf8_type`
- `literal_boolean_type`
- `literal_null_has_no_type`

**Stop here before continuing.**

At this point, our engine knows how to determine the type of a literal without evaluating any expression.

---

## 3. Step B — Resolving Column Types

We already have:

```rust
pub struct Schema {
    pub fields: Vec<Field>,
}
```

And:

```rust
pub fn field(&self, name: &str) -> Option<&Field>
```

We can reuse this method to look up a column.

### Exercise B1

Create:

```text
src/validation.rs
```

For now, introduce a small error enum:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    UnknownColumn(String),
    UntypedNull,
    TypeMismatch,
    InvalidOperator,
}
```

We're keeping errors deliberately simple.

Later, we'll add better diagnostic information.

### Exercise B2

Implement:

```rust
pub fn resolve_column_type(
    name: &str,
    schema: &Schema,
) -> Result<DataType, ValidationError> {
    todo!()
}
```

Expected behavior:

```rust
resolve_column_type("fare", &schema)
```

Returns:

```rust
Ok(DataType::Float64)
```

And:

```rust
resolve_column_type("unknown", &schema)
```

Returns:

```rust
Err(ValidationError::UnknownColumn(
    "unknown".to_string()
))
```

### Rust concepts

Pay attention to:

- Borrowing a `Schema`
- Returning owned values
- `Option` versus `Result`
- Converting missing lookups into domain-specific errors

One useful method to explore is:

```rust
Option::ok_or_else()
```

### Tests

- `resolve_existing_column`
- `reject_unknown_column`
- `resolve_empty_schema`

**Stop here.**

We can now resolve column names without reading table rows.

---

## 4. Step C — Expression Type Inference

This is the central exercise.

Remember our expression model:

```rust
pub enum Expr {
    Column(String),
    Literal(Value),
    Binary {
        left: Box<Expr>,
        op: Operator,
        right: Box<Expr>,
    },
}
```

An expression can contain other expressions.

Therefore, type inference must be recursive.

### Exercise C1

Implement:

```rust
pub fn infer_expr_type(
    expr: &Expr,
    schema: &Schema,
) -> Result<DataType, ValidationError> {
    todo!()
}
```

Begin with only two variants:

```rust
match expr {
    Expr::Column(name) => {
        // Resolve the column type.
    }

    Expr::Literal(value) => {
        // Determine the literal type.
    }

    Expr::Binary { .. } => {
        todo!("Binary expressions come next")
    }
}
```

The first two cases should reuse the functionality implemented in Steps A and B.

Do not duplicate that logic.

### Tests

Before implementing Binary expressions, verify:

```rust
infer_expr_type(&col("fare"), &schema)
```

Returns `Float64`.

And:

```rust
infer_expr_type(&lit(Value::Int64(10)), &schema)
```

Returns `Int64`.

Also verify that unknown columns and untyped null literals are rejected.

---

## 5. Step D — Recursive Binary Type Inference

Now things get interesting.

Consider:

```text
fare > 100.0
```

The expression tree is:

```text
       Gt
      /  \
   fare  100.0
```

To determine the type of the complete expression, we need to know the types of both children.

That suggests a recursive approach.

### Exercise D1

Inside the Binary branch of `infer_expr_type`, recursively determine:

```rust
let left_type = infer_expr_type(left, schema)?;
let right_type = infer_expr_type(right, schema)?;
```

These calls walk down the expression tree until they reach columns or literals.

Now introduce a helper:

```rust
fn infer_binary_type(
    op: &Operator,
    left: &DataType,
    right: &DataType,
) -> Result<DataType, ValidationError> {
    todo!()
}
```

Its only responsibility is to validate an operator and the types of its operands.

It should not inspect expressions, schemas, or actual values.

### Exercise D2 — Implement operators

Implement the type rules defined in Section 1.

Start with `And`, because its rule is simple:

```text
Boolean AND Boolean → Boolean
```

Then implement:

```text
Int64 > Int64     → Boolean
Float64 > Float64 → Boolean
Int64 < Int64     → Boolean
Float64 < Float64 → Boolean
```

Finally, implement equality for matching types:

```text
Int64 = Int64     → Boolean
Float64 = Float64 → Boolean
Utf8 = Utf8       → Boolean
Boolean = Boolean → Boolean
```

Any unsupported combination must produce an error.

Use `TypeMismatch` when the operand types are incompatible and `InvalidOperator` when the operand types match but the operator is unsupported for that type.

### Tests

- `infer_numeric_comparison`
- `infer_string_equality`
- `infer_boolean_and`
- `reject_string_greater_than_number`
- `reject_numeric_and`
- `reject_mixed_numeric_types`
- `reject_untyped_null`
- `infer_nested_boolean_expression`

### Important observation

For the expression:

```text
(fare > 100.0) AND (zone = "A")
```

The engine can infer:

```text
fare                  → Float64
100.0                 → Float64
fare > 100.0          → Boolean

zone                  → Utf8
"A"                   → Utf8
zone = "A"            → Boolean

Boolean AND Boolean   → Boolean
```

Not one row has been evaluated.

**This is static analysis of the query.**

---

## 6. Step E — Validating a Filter

Now we're ready to connect expression validation to logical plans.

Recall:

```rust
pub enum LogicalPlan {
    Scan {
        table: String,
    },
    Filter {
        predicate: Expr,
        input: Box<LogicalPlan>,
    },
    Projection {
        columns: Vec<String>,
        input: Box<LogicalPlan>,
    },
}
```

A Filter requires a predicate that returns Boolean.

This means:

```rust
.filter(col("fare").gt(lit(Value::Float64(100.0))))
```

Is valid.

But:

```rust
.filter(col("fare"))
```

Should fail validation.

Why?

Because `fare` is `Float64`, not `Boolean`.

### Exercise E1

Extend `ValidationError`:

```rust
pub enum ValidationError {
    UnknownColumn(String),
    UntypedNull,
    TypeMismatch,
    InvalidOperator,
    ExpectedBoolean,
}
```

Implement:

```rust
pub fn validate_filter(
    predicate: &Expr,
    schema: &Schema,
) -> Result<(), ValidationError> {
    todo!()
}
```

This function should:

1. Infer the predicate's type.
2. Check whether the result is Boolean.
3. Return `Ok(())` if valid.
4. Return an appropriate error otherwise.

### Tests

- `accept_boolean_filter`
- `reject_numeric_filter`
- `reject_unknown_filter_column`
- `reject_invalid_filter_expression`

**Stop here.**

We have now connected expression type inference to a relational operator.

---

## 7. Step F — Validating Projection

Projection selects columns from the input schema.

For example:

```rust
.project(vec!["zone".to_string(), "fare".to_string()])
```

Given:

```text
trip_id       Int64
zone          Utf8
distance_km   Float64
fare          Float64
```

The output schema should be:

```text
zone          Utf8
fare          Float64
```

Notice that Projection changes the schema.

Filter, on the other hand, preserves its input schema.

### Exercise F1

Implement:

```rust
pub fn project_schema(
    columns: &[String],
    input_schema: &Schema,
) -> Result<Schema, ValidationError> {
    todo!()
}
```

The function should:

1. Look up each requested column.
2. Return an error if any column is missing.
3. Preserve the requested column order.
4. Construct a new Schema containing the selected fields.

For this exercise, allow an empty projection and repeated column names. We will revisit whether to restrict those cases later.

### Tests

- `project_existing_columns`
- `project_preserves_order`
- `reject_unknown_projection_column`
- `project_empty_column_list`

### Design question

Why should Projection return a new `Schema`, while Filter can reuse its input schema?

Think about how relational operations transform data structure, not just data values.

---

## 8. Step G — Validating a Complete Logical Plan

We now have enough building blocks to validate a complete query tree.

But there is one missing piece.

Consider:

```rust
LogicalPlan::scan("trips")
```

How does the validator discover the schema of `trips`?

We need a tiny table catalog.

### Exercise G1 — A Minimal Catalog

For this checkpoint, use a standard Rust collection:

```rust
use std::collections::HashMap;

pub struct Catalog {
    tables: HashMap<String, Schema>,
}
```

Implement:

```rust
impl Catalog {
    pub fn new() -> Self {
        todo!()
    }

    pub fn register(
        &mut self,
        table: String,
        schema: Schema,
    ) {
        todo!()
    }

    pub fn schema(&self, table: &str) -> Option<&Schema> {
        todo!()
    }
}
```

This catalog contains only table names and schemas.

It does not store rows.

You can keep it in `src/validation.rs` for this exercise. We can extract it into its own module if it grows.

### Exercise G2 — Validate the Tree

Extend `ValidationError` with:

```rust
UnknownTable(String),
```

Then implement:

```rust
pub fn validate_plan(
    plan: &LogicalPlan,
    catalog: &Catalog,
) -> Result<Schema, ValidationError> {
    todo!()
}
```

This function should be recursive.

Its responsibility is to determine the output schema of every logical operator.

### Scan

Look up the table in the catalog.

Return its schema or an `UnknownTable` error.

### Filter

1. Validate the input logical plan.
2. Retrieve its output schema.
3. Validate the predicate against that schema.
4. Return the unchanged input schema.

### Projection

1. Validate the input logical plan.
2. Retrieve its output schema.
3. Validate the requested columns.
4. Return the projected schema.

### Expected usage

```rust
let mut catalog = Catalog::new();

catalog.register("trips".to_string(), trips_schema());

let plan = LogicalPlan::scan("trips")
    .filter(
        col("distance_km")
            .gt(lit(Value::Float64(5.0)))
    )
    .project(vec![
        "zone".to_string(),
        "fare".to_string(),
    ]);

let result_schema = validate_plan(&plan, &catalog)?;
```

The expected result schema is:

```text
zone    Utf8
fare    Float64
```

### Tests

- `validate_complete_plan`
- `reject_unknown_table`
- `reject_unknown_filter_column`
- `reject_unknown_projection_column`
- `reject_non_boolean_filter`
- `validate_nested_filters`
- `reject_filter_after_column_removed`

The final test is especially important.

Consider:

```text
Filter: fare > 100.0
  Projection: [zone]
    Scan: trips
```

This plan should fail validation because `fare` is no longer available after Projection.

It demonstrates that **operator ordering affects schema availability**.

---

## 9. What We Are Not Building Yet

Even though this exercise introduces validation, we are intentionally avoiding several advanced concepts.

### No numeric coercion

We reject mixed numeric expressions such as:

```text
Float64 > Int64
```

Real query engines often insert casts or apply coercion rules.

That is future work.

### No SQL null semantics

Although `Value` contains `Null`, our inference model cannot yet assign an untyped null a concrete data type.

We reject it during type inference.

Typed nulls, nullable fields, and SQL three-valued logic will be separate exercises.

### No physical execution

Validation never reads rows.

The catalog only stores schemas.

### No optimizer

We inspect logical plans but do not rewrite them.

### No separate QueryBuilder yet

Our existing `LogicalPlan` constructors are sufficient for constructing query trees.

We will only introduce a separate builder if it provides a meaningful advantage.

---

## 10. Final Integration Challenge

Construct these two plans using the same catalog.

### Query A — Valid

```sql
SELECT zone, fare
FROM trips
WHERE distance_km > 5.0;
```

Expected output schema:

```text
zone    Utf8
fare    Float64
```

### Query B — Invalid

```sql
SELECT zone
FROM trips
WHERE unknown > 5;
```

Expected outcome:

```text
ValidationError::UnknownColumn("unknown")
```

Your tests should prove that validation succeeds or fails appropriately without accessing any actual data.

You may also check that validation does not modify the original logical plan.

---

## 11. Questions to Think About

1. Why can we infer an expression's type without processing rows?
2. Why does `validate_plan()` return a Schema instead of just `Result<(), ValidationError>`?
3. Why do we validate the input operator before validating its parent?
4. Why is it useful to separate `infer_expr_type()` from `validate_filter()`?
5. What would need to change if Projection supported expressions such as `fare * 2`?

For the fifth question, consider the relationship between an expression's resulting type and the output schema of a projection.

---

## 12. Definition of Done

### Values and columns

- [ ] `Value::data_type()` is implemented.
- [ ] Column types resolve through `Schema`.
- [ ] Unknown columns produce errors.

### Expressions

- [ ] Literal type inference works.
- [ ] Recursive binary type inference works.
- [ ] Operator type rules are enforced.
- [ ] Invalid combinations are rejected.

### Logical operators

- [ ] Filter requires a Boolean predicate.
- [ ] Projection produces a new output schema.
- [ ] Scan resolves its schema through a catalog.

### Complete plans

- [ ] `validate_plan()` recursively validates logical plans.
- [ ] Every node produces the appropriate output schema.
- [ ] Operator ordering is respected.
- [ ] Unknown tables and columns are rejected.
- [ ] No rows are loaded.
- [ ] No external dependencies are introduced.

Run:

```bash
cargo fmt
cargo test
cargo clippy --all-targets -- -D warnings
```

---

## 13. What's Next?

**CP1 — Step 06: Final Integration and Architecture Review**

Before moving to CP2 (Physical Planning), we'll inspect our implementation as a whole.

We will review:

- Module boundaries
- Ownership and borrowing
- Expression trees and logical plan trees
- Error propagation
- Schema invariants
- Public API design
- Test coverage

The objective will be to identify whether our abstractions are ready for a physical planner.

We'll also compare our MiniFusion concepts to their equivalents in Apache DataFusion.

After that, we'll be ready to begin CP2:

```text
Validated Logical Plan
          |
          v
    Physical Planner
          |
          v
     Physical Plan
```

And eventually, our plans will start processing real data.

**Remember: the query is still only an intention. We're learning how to verify that the intention makes sense before turning it into executable work.**
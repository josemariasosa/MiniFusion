# CP1 — Step 03: Expressions

## Objective

Build a recursive expression tree capable of representing column references, literal values, and binary operations.

By the end of this step, MiniFusion should be able to represent an expression such as:

```sql
distance_km > 5
```

Without evaluating it, accessing data, or executing a query.

This is our first introduction to one of the most important abstractions in a query engine: **the expression tree**.

## 1. Background: What is an Expression?

Consider this SQL query:

```sql
SELECT zone, fare
FROM trips
WHERE distance_km > 5 AND fare < 200;
```

The `WHERE` clause contains an expression:

```text
distance_km > 5 AND fare < 200
```

This expression consists of smaller expressions:

- `distance_km`: a column reference.
- `5`: a literal value.
- `distance_km > 5`: a binary expression.
- `fare < 200`: another binary expression.
- `AND`: a logical operator combining two expressions.

We can represent the entire expression as a tree:

```text
             AND
            /   \
           >     <
          / \   / \
  distance  5 fare 200
     _km
```

Each node represents an expression, and an expression may contain other expressions.

This is why recursive data structures are so important in query engines.

### Expressions vs. Values

We already implemented `Value` in Step 01.

A `Value` represents concrete data:

```rust
Value::Int64(5)
Value::Utf8("A".to_string())
```

An `Expr`, however, describes a computation that may produce a value.

For example:

```rust
Expr::Column("distance_km".to_string())
```

This does not contain the actual distance of any particular trip.

It represents a reference to a column whose value will eventually be resolved during query execution.

**An expression describes computation. A value represents data.**

## 2. Our Expression Model

Create a new module:

```text
src/expr.rs
```

Start with the following enums:

```rust
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
```

Expose the module in `main.rs`:

```rust
mod expr;
```

### Why `Box<Expr>`?

Rust needs to know the size of every type at compile time.

Consider:

```rust
enum Expr {
    Binary {
        left: Expr,
        right: Expr,
    },
}
```

This definition cannot compile because `Expr` would contain another `Expr` directly, creating an infinitely sized recursive type.

`Box<Expr>` solves this problem by introducing indirection.

A `Box<T>` occupies a known amount of space because it holds a pointer to heap-allocated data.

This allows expressions to contain arbitrarily nested expressions.

Understanding recursive enums is essential when working with query planners, compilers, and interpreters.

## 3. Implementation Task A — Expression Constructors

Writing nested enum variants manually can become cumbersome.

For example:

```rust
let expr = Expr::Binary {
    left: Box::new(Expr::Column("distance_km".to_string())),
    op: Operator::Gt,
    right: Box::new(Expr::Literal(Value::Int64(5))),
};
```

We want a cleaner API.

Implement these two helper functions:

```rust
pub fn col(name: &str) -> Expr {
    todo!()
}

pub fn lit(value: Value) -> Expr {
    todo!()
}
```

They should allow us to write:

```rust
let column = col("distance_km");
let literal = lit(Value::Int64(5));
```

These constructors do not validate schemas or evaluate anything.

Their only responsibility is to construct expressions.

## 4. Implementation Task B — Binary Expressions

Now implement a method to combine two expressions using an operator.

```rust
impl Expr {
    pub fn binary(self, op: Operator, right: Expr) -> Expr {
        todo!()
    }
}
```

The method takes ownership of `self` and `right`.

This is intentional.

Expression trees own their child expressions, and we do not need to clone them just to construct a larger tree.

After implementing the method, this should work:

```rust
let expr = col("distance_km")
    .binary(Operator::Gt, lit(Value::Int64(5)));
```

Notice that the method does not execute the comparison.

It constructs a tree representing the comparison.

## 5. Implementation Task C — Fluent Expression API

Let's make our API more convenient.

We want to write:

```rust
let expr = col("distance_km").gt(lit(Value::Int64(5)));
```

Instead of:

```rust
let expr = col("distance_km")
    .binary(Operator::Gt, lit(Value::Int64(5)));
```

Implement these methods:

```rust
impl Expr {
    pub fn eq(self, right: Expr) -> Expr {
        todo!()
    }

    pub fn gt(self, right: Expr) -> Expr {
        todo!()
    }

    pub fn lt(self, right: Expr) -> Expr {
        todo!()
    }

    pub fn and(self, right: Expr) -> Expr {
        todo!()
    }
}
```

Each method should delegate to the `binary` method implemented in Task B.

Do not duplicate tree-construction logic.

### Expected Usage

We should now be able to express:

```sql
distance_km > 5 AND fare < 200
```

Using:

```rust
let expr = col("distance_km")
    .gt(lit(Value::Int64(5)))
    .and(
        col("fare").lt(lit(Value::Int64(200)))
    );
```

This produces a nested expression tree.

No data has been read.

No comparison has been performed.

We have only represented the query's intended computation.

## 6. Implementation Task D — Expression Display

Our next challenge is to make expressions observable.

Currently, Rust's `Debug` trait allows us to inspect an expression, but its representation is quite verbose.

We want the following:

```rust
println!("{expr}");
```

To produce:

```text
((distance_km > 5) AND (fare < 200))
```

Implement `std::fmt::Display` for both `Operator` and `Expr`.

Suggested skeleton:

```rust
use std::fmt;

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
```

You may also need to implement `Display` for `Value` in `value.rs`.

Keep the formatting straightforward:

- `Column`: print the column name.
- `Literal`: print its value.
- `Binary`: print both children and the operator inside parentheses.
- `Operator`: print its SQL-like symbol.

For now, we are not implementing a complete SQL formatter. Parentheses around every binary expression are sufficient to make tree structure and precedence explicit.

**Design constraint:** Formatting must not modify the expression tree.

## 7. Tests

Create a `#[cfg(test)]` module in `expr.rs`.

Write the following tests.

### Test 1 — Column Expression

Construct:

```rust
col("distance_km")
```

Verify that the result is:

```rust
Expr::Column("distance_km".to_string())
```

### Test 2 — Literal Expression

Construct:

```rust
lit(Value::Int64(42))
```

Verify that it contains the expected literal value.

### Test 3 — Binary Expression

Construct:

```rust
col("fare").gt(lit(Value::Int64(100)))
```

Verify that:

- The root is a `Binary` expression.
- The operator is `Gt`.
- The left child is a column reference.
- The right child is a literal.

### Test 4 — Nested Expressions

Construct:

```rust
col("distance_km")
    .gt(lit(Value::Int64(5)))
    .and(
        col("fare").lt(lit(Value::Int64(200)))
    )
```

Verify that the root operator is `And`, with two binary expressions as children.

### Test 5 — Expression Display

Verify that formatting the nested expression produces:

```text
((distance_km > 5) AND (fare < 200))
```

### Test 6 — Expression Construction Does Not Evaluate

Construct a comparison between two different types:

```rust
col("zone").gt(lit(Value::Int64(5)))
```

Verify that the expression can still be constructed.

Why?

Because our expression constructors are not responsible for type checking.

Later, a schema-aware validation phase will determine whether the expression is valid.

This is an important architectural separation.

## 8. Design Decisions

### Expressions are immutable descriptions

Our methods construct and return new expression trees.

They do not maintain mutable execution state.

This makes expressions easy to inspect, clone when necessary, and eventually rewrite through optimizer rules.

### Expression construction is separate from validation

We can construct an expression without knowing the dataset schema.

For example:

```rust
col("unknown_column").gt(lit(Value::Int64(10)))
```

The expression is structurally valid.

Whether the column exists is a different question.

Schema validation will be introduced when we connect expressions to logical plans.

### Evaluation belongs to a later stage

We are intentionally not implementing:

```rust
expr.evaluate(row)
```

Yet.

Our current responsibility is to describe computations, not execute them.

This distinction will become especially important when we separate logical planning from physical execution.

## 9. Questions to Think About

Before moving on, consider these questions:

1. Why does `Expr::Binary` contain two `Box<Expr>` values instead of storing `Value` directly?
2. Why can an expression reference a column that does not exist?
3. What is the difference between constructing `distance_km > 5` and actually evaluating it?
4. Why might an optimizer need to inspect and modify an expression tree?
5. What would happen if we wanted to support expressions such as `(fare + 10) * 2`?

You do not need to implement arithmetic operators yet.

The objective is to understand how the existing abstraction could accommodate them.

## 10. Definition of Done

- [ ] `src/expr.rs` exists.
- [ ] `Expr` and `Operator` are defined.
- [ ] `col()` and `lit()` constructors are implemented.
- [ ] `Expr::binary()` is implemented.
- [ ] Fluent methods `eq()`, `gt()`, `lt()`, and `and()` are implemented.
- [ ] Expressions can be nested recursively.
- [ ] `Display` is implemented for expressions and operators.
- [ ] All six tests pass.
- [ ] No external dependencies have been introduced.

Run:

```bash
cargo fmt
cargo test
cargo clippy --all-targets -- -D warnings
```

## 11. What's Next?

**CP1 — Step 04: Logical Plans**

Now that we can represent expressions, we can begin describing operations over entire datasets.

We will introduce:

```rust
enum LogicalPlan {
    Scan { /* ... */ },
    Filter { /* ... */ },
    Projection { /* ... */ },
}
```

We'll learn the difference between a scalar expression and a relational operator, and how an entire query can be represented as a tree.

Our first objective will be to construct:

```text
Projection: [zone, fare]
  Filter: distance_km > 5
    Scan: trips
```

At that point, MiniFusion will have its first complete logical query representation.

Still no execution.

Still no Arrow.

Just the fundamental abstractions that make query engines possible.
# CP1 — Step 04: Logical Plans

## Objective

Build a logical plan representation capable of describing relational operations over datasets.

By the end of this step, MiniFusion should be able to represent:

```sql
SELECT zone, fare
FROM trips
WHERE distance_km > 5;
```

As a tree of relational operations:

```text
Projection: [zone, fare]
    Filter: (distance_km > 5)
        Scan: trips
```

We will construct and inspect this tree without loading or processing any data.

**The goal is to understand how query engines describe transformations over datasets before deciding how to execute them.**

## 1. Background: Expressions vs. Logical Plans

In Step 03, we introduced `Expr`.

An expression describes a computation that eventually produces a value.

For example:

```rust
let predicate = col("distance_km").gt(lit(Value::Int64(5)));
```

This expression represents:

```sql
distance_km > 5
```

But it does not tell us which dataset contains `distance_km`, which rows should be examined, or what should happen with the result.

We need another abstraction.

### Relational operators

A relational operator describes an operation over a dataset.

For our initial engine, we will support three operators:

| Operator | Responsibility |
|---|---|
| Scan | Reference a dataset |
| Filter | Retain rows satisfying a predicate |
| Projection | Select columns from a dataset |

These operators compose into a logical plan.

For example:

```text
Projection
    |
  Filter
    |
   Scan
```

A logical plan describes **what operations should be performed**, not the algorithm used to perform them.

### Scalar expressions vs. relational operations

Consider:

```sql
SELECT zone, fare
FROM trips
WHERE distance_km > 5;
```

The comparison:

```text
distance_km > 5
```

Is a scalar expression.

The `Filter` operator applies that expression to a dataset.

The `Projection` operator determines which columns appear in the output.

This separation allows the same expression model to be reused in different relational operations.

## 2. Understanding Logical Plan Trees

Our first logical plan contains three nodes.

```text
       Projection
      [zone, fare]
           |
         Filter
    distance_km > 5
           |
          Scan
         trips
```

Each operator has a specific responsibility.

### Scan

Represents a reference to a table or dataset.

```text
Scan: trips
```

At this stage, Scan does not load anything.

It only identifies the source.

### Filter

Represents a row-filtering operation.

```text
Filter: distance_km > 5
```

A Filter needs two things:

- An input logical plan.
- A predicate expression.

Its input could be a Scan, another Filter, or any supported logical plan.

### Projection

Represents the columns we want in the output.

```text
Projection: [zone, fare]
```

A Projection also requires an input logical plan.

We will initially limit Projection to selecting columns by name. Support for computed columns, aliases, and arbitrary projection expressions will come later.

### Important observation

The logical plan is a tree, just like our expression trees.

However, they represent different things:

- `Expr` represents computations over values.
- `LogicalPlan` represents transformations over datasets.

An expression tree can live inside a logical plan node.

## 3. Implementation Task A — Define LogicalPlan

Create:

```text
src/logical_plan.rs
```

Define:

```rust
use crate::expr::Expr;

#[derive(Debug, Clone, PartialEq)]
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

Expose the module from your crate root:

```rust
mod logical_plan;
```

### Why does Filter contain an Expr?

Because a filter needs a predicate to decide which rows should be retained.

However, that predicate is still only a description of computation.

It has not been evaluated.

### Why does Filter contain a LogicalPlan?

Because a filter operates on the result of another relational operation.

For example:

```text
Filter
    |
  Scan
```

But we could also have:

```text
Filter
    |
  Filter
    |
  Scan
```

This represents two consecutive filtering operations.

Our recursive structure supports both without changing the enum.

### Why Box again?

For the same reason we introduced `Box<Expr>` in Step 03.

A recursive enum must use indirection to have a finite, known size.

Additionally, `Box<LogicalPlan>` allows each parent node to own its input subtree.

## 4. Implementation Task B — Constructors

Writing nested logical plans manually is verbose.

For example:

```rust
let plan = LogicalPlan::Filter {
    predicate: col("distance_km").gt(lit(Value::Int64(5))),
    input: Box::new(LogicalPlan::Scan {
        table: "trips".to_string(),
    }),
};
```

We want a simpler interface.

Implement these constructors:

```rust
impl LogicalPlan {
    pub fn scan(table: &str) -> Self {
        todo!()
    }

    pub fn filter(self, predicate: Expr) -> Self {
        todo!()
    }

    pub fn project(self, columns: Vec<String>) -> Self {
        todo!()
    }
}
```

### Expected usage

After implementing the constructors, we should be able to write:

```rust
let plan = LogicalPlan::scan("trips")
    .filter(col("distance_km").gt(lit(Value::Int64(5))))
    .project(vec![
        "zone".to_string(),
        "fare".to_string(),
    ]);
```

Notice the ownership model.

The `filter` and `project` methods consume `self`.

Each new operator becomes the root of a new logical plan while taking ownership of the previous plan as its input.

For example:

```rust
let scan = LogicalPlan::scan("trips");
```

Produces:

```text
Scan
```

Then:

```rust
let filtered = scan.filter(predicate);
```

Produces:

```text
Filter
    |
  Scan
```

Finally:

```rust
let projected = filtered.project(columns);
```

Produces:

```text
Projection
    |
  Filter
    |
  Scan
```

**We build the plan from the data source outward, even though the completed tree is normally displayed from the root downward.**

## 5. Implementation Task C — Display Logical Plans

In Step 03, we implemented `Display` for expressions.

Now we want to display an entire logical plan.

Expected output:

```text
Projection: [zone, fare]
  Filter: (distance_km > 5)
    Scan: trips
```

Unlike expressions, logical plans should be printed with indentation.

Each level of the tree adds two spaces.

### Recommended approach

Implement a recursive formatting helper:

```rust
use std::fmt;

impl LogicalPlan {
    fn fmt_with_indent(
        &self,
        f: &mut fmt::Formatter<'_>,
        indent: usize,
    ) -> fmt::Result {
        todo!()
    }
}
```

Then implement `Display`:

```rust
impl fmt::Display for LogicalPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_indent(f, 0)
    }
}
```

The helper should:

1. Print indentation according to the current depth.
2. Print the current node.
3. Recursively print its input with an increased indentation.

### Rust hints

You may find these useful:

```rust
write!(f, "{}", " ".repeat(indent))
```

And:

```rust
writeln!(f, "Scan: {table}")
```

Remember that `writeln!` returns `fmt::Result`.

Also, use `?` to propagate formatting errors when writing multiple lines.

### Design constraint

Display should only inspect the logical plan.

It must not modify, validate, optimize, or execute anything.

## 6. Implementation Task D — Inspecting Plan Structure

One reason query engines represent queries as trees is that other components need to inspect the structure.

Eventually, our optimizer will traverse these trees and transform them.

Before introducing optimization, let's practice inspecting them.

Consider:

```rust
let plan = LogicalPlan::scan("trips")
    .filter(col("fare").gt(lit(Value::Int64(100))))
    .project(vec!["zone".to_string()]);
```

Write a test that inspects this plan structurally.

It should confirm:

- The root is a Projection.
- Projection contains the `zone` column.
- Its input is a Filter.
- The Filter predicate is `fare > 100`.
- The Filter input is a Scan.
- The Scan references the `trips` table.

You can use `match`, `matches!`, and assertions.

### Challenge: Borrow without consuming

Try inspecting the plan using:

```rust
match &plan {
    // ...
}
```

Rather than:

```rust
match plan {
    // ...
}
```

Why?

Because we want to inspect the tree without moving its owned contents.

This will be useful when our optimizer needs to analyze a plan before deciding whether to transform it.

## 7. Tests

Create a `#[cfg(test)]` module in `logical_plan.rs`.

Implement the following tests.

### Test 1 — Scan Construction

Construct:

```rust
LogicalPlan::scan("trips")
```

Verify:

- The root is Scan.
- The table name is `trips`.

### Test 2 — Filter Construction

Construct:

```rust
LogicalPlan::scan("trips")
    .filter(col("fare").gt(lit(Value::Int64(100))))
```

Verify:

- The root is Filter.
- The predicate is the expected expression.
- The input is Scan.

### Test 3 — Projection Construction

Construct:

```rust
LogicalPlan::scan("trips")
    .project(vec!["zone".to_string(), "fare".to_string()])
```

Verify:

- The root is Projection.
- Columns preserve their declared order.
- The input is Scan.

### Test 4 — Complete Logical Plan

Construct the full query:

```rust
let plan = LogicalPlan::scan("trips")
    .filter(col("distance_km").gt(lit(Value::Int64(5))))
    .project(vec![
        "zone".to_string(),
        "fare".to_string(),
    ]);
```

Verify the full tree structure:

```text
Projection
    Filter
        Scan
```

Also verify the relevant columns, predicate, and table name.

### Test 5 — Logical Plan Display

Verify that formatting the plan produces:

```text
Projection: [zone, fare]
  Filter: (distance_km > 5)
    Scan: trips
```

Choose a consistent convention for the final trailing newline and test it explicitly.

### Test 6 — Nested Filters

Construct:

```rust
LogicalPlan::scan("trips")
    .filter(col("distance_km").gt(lit(Value::Int64(5))))
    .filter(col("fare").lt(lit(Value::Int64(200))))
```

Verify that both Filter nodes exist and preserve their construction order.

Expected tree:

```text
Filter: (fare < 200)
  Filter: (distance_km > 5)
    Scan: trips
```

### Test 7 — Construction Does Not Validate

Construct a plan referencing a nonexistent column:

```rust
LogicalPlan::scan("trips")
    .filter(col("unknown_column").gt(lit(Value::Int64(10))))
```

Verify that construction succeeds.

This is intentional for this exercise: our constructors currently build syntax trees, not validated plans.

Later, we will use our schema abstraction to determine whether the query makes sense.

### Test 8 — Construction Does Not Execute

Construct a logical plan referencing a table that has not been registered or loaded:

```rust
LogicalPlan::scan("imaginary_table")
    .project(vec!["missing_column".to_string()])
```

Verify that the plan can be constructed and inspected.

No table lookup should occur.

The builder is only capturing a query description.

## 8. Important Architectural Distinction

Let's examine the responsibilities we have established.

### Expression construction

```rust
col("fare").gt(lit(Value::Int64(100)))
```

Produces an expression tree.

### Logical plan construction

```rust
LogicalPlan::scan("trips")
    .filter(predicate)
```

Produces a relational operation tree.

### Schema validation

Not implemented yet.

Eventually, the validator will determine:

- Whether the table exists.
- Whether referenced columns exist.
- Whether expression operand types are compatible.
- Whether a filter predicate has a Boolean result.
- What columns and types each logical operator produces.

### Optimization

Not implemented yet.

The optimizer will transform logical plans while preserving their meaning.

### Execution

Not implemented yet.

The executor will process actual data using physical operators.

**Each abstraction should have one clearly defined responsibility.**

## 9. A Subtle but Important Concept: Operator Ordering

Consider these two logical plans.

### Plan A

```text
Projection: [zone]
  Filter: (fare > 100)
    Scan: trips
```

### Plan B

```text
Filter: (fare > 100)
  Projection: [zone]
    Scan: trips
```

Are these equivalent?

Not necessarily.

In Plan A, the Filter can access the `fare` column before Projection removes it.

In Plan B, Projection removes `fare` before Filter attempts to use it.

The second plan would be invalid under the usual relational semantics.

This is our first glimpse into why query optimization is more complicated than simply rearranging operators.

**An optimizer must preserve both query semantics and schema requirements.**

We will revisit this problem in CP4 when implementing optimizer rules.

## 10. Questions to Think About

Before moving on, consider:

1. Why does Filter contain an `Expr`, while Projection currently contains a `Vec<String>`?
2. Why do Filter and Projection each contain an input `LogicalPlan`, while Scan does not?
3. Could two consecutive Filter nodes be combined into one Filter with an `AND` expression?
4. Why does the order of relational operators matter?
5. How would you represent a query involving two different tables?

For question 5, think about whether a logical operator always has exactly one input.

You do not need to implement joins yet.

## 11. Definition of Done

- [ ] `src/logical_plan.rs` exists.
- [ ] `LogicalPlan` supports Scan, Filter, and Projection.
- [ ] All three constructors are implemented.
- [ ] Logical plans can be nested recursively.
- [ ] Constructors preserve ownership without unnecessary clones.
- [ ] `Display` produces an indented tree.
- [ ] Logical plans can be inspected without consuming them.
- [ ] All eight tests pass.
- [ ] No data is loaded or processed.
- [ ] No schema validation or query optimization is performed.
- [ ] No external dependencies are required.

Run:

```bash
cargo fmt
cargo test
cargo clippy --all-targets -- -D warnings
```

## 12. What's Next?

**CP1 — Step 05: Query Builder and Schema-Aware Validation**

We now have two fundamental abstractions:

```text
Expr
  └── Describes computations

LogicalPlan
  └── Describes relational operations
```

The next step will connect these abstractions to our previously implemented `Schema`.

We'll explore:

- Resolving column references against schemas
- Type inference for expressions
- Validating Filter predicates
- Determining output schemas for relational operators
- Reporting invalid queries before execution

We will also consider whether `QueryBuilder` should be a separate abstraction or whether our existing `LogicalPlan` constructors already provide everything we need.

Our intended pipeline is gradually taking shape:

```text
Query Construction       [In progress]
        |
        v
Schema-Aware Validation  [Next]
        |
        v
Logical Optimization     [Future]
        |
        v
Physical Planning        [Future]
        |
        v
Execution                [Future]
```

Once CP1 is complete, MiniFusion will be able to represent and validate complete queries without reading a single data row.
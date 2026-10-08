# MiniFusion

**A miniature query engine written in Rust to understand the internals of Apache DataFusion.**

MiniFusion is an educational project focused on learning how modern analytical query engines represent, optimize, and execute queries.

The goal is **not to build a production-ready database or recreate Apache DataFusion**. Instead, we will implement a simplified query engine from scratch, progressively introducing the architectural concepts used by real query engines.

Think of this repository as the *shallow end of the pool* before diving into [Apache DataFusion](https://datafusion.apache.org/).

## 1. Motivation

Apache DataFusion is a powerful, extensible query engine written in Rust, built around Apache Arrow's columnar memory model.

However, understanding its internals requires familiarity with several concepts:

- Expression trees and logical query plans
- Logical and physical planning
- Query optimization and tree rewriting
- Physical execution operators
- Iterator-based and streaming execution
- Columnar data representation
- Apache Arrow and `RecordBatch`
- Schema validation and type resolution

Rather than learning all these concepts simultaneously by navigating a large codebase, MiniFusion will introduce them incrementally.

**The ultimate objective is to develop enough architectural understanding to navigate DataFusion's source code and begin making meaningful contributions.**

## 2. Learning Philosophy

This repository follows a few principles:

1. **Understand before implementing.** Every abstraction should solve a problem we have already identified.
2. **Start simple, then evolve.** Begin with an in-memory, row-oriented engine before introducing columnar execution.
3. **Separate responsibilities.** Query construction, optimization, planning, and execution should have distinct boundaries.
4. **Make everything observable.** We should be able to inspect logical plans, optimized plans, physical plans, and execution results.
5. **Test behavior and invariants.** Each checkpoint should include unit or integration tests.
6. **Avoid unnecessary complexity.** We will not introduce features merely because DataFusion has them.
7. **Learn by building.** Implementations should be developed incrementally, with attention to the reasoning behind each design decision.

The objective is not simply to make queries work. It is to understand *why* query engines are designed the way they are.

## 3. Target Architecture

MiniFusion will gradually implement the following pipeline:

```text
             Query Builder
                   |
                   v
              Logical Plan
                   |
                   v
            Logical Optimizer
                   |
                   v
             Physical Planner
                   |
                   v
              Physical Plan
                   |
                   v
             Query Executor
                   |
                   v
                 Results
```

Each stage has a distinct responsibility.

| Component | Responsibility |
|---|---|
| Query Builder | Construct queries through a Rust API |
| Logical Plan | Represent what the query should compute |
| Logical Optimizer | Rewrite a logical plan while preserving its meaning |
| Physical Planner | Translate logical operations into executable operators |
| Physical Plan | Describe the concrete execution strategy |
| Query Executor | Execute operators and produce results |

A query plan is a **tree of operations**, not necessarily a sequential list of instructions.

The execution engine evaluates this tree to produce the requested results.

## 4. Example Query

Consider a small dataset:

| trip_id | zone | distance_km | fare |
|---|---|---|---|
| 1 | A | 5 | 100 |
| 2 | B | 12 | 250 |
| 3 | A | 2 | 40 |
| 4 | C | 8 | 180 |
| 5 | B | 15 | 300 |

We want the equivalent of:

```sql
SELECT zone, fare
FROM trips
WHERE distance_km > 5;
```

Instead of implementing SQL parsing, MiniFusion will expose a Rust query-building API.

Our intended API might look like:

```rust
let query = QueryBuilder::scan("trips")
    .filter(col("distance_km").gt(lit(5)))
    .select(["zone", "fare"])
    .build()?;
```

This produces a logical plan:

```text
Projection: [zone, fare]
  Filter: distance_km > 5
    Scan: trips
```

The query is then optimized, converted into a physical plan, and executed.

Expected result:

```text
+------+------+
| zone | fare |
+------+------+
| B    | 250  |
| C    | 180  |
| B    | 300  |
+------+------+
```

The API above is illustrative and will evolve as the project progresses.

## 5. Development Roadmap

The project is divided into six checkpoints.

Each checkpoint introduces a new concept while preserving the functionality implemented in previous stages.

### CP1 — Logical Plans and Expression Trees

**Goal:** Represent queries without executing them.

Implement the fundamental structures required to describe a query.

Core abstractions:

- `Value`
- `Expr`
- `Operator`
- `LogicalPlan`
- `QueryBuilder`
- Basic schemas and column resolution

Initial logical operators:

- `Scan`
- `Filter`
- `Projection`

Key concepts:

- Abstract syntax trees (ASTs)
- Recursive enums and `Box<T>`
- Expression trees
- Logical operators
- Schema resolution
- Query validation
- Separating query representation from execution

**Deliverable:** Construct and print a valid logical query plan using a Rust API.

**Completion criteria:**

- Queries can be constructed programmatically.
- Logical plans can be displayed as trees.
- Nested expressions are supported.
- Unknown columns and invalid expressions produce meaningful errors.
- Unit tests validate plan structure and basic semantics.

### CP2 — Physical Planning

**Goal:** Translate a logical plan into an executable representation.

Introduce a physical planning phase that maps logical operations to concrete execution operators.

Potential physical operators:

- `ScanExec`
- `FilterExec`
- `ProjectionExec`

Key concepts:

- Logical vs. physical plans
- Operator interfaces
- Planning transformations
- Schema preservation
- Separating declarative computation from execution strategy

**Deliverable:** Convert a logical plan into a physical plan and inspect both representations.

**Completion criteria:**

- Every supported logical operator has a physical equivalent.
- The physical planner recursively constructs executable operator trees.
- Physical plans can be inspected without execution.
- Unsupported operations produce explicit errors.
- Planning does not read or process dataset rows.

### CP3 — Pull-Based Query Execution

**Goal:** Execute physical plans using an iterator-based model.

Start with an in-memory, row-oriented executor.

Each physical operator requests rows from its child and processes them incrementally.

Conceptually:

```text
ProjectionExec
      |
      v
  FilterExec
      |
      v
   ScanExec
      |
      v
 In-memory data
```

Actual data flows upward through the operator tree.

Key concepts:

- Volcano iterator model
- Pull-based execution
- Lazy evaluation
- Operator composition
- Expression evaluation
- Memory usage and incremental processing
- Execution errors

**Deliverable:** Execute a query and return the correct rows.

**Completion criteria:**

- Scan produces rows from an in-memory dataset.
- Filter evaluates predicates.
- Projection selects the requested columns.
- Operators consume rows incrementally.
- Results match expected query semantics.
- Execution tests cover empty inputs, filtered results, and expression evaluation.

### CP4 — Logical Query Optimization

**Goal:** Understand how equivalent queries can be executed more efficiently.

Implement a simple rule-based logical optimizer.

Initial optimization rules:

- Predicate pushdown
- Projection pushdown / column pruning

For example, a filter can sometimes be moved closer to the underlying scan, reducing the amount of data processed by intermediate operators.

Key concepts:

- Query rewriting
- Recursive tree transformations
- Optimizer rules
- Predicate dependencies
- Semantic equivalence
- Operator ordering
- Basic execution metrics

**Deliverable:** Compare original and optimized plans and verify that they produce equivalent results.

**Completion criteria:**

- Optimization rules transform eligible logical plans.
- Optimizations preserve query results.
- Invalid transformations are avoided.
- Original and optimized plans can be inspected.
- Basic metrics demonstrate reduced processing where applicable.
- Tests verify both correctness and plan transformations.

### CP5 — Columnar and Batch Execution

**Goal:** Understand why analytical engines operate on columnar batches rather than individual rows.

Replace or extend the initial row-oriented executor with a batch-oriented execution model using Apache Arrow.

Introduce:

- Arrow arrays
- Arrow schemas
- `RecordBatch`
- Columnar filtering
- Columnar projection
- Batch-oriented execution operators

Key concepts:

- Row-oriented vs. column-oriented storage
- Vectorized computation
- Memory layout
- Batch processing
- Arrow data types
- Null handling
- Execution pipelines

**Deliverable:** Execute our existing queries using Arrow `RecordBatch`es.

**Completion criteria:**

- Input datasets can be represented as Arrow batches.
- Scan produces batches incrementally.
- Filter and Projection operate on columnar data.
- Query results remain equivalent to the row-oriented implementation.
- Tests cover multiple batches, empty batches, and null values.
- We can explain the architectural tradeoffs between row-based and columnar execution.

### CP6 — Bridging to Apache DataFusion

**Goal:** Connect what we built with the real DataFusion architecture.

Reimplement our sample queries using Apache DataFusion and trace the corresponding components through its source code.

Explore:

- `SessionContext`
- `LogicalPlan`
- `Expr`
- `ExecutionPlan`
- Logical optimizer rules
- Physical expressions
- Execution streams
- Arrow `RecordBatch`
- Custom `TableProvider` implementations

Key concepts:

- Real-world query planning interfaces
- Async execution and batch streams
- Partitioned execution
- Data source abstractions
- Extensibility
- Navigating a mature Rust codebase

**Deliverable:** Execute equivalent queries in MiniFusion and DataFusion, inspect the plans, and document how their architectures correspond.

**Completion criteria:**

- Reproduce a MiniFusion query using DataFusion.
- Inspect logical and physical plans.
- Identify the DataFusion equivalents of our major abstractions.
- Trace a query from planning through execution.
- Implement or experiment with a small DataFusion extension.
- Identify a realistic first contribution opportunity.

## 6. Concept Mapping

MiniFusion's abstractions are intentionally inspired by DataFusion.

| MiniFusion | Apache DataFusion |
|---|---|
| `QueryBuilder` | `LogicalPlanBuilder` / DataFrame API |
| `Expr` | `datafusion_expr::Expr` |
| `LogicalPlan` | `datafusion_expr::LogicalPlan` |
| Optimizer rules | `OptimizerRule` |
| Physical planner | DataFusion physical planning |
| `PhysicalPlan` | `ExecutionPlan` |
| Row iterator | Simplified execution model |
| Batch executor | `RecordBatch` streams |
| In-memory table | In-memory data sources / `TableProvider` |

These mappings are conceptual rather than guarantees of identical APIs.

MiniFusion will intentionally omit many complexities found in the production engine.

## 7. Scope and Non-Goals

MiniFusion is a learning repository.

We will focus on understanding query engine architecture through small, testable implementations.

**Initial scope:**

- In-memory data
- Basic scalar expressions
- Scans, filters, and projections
- Logical and physical planning
- Rule-based logical optimization
- Incremental execution
- Arrow batch processing

**Out of scope for the initial roadmap:**

- Full SQL parsing
- Persistent database storage
- Transactions
- Distributed execution
- Complex joins
- Aggregations and window functions
- Cost-based optimization
- Advanced statistics
- Multi-threaded scheduling
- Production-grade error handling and performance tuning

These features may become future exercises, but they are not required to achieve the primary learning objective.

## 8. Engineering Practices

MiniFusion will be developed like a small, real Rust project.

Each checkpoint should include:

- Clear module responsibilities
- Small public interfaces
- Unit and integration tests
- Meaningful error handling
- Useful debugging output
- Documentation of important design decisions

We will avoid overengineering, unnecessary abstractions, and premature optimization.

Refactoring is expected as new concepts are introduced.

A successful checkpoint is not measured by lines of code but by whether its concepts can be explained and tested independently.

## 9. Progress Tracker

- [ ] CP1 — Logical Plans and Expression Trees
- [ ] CP2 — Physical Planning
- [ ] CP3 — Pull-Based Query Execution
- [ ] CP4 — Logical Query Optimization
- [ ] CP5 — Columnar and Batch Execution
- [ ] CP6 — Bridging to Apache DataFusion

## 10. Graduation Criteria

MiniFusion is considered complete when we can confidently explain:

1. Why query engines separate logical and physical plans.
2. How expressions and relational operators form query trees.
3. How a physical planner turns declarative operations into executable operators.
4. How pull-based execution processes data incrementally.
5. How optimizer rules transform plans while preserving semantics.
6. Why analytical engines benefit from columnar batch processing.
7. How Apache Arrow represents data in memory.
8. How DataFusion organizes planning, optimization, and execution.
9. Where new data sources, physical operators, and optimizer rules fit into DataFusion.
10. How to approach a small contribution to the real project.

## 11. References

- [Apache DataFusion](https://datafusion.apache.org/)
- [DataFusion GitHub Repository](https://github.com/apache/datafusion)
- [Apache Arrow](https://arrow.apache.org/)
- [DataFusion Architecture](https://datafusion.apache.org/contributor-guide/architecture.html)
- [DataFusion Contributor Guide](https://datafusion.apache.org/contributor-guide/)
- [Custom Table Providers](https://datafusion.apache.org/library-user-guide/custom-table-providers.html)

---

## Final Objective

This repository is successful not when MiniFusion becomes a sophisticated query engine, but when its implementation makes Apache DataFusion's internals understandable.

**Build the small engine. Understand the abstractions. Explore the real engine. Contribute.**
use std::fmt;

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

impl LogicalPlan {
    pub fn scan(table: &str) -> Self {
        Self::Scan {
            table: table.to_owned(),
        }
    }

    pub fn filter(self, predicate: Expr) -> Self {
        Self::Filter {
            predicate,
            input: Box::new(self),
        }
    }

    pub fn project(self, columns: Vec<String>) -> Self {
        Self::Projection {
            columns,
            input: Box::new(self),
        }
    }

    fn fmt_with_indent(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        match self {
            LogicalPlan::Scan { table } => {
                writeln!(f, "{}Scan: {table}", " ".repeat(indent))
            }

            LogicalPlan::Filter { predicate, input } => {
                writeln!(f, "{}Filter: {predicate}", " ".repeat(indent))?;
                input.fmt_with_indent(f, indent + 2)
            }

            LogicalPlan::Projection { columns, input } => {
                writeln!(
                    f,
                    "{}Projection: [{}]",
                    " ".repeat(indent),
                    columns.join(", ")
                )?;

                input.fmt_with_indent(f, indent + 2)
            }
        }
    }
}

impl fmt::Display for LogicalPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_indent(f, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{col, lit};
    use crate::value::Value;

    // Test 1 — Scan Construction
    #[test]
    fn scan_construction() {
        let plan = LogicalPlan::scan("trips");

        match plan {
            LogicalPlan::Scan { table } => {
                assert_eq!(table, "trips");
            }
            _ => panic!(),
        }
    }

    // Test 2 — Filter Construction
    #[test]
    fn filter_construction() {
        let plan = LogicalPlan::scan("trips").filter(col("fare").gt(lit(Value::Int64(100))));

        match plan {
            LogicalPlan::Filter { predicate, input } => {
                assert!(matches!(predicate, Expr::Binary { .. }));
                assert!(matches!(*input, LogicalPlan::Scan { .. }));
            }
            _ => panic!(),
        }
    }

    // Test 3 — Projection Construction
    #[test]
    fn projection_construction() {
        let plan = LogicalPlan::scan("trips").project(vec!["zone".to_string(), "fare".to_string()]);

        match plan {
            LogicalPlan::Projection { columns, input } => {
                assert_eq!(
                    (columns[0].clone(), columns[1].clone()),
                    ("zone".to_string(), "fare".to_string())
                );
                assert!(matches!(*input, LogicalPlan::Scan { .. }));
            }
            _ => panic!(),
        }
    }

    // Test 4 — Logical Plan Display
    #[test]
    fn logical_plan_display() {
        let plan = LogicalPlan::scan("trips")
            .filter(col("distance_km").gt(lit(Value::Int64(5))))
            .project(vec!["zone".to_string(), "fare".to_string()]);

        let target = "Projection: [zone, fare]\n  Filter: (distance_km > 5)\n    Scan: trips\n";

        assert_eq!(format!("{}", plan), target);
    }

    // Test 5 — Nested Filters
    #[test]
    fn nested_filters() {
        let plan = LogicalPlan::scan("trips")
            .filter(col("distance_km").gt(lit(Value::Int64(5))))
            .filter(col("fare").lt(lit(Value::Int64(200))));

        match &plan {
            LogicalPlan::Filter { predicate, input } => {
                assert!(matches!(*predicate, Expr::Binary { .. }));
                assert!(matches!(**input, LogicalPlan::Filter { .. }));
                let LogicalPlan::Filter {
                    predicate: _,
                    input,
                } = *input.clone()
                else {
                    panic!();
                };
                assert!(matches!(*input, LogicalPlan::Scan { .. }));
            }
            _ => panic!(),
        };

        let target = "Filter: (fare < 200)\n  Filter: (distance_km > 5)\n    Scan: trips\n";

        assert_eq!(format!("{}", plan), target);
    }

    // Test 6 — Construction Does Not Validate
    // Verify that construction succeeds.
    // This is intentional for this exercise: our constructors currently build syntax trees, not validated plans.
    // Later, we will use our schema abstraction to determine whether the query makes sense.
    #[test]
    fn construction_does_not_validate() {
        let plan =
            LogicalPlan::scan("trips").filter(col("unknown_column").gt(lit(Value::Int64(10))));

        match plan {
            LogicalPlan::Filter { predicate, input } => {
                let Expr::Binary { left, .. } = predicate else {
                    panic!();
                };
                assert!(matches!(*left, Expr::Column { .. }));
                let Expr::Column(name) = *left else { panic!() };
                assert_eq!(name, "unknown_column");
                assert!(matches!(*input, LogicalPlan::Scan { .. }));
            }
            _ => panic!(),
        };
    }

    // Test 7 — Construction Does Not Execute
    #[test]
    fn construction_does_not_execute() {
        let plan = LogicalPlan::scan("imaginary_table")
            .filter(col("missing_column").gt(lit(Value::Int64(10))));

        match plan {
            LogicalPlan::Filter { predicate, input } => {
                let Expr::Binary { left, .. } = predicate else {
                    panic!();
                };
                assert!(matches!(*left, Expr::Column { .. }));
                let Expr::Column(name) = *left else { panic!() };
                assert_eq!(name, "missing_column");
                let LogicalPlan::Scan { table } = *input else {
                    panic!()
                };
                assert_eq!(table, "imaginary_table");
            }
            _ => panic!(),
        };
    }

    // Construct a logical plan referencing a table that has not been registered or loaded:

    // LogicalPlan::scan("imaginary_table")
    //     .project(vec!["missing_column".to_string()])

    // Verify that the plan can be constructed and inspected.

    // No table lookup should occur.

    // The builder is only capturing a query description.
}

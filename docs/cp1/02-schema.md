CP1 — Step 2: Data types and schemas
Our next objective is to describe the structure of a table, independently of its contents.
We'll introduce three abstractions:
DataType
What kind of data can this field hold?
Int64, Float64, Utf8, Boolean


Field
A named column with a declared type.
distance_km: Float64


Schema
An ordered collection of fields describing a dataset.
[trip_id, zone, distance_km, fare]


1. Create src/schema.rs
Start with these definitions:
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Int64,
    Float64,
    Utf8,
    Boolean,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub fields: Vec<Field>,
}


Notice that DataType doesn't include Null. Nullability is a separate property of a field. We'll add that later when we introduce proper null handling.
Now expose this module in main.rs:
mod value;
mod schema;


2. Your implementation challenge
Implement these methods yourself:
impl Schema {
    pub fn new(fields: Vec<Field>) -> Self {
        todo!()
    }

    pub fn field(&self, name: &str) -> Option<&Field> {
        todo!()
    }

    pub fn index_of(&self, name: &str) -> Option<usize> {
        todo!()
    }
}


Their contracts are:
- new constructs a schema from its fields.
- field retrieves a field by name, without cloning it.
- index_of returns the position of a field in the schema.
For now, assume field names are unique. Later we can revisit schema validation.
Rust hint: iter(), find(), and position() are your friends here.
3. Tests to implement
Use this fixture:
fn trips_schema() -> Schema {
    Schema::new(vec![
        Field {
            name: "trip_id".to_string(),
            data_type: DataType::Int64,
        },
        Field {
            name: "zone".to_string(),
            data_type: DataType::Utf8,
        },
        Field {
            name: "distance_km".to_string(),
            data_type: DataType::Float64,
        },
        Field {
            name: "fare".to_string(),
            data_type: DataType::Float64,
        },
    ])
}


Write four tests:
Test	Expected behavior
find_existing_field	field("zone") returns a Field with type Utf8
find_unknown_field	field("unknown") returns None
find_field_index	index_of("distance_km") returns Some(2)
empty_schema	An empty schema returns None for any lookup
4. One important design detail
Why does index_of matter?
Imagine our future executor receives a row:
let row = vec![
    Value::Int64(1),
    Value::Utf8("A".to_string()),
    Value::Float64(5.0),
    Value::Float64(100.0),
];


When evaluating:
WHERE distance_km > 3


We can resolve distance_km to column index 2 using the schema.
Then the executor knows which value to read:
let distance = &row[2];


This introduces an important principle:
The planner resolves names and types; the executor should not have to rediscover them for every row.
We aren't implementing that optimization yet, but our schema abstraction will make it possible.

Definition of done — Step 2
Run:
cargo fmt
cargo test
cargo clippy --all-targets -- -D warnings


Once these pass, we can move to Step 3: Expressions (Expr), where things get much more interesting.
We'll finally represent:
distance_km > 5


as a recursive Rust tree and distinguish a column reference from a literal value and an operator.
No dependencies yet, no query execution, and no unnecessary abstractions.
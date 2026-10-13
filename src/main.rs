#[cfg(test)]
use crate::schema::{DataType, Field, Schema};

pub mod expr;
pub mod logical_plan;
pub mod schema;
pub mod validation;
pub mod value;

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
pub fn trips_schema() -> Schema {
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

#[cfg(test)]
pub fn trips_null_schema() -> Schema {
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

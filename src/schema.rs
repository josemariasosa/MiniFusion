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

impl Schema {
    // new constructs a schema from its fields.
    pub fn new(fields: Vec<Field>) -> Self {
        Self { fields }
    }

    // field retrieves a field by name, without cloning it.
    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }

    // index_of returns the position of a field in the schema.
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|f| f.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trips_schema;

    // find_existing_field	field("zone") returns a Field with type Utf8
    #[test]
    fn find_existing_field() {
        let trips = trips_schema();
        let field = trips.field("zone").unwrap();
        assert_eq!(field.data_type, DataType::Utf8);
    }

    // find_unknown_field	field("unknown") returns None
    #[test]
    fn find_unknown_field() {
        let trips = trips_schema();
        assert!(trips.field("unknown").is_none());
    }

    // find_field_index	index_of("distance_km") returns Some(2)
    #[test]
    fn find_field_index() {
        let trips = trips_schema();
        let field = trips.index_of("distance_km").unwrap();
        assert_eq!(field, 2);
    }

    // empty_schema	An empty schema returns None for any lookup
    #[test]
    fn empty_schema() {
        let schema = Schema::new(vec![]);
        assert!(schema.field("trip_id").is_none());
        assert!(schema.index_of("trip_id").is_none());
    }
}

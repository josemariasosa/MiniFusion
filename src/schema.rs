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

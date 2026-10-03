//! Draft entity representation. Executable commands are not implemented yet.
use serde::{Deserialize, Serialize};

pub const VERSION: &str = "0.1-draft";

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Application {
    pub sir_version: String,
    pub name: String,
    pub entities: Vec<Entity>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entity {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FieldType {
    String,
    Int,
    Bool,
    Ref { entity: String },
}

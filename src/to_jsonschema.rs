use openapiv3::Type;
use serde_json::json;

pub trait ToJSONSchema {
    fn to_json_schema(&self) -> serde_json::Value;
}

impl ToJSONSchema for openapiv3::Schema {
    fn to_json_schema(&self) -> serde_json::Value {
        match &self.schema_kind {
            openapiv3::SchemaKind::Type(Type::Boolean {}) => json!({"type": "boolean"}),
            openapiv3::SchemaKind::Type(Type::String(string_schema)) => {
                string_schema.to_json_schema()
            }
            openapiv3::SchemaKind::Type(Type::Number(number_schema)) => {
                number_schema.to_json_schema()
            }
            openapiv3::SchemaKind::Type(Type::Integer(integer_schema)) => {
                integer_schema.to_json_schema()
            }
            openapiv3::SchemaKind::Type(Type::Object(object_schema)) => {
                object_schema.to_json_schema()
            }
            openapiv3::SchemaKind::Type(Type::Array(array_schema)) => array_schema.to_json_schema(),
            openapiv3::SchemaKind::OneOf { one_of } => {
                let mut json = serde_json::Map::new();
                json.insert("oneOf".to_string(), one_of.to_json_schema());
                json.into()
            }
            openapiv3::SchemaKind::AllOf { all_of } => {
                let mut json = serde_json::Map::new();
                json.insert("allOf".to_string(), all_of.to_json_schema());
                json.into()
            }
            openapiv3::SchemaKind::AnyOf { any_of } => {
                let mut json = serde_json::Map::new();
                json.insert("anyOf".to_string(), any_of.to_json_schema());
                json.into()
            }
            openapiv3::SchemaKind::Not { not } => {
                let mut json = serde_json::Map::new();
                json.insert("not".to_string(), not.to_json_schema());
                json.into()
            }
            _ => todo!(),
        }
    }
}

impl ToJSONSchema for openapiv3::StringType {
    fn to_json_schema(&self) -> serde_json::Value {
        let mut json = serde_json::Map::new();
        json.insert("type".to_string(), serde_json::Value::from("string"));
        json.insert_if_some("minLength", &self.min_length);
        json.insert_if_some("maxLength", &self.max_length);
        json.insert_if_not_empty("enum", &self.enumeration);
        json.insert_if_some("pattern", &self.pattern);
        if let openapiv3::VariantOrUnknownOrEmpty::Item(format) = &self.format {
            match format {
                openapiv3::StringFormat::DateTime => {
                    json.insert("format".to_string(), "date-time".into());
                }
                openapiv3::StringFormat::Date => {
                    json.insert("format".to_string(), "date".into());
                }
                openapiv3::StringFormat::Password => {
                    json.insert("format".to_string(), "password".into());
                }
                openapiv3::StringFormat::Byte => {
                    json.insert("format".to_string(), "byte".into());
                }
                openapiv3::StringFormat::Binary => {
                    json.insert("format".to_string(), "binary".into());
                }
            }
        }
        json.into()
    }
}

impl ToJSONSchema for openapiv3::NumberType {
    fn to_json_schema(&self) -> serde_json::Value {
        let mut json = serde_json::Map::new();
        json.insert("type".to_string(), serde_json::Value::from("number"));
        json.insert_if_some("minimum", &self.minimum);
        json.insert_if_some("maximum", &self.maximum);
        json.insert_if_true("exclusiveMinimum", self.exclusive_minimum);
        json.insert_if_true("exclusiveMaximum", self.exclusive_maximum);
        json.insert_if_some("multipleOf", &self.multiple_of);
        json.insert_if_not_empty("enum", &self.enumeration);
        json.into()
    }
}

impl ToJSONSchema for openapiv3::IntegerType {
    fn to_json_schema(&self) -> serde_json::Value {
        let mut json = serde_json::Map::new();
        json.insert("type".to_string(), serde_json::Value::from("integer"));
        json.insert_if_some("minimum", &self.minimum);
        json.insert_if_some("maximum", &self.maximum);
        json.insert_if_true("exclusiveMinimum", self.exclusive_minimum);
        json.insert_if_true("exclusiveMaximum", self.exclusive_maximum);
        json.insert_if_some("multipleOf", &self.multiple_of);
        json.insert_if_not_empty("enum", &self.enumeration);
        json.into()
    }
}

impl ToJSONSchema for openapiv3::ArrayType {
    fn to_json_schema(&self) -> serde_json::Value {
        let mut json = serde_json::Map::new();
        json.insert("type".to_string(), serde_json::Value::from("array"));
        json.insert_if_some("minItems", &self.min_items);
        json.insert_if_some("maxItems", &self.max_items);
        json.insert_if_true("uniqueItems", self.unique_items);
        json.insert_if_some(
            "items",
            &self
                .items
                .as_ref()
                .and_then(openapiv3::ReferenceOr::as_item)
                .map(|schema| schema.to_json_schema()),
        );
        json.into()
    }
}

impl ToJSONSchema for openapiv3::ObjectType {
    fn to_json_schema(&self) -> serde_json::Value {
        let mut json = serde_json::Map::new();
        json.insert("type".to_string(), serde_json::Value::from("object"));
        json.insert_if_some("minProperties", &self.min_properties);
        json.insert_if_some("maxProperties", &self.max_properties);
        if let Some(additional_properties) = &self.additional_properties {
            json.insert(
                "additionalProperties".to_string(),
                match additional_properties {
                    openapiv3::AdditionalProperties::Any(value) => value.clone().into(),
                    openapiv3::AdditionalProperties::Schema(schema) => schema.to_json_schema(),
                },
            );
        }
        json.insert_if_map_not_empty("properties", &self.properties);
        json.insert_if_not_empty("required", &self.required);
        json.into()
    }
}

impl<T: ToJSONSchema + Clone> ToJSONSchema for openapiv3::ReferenceOr<T> {
    fn to_json_schema(&self) -> serde_json::Value {
        self.clone().as_item().unwrap().to_json_schema()
    }
}

impl<T: ToJSONSchema> ToJSONSchema for Vec<T> {
    fn to_json_schema(&self) -> serde_json::Value {
        self.iter().map(|schema| schema.to_json_schema()).collect()
    }
}

impl<T: ToJSONSchema> ToJSONSchema for Box<T> {
    fn to_json_schema(&self) -> serde_json::Value {
        self.as_ref().to_json_schema()
    }
}

trait InsertIf {
    fn insert_if_some<T: Into<serde_json::Value> + Clone>(
        &mut self,
        key: &str,
        optional_value: &Option<T>,
    );
    fn insert_if_true(&mut self, key: &str, value: bool);
    fn insert_if_not_empty<T: Into<serde_json::Value> + Clone>(
        &mut self,
        key: &str,
        value: &Vec<T>,
    );
    fn insert_if_map_not_empty<T: ToJSONSchema + Clone>(
        &mut self,
        key: &str,
        value: &indexmap::map::IndexMap<String, T>,
    );
}

impl InsertIf for serde_json::Map<String, serde_json::Value> {
    fn insert_if_some<T: Into<serde_json::Value> + Clone>(
        &mut self,
        key: &str,
        optional_value: &Option<T>,
    ) {
        if let Some(value) = optional_value {
            self.insert(key.to_string(), value.clone().into());
        }
    }

    fn insert_if_true(&mut self, key: &str, value: bool) {
        if value {
            self.insert(key.to_string(), value.into());
        }
    }

    fn insert_if_not_empty<T: Into<serde_json::Value> + Clone>(
        &mut self,
        key: &str,
        value: &Vec<T>,
    ) {
        if !value.is_empty() {
            self.insert(key.to_string(), value.clone().into());
        }
    }

    fn insert_if_map_not_empty<T: ToJSONSchema + Clone>(
        &mut self,
        key: &str,
        value: &indexmap::map::IndexMap<String, T>,
    ) {
        if !value.is_empty() {
            self.insert(
                key.to_string(),
                value
                    .iter()
                    .map(|(key, value)| (key.to_string(), value.to_json_schema()))
                    .collect::<serde_json::Map<_, _>>()
                    .into(),
            );
        }
    }
}

#[cfg(test)]
mod test_boolean {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Boolean {})
            }
            .to_json_schema(),
            json!({"type": "boolean"})
        )
    }
}

#[cfg(test)]
mod test_string {
    use super::*;
    use openapiv3::StringType;

    #[test]
    fn basic() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::String(StringType {
                    format: Default::default(),
                    pattern: None,
                    enumeration: vec![],
                    min_length: None,
                    max_length: None,
                }))
            }
            .to_json_schema(),
            json!({"type": "string"})
        )
    }

    #[test]
    fn min_length() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::String(StringType {
                    format: Default::default(),
                    pattern: None,
                    enumeration: vec![],
                    min_length: Some(5),
                    max_length: None,
                }))
            }
            .to_json_schema(),
            json!({"type": "string", "minLength": 5})
        )
    }

    #[test]
    fn min_and_max_length() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::String(StringType {
                    format: Default::default(),
                    pattern: None,
                    enumeration: vec![],
                    min_length: Some(5),
                    max_length: Some(10),
                }))
            }
            .to_json_schema(),
            json!({"type": "string", "minLength": 5, "maxLength": 10})
        )
    }

    #[test]
    fn enumeration() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::String(StringType {
                    format: Default::default(),
                    pattern: None,
                    enumeration: vec![Some("one".to_string()), Some("two".to_string())],
                    min_length: None,
                    max_length: None,
                }))
            }
            .to_json_schema(),
            json!({"type": "string", "enum": ["one", "two"]})
        )
    }

    #[test]
    fn pattern() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::String(StringType {
                    format: Default::default(),
                    pattern: Some("^(\\([0-9]{3}\\))?[0-9]{3}-[0-9]{4}$".to_string()),
                    enumeration: vec![],
                    min_length: None,
                    max_length: None,
                }))
            }
            .to_json_schema(),
            json!({"type": "string", "pattern": "^(\\([0-9]{3}\\))?[0-9]{3}-[0-9]{4}$"})
        )
    }

    #[test]
    fn format_date_time() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::String(StringType {
                    format: openapiv3::VariantOrUnknownOrEmpty::Item(
                        openapiv3::StringFormat::DateTime
                    ),
                    pattern: None,
                    enumeration: vec![],
                    min_length: None,
                    max_length: None,
                }))
            }
            .to_json_schema(),
            json!({"type": "string", "format": "date-time"})
        )
    }

    #[test]
    fn format_date() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::String(StringType {
                    format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::StringFormat::Date),
                    pattern: None,
                    enumeration: vec![],
                    min_length: None,
                    max_length: None,
                }))
            }
            .to_json_schema(),
            json!({"type": "string", "format": "date"})
        )
    }

    #[test]
    fn format_password() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::String(StringType {
                    format: openapiv3::VariantOrUnknownOrEmpty::Item(
                        openapiv3::StringFormat::Password
                    ),
                    pattern: None,
                    enumeration: vec![],
                    min_length: None,
                    max_length: None,
                }))
            }
            .to_json_schema(),
            json!({"type": "string", "format": "password"})
        )
    }

    #[test]
    fn format_byte() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::String(StringType {
                    format: openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::StringFormat::Byte),
                    pattern: None,
                    enumeration: vec![],
                    min_length: None,
                    max_length: None,
                }))
            }
            .to_json_schema(),
            json!({"type": "string", "format": "byte"})
        )
    }

    #[test]
    fn format_binary() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::String(StringType {
                    format: openapiv3::VariantOrUnknownOrEmpty::Item(
                        openapiv3::StringFormat::Binary
                    ),
                    pattern: None,
                    enumeration: vec![],
                    min_length: None,
                    max_length: None,
                }))
            }
            .to_json_schema(),
            json!({"type": "string", "format": "binary"})
        )
    }
}

#[cfg(test)]
mod test_number {
    use super::*;
    use openapiv3::NumberType;

    #[test]
    fn basic() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Number(NumberType {
                    format: Default::default(),
                    multiple_of: None,
                    exclusive_minimum: false,
                    exclusive_maximum: false,
                    minimum: None,
                    maximum: None,
                    enumeration: vec![],
                }))
            }
            .to_json_schema(),
            json!({"type": "number"})
        )
    }

    #[test]
    fn minimum() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Number(NumberType {
                    format: Default::default(),
                    multiple_of: None,
                    exclusive_minimum: false,
                    exclusive_maximum: false,
                    minimum: Some(2.1),
                    maximum: None,
                    enumeration: vec![],
                }))
            }
            .to_json_schema(),
            json!({"type": "number", "minimum": 2.1})
        )
    }

    #[test]
    fn minimum_and_maximum() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Number(NumberType {
                    format: Default::default(),
                    multiple_of: None,
                    exclusive_minimum: false,
                    exclusive_maximum: false,
                    minimum: Some(2.1),
                    maximum: Some(5.6),
                    enumeration: vec![],
                }))
            }
            .to_json_schema(),
            json!({"type": "number", "minimum": 2.1, "maximum": 5.6})
        )
    }

    #[test]
    fn exclusive_minimum_and_maximum() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Number(NumberType {
                    format: Default::default(),
                    multiple_of: None,
                    exclusive_minimum: true,
                    exclusive_maximum: true,
                    minimum: Some(2.1),
                    maximum: Some(5.6),
                    enumeration: vec![],
                }))
            }
            .to_json_schema(),
            json!({"type": "number", "minimum": 2.1, "maximum": 5.6, "exclusiveMinimum": true, "exclusiveMaximum": true})
        )
    }

    #[test]
    fn multiple_of() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Number(NumberType {
                    format: Default::default(),
                    multiple_of: Some(1.1),
                    exclusive_minimum: false,
                    exclusive_maximum: false,
                    minimum: None,
                    maximum: None,
                    enumeration: vec![],
                }))
            }
            .to_json_schema(),
            json!({"type": "number", "multipleOf": 1.1})
        )
    }

    #[test]
    fn enumeration() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Number(NumberType {
                    format: Default::default(),
                    multiple_of: None,
                    exclusive_minimum: false,
                    exclusive_maximum: false,
                    minimum: None,
                    maximum: None,
                    enumeration: vec![Some(1.1), Some(2.2)],
                }))
            }
            .to_json_schema(),
            json!({"type": "number", "enum": [1.1, 2.2]})
        )
    }
}

#[cfg(test)]
mod test_integer {
    use super::*;
    use openapiv3::IntegerType;

    #[test]
    fn basic() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Integer(IntegerType {
                    format: Default::default(),
                    multiple_of: None,
                    exclusive_minimum: false,
                    exclusive_maximum: false,
                    minimum: None,
                    maximum: None,
                    enumeration: vec![],
                }))
            }
            .to_json_schema(),
            json!({"type": "integer"})
        )
    }

    #[test]
    fn minimum() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Integer(IntegerType {
                    format: Default::default(),
                    multiple_of: None,
                    exclusive_minimum: false,
                    exclusive_maximum: false,
                    minimum: Some(2),
                    maximum: None,
                    enumeration: vec![],
                }))
            }
            .to_json_schema(),
            json!({"type": "integer", "minimum": 2})
        )
    }

    #[test]
    fn minimum_and_maximum() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Integer(IntegerType {
                    format: Default::default(),
                    multiple_of: None,
                    exclusive_minimum: false,
                    exclusive_maximum: false,
                    minimum: Some(2),
                    maximum: Some(5),
                    enumeration: vec![],
                }))
            }
            .to_json_schema(),
            json!({"type": "integer", "minimum": 2, "maximum": 5})
        )
    }

    #[test]
    fn exclusive_minimum_and_maximum() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Integer(IntegerType {
                    format: Default::default(),
                    multiple_of: None,
                    exclusive_minimum: true,
                    exclusive_maximum: true,
                    minimum: Some(2),
                    maximum: Some(5),
                    enumeration: vec![],
                }))
            }
            .to_json_schema(),
            json!({"type": "integer", "minimum": 2, "maximum": 5, "exclusiveMinimum": true, "exclusiveMaximum": true})
        )
    }

    #[test]
    fn multiple_of() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Integer(IntegerType {
                    format: Default::default(),
                    multiple_of: Some(10),
                    exclusive_minimum: false,
                    exclusive_maximum: false,
                    minimum: None,
                    maximum: None,
                    enumeration: vec![],
                }))
            }
            .to_json_schema(),
            json!({"type": "integer", "multipleOf": 10})
        )
    }

    #[test]
    fn enumeration() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Integer(IntegerType {
                    format: Default::default(),
                    multiple_of: None,
                    exclusive_minimum: false,
                    exclusive_maximum: false,
                    minimum: None,
                    maximum: None,
                    enumeration: vec![Some(1), Some(2)],
                }))
            }
            .to_json_schema(),
            json!({"type": "integer", "enum": [1, 2]})
        )
    }
}

#[cfg(test)]
mod test_array {
    use super::*;
    use openapiv3::{ArrayType, IntegerType, NumberType, ReferenceOr, StringType};

    #[test]
    fn basic() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Array(ArrayType {
                    items: None,
                    min_items: None,
                    max_items: None,
                    unique_items: false,
                }))
            }
            .to_json_schema(),
            json!({"type": "array"})
        )
    }

    #[test]
    fn min_items() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Array(ArrayType {
                    items: None,
                    min_items: Some(2),
                    max_items: None,
                    unique_items: false,
                }))
            }
            .to_json_schema(),
            json!({"type": "array", "minItems": 2})
        )
    }

    #[test]
    fn min_and_max_items() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Array(ArrayType {
                    items: None,
                    min_items: Some(2),
                    max_items: Some(5),
                    unique_items: false,
                }))
            }
            .to_json_schema(),
            json!({"type": "array", "minItems": 2, "maxItems": 5})
        )
    }

    #[test]
    fn unique_items() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Array(ArrayType {
                    items: None,
                    min_items: None,
                    max_items: None,
                    unique_items: true,
                }))
            }
            .to_json_schema(),
            json!({"type": "array", "uniqueItems": true})
        )
    }

    #[test]
    fn number_items() {
        let number_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Number(NumberType {
                format: Default::default(),
                multiple_of: None,
                exclusive_minimum: false,
                exclusive_maximum: false,
                minimum: None,
                maximum: None,
                enumeration: vec![],
            })),
        };
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Array(ArrayType {
                    items: Some(ReferenceOr::Item(Box::from(number_schema))),
                    min_items: None,
                    max_items: None,
                    unique_items: false,
                }))
            }
            .to_json_schema(),
            json!({"type": "array", "items": {"type": "number"}})
        )
    }

    #[test]
    fn integer_items() {
        let integer_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Integer(IntegerType {
                format: Default::default(),
                multiple_of: None,
                exclusive_minimum: false,
                exclusive_maximum: false,
                minimum: None,
                maximum: None,
                enumeration: vec![],
            })),
        };
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Array(ArrayType {
                    items: Some(ReferenceOr::Item(Box::from(integer_schema))),
                    min_items: None,
                    max_items: None,
                    unique_items: false,
                }))
            }
            .to_json_schema(),
            json!({"type": "array", "items": {"type": "integer"}})
        )
    }

    #[test]
    fn string_items() {
        let string_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::String(StringType {
                format: Default::default(),
                pattern: None,
                enumeration: vec![],
                min_length: None,
                max_length: None,
            })),
        };
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Array(ArrayType {
                    items: Some(ReferenceOr::Item(Box::from(string_schema))),
                    min_items: None,
                    max_items: None,
                    unique_items: false,
                }))
            }
            .to_json_schema(),
            json!({"type": "array", "items": {"type": "string"}})
        )
    }

    #[test]
    fn boolean_items() {
        let boolean_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Boolean {}),
        };
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Array(ArrayType {
                    items: Some(ReferenceOr::Item(Box::from(boolean_schema))),
                    min_items: None,
                    max_items: None,
                    unique_items: false,
                }))
            }
            .to_json_schema(),
            json!({"type": "array", "items": {"type": "boolean"}})
        )
    }

    #[test]
    fn array_items() {
        let array_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Array(ArrayType {
                items: None,
                min_items: None,
                max_items: None,
                unique_items: false,
            })),
        };
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Array(ArrayType {
                    items: Some(ReferenceOr::Item(Box::from(array_schema))),
                    min_items: None,
                    max_items: None,
                    unique_items: false,
                }))
            }
            .to_json_schema(),
            json!({"type": "array", "items": {"type": "array"}})
        )
    }

    #[test]
    fn invalid_items() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Array(ArrayType {
                    items: Some(ReferenceOr::Reference {
                        reference: "not valid".to_string()
                    }),
                    min_items: None,
                    max_items: None,
                    unique_items: false,
                }))
            }
            .to_json_schema(),
            json!({"type": "array"})
        )
    }
}

#[cfg(test)]
mod test_object {
    use super::*;
    use openapiv3::{IntegerType, NumberType, ObjectType, ReferenceOr, StringType};

    #[test]
    fn basic() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Object(ObjectType {
                    properties: Default::default(),
                    required: vec![],
                    additional_properties: None,
                    min_properties: None,
                    max_properties: None,
                }))
            }
            .to_json_schema(),
            json!({"type": "object"})
        )
    }

    #[test]
    fn min_properties() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Object(ObjectType {
                    properties: Default::default(),
                    required: vec![],
                    additional_properties: None,
                    min_properties: Some(2),
                    max_properties: None,
                }))
            }
            .to_json_schema(),
            json!({"type": "object", "minProperties": 2})
        )
    }

    #[test]
    fn min_and_max_properties() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Object(ObjectType {
                    properties: Default::default(),
                    required: vec![],
                    additional_properties: None,
                    min_properties: Some(2),
                    max_properties: Some(5),
                }))
            }
            .to_json_schema(),
            json!({"type": "object", "minProperties": 2, "maxProperties": 5})
        )
    }

    #[test]
    fn number_properties() {
        let number_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Number(NumberType {
                format: Default::default(),
                multiple_of: None,
                exclusive_minimum: false,
                exclusive_maximum: false,
                minimum: None,
                maximum: None,
                enumeration: vec![],
            })),
        };
        let mut properties = indexmap::map::IndexMap::new();
        properties.insert(
            "count".to_string(),
            ReferenceOr::Item(Box::from(number_schema)),
        );
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Object(ObjectType {
                    properties,
                    required: vec![],
                    additional_properties: None,
                    min_properties: None,
                    max_properties: None,
                }))
            }
            .to_json_schema(),
            json!({"type": "object", "properties": {"count": {"type": "number"}}})
        )
    }

    #[test]
    fn multiple_properties() {
        let string_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::String(StringType {
                format: Default::default(),
                pattern: None,
                enumeration: vec![],
                min_length: Some(5),
                max_length: Some(10),
            })),
        };
        let integer_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Integer(IntegerType {
                format: Default::default(),
                multiple_of: Some(10),
                exclusive_minimum: false,
                exclusive_maximum: false,
                minimum: None,
                maximum: None,
                enumeration: vec![],
            })),
        };
        let boolean_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Boolean {}),
        };
        let mut properties = indexmap::map::IndexMap::new();
        properties.insert(
            "string".to_string(),
            ReferenceOr::Item(Box::from(string_schema)),
        );
        properties.insert(
            "integer".to_string(),
            ReferenceOr::Item(Box::from(integer_schema)),
        );
        properties.insert(
            "boolean".to_string(),
            ReferenceOr::Item(Box::from(boolean_schema)),
        );
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Object(ObjectType {
                    properties,
                    required: vec![],
                    additional_properties: None,
                    min_properties: Some(3),
                    max_properties: Some(5),
                }))
            }
            .to_json_schema(),
            json!({"type": "object", 
                    "properties": {"string": {"type": "string", "minLength": 5, "maxLength": 10}, 
                                   "integer": {"type": "integer", "multipleOf": 10}, 
                                   "boolean": {"type": "boolean"}}, 
                    "minProperties": 3, "maxProperties": 5})
        )
    }

    #[test]
    fn required_properties() {
        let number_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Number(NumberType {
                format: Default::default(),
                multiple_of: None,
                exclusive_minimum: false,
                exclusive_maximum: false,
                minimum: None,
                maximum: None,
                enumeration: vec![],
            })),
        };
        let mut properties = indexmap::map::IndexMap::new();
        properties.insert(
            "count".to_string(),
            ReferenceOr::Item(Box::from(number_schema)),
        );
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Object(ObjectType {
                    properties,
                    required: vec!["count".to_string()],
                    additional_properties: None,
                    min_properties: None,
                    max_properties: None,
                }))
            }
            .to_json_schema(),
            json!({"type": "object", "required": ["count"], "properties": {"count": {"type": "number"}}})
        )
    }

    #[test]
    fn multiple_required_properties() {
        let number_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Number(NumberType {
                format: Default::default(),
                multiple_of: None,
                exclusive_minimum: false,
                exclusive_maximum: false,
                minimum: None,
                maximum: None,
                enumeration: vec![],
            })),
        };
        let boolean_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Boolean {}),
        };
        let mut properties = indexmap::map::IndexMap::new();
        properties.insert(
            "count".to_string(),
            ReferenceOr::Item(Box::from(number_schema)),
        );
        properties.insert(
            "is_working".to_string(),
            ReferenceOr::Item(Box::from(boolean_schema)),
        );
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Object(ObjectType {
                    properties,
                    required: vec!["count".to_string(), "is_working".to_string()],
                    additional_properties: None,
                    min_properties: None,
                    max_properties: None,
                }))
            }
            .to_json_schema(),
            json!({"type": "object", "required": ["count", "is_working"], "properties": {"count": {"type": "number"}, "is_working": {"type": "boolean"}}})
        )
    }

    #[test]
    fn additional_properties_false() {
        let boolean_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Boolean {}),
        };
        let mut properties = indexmap::map::IndexMap::new();
        properties.insert(
            "is_working".to_string(),
            ReferenceOr::Item(Box::from(boolean_schema)),
        );
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Object(ObjectType {
                    properties: properties,
                    required: vec!["is_working".to_string()],
                    additional_properties: Some(openapiv3::AdditionalProperties::Any(false)),
                    min_properties: None,
                    max_properties: None,
                }))
            }
            .to_json_schema(),
            json!({"type": "object", "properties": {"is_working": {"type": "boolean"}}, "additionalProperties": false, "required": ["is_working"]})
        )
    }

    #[test]
    fn additional_properties_schema() {
        let number_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Number(NumberType {
                format: Default::default(),
                multiple_of: None,
                exclusive_minimum: false,
                exclusive_maximum: false,
                minimum: None,
                maximum: None,
                enumeration: vec![],
            })),
        };
        let boolean_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Boolean {}),
        };
        let mut properties = indexmap::map::IndexMap::new();
        properties.insert(
            "is_working".to_string(),
            ReferenceOr::Item(Box::from(boolean_schema)),
        );
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Object(ObjectType {
                    properties,
                    required: vec!["is_working".to_string()],
                    additional_properties: Some(openapiv3::AdditionalProperties::Schema(
                        Box::from(ReferenceOr::Item(number_schema))
                    )),
                    min_properties: None,
                    max_properties: None,
                }))
            }
            .to_json_schema(),
            json!({"type": "object", "properties": {"is_working": {"type": "boolean"}}, "additionalProperties": {"type": "number"}, "required": ["is_working"]})
        )
    }
}

#[cfg(test)]
mod test_one_of {
    use super::*;
    use openapiv3::{IntegerType, ReferenceOr};

    #[test]
    fn basic() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::OneOf {
                    one_of: vec![ReferenceOr::Item(openapiv3::Schema {
                        schema_data: Default::default(),
                        schema_kind: openapiv3::SchemaKind::Type(Type::Boolean {})
                    })]
                }
            }
            .to_json_schema(),
            json!({"oneOf": [{"type": "boolean"}]})
        )
    }

    #[test]
    fn multiple() {
        let integer_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Integer(IntegerType {
                format: Default::default(),
                multiple_of: None,
                exclusive_minimum: false,
                exclusive_maximum: false,
                minimum: None,
                maximum: None,
                enumeration: vec![],
            })),
        };
        let boolean_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Boolean {}),
        };
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::OneOf {
                    one_of: vec![
                        ReferenceOr::Item(boolean_schema),
                        ReferenceOr::Item(integer_schema)
                    ]
                }
            }
            .to_json_schema(),
            json!({"oneOf": [{"type": "boolean"}, {"type": "integer"}]})
        )
    }
}

#[cfg(test)]
mod test_all_of {
    use super::*;
    use openapiv3::{IntegerType, ReferenceOr};

    #[test]
    fn basic() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::AllOf {
                    all_of: vec![ReferenceOr::Item(openapiv3::Schema {
                        schema_data: Default::default(),
                        schema_kind: openapiv3::SchemaKind::Type(Type::Boolean {})
                    })]
                }
            }
            .to_json_schema(),
            json!({"allOf": [{"type": "boolean"}]})
        )
    }

    #[test]
    fn multiple() {
        let integer_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Integer(IntegerType {
                format: Default::default(),
                multiple_of: None,
                exclusive_minimum: false,
                exclusive_maximum: false,
                minimum: None,
                maximum: None,
                enumeration: vec![],
            })),
        };
        let boolean_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Boolean {}),
        };
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::AllOf {
                    all_of: vec![
                        ReferenceOr::Item(boolean_schema),
                        ReferenceOr::Item(integer_schema)
                    ]
                }
            }
            .to_json_schema(),
            json!({"allOf": [{"type": "boolean"}, {"type": "integer"}]})
        )
    }
}

#[cfg(test)]
mod test_any_of {
    use super::*;
    use openapiv3::{IntegerType, ReferenceOr};

    #[test]
    fn basic() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::AnyOf {
                    any_of: vec![ReferenceOr::Item(openapiv3::Schema {
                        schema_data: Default::default(),
                        schema_kind: openapiv3::SchemaKind::Type(Type::Boolean {})
                    })]
                }
            }
            .to_json_schema(),
            json!({"anyOf": [{"type": "boolean"}]})
        )
    }

    #[test]
    fn multiple() {
        let integer_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Integer(IntegerType {
                format: Default::default(),
                multiple_of: None,
                exclusive_minimum: false,
                exclusive_maximum: false,
                minimum: None,
                maximum: None,
                enumeration: vec![],
            })),
        };
        let boolean_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Boolean {}),
        };
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::AnyOf {
                    any_of: vec![
                        ReferenceOr::Item(boolean_schema),
                        ReferenceOr::Item(integer_schema)
                    ]
                }
            }
            .to_json_schema(),
            json!({"anyOf": [{"type": "boolean"}, {"type": "integer"}]})
        )
    }
}

#[cfg(test)]
mod test_not {
    use super::*;
    use openapiv3::ReferenceOr;

    #[test]
    fn basic() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Not {
                    not: Box::from(ReferenceOr::Item(openapiv3::Schema {
                        schema_data: Default::default(),
                        schema_kind: openapiv3::SchemaKind::Type(Type::Boolean {})
                    }))
                }
            }
            .to_json_schema(),
            json!({"not": {"type": "boolean"}})
        )
    }
}

#[cfg(test)]
mod test_validation {
    use super::*;
    use jsonschema::JSONSchema;
    use openapiv3::{NumberType, ObjectType, ReferenceOr, StringType};

    #[test]
    fn boolean() {
        let schema_json = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Boolean {}),
        }
        .to_json_schema();
        assert_eq!(json!({"type": "boolean"}), schema_json);

        let instance = json!(true);
        let schema = JSONSchema::compile(&schema_json).expect("a valid schema");
        assert_eq!(true, schema.is_valid(&instance));
    }

    #[test]
    fn string() {
        let schema_json = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::String(StringType {
                format: Default::default(),
                pattern: None,
                enumeration: vec![],
                min_length: Some(5),
                max_length: Some(10),
            })),
        }
        .to_json_schema();
        assert_eq!(
            json!({"type": "string", "minLength": 5, "maxLength": 10}),
            schema_json
        );

        let good_json = json!("length");
        let bad_json = json!("length_too_long");
        let schema = JSONSchema::compile(&schema_json).expect("a valid schema");
        assert_eq!(true, schema.is_valid(&good_json));
        assert_eq!(false, schema.is_valid(&bad_json));
    }

    #[test]
    fn required_properties_in_object() {
        let number_schema = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Number(NumberType {
                format: Default::default(),
                multiple_of: None,
                exclusive_minimum: false,
                exclusive_maximum: false,
                minimum: None,
                maximum: None,
                enumeration: vec![],
            })),
        };
        let mut properties = indexmap::map::IndexMap::new();
        properties.insert(
            "count".to_string(),
            ReferenceOr::Item(Box::from(number_schema)),
        );

        let schema_json = openapiv3::Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Type::Object(ObjectType {
                properties,
                required: vec!["count".to_string()],
                additional_properties: None,
                min_properties: None,
                max_properties: None,
            })),
        }
        .to_json_schema();
        assert_eq!(
            json!({"type": "object", "required": ["count"], "properties": {"count": {"type": "number"}}}),
            schema_json
        );

        let good_json = json!({"count": 10.1});
        let wrong_key = json!({"not_count": 10.1});
        let wrong_type = json!({"count": "string"});
        let schema = JSONSchema::compile(&schema_json).expect("a valid schema");
        assert_eq!(true, schema.is_valid(&good_json));
        assert_eq!(false, schema.is_valid(&wrong_key));
        assert_eq!(false, schema.is_valid(&wrong_type));
    }
}

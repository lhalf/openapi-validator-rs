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
                let schemas: serde_json::Value = one_of
                    .iter()
                    .map(|schema| schema.to_owned().into_item().unwrap().to_json_schema())
                    .collect();
                json.insert("oneOf".to_string(), schemas);
                json.into()
            }
            openapiv3::SchemaKind::AllOf { all_of } => {
                let mut json = serde_json::Map::new();
                let schemas: serde_json::Value = all_of
                    .iter()
                    .map(|schema| schema.to_owned().into_item().unwrap().to_json_schema())
                    .collect();
                json.insert("allOf".to_string(), schemas);
                json.into()
            }
            openapiv3::SchemaKind::AnyOf { any_of } => {
                let mut json = serde_json::Map::new();
                let schemas: serde_json::Value = any_of
                    .iter()
                    .map(|schema| schema.to_owned().into_item().unwrap().to_json_schema())
                    .collect();
                json.insert("anyOf".to_string(), schemas);
                json.into()
            }
            openapiv3::SchemaKind::Not { not } => {
                let mut json = serde_json::Map::new();
                json.insert(
                    "not".to_string(),
                    not.to_owned().into_item().unwrap().to_json_schema(),
                );
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
        if let Some(min_length) = self.min_length {
            json.insert("minLength".to_string(), min_length.into());
        }
        if let Some(max_length) = self.max_length {
            json.insert("maxLength".to_string(), max_length.into());
        }
        if let Some(pattern) = &self.pattern {
            json.insert("pattern".to_string(), pattern.to_string().into());
        }
        if let openapiv3::VariantOrUnknownOrEmpty::Item(format) = &self.format {
            match format {
                openapiv3::StringFormat::DateTime => {
                    json.insert("format".to_string(), "date-time".into());
                }
                openapiv3::StringFormat::Date => {
                    json.insert("format".to_string(), "date".into());
                }
                _ => (),
            }
        }
        json.into()
    }
}

impl ToJSONSchema for openapiv3::NumberType {
    fn to_json_schema(&self) -> serde_json::Value {
        let mut json = serde_json::Map::new();
        json.insert("type".to_string(), serde_json::Value::from("number"));
        if let Some(minimum) = self.minimum {
            json.insert("minimum".to_string(), minimum.into());
        }
        if let Some(maximum) = self.maximum {
            json.insert("maximum".to_string(), maximum.into());
        }
        if self.exclusive_minimum {
            json.insert(
                "exclusiveMinimum".to_string(),
                self.exclusive_minimum.into(),
            );
        }
        if self.exclusive_maximum {
            json.insert(
                "exclusiveMaximum".to_string(),
                self.exclusive_minimum.into(),
            );
        }
        if let Some(multiple_of) = self.multiple_of {
            json.insert("multipleOf".to_string(), multiple_of.into());
        }
        json.into()
    }
}

impl ToJSONSchema for openapiv3::IntegerType {
    fn to_json_schema(&self) -> serde_json::Value {
        let mut json = serde_json::Map::new();
        json.insert("type".to_string(), serde_json::Value::from("integer"));
        if let Some(minimum) = self.minimum {
            json.insert("minimum".to_string(), minimum.into());
        }
        if let Some(maximum) = self.maximum {
            json.insert("maximum".to_string(), maximum.into());
        }
        if self.exclusive_minimum {
            json.insert(
                "exclusiveMinimum".to_string(),
                self.exclusive_minimum.into(),
            );
        }
        if self.exclusive_maximum {
            json.insert(
                "exclusiveMaximum".to_string(),
                self.exclusive_minimum.into(),
            );
        }
        if let Some(multiple_of) = self.multiple_of {
            json.insert("multipleOf".to_string(), multiple_of.into());
        }
        json.into()
    }
}

impl ToJSONSchema for openapiv3::ArrayType {
    fn to_json_schema(&self) -> serde_json::Value {
        let mut json = serde_json::Map::new();
        json.insert("type".to_string(), serde_json::Value::from("array"));
        if let Some(min_items) = self.min_items {
            json.insert("minItems".to_string(), min_items.into());
        }
        if let Some(max_items) = self.max_items {
            json.insert("maxItems".to_string(), max_items.into());
        }
        if self.unique_items {
            json.insert("uniqueItems".to_string(), self.unique_items.into());
        }
        if let Some(items) = &self.items {
            if let Some(schema) = &items.as_item() {
                json.insert("items".to_string(), schema.to_json_schema());
            }
        }
        json.into()
    }
}

impl ToJSONSchema for openapiv3::ObjectType {
    fn to_json_schema(&self) -> serde_json::Value {
        let mut json = serde_json::Map::new();
        json.insert("type".to_string(), serde_json::Value::from("object"));
        if let Some(min_properties) = self.min_properties {
            json.insert("minProperties".to_string(), min_properties.into());
        }
        if let Some(max_properties) = self.max_properties {
            json.insert("maxProperties".to_string(), max_properties.into());
        }
        if let Some(additional_properties) = &self.additional_properties {
            json.insert(
                "additionalProperties".to_string(),
                match additional_properties {
                    openapiv3::AdditionalProperties::Any(value) => value.to_owned().into(),
                    openapiv3::AdditionalProperties::Schema(schema) => {
                        schema.to_owned().as_item().unwrap().to_json_schema()
                    }
                },
            );
        }
        if !self.properties.is_empty() {
            let properties: serde_json::Map<_, _> = self
                .properties
                .iter()
                .map(|(key, value)| (key.to_string(), value.as_item().unwrap().to_json_schema()))
                .collect();
            json.insert("properties".to_string(), properties.into());
        }
        if !self.required.is_empty() {
            json.insert("required".to_string(), self.required.to_owned().into());
        }
        json.into()
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

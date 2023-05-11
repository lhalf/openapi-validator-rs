use openapiv3::Type;
use serde_json::json;

trait JSONSchema {
    fn to_json_schema(&self) -> serde_json::Value;
}

impl JSONSchema for openapiv3::Schema {
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
            openapiv3::SchemaKind::Type(Type::Array(array_schema)) => array_schema.to_json_schema(),
            _ => todo!(),
        }
    }
}

impl JSONSchema for openapiv3::StringType {
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

impl JSONSchema for openapiv3::NumberType {
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

impl JSONSchema for openapiv3::IntegerType {
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

impl JSONSchema for openapiv3::ArrayType {
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

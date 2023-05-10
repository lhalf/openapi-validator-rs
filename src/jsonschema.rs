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
                    json.insert("format".to_string(), serde_json::Value::from("date-time"));
                }
                _ => todo!(),
            }
        }
        json.into()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use openapiv3::StringType;

    #[test]
    fn boolean() {
        assert_eq!(
            openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(Type::Boolean {})
            }
            .to_json_schema(),
            json!({"type": "boolean"})
        )
    }

    #[test]
    fn string() {
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
    fn min_length_string() {
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
    fn min_and_max_length_string() {
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
    fn pattern_string() {
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
    fn format_string() {
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
}

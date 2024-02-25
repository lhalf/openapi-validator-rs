use jsonschema::JSONSchema;

pub trait JSONSchemaValidator {
    fn validates(&self, input: &str) -> Result<(), ()>;
}

impl JSONSchemaValidator for serde_json::Value {
    fn validates(&self, input: &str) -> Result<(), ()> {
        let json_parameter: serde_json::Value = serde_json::from_str(input).map_err(|_| ())?;

        let schema = JSONSchema::compile(&self).map_err(|_| ())?;

        if !schema.is_valid(&json_parameter) {
            return Err(());
        }

        Ok(())
    }
}

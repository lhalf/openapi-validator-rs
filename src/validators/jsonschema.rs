use jsonschema::JSONSchema;

pub trait JSONSchemaValidator {
    fn validates(&self, input: &str) -> Result<(), ()>;
}

impl JSONSchemaValidator for serde_json::Value {
    fn validates(&self, input: &str) -> Result<(), ()> {
        let json_parameter = match serde_json::from_slice::<serde_json::Value>(input.as_bytes()) {
            Ok(json_parameter) => json_parameter,
            Err(_) => return Err(()),
        };

        let schema = match JSONSchema::compile(&self) {
            Ok(schema) => schema,
            Err(_) => return Err(()),
        };

        if !schema.is_valid(&json_parameter) {
            return Err(());
        }

        Ok(())
    }
}

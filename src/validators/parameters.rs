use jsonschema::JSONSchema;

use super::content_type::ContentTypeValidator;
use super::request::Request;
use crate::jsonschema::ToJSONSchema;

pub struct ParametersValidator<'api> {
    pub operation_spec: &'api openapiv3::Operation,
    pub components: &'api Option<openapiv3::Components>,
}

impl<'api> ParametersValidator<'api> {
    pub fn validate_parameters(&self, request: &Request) -> Result<ContentTypeValidator, ()> {
        let all_parameters_valid = self.operation_spec.parameters.iter().all(|parameter| {
            parameter
                .as_item()
                .unwrap()
                .to_parameter_validator()
                .validate(request)
        });

        if !all_parameters_valid {
            return Err(());
        }

        Ok(ContentTypeValidator {
            operation_spec: self.operation_spec,
            components: self.components,
        })
    }
}

trait ToParameterValidator {
    fn to_parameter_validator(&self) -> ParameterValidator;
}

impl ToParameterValidator for openapiv3::Parameter {
    fn to_parameter_validator(&self) -> ParameterValidator {
        let parameter_data = match self {
            openapiv3::Parameter::Header { parameter_data, .. } => parameter_data,
            _ => todo!(),
        };

        match &parameter_data.format {
            openapiv3::ParameterSchemaOrContent::Schema(openapiv3::ReferenceOr::Item(schema)) => {
                ParameterValidator::Header {
                    jsonschema: schema.to_json_schema(),
                    name: parameter_data.name.clone(),
                    required: parameter_data.required,
                }
            }
            _ => todo!(),
        }
    }
}

enum ParameterValidator {
    Header {
        jsonschema: serde_json::Value,
        name: String,
        required: bool,
    },
}

impl ParameterValidator {
    fn validate(&self, request: &Request) -> bool {
        match self {
            ParameterValidator::Header {
                jsonschema,
                name,
                required,
            } => Self::validate_header_parameter(jsonschema, required, request.get_header(name)),
        }
    }

    fn validate_header_parameter(
        jsonschema: &serde_json::Value,
        required: &bool,
        header_value: Option<&str>,
    ) -> bool {
        let header_value = match header_value {
            None => return !*required,
            Some(header_value) => header_value,
        };

        let json_parameter =
            match serde_json::from_slice::<serde_json::Value>(header_value.as_bytes()) {
                Ok(json_parameter) => json_parameter,
                Err(_) => return false,
            };

        let schema = JSONSchema::compile(jsonschema).unwrap();

        schema.is_valid(&json_parameter)
    }
}

#[cfg(test)]
mod test_header_parameters {
    use crate::validators::request::make_validator_from_spec;
    use crate::validators::request::Request;
    use indoc::indoc;
    use std::collections::HashMap;

    #[test]
    fn reject_a_request_with_missing_header_parameter() {
        let path_spec = indoc!(
            r#"
            paths:
              /requires/header/parameter:
                post:
                  parameters:
                    - in: header
                      name: thing
                      required: true
                      schema:
                        type: boolean
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            url: "http://test.com/requires/header/parameter".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(request)
        );
    }

    #[test]
    fn reject_a_request_expecting_two_header_parameters() {
        let path_spec = indoc!(
            r#"
            paths:
              /requires/header/parameter:
                post:
                  parameters:
                    - in: header
                      name: thing
                      required: true
                      schema:
                        type: boolean
                    - in: header
                      name: another_thing
                      required: true
                      schema:
                        type: boolean
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            url: "http://test.com/requires/header/parameter".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::from([("thing".to_string(), "true".to_string())]),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(request)
        );
    }

    #[test]
    fn reject_a_request_with_invalid_header_parameter_type() {
        let path_spec = indoc!(
            r#"
            paths:
              /requires/header/parameter:
                post:
                  parameters:
                    - in: header
                      name: thing
                      required: true
                      schema:
                        type: boolean
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            url: "http://test.com/requires/header/parameter".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::from([("thing".to_string(), "1".to_string())]),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(request)
        );
    }

    #[test]
    fn reject_a_request_with_one_of_multiple_invalid_header_parameter_type() {
        let path_spec = indoc!(
            r#"
            paths:
              /requires/header/parameter:
                post:
                  parameters:
                    - in: header
                      name: thing
                      required: true
                      schema:
                        type: boolean
                    - in: header
                      name: another_thing
                      required: true
                      schema:
                        type: boolean
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            url: "http://test.com/requires/header/parameter".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::from([
                ("thing".to_string(), "true".to_string()),
                ("another_thing".to_string(), "1".to_string()),
            ]),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(request)
        );
    }

    #[test]
    fn accept_a_request_with_multiple_valid_header_parameters() {
        let path_spec = indoc!(
            r#"
            paths:
              /requires/header/parameter:
                post:
                  parameters:
                    - in: header
                      name: thing
                      required: true
                      schema:
                        type: boolean
                    - in: header
                      name: another_thing
                      required: true
                      schema:
                        type: integer
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            url: "http://test.com/requires/header/parameter".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::from([
                ("thing".to_string(), "true".to_string()),
                ("another_thing".to_string(), "1".to_string()),
            ]),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(request)
            .is_ok());
    }

    #[test]
    fn reject_a_request_with_non_json_header_parameter() {
        let path_spec = indoc!(
            r#"
            paths:
              /requires/header/parameter:
                post:
                  parameters:
                    - in: header
                      name: thing
                      required: true
                      schema:
                        type: string
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            url: "http://test.com/requires/header/parameter".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::from([("thing".to_string(), "p".to_string())]),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(request)
        );
    }
}
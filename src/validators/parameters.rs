use url::Url;

use super::content_type::ContentTypeValidator;
use super::request::Request;
use crate::to_jsonschema::ToJSONSchema;
use crate::validators::jsonschema::JSONSchemaValidator;

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
                .validate(request).unwrap_or(false)
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

trait ParameterValidator {
    fn validate(&self, request: &Request) -> Result<bool, ()>;
    fn validate_parameter(jsonschema: &serde_json::Value, required: &bool, parameter_value: Option<&str>) -> Result<bool, ()>;
}

impl ParameterValidator for openapiv3::Parameter {
    fn validate(&self, request: &Request) -> Result<bool, ()> {
        let parameter_data = self.clone().parameter_data();

        match &parameter_data.format {
            openapiv3::ParameterSchemaOrContent::Schema(openapiv3::ReferenceOr::Item(schema)) => {
                match self {
                    openapiv3::Parameter::Header { .. } => Self::validate_parameter(&schema.to_json_schema(),
                                                                                    &parameter_data.required,
                                                                                    request.get_header(&parameter_data.name)),
                    openapiv3::Parameter::Query { .. } => {
                        let url = Url::parse(request.url()).unwrap();
                        Self::validate_parameter(&schema.to_json_schema(),
                                                 &parameter_data.required,
                                                 url.extract_query_parameter(&parameter_data.name).as_deref())
                    }
                    _ => todo!()
                }
            }
            _ => todo!(),
        }
    }

    fn validate_parameter(
        jsonschema: &serde_json::Value,
        required: &bool,
        parameter_value: Option<&str>,
    ) -> Result<bool, ()> {
        let parameter_value = match parameter_value {
            None => return Ok(!*required),
            Some(parameter_value) => parameter_value,
        };

        jsonschema.validates(parameter_value)
    }
}

trait ExtractQueryParameter {
    fn extract_query_parameter(&self, name: &String) -> Option<String>;
}

impl ExtractQueryParameter for Url {
    fn extract_query_parameter(&self, name: &String) -> Option<String> {
        match self.query_pairs().find(|(key, ..)| key == name) {
            Some((.., value)) => Some(value.to_string()),
            None => None,
        }
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

#[cfg(test)]
mod test_query_parameters {
    use crate::validators::request::make_validator_from_spec;
    use crate::validators::request::Request;
    use indoc::indoc;
    use std::collections::HashMap;

    #[test]
    fn reject_a_request_with_missing_query_parameter() {
        let path_spec = indoc!(
            r#"
            paths:
              /requires/query/parameter:
                post:
                  parameters:
                    - in: query
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
            url: "http://test.com/requires/query/parameter".to_string(),
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
    fn reject_a_request_with_one_of_multiple_missing_query_parameter() {
        let path_spec = indoc!(
            r#"
            paths:
              /requires/two/query/parameter:
                post:
                  parameters:
                    - in: query
                      name: thing
                      required: true
                      schema:
                        type: boolean
                    - in: query
                      name: another
                      required: true
                      schema:
                        type: boolean
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            url: "http://test.com/requires/two/query/parameter?thing=true".to_string(),
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
    fn reject_a_request_with_wrong_query_parameter_type() {
        let path_spec = indoc!(
            r#"
            paths:
              /requires/query/parameter:
                post:
                  parameters:
                    - in: query
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
            url: "http://test.com/requires/query/parameter?thing=string".to_string(),
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
    fn accept_a_request_with_valid_query_parameter() {
        let path_spec = indoc!(
            r#"
            paths:
              /requires/query/parameter:
                post:
                  parameters:
                    - in: query
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
            url: "http://test.com/requires/query/parameter?thing=true".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(request)
            .is_ok());
    }

    #[test]
    fn accept_a_request_with_multiple_valid_query_parameter() {
        let path_spec = indoc!(
            r#"
            paths:
              /requires/multiple/query/parameter:
                post:
                  parameters:
                    - in: query
                      name: thing
                      required: true
                      schema:
                        type: boolean
                    - in: query
                      name: another
                      required: true
                      schema:
                        type: string
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            url: "http://test.com/requires/multiple/query/parameter?thing=true&another=\"cheese\"".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(request)
            .is_ok());
    }

    #[test]
    fn reject_a_request_with_non_json_query_parameter() {
        let path_spec = indoc!(
            r#"
            paths:
              /requires/multiple/query/parameter:
                post:
                  parameters:
                    - in: query
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
            url: "http://test.com/requires/multiple/query/parameter?another=cheese".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(request)
        );
    }
}
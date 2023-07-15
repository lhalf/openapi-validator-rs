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
            openapiv3::Parameter::Query { parameter_data, .. } => parameter_data,
            _ => todo!(),
        };

        match &parameter_data.format {
            openapiv3::ParameterSchemaOrContent::Schema(openapiv3::ReferenceOr::Item(schema)) => {
                match self {
                    openapiv3::Parameter::Header { .. } => ParameterValidator::Header {
                        jsonschema: schema.to_json_schema(),
                        name: parameter_data.name.clone(),
                        required: parameter_data.required,
                    },
                    openapiv3::Parameter::Query { .. } => ParameterValidator::Query {
                        jsonschema: schema.to_json_schema(),
                        name: parameter_data.name.clone(),
                        required: parameter_data.required,
                    },
                    _ => todo!()
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
    Query {
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
            ParameterValidator::Query {
                jsonschema,
                name,
                required,
            } => {
                let url = Url::parse(request.url()).unwrap();
                Self::validate_query_parameter(jsonschema, required, Self::extract_query_parameter_from_url(&url, name).as_deref())
            }
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

        match jsonschema.validates(header_value) {
            Ok(result) => result,
            Err(..) => false
        }
    }

    fn extract_query_parameter_from_url(url: &Url, name: &String) -> Option<String> {
        match url.query_pairs().find(|(key, ..)| key == name) {
            Some((.., value)) => Some(value.to_string()),
            None => None,
        }
    }

    fn validate_query_parameter(
        jsonschema: &serde_json::Value,
        required: &bool,
        query_value: Option<&str>,
    ) -> bool {
        let query_value = match query_value {
            None => return !*required,
            Some(query_value) => query_value,
        };

        match jsonschema.validates(query_value) {
            Ok(result) => result,
            Err(..) => false
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
    fn reject_a_request_with_invalid_query_parameter_type() {
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
}
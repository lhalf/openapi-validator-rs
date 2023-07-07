use crate::jsonschema::ToJSONSchema;
use jsonschema::JSONSchema;
use std::collections::HashMap;
use std::ops::Index;

struct Validator {
    api: openapiv3::OpenAPI,
}

#[allow(dead_code)]
impl Validator {
    fn new(api: openapiv3::OpenAPI) -> Self {
        Self { api }
    }

    //take &self rather than self otherwise Validator is consumed by validate_request (dropped)
    fn validate_request(&self, request: Request) -> Result<Request, ()> {
        self.validate_path(request.path())?
            .validate_operation(request.operation())?
            .validate_parameters(request.get_header("thing"))?
            .validate_content_type(request.get_header("Content-Type"))?
            .validate_body(request.body())?;
        Ok(request)
    }

    fn validate_path(&self, path: &str) -> Result<ValidatedPath, ()> {
        if let Some(path_spec) = self
            .api
            .paths
            .paths
            .get(path)
            .and_then(openapiv3::ReferenceOr::as_item)
        {
            return Ok(ValidatedPath {
                path_spec,
                components: &self.api.components,
            });
        }
        Err(())
    }
}

struct ValidatedPath<'api> {
    path_spec: &'api openapiv3::PathItem,
    components: &'api Option<openapiv3::Components>,
}

impl<'api> ValidatedPath<'api> {
    fn validate_operation(&self, operation: &str) -> Result<ValidatedOperation, ()> {
        let operation_spec = match operation {
            "get" => self.path_spec.get.as_ref().ok_or(()),
            "put" => self.path_spec.put.as_ref().ok_or(()),
            "delete" => self.path_spec.delete.as_ref().ok_or(()),
            "post" => self.path_spec.post.as_ref().ok_or(()),
            _ => Err(()),
        }?;
        Ok(ValidatedOperation {
            operation_spec,
            components: self.components,
        })
    }
}

struct ValidatedOperation<'api> {
    operation_spec: &'api openapiv3::Operation,
    components: &'api Option<openapiv3::Components>,
}

impl<'api> ValidatedOperation<'api> {
    fn validate_parameters(&self, header_value: Option<&str>) -> Result<ValidatedParameters, ()> {
        let thing_header_required = self
            .operation_spec
            .parameters
            .iter()
            .map(|parameter| parameter.as_item().unwrap())
            .filter_map(|parameter| match parameter {
                openapiv3::Parameter::Header { parameter_data, .. } => Some(parameter_data),
                _ => None,
            })
            .any(|parameter_data| parameter_data.name == "thing" && parameter_data.required);

        if thing_header_required && header_value.is_none() {
            return Err(());
        }

        Ok(ValidatedParameters {
            operation_spec: self.operation_spec,
            components: self.components,
        })
    }
}

struct ValidatedParameters<'api> {
    operation_spec: &'api openapiv3::Operation,
    components: &'api Option<openapiv3::Components>,
}

impl<'api> ValidatedParameters<'api> {
    fn validate_content_type(
        &self,
        content_type: Option<&str>,
    ) -> Result<ValidatedContentType, ()> {
        let body_spec = match self
            .operation_spec
            .request_body
            .as_ref()
            .and_then(openapiv3::ReferenceOr::as_item)
        {
            Some(body_spec) => body_spec,
            None => return Ok(ValidatedContentType::NoSpecification),
        };

        let content_type = match content_type {
            Some(content_type) => content_type,
            _ => return Ok(ValidatedContentType::EmptyContentType { body_spec }),
        };

        if !body_spec.content.contains_key(content_type) {
            return Err(());
        }

        match content_type {
            "application/json" => Ok(ValidatedContentType::JSONBody {
                body_spec,
                components: self.components,
            }),
            "text/plain; charset=utf-8" => Ok(ValidatedContentType::PlainUTF8Body),
            _ => Err(()),
        }
    }
}

enum ValidatedContentType<'api> {
    NoSpecification,
    EmptyContentType {
        body_spec: &'api openapiv3::RequestBody,
    },
    JSONBody {
        body_spec: &'api openapiv3::RequestBody,
        components: &'api Option<openapiv3::Components>,
    },
    PlainUTF8Body,
}

impl<'api> ValidatedContentType<'api> {
    fn validate_body(&self, body: &[u8]) -> Result<(), ()> {
        match self {
            Self::JSONBody {
                body_spec,
                components,
            } => {
                if let Some(body_schema) = body_spec.content["application/json"]
                    .schema
                    .as_ref()
                    .map(|reference_or| reference_or.item_or_fetch(components))
                {
                    return validate_json_body(body_schema, body);
                }

                if serde_json::from_slice::<serde_json::Value>(body).is_ok() {
                    return Ok(());
                }

                Err(())
            }
            Self::PlainUTF8Body { .. } => match std::str::from_utf8(body) {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            },
            Self::EmptyContentType { body_spec } => match !body_spec.required && body.is_empty() {
                true => Ok(()),
                false => Err(()),
            },
            Self::NoSpecification => Ok(()),
        }
    }
}

fn validate_json_body(schema: &openapiv3::Schema, body: &[u8]) -> Result<(), ()> {
    let json_body = serde_json::from_slice::<serde_json::Value>(body).or(Err(()))?;

    let schema = JSONSchema::compile(&schema.clone().to_json_schema()).or(Err(()))?;

    if schema.is_valid(&json_body) {
        return Ok(());
    }

    Err(())
}

trait ItemOrFetch<T> {
    fn item_or_fetch<'api>(&'api self, components: &'api Option<openapiv3::Components>) -> &T;
}

impl ItemOrFetch<openapiv3::Schema> for openapiv3::ReferenceOr<openapiv3::Schema> {
    fn item_or_fetch<'api>(
        &'api self,
        components: &'api Option<openapiv3::Components>,
    ) -> &openapiv3::Schema {
        match self {
            Self::Item(item) => item,
            Self::Reference { reference } => components
                .as_ref()
                .unwrap()
                .schemas
                .index(reference.trim_start_matches("#/components/schemas/"))
                .item_or_fetch(components),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Request {
    path: String,
    operation: String,
    body: Vec<u8>,
    headers: HashMap<String, String>,
}

impl Request {
    fn path(&self) -> &str {
        &self.path
    }

    fn operation(&self) -> &str {
        &self.operation
    }

    fn body(&self) -> &[u8] {
        &self.body
    }

    fn get_header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).map(String::as_str)
    }
}

#[cfg(test)]
fn make_validator_from_spec(path_spec: &str) -> Validator {
    let openapi = indoc::indoc!(
        r#"
            openapi: 3.0.0
            info:
                description: API to handle generic two-way HTTP requests
                version: "1.0.0"
                title: Swagger ReST Article
            "#
    )
    .to_string()
        + path_spec;
    Validator::new(serde_yaml::from_str(&openapi).unwrap())
}

#[cfg(test)]
fn make_validator() -> Validator {
    let spec: String = std::fs::read_to_string("./specs/openapi.yaml").unwrap();
    let api: openapiv3::OpenAPI = serde_yaml::from_str(&spec).unwrap();
    Validator::new(api)
}

#[cfg(test)]
mod test_path {
    use crate::validator::Request;
    use crate::validator::{make_validator, make_validator_from_spec};
    use indoc::indoc;
    use std::collections::HashMap;

    #[test]
    fn accept_a_request_with_valid_path() {
        let validator = make_validator();
        let request = Request {
            path: "/ping".to_string(),
            operation: "get".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert!(validator.validate_request(request).is_ok());
    }

    #[test]
    fn reject_a_request_with_invalid_path() {
        let path_spec = indoc!(
            r#"
           paths:
             /ping:
               get:
                 summary: Ping
                 responses:
                   200:
                     description: API call successful
           "#
        );
        let request = Request {
            path: "/invalid/path".to_string(),
            operation: "get".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(request)
        );
    }
}

#[cfg(test)]
mod test_parameters {
    use crate::validator::make_validator_from_spec;
    use crate::validator::Request;
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
                        type: bool
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            path: "/requires/header/parameter".to_string(),
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

#[cfg(test)]
mod test_operations {
    use crate::validator::make_validator_from_spec;
    use crate::validator::Request;
    use indoc::indoc;
    use std::collections::HashMap;

    #[test]
    fn accept_a_request_with_put_operation() {
        let path_spec = indoc!(
            r#"
            paths:
              /allowed/put:
                put:
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            path: "/allowed/put".to_string(),
            operation: "put".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(request)
            .is_ok());
    }

    #[test]
    fn accept_a_request_with_post_operation() {
        let path_spec = indoc!(
            r#"
            paths:
              /allowed/post:
                post:
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            path: "/allowed/post".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(request)
            .is_ok());
    }

    #[test]
    fn accept_a_request_with_delete_operation() {
        let path_spec = indoc!(
            r#"
            paths:
              /allowed/delete:
                delete:
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            path: "/allowed/delete".to_string(),
            operation: "delete".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(request)
            .is_ok());
    }
}

#[cfg(test)]
mod test_headers {
    use crate::validator::make_validator_from_spec;
    use crate::validator::Request;
    use indoc::indoc;
    use std::collections::HashMap;

    #[test]
    fn reject_a_request_where_body_required_and_content_type_in_header_but_not_in_spec() {
        let path_spec = indoc!(
            r#"
            paths:
              /required/body:
                post:
                  summary: Requires a body
                  requestBody:
                    required: true
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            path: "/required/body".to_string(),
            operation: "post".to_string(),
            body: "babe".as_bytes().to_vec(),
            headers: HashMap::from([(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            )]),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(request)
        );
    }

    #[test]
    fn reject_a_request_where_body_is_optional_but_specified_content_type_is_not_in_spec() {
        let path_spec = indoc!(
            r#"
            paths:
              /not/required/body:
                post:
                  summary: Requires a body
                  requestBody:
                    required: false
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            path: "/not/required/body".to_string(),
            operation: "post".to_string(),
            body: "babe".as_bytes().to_vec(),
            headers: HashMap::from([(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            )]),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(request)
        );
    }

    #[test]
    fn select_which_content_to_validate_given_content_type_header_invalid_case() {
        let path_spec = indoc!(
            r#"
            paths:
              /allows/utf8/or/json/body:
                post:
                  summary: Requires a JSON body
                  requestBody:
                    required: true
                    content:
                      application/json:
                        schema:
                      text/plain; charset=utf-8:
                        schema:
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            path: "/allows/utf8/or/json/body".to_string(),
            operation: "post".to_string(),
            body: "ab".as_bytes().to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(request)
        );
    }

    #[test]
    fn select_which_content_to_validate_given_content_type_header_valid_case() {
        let path_spec = indoc!(
            r#"
            paths:
              /allows/utf8/or/json/body:
                post:
                  summary: Requires a JSON body
                  requestBody:
                    required: true
                    content:
                      application/json:
                        schema:
                      text/plain; charset=utf-8:
                        schema:
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            path: "/allows/utf8/or/json/body".to_string(),
            operation: "post".to_string(),
            body: "ab".as_bytes().to_vec(),
            headers: HashMap::from([(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            )]),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(request)
            .is_ok());
    }
}

#[cfg(test)]
mod test_body {
    use crate::validator::make_validator_from_spec;
    use crate::validator::Request;
    use indoc::indoc;
    use std::collections::HashMap;

    #[test]
    fn reject_a_request_with_no_body_if_required() {
        let path_spec = indoc!(
            r#"
            paths:
              /required/body:
                post:
                  summary: Requires a body
                  requestBody:
                    required: true
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            path: "/required/body".to_string(),
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
    fn accept_a_request_with_no_body_if_not_required() {
        let path_spec = indoc!(
            r#"
            paths:
              /not/required/body:
                post:
                  summary: Requires a body
                  requestBody:
                    required: false
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            path: "/not/required/body".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(request)
            .is_ok());
    }

    #[test]
    fn accept_a_request_with_a_json_body_if_required() {
        let path_spec = indoc!(
            r#"
            paths:
              /required/json/body:
                post:
                  summary: Requires a body
                  requestBody:
                    required: true
                    content:
                      application/json:
                        schema:
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            path: "/required/json/body".to_string(),
            operation: "post".to_string(),
            body: "{}".as_bytes().to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(request)
            .is_ok());
    }

    #[test]
    fn reject_a_request_with_invalid_json_body_if_required() {
        let path_spec = indoc!(
            r#"
            paths:
              /required/json/body:
                post:
                  summary: Requires a body
                  requestBody:
                    required: true
                    content:
                      application/json:
                        schema:
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            path: "/required/json/body".to_string(),
            operation: "post".to_string(),
            body: "babe".as_bytes().to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(request)
        );
    }

    #[test]
    fn accept_a_request_with_valid_utf8_body_if_required() {
        let path_spec = indoc!(
            r#"
            paths:
              /required/utf8/body:
                post:
                  summary: Requires a JSON body
                  requestBody:
                    required: true
                    content:
                      text/plain; charset=utf-8:
                        schema:
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            path: "/required/utf8/body".to_string(),
            operation: "post".to_string(),
            body: "ab".as_bytes().to_vec(),
            headers: HashMap::from([(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            )]),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(request)
            .is_ok());
    }

    #[test]
    fn reject_a_request_with_invalid_utf8_body_if_required() {
        let path_spec = indoc!(
            r#"
            paths:
              /required/utf8/body:
                post:
                  summary: Requires a JSON body
                  requestBody:
                    required: true
                    content:
                      text/plain; charset=utf-8:
                        schema:
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            path: "/required/utf8/body".to_string(),
            operation: "post".to_string(),
            body: vec![b'\xc3', b'\x28'],
            headers: HashMap::from([(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            )]),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(request)
        );
    }

    #[test]
    fn reject_a_json_body_given_a_schema() {
        let path_spec = indoc!(
            r#"
            paths:
              /rejects/invalid/json/against/schema:
                post:
                  summary: Requires a JSON body
                  requestBody:
                    required: true
                    content:
                      application/json:
                        schema:
                          type: object
                          required:
                            - key
                          properties:
                            key:
                              type: string
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            path: "/rejects/invalid/json/against/schema".to_string(),
            operation: "post".to_string(),
            body: r#"{"not key": "value"}"#.as_bytes().to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(request)
        );
    }

    #[test]
    fn accept_a_valid_json_body_given_a_schema() {
        let path_spec = indoc!(
            r#"
            paths:
              /json/against/schema:
                post:
                  summary: Requires a JSON body
                  requestBody:
                    required: true
                    content:
                      application/json:
                        schema:
                          type: object
                          required:
                            - name
                            - count
                            - date
                          properties:
                            name:
                              type: string
                            count:
                              type: integer
                            date:
                              type: string
                              format: date
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            path: "/json/against/schema".to_string(),
            operation: "post".to_string(),
            body: r#"{"name": "laurence", "count": 10, "date": "2023-05-11"}"#
                .as_bytes()
                .to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(request)
            .is_ok());
    }

    #[test]
    fn accept_a_valid_json_body_given_component_schema_reference() {
        let path_spec = indoc!(
            r#"
            paths:
              /json/against/schema:
                post:
                  summary: Requires a JSON body
                  requestBody:
                    required: true
                    content:
                      application/json:
                        schema:
                          $ref: '#/components/schemas/Test'
                  responses:
                    200:
                      description: API call successful
            
            components:
              schemas:
                Test:
                  type: boolean
            "#
        );
        let request = Request {
            path: "/json/against/schema".to_string(),
            operation: "post".to_string(),
            body: r#"true"#.as_bytes().to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(request)
            .is_ok());
    }

    #[test]
    fn accept_a_valid_json_body_given_component_schema_nested_reference() {
        let path_spec = indoc!(
            r#"
            paths:
              /json/against/schema:
                post:
                  summary: Requires a JSON body
                  requestBody:
                    required: true
                    content:
                      application/json:
                        schema:
                          $ref: '#/components/schemas/Test'
                  responses:
                    200:
                      description: API call successful
            
            components:
              schemas:
                Test:
                  $ref: '#/components/schemas/Next'
                Next:
                  type: boolean
            "#
        );
        let request = Request {
            path: "/json/against/schema".to_string(),
            operation: "post".to_string(),
            body: r#"true"#.as_bytes().to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(request)
            .is_ok());
    }

    #[test]
    #[should_panic]
    fn reject_given_component_schema_reference_with_incorrect_reference_panics() {
        let path_spec = indoc!(
            r#"
            paths:
              /json/against/schema:
                post:
                  summary: Requires a JSON body
                  requestBody:
                    required: true
                    content:
                      application/json:
                        schema:
                          $ref: '#/components/schemas/NotThere'
                  responses:
                    200:
                      description: API call successful
            
            components:
              schemas:
                There:
                  type: boolean
            "#
        );
        let request = Request {
            path: "/json/against/schema".to_string(),
            operation: "post".to_string(),
            body: r#"true"#.as_bytes().to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        let _ = make_validator_from_spec(path_spec).validate_request(request);
    }
}

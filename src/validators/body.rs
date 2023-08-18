use crate::item_or_fetch::ItemOrFetch;
use crate::to_jsonschema::ToJSONSchema;
use crate::validators::jsonschema::JSONSchemaValidator;
use crate::validators::response::ResponseValidator;

pub enum BodyValidator<'api> {
    NoSpecification {
        response_spec: &'api openapiv3::Responses,
        components: &'api Option<openapiv3::Components>,
    },
    EmptyContentType {
        body_spec: &'api openapiv3::RequestBody,
        components: &'api Option<openapiv3::Components>,
        response_spec: &'api openapiv3::Responses,
    },
    JSONBody {
        body_spec: &'api openapiv3::RequestBody,
        components: &'api Option<openapiv3::Components>,
        response_spec: &'api openapiv3::Responses,
    },
    PlainUTF8Body {
        response_spec: &'api openapiv3::Responses,
        components: &'api Option<openapiv3::Components>,
    },
}

impl<'api> BodyValidator<'api> {
    pub fn validate_body(self, body: &[u8]) -> Result<ResponseValidator<'api>, ()> {
        match self {
            Self::JSONBody {
                body_spec,
                components,
                response_spec,
            } => {
                if let Some(body_schema) =
                    body_spec
                        .content
                        .get("application/json")
                        .and_then(|content| {
                            content
                                .schema
                                .as_ref()
                                .map(|schema| schema.item_or_fetch(components))
                        })
                {
                    if validate_json_body(body_schema, body).is_ok() {
                        return Ok(ResponseValidator {
                            response_spec,
                            components,
                        });
                    }
                    return Err(());
                }

                if serde_json::from_slice::<serde_json::Value>(body).is_ok() {
                    return Ok(ResponseValidator {
                        response_spec,
                        components,
                    });
                }

                Err(())
            }
            Self::PlainUTF8Body {
                response_spec,
                components,
            } => match std::str::from_utf8(body) {
                Ok(_) => Ok(ResponseValidator {
                    response_spec,
                    components,
                }),
                Err(_) => Err(()),
            },
            Self::EmptyContentType {
                body_spec,
                response_spec,
                components,
            } => {
                if !body_spec.required && body.is_empty() {
                    Ok(ResponseValidator {
                        response_spec,
                        components,
                    })
                } else {
                    Err(())
                }
            }
            Self::NoSpecification {
                response_spec,
                components,
            } => Ok(ResponseValidator {
                response_spec,
                components,
            }),
        }
    }
}

fn validate_json_body(schema: &openapiv3::Schema, body: &[u8]) -> Result<(), ()> {
    let body = match std::str::from_utf8(body) {
        Ok(body) => body,
        Err(..) => return Err(()),
    };

    schema.clone().to_json_schema().validates(body)
}

#[cfg(test)]
mod test_body {
    use crate::validators::request::test_helpers::*;
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
        let request = FakeRequest {
            url: "http://test.com/required/body".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(&request)
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
        let request = FakeRequest {
            url: "http://test.com/not/required/body".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(&request)
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
        let request = FakeRequest {
            url: "http://test.com/required/json/body".to_string(),
            operation: "post".to_string(),
            body: "{}".as_bytes().to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(&request)
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
        let request = FakeRequest {
            url: "http://test.com/required/json/body".to_string(),
            operation: "post".to_string(),
            body: "babe".as_bytes().to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(&request)
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
        let request = FakeRequest {
            url: "http://test.com/required/utf8/body".to_string(),
            operation: "post".to_string(),
            body: "ab".as_bytes().to_vec(),
            headers: HashMap::from([(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            )]),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(&request)
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
        let request = FakeRequest {
            url: "http://test.com/required/utf8/body".to_string(),
            operation: "post".to_string(),
            body: vec![b'\xc3', b'\x28'],
            headers: HashMap::from([(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            )]),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(&request)
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
        let request = FakeRequest {
            url: "http://test.com/rejects/invalid/json/against/schema".to_string(),
            operation: "post".to_string(),
            body: r#"{"not key": "value"}"#.as_bytes().to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(&request)
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
        let request = FakeRequest {
            url: "http://test.com/json/against/schema".to_string(),
            operation: "post".to_string(),
            body: r#"{"name": "laurence", "count": 10, "date": "2023-05-11"}"#
                .as_bytes()
                .to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(&request)
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
        let request = FakeRequest {
            url: "http://test.com/json/against/schema".to_string(),
            operation: "post".to_string(),
            body: r#"true"#.as_bytes().to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(&request)
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
        let request = FakeRequest {
            url: "http://test.com/json/against/schema".to_string(),
            operation: "post".to_string(),
            body: r#"true"#.as_bytes().to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(&request)
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
        let request = FakeRequest {
            url: "http://test.com/json/against/schema".to_string(),
            operation: "post".to_string(),
            body: r#"true"#.as_bytes().to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        let _ = make_validator_from_spec(path_spec).validate_request(&request);
    }

    #[test]
    fn accept_a_valid_json_body_given_a_request_body_reference() {
        let path_spec = indoc!(
            r#"
            paths:
              /body/against/schema:
                post:
                  summary: Requires a body through a reference
                  requestBody:
                    $ref: '#/components/requestBodies/Test'
                  responses:
                    200:
                      description: API call successful
            
            components:
              requestBodies:
                Test:
                  required: true
                  content:
                    application/json:
                      schema:
                        type: boolean
            "#
        );
        let request = FakeRequest {
            url: "http://test.com/body/against/schema".to_string(),
            operation: "post".to_string(),
            body: r#"true"#.as_bytes().to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(&request)
            .is_ok());
    }
}

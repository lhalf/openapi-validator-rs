use super::body::BodyValidator;
use crate::item_or_fetch::ItemOrFetch;

pub struct ContentTypeValidator<'api> {
    pub operation_spec: &'api openapiv3::Operation,
    pub components: &'api Option<openapiv3::Components>,
}

impl<'api> ContentTypeValidator<'api> {
    pub fn validate_content_type(
        self,
        content_type: Option<String>,
    ) -> Result<BodyValidator<'api>, ()> {
        let body_spec = match &self.operation_spec.request_body {
            Some(body_spec) => body_spec.item_or_fetch(self.components),
            None => return Ok(BodyValidator::NoSpecification),
        };

        match content_type {
            Some(content_type) if body_spec.content.contains_key(&content_type) => {
                match content_type.as_str() {
                    "application/json" => Ok(BodyValidator::JSONBody {
                        body_spec,
                        components: self.components,
                    }),
                    "text/plain; charset=utf-8" => Ok(BodyValidator::PlainUTF8Body),
                    _ => Err(()),
                }
            }
            Some(_) => Err(()),
            None => Ok(BodyValidator::EmptyContentType { body_spec }),
        }
    }
}

#[cfg(test)]
mod test_content_type {
    use crate::request::test_helpers::*;
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
        let request = FakeRequest {
            url: "http://test.com/required/body".to_string(),
            operation: "post".to_string(),
            body: "babe".as_bytes().to_vec(),
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
        let request = FakeRequest {
            url: "http://test.com/not/required/body".to_string(),
            operation: "post".to_string(),
            body: "babe".as_bytes().to_vec(),
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
        let request = FakeRequest {
            url: "http://test.com/allows/utf8/or/json/body".to_string(),
            operation: "post".to_string(),
            body: "ab".as_bytes().to_vec(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(&request)
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
        let request = FakeRequest {
            url: "http://test.com/allows/utf8/or/json/body".to_string(),
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
}

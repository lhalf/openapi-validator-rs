use super::parameters::ParametersValidator;
use crate::validators::content_type::ContentTypeValidator;
use crate::validators::request::Request;
use crate::validators::response::ResponseValidator;
use std::collections::HashMap;

pub struct OperationValidator<'api, 'request> {
    pub path_spec: &'api openapiv3::PathItem,
    pub components: &'api Option<openapiv3::Components>,
    pub path_parameters: HashMap<&'api str, &'request str>,
}

impl<'api, 'request> OperationValidator<'api, 'request> {
    pub fn validate_operation(self, request: &dyn Request) -> Result<ResponseValidator<'api>, ()> {
        let operation_spec = match request.operation() {
            "get" => self.path_spec.get.as_ref().ok_or(()),
            "put" => self.path_spec.put.as_ref().ok_or(()),
            "delete" => self.path_spec.delete.as_ref().ok_or(()),
            "post" => self.path_spec.post.as_ref().ok_or(()),
            _ => Err(()),
        }?;

        ParametersValidator {
            operation_spec,
            components: self.components,
            path_parameters: self.path_parameters,
        }
        .validate_parameters(request)?;

        ContentTypeValidator {
            operation_spec,
            components: self.components,
        }
        .validate_content_type(request.get_header("Content-Type"))?
        .validate_body(request.body())?;

        Ok(ResponseValidator {
            response_spec: &operation_spec.responses,
            components: self.components,
        })
    }
}

#[cfg(test)]
mod test_operations {
    use crate::validators::request::test_helpers::*;
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
        let request = FakeRequest {
            url: "http://test.com/allowed/put".to_string(),
            operation: "put".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(&request)
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
        let request = FakeRequest {
            url: "http://test.com/allowed/post".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(&request)
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
        let request = FakeRequest {
            url: "http://test.com/allowed/delete".to_string(),
            operation: "delete".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(&request)
            .is_ok());
    }
}

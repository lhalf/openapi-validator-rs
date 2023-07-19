use super::parameters::ParametersValidator;
use std::collections::HashMap;

pub struct OperationValidator<'api> {
    pub path_spec: &'api openapiv3::PathItem,
    pub components: &'api Option<openapiv3::Components>,
    pub path_parameters: HashMap<String, String>,
}

impl<'api> OperationValidator<'api> {
    pub fn validate_operation(self, operation: &str) -> Result<ParametersValidator<'api>, ()> {
        let operation_spec = match operation {
            "get" => self.path_spec.get.as_ref().ok_or(()),
            "put" => self.path_spec.put.as_ref().ok_or(()),
            "delete" => self.path_spec.delete.as_ref().ok_or(()),
            "post" => self.path_spec.post.as_ref().ok_or(()),
            _ => Err(()),
        }?;
        Ok(ParametersValidator {
            operation_spec,
            components: self.components,
            path_parameters: self.path_parameters,
        })
    }
}

#[cfg(test)]
mod test_operations {
    use crate::validators::request::make_validator_from_spec;
    use crate::validators::request::Request;
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
            url: "http://test.com/allowed/put".to_string(),
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
            url: "http://test.com/allowed/post".to_string(),
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
            url: "http://test.com/allowed/delete".to_string(),
            operation: "delete".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert!(make_validator_from_spec(path_spec)
            .validate_request(request)
            .is_ok());
    }
}

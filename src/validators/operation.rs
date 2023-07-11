use super::parameters::ParametersValidator;

pub struct OperationValidator<'api> {
    pub path_spec: &'api openapiv3::PathItem,
    pub components: &'api Option<openapiv3::Components>,
}

impl<'api> OperationValidator<'api> {
    pub fn validate_operation(&self, operation: &str) -> Result<ParametersValidator, ()> {
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
        })
    }
}

#[cfg(test)]
mod test_paths {
    use crate::validators::request::Request;
    use crate::validators::request::{make_validator, make_validator_from_spec};
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

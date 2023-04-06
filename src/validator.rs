use std::collections::HashMap;

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
            .validate_body(request.body(), request.get_header("Content-Type"))?;
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
            return Ok(ValidatedPath { path_spec });
        }
        Err(())
    }
}

struct ValidatedPath<'path> {
    path_spec: &'path openapiv3::PathItem,
}

impl<'path> ValidatedPath<'path> {
    fn validate_operation(&self, operation: &str) -> Result<ValidatedOperation, ()> {
        let operation_spec = match operation {
            "get" => self.path_spec.get.as_ref().ok_or(()),
            "put" => self.path_spec.put.as_ref().ok_or(()),
            "delete" => self.path_spec.delete.as_ref().ok_or(()),
            "post" => self.path_spec.post.as_ref().ok_or(()),
            _ => Err(()),
        }?;
        Ok(ValidatedOperation { operation_spec })
    }
}

struct ValidatedOperation<'operation> {
    operation_spec: &'operation openapiv3::Operation,
}

impl<'operation> ValidatedOperation<'operation> {
    fn validate_body(&self, body: &[u8], content_type: Option<&str>) -> Result<(), ()> {
        let body_spec = match self
            .operation_spec
            .request_body
            .as_ref()
            .and_then(openapiv3::ReferenceOr::as_item)
        {
            Some(body_spec) => body_spec,
            None => return Ok(()),
        };

        if body_spec.required && body.is_empty() {
            return Err(());
        }

        if !body_spec.required && body.is_empty() {
            return Ok(());
        }

        let content_type = match content_type {
            Some(content_type) if !body.is_empty() => content_type,
            _ => return Err(()),
        };

        if !body_spec.content.contains_key(content_type) {
            return Err(());
        }

        return match content_type {
            "application/json" => match serde_json::from_slice::<serde_json::Value>(body) {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            },
            "text/plain; charset=utf-8" => match std::str::from_utf8(body) {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            },
            _ => Err(()),
        };
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
mod test {
    use crate::validator::{Request, Validator};
    use std::collections::HashMap;

    fn make_validator() -> Validator {
        let spec: String = std::fs::read_to_string("./specs/openapi.yaml").unwrap();
        let api: openapiv3::OpenAPI = serde_yaml::from_str(&spec).unwrap();
        Validator::new(api)
    }

    #[test]
    fn validator_can_be_built_with_spec() {
        make_validator();
    }

    #[test]
    fn validator_can_accept_a_request_with_valid_path() {
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
    fn validator_can_reject_a_request_with_invalid_path() {
        let validator = make_validator();
        let request = Request {
            path: "/not/ping".to_string(),
            operation: "get".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert_eq!(Err(()), validator.validate_request(request));
    }

    #[test]
    fn validator_can_accept_a_request_with_put_operation() {
        let validator = make_validator();
        let request = Request {
            path: "/multiple/allowed/operations".to_string(),
            operation: "put".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert!(validator.validate_request(request).is_ok());
    }

    #[test]
    fn validator_can_accept_a_request_with_post_operation() {
        let validator = make_validator();
        let request = Request {
            path: "/multiple/allowed/operations".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert!(validator.validate_request(request).is_ok());
    }

    #[test]
    fn validator_can_accept_a_request_with_delete_operation() {
        let validator = make_validator();
        let request = Request {
            path: "/multiple/allowed/operations".to_string(),
            operation: "delete".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert!(validator.validate_request(request).is_ok());
    }

    #[test]
    fn validator_can_reject_a_request_with_no_body_if_required() {
        let validator = make_validator();
        let request = Request {
            path: "/required/body".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert_eq!(Err(()), validator.validate_request(request));
    }

    #[test]
    fn validator_can_reject_a_request_where_body_required_and_content_type_in_header_but_not_in_spec(
    ) {
        let validator = make_validator();
        let request = Request {
            path: "/required/body".to_string(),
            operation: "post".to_string(),
            body: vec![b'b', b'a', b'b', b'e'],
            headers: HashMap::from([(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            )]),
        };
        assert_eq!(Err(()), validator.validate_request(request));
    }

    #[test]
    fn validator_can_accept_a_request_with_no_body_if_not_required() {
        let validator = make_validator();
        let request = Request {
            path: "/not/required/body".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert!(validator.validate_request(request).is_ok());
    }

    #[test]
    fn validator_can_reject_a_request_where_body_is_optional_but_specified_content_type_is_not_in_spec(
    ) {
        let validator = make_validator();
        let request = Request {
            path: "/not/required/body".to_string(),
            operation: "post".to_string(),
            body: vec![b'b', b'a', b'b', b'e'],
            headers: HashMap::from([(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            )]),
        };
        assert_eq!(Err(()), validator.validate_request(request));
    }

    #[test]
    fn validator_can_accept_a_request_with_a_json_body_if_required() {
        let validator = make_validator();
        let request = Request {
            path: "/required/json/body".to_string(),
            operation: "post".to_string(),
            body: vec![b'{', b'}'],
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert!(validator.validate_request(request).is_ok());
    }

    #[test]
    fn validator_can_reject_a_request_with_invalid_json_body_if_required() {
        let validator = make_validator();
        let request = Request {
            path: "/required/json/body".to_string(),
            operation: "post".to_string(),
            body: vec![b'b', b'a', b'b', b'e'],
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert_eq!(Err(()), validator.validate_request(request));
    }

    #[test]
    fn validator_can_accept_a_request_with_valid_utf8_body_if_required() {
        let validator = make_validator();
        let request = Request {
            path: "/required/utf8/body".to_string(),
            operation: "post".to_string(),
            body: vec![b'a', b'b'],
            headers: HashMap::from([(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            )]),
        };
        assert!(validator.validate_request(request).is_ok());
    }

    #[test]
    fn validator_can_reject_a_request_with_invalid_utf8_body_if_required() {
        let validator = make_validator();
        let request = Request {
            path: "/required/utf8/body".to_string(),
            operation: "post".to_string(),
            body: vec![b'\xc3', b'\x28'],
            headers: HashMap::from([(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            )]),
        };
        assert_eq!(Err(()), validator.validate_request(request));
    }

    #[test]
    fn validator_can_select_which_content_to_validate_given_content_type_header_invalid_case() {
        let validator = make_validator();
        let request = Request {
            path: "/allows/utf8/or/json/body".to_string(),
            operation: "post".to_string(),
            body: vec![b'a', b'b'],
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
        };
        assert_eq!(Err(()), validator.validate_request(request));
    }

    #[test]
    fn validator_can_select_which_content_to_validate_given_content_type_header_valid_case() {
        let validator = make_validator();
        let request = Request {
            path: "/allows/utf8/or/json/body".to_string(),
            operation: "post".to_string(),
            body: vec![b'a', b'b'],
            headers: HashMap::from([(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            )]),
        };
        assert!(validator.validate_request(request).is_ok());
    }
}

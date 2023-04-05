struct Validator {
    api: openapiv3::OpenAPI,
}

#[allow(dead_code)]
impl Validator {
    fn new(api: openapiv3::OpenAPI) -> Self {
        Self{api}
    }

    //take &self rather than self otherwise Validator is consumed by validate_request (dropped)
    fn validate_request(&self, request: Request) -> Result<Request, ()> {
        self.validate_path(request.path())?
            .validate_operation(request.operation())?
            .validate_body(request.body())?;
        Ok(request)
    }

    fn validate_path(&self, path: &str) -> Result<ValidatedPath, ()> {
        if let Some(path) = self.api.paths.paths.get(path) {
            return Ok(ValidatedPath{path: path.as_item().unwrap()})
        }
        Err(())
    }
}

struct ValidatedPath<'path> {
    path: &'path openapiv3::PathItem
}

impl<'path>  ValidatedPath<'path>  {
    fn validate_operation(&self, operation: &str) -> Result<ValidatedOperation, ()> {
        match operation {
            "get" => self.validate_get(),
            "put" => self.validate_put(),
            "delete" => self.validate_delete(),
            "post" => self.validate_post(),
            _ => Err(())
        }
    }

    fn validate_get(&self) -> Result<ValidatedOperation, ()> {
        if let Some(operation) = self.path.get.as_ref() {
            return Ok(ValidatedOperation{operation});
        }
        Err(())
    }

    fn validate_put(&self) -> Result<ValidatedOperation, ()> {
        if let Some(operation) = self.path.put.as_ref() {
            return Ok(ValidatedOperation{operation});
        }
        Err(())
    }

    fn validate_post(&self) -> Result<ValidatedOperation, ()> {
        if let Some(operation) = self.path.post.as_ref() {
            return Ok(ValidatedOperation{operation});
        }
        Err(())
    }

    fn validate_delete(&self) -> Result<ValidatedOperation, ()> {
        if let Some(operation) = self.path.delete.as_ref() {
            return Ok(ValidatedOperation{operation});
        }
        Err(())
    }
}

struct ValidatedOperation<'operation> {
    operation: &'operation openapiv3::Operation
}

impl<'operation>  ValidatedOperation<'operation> {
    fn validate_body(&self, body: &[u8]) -> Result<(), ()> {
        if let Some(spec_body) = self.operation.request_body.as_ref().and_then(openapiv3::ReferenceOr::as_item) {
            if spec_body.required && body.is_empty() {
                return Err(());
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
struct Request {
    path: String,
    operation: String,
    body: Vec<u8>
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
}

#[cfg(test)]
mod test {
    use crate::validator::{Request, Validator};

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
    fn validator_can_work_a_request_with_valid_path() {
        let validator = make_validator();
        let request = Request{path: "/ping".to_string(),
                              operation: "get".to_string(),
                              body: vec![]};
        assert!(validator.validate_request(request).is_ok());
    }

    #[test]
    fn validator_can_reject_a_request_with_invalid_path() {
        let validator = make_validator();
        let request = Request{path: "/not/ping".to_string(),
                              operation: "get".to_string(),
                              body: vec![]};
        assert_eq!(Err(()), validator.validate_request(request));
    }

    #[test]
    fn validator_can_work_with_a_request_with_put_operation() {
        let validator = make_validator();
        let request = Request{path: "/multiple/allowed/operations".to_string(),
                              operation: "put".to_string(),
                              body: vec![]};
        assert!(validator.validate_request(request).is_ok());
    }

    #[test]
    fn validator_can_work_with_a_request_with_post_operation() {
        let validator = make_validator();
        let request = Request{path: "/multiple/allowed/operations".to_string(),
                              operation: "post".to_string(),
                              body: vec![]};
        assert!(validator.validate_request(request).is_ok());
    }

    #[test]
    fn validator_can_work_with_a_request_with_delete_operation() {
        let validator = make_validator();
        let request = Request{path: "/multiple/allowed/operations".to_string(),
                              operation: "delete".to_string(),
                              body: vec![]};
        assert!(validator.validate_request(request).is_ok());
    }

    #[test]
    fn validator_can_reject_a_request_with_no_body_if_required() {
        let validator = make_validator();
        let request = Request{path: "/required/body".to_string(),
                              operation: "post".to_string(),
                              body: vec![]};
        assert_eq!(Err(()), validator.validate_request(request));
    }
}
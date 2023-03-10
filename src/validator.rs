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
        self.validate_path(request).and_then(|request| self.validate_method(request))
    }

    fn validate_path(&self, request: Request) -> Result<Request, ()> {
        if self.api.paths.paths.keys().any(|path| request.path() == path) {
            return Ok(request);
        }
        Err(())
    }

    fn validate_method(&self, request: Request) -> Result<Request, ()> {
        match request.method() {
            "get" => self.validate_get(request),
            _ => Err(())
        }
    }

    fn validate_get(&self, request: Request) -> Result<Request, ()> {
        if self.api.paths.paths[request.path.as_str()].as_item().unwrap().get.is_some() {
            return Ok(request);
        }
        Err(())
    }
}

#[derive(Debug, PartialEq)]
struct Request {
    path: String,
    method: String
}

impl Request {
    fn path(&self) -> &str {
        &self.path
    }

    fn method(&self) -> &str {
        &self.method
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
        let request = Request{path: "/ping".to_string(), method: "get".to_string()};
        assert!(validator.validate_request(request).is_ok());
    }

    #[test]
    fn validator_can_reject_a_request_with_invalid_path() {
        let validator = make_validator();
        let request = Request{path: "/not/ping".to_string(), method: "get".to_string()};
        assert_eq!(Err(()), validator.validate_request(request));
    }

    #[test]
    fn validator_can_reject_a_request_with_invalid_method() {
        let validator = make_validator();
        let request = Request{path: "/ping".to_string(), method: "put".to_string()};
        assert_eq!(Err(()), validator.validate_request(request));
    }
}
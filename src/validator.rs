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
        self.get_path(request.path())?
            .get_method(request.method())?;
        Ok(request)
    }

    fn get_path(&self, path: &str) -> Result<PathValidator, ()> {
        if let Some(path) = self.api.paths.paths.get(path) {
            return Ok(PathValidator{path: path.as_item().unwrap()})
        }
        Err(())
    }
}

struct PathValidator<'path> {
    path: &'path openapiv3::PathItem
}

impl<'path>  PathValidator<'path>  {
    fn get_method(&self, method: &str) -> Result<(), ()> {
        match method {
            "get" => self.get_get(),
            "put" => self.get_put(),
            "delete" => self.get_delete(),
            "post" => self.get_post(),
            _ => Err(())
        }
    }

    fn get_get(&self) -> Result<(), ()> {
        if self.path.get.is_some() {
            return Ok(());
        }
        Err(())
    }

    fn get_put(&self) -> Result<(), ()> {
        if self.path.put.is_some() {
            return Ok(());
        }
        Err(())
    }

    fn get_post(&self) -> Result<(), ()> {
        if self.path.post.is_some() {
            return Ok(());
        }
        Err(())
    }

    fn get_delete(&self) -> Result<(), ()> {
        if self.path.delete.is_some() {
            return Ok(());
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
    fn validator_can_work_with_a_request_with_put_method() {
        let validator = make_validator();
        let request = Request{path: "/multiple/allowed/methods".to_string(), method: "put".to_string()};
        assert!(validator.validate_request(request).is_ok());
    }

    #[test]
    fn validator_can_work_with_a_request_with_post_method() {
        let validator = make_validator();
        let request = Request{path: "/multiple/allowed/methods".to_string(), method: "post".to_string()};
        assert!(validator.validate_request(request).is_ok());
    }

    #[test]
    fn validator_can_work_with_a_request_with_delete_method() {
        let validator = make_validator();
        let request = Request{path: "/multiple/allowed/methods".to_string(), method: "delete".to_string()};
        assert!(validator.validate_request(request).is_ok());
    }
}
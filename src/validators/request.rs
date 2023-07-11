use super::operation::OperationValidator;
use std::collections::HashMap;

pub struct Validator {
    api: openapiv3::OpenAPI,
}

#[allow(dead_code)]
impl Validator {
    fn new(api: openapiv3::OpenAPI) -> Self {
        Self { api }
    }

    //take &self rather than self otherwise Validator is consumed by validate_request (dropped)
    pub fn validate_request(&self, request: Request) -> Result<Request, ()> {
        self.validate_path(request.path())?
            .validate_operation(request.operation())?
            .validate_parameters(&request)?
            .validate_content_type(request.get_header("Content-Type"))?
            .validate_body(request.body())?;
        Ok(request)
    }

    fn validate_path(&self, path: &str) -> Result<OperationValidator, ()> {
        if let Some(path_spec) = self
            .api
            .paths
            .paths
            .get(path)
            .and_then(openapiv3::ReferenceOr::as_item)
        {
            return Ok(OperationValidator {
                path_spec,
                components: &self.api.components,
            });
        }
        Err(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Request {
    pub path: String,
    pub operation: String,
    pub body: Vec<u8>,
    pub headers: HashMap<String, String>,
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

    pub fn get_header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).map(String::as_str)
    }
}

#[cfg(test)]
pub fn make_validator_from_spec(path_spec: &str) -> Validator {
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
pub fn make_validator() -> Validator {
    let spec: String = std::fs::read_to_string("./specs/openapi.yaml").unwrap();
    let api: openapiv3::OpenAPI = serde_yaml::from_str(&spec).unwrap();
    Validator::new(api)
}

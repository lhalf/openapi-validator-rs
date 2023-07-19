use regex::Regex;
use std::collections::HashMap;
use url::Url;

use super::operation::OperationValidator;

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
        let url = self.parse_url(request.url())?;
        self.validate_path(url.path())?
            .validate_operation(request.operation())?
            .validate_parameters(&request)?
            .validate_content_type(request.get_header("Content-Type"))?
            .validate_body(request.body())?;
        Ok(request)
    }

    fn parse_url(&self, url: &str) -> Result<Url, ()> {
        match Url::parse(url) {
            Ok(url) => Ok(url),
            Err(..) => Err(()),
        }
    }

    fn validate_path(&self, request_path: &str) -> Result<OperationValidator, ()> {
        let api_paths = &self.api.paths.paths;

        let matching_path = api_paths.keys().find(|path| {
            path.to_component_list()
                .matches(request_path.to_string().to_str_list())
        });

        if let Some(path) = matching_path {
            return Ok(OperationValidator {
                //unwrap as we currently don't have references
                path_spec: api_paths
                    .get(path)
                    .and_then(openapiv3::ReferenceOr::as_item)
                    .unwrap(),
                components: &self.api.components,
            });
        }

        Err(())
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum Segment<'path> {
    Fixed { literal: &'path str },
    Parameter { name: &'path str },
}

trait MatchesPath {
    fn matches(&self, request_segments: Vec<&str>) -> bool;
}

impl MatchesPath for Vec<Segment<'_>> {
    fn matches(&self, request_segments: Vec<&str>) -> bool {
        if self.len() != request_segments.len() {
            return false;
        }
        self.iter()
            .zip(request_segments.iter())
            .all(|(spec_segment, request_segment)| spec_segment.matches(request_segment))
    }
}

impl Segment<'_> {
    fn matches(&self, request_segment: &str) -> bool {
        match self {
            Segment::Fixed { literal } => literal == &request_segment,
            Segment::Parameter { .. } => true,
        }
    }
}

trait SplitPath {
    fn to_component_list(&self) -> Vec<Segment>;
    fn to_str_list(&self) -> Vec<&str>;
}

impl SplitPath for String {
    fn to_component_list(&self) -> Vec<Segment> {
        self.to_str_list()
            .iter()
            .map(|segment| {
                let re = Regex::new(r"^\{[^}]*\}$").unwrap();
                match re.is_match(segment) {
                    true => Segment::Parameter {
                        name: &segment[1..segment.len() - 1],
                    },
                    false => Segment::Fixed { literal: segment },
                }
            })
            .collect::<Vec<Segment>>()
    }

    fn to_str_list(&self) -> Vec<&str> {
        self.split('/')
            .filter(|component| !component.is_empty())
            .collect::<Vec<&str>>()
    }
}

#[derive(Debug, PartialEq)]
pub struct Request {
    pub url: String,
    pub operation: String,
    pub body: Vec<u8>,
    pub headers: HashMap<String, String>,
}

impl Request {
    pub fn url(&self) -> &str {
        &self.url
    }

    fn operation(&self) -> &str {
        &self.operation
    }

    fn body(&self) -> &[u8] {
        &self.body
    }

    pub fn get_header(&self, key: &str) -> Option<String> {
        self.headers.get(key).cloned()
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
pub fn make_validator() -> Result<Validator, ()> {
    let spec = match std::fs::read_to_string("./specs/openapi.yaml") {
        Ok(spec) => spec,
        Err(..) => return Err(()),
    };
    let api = match serde_yaml::from_str(&spec) {
        Ok(api) => api,
        Err(..) => return Err(()),
    };
    Ok(Validator::new(api))
}

#[cfg(test)]
mod test_url {
    use crate::validators::request::make_validator_from_spec;
    use crate::validators::request::Request;
    use indoc::indoc;
    use std::collections::HashMap;

    #[test]
    fn reject_a_request_with_no_host_in_url() {
        let path_spec = indoc!(
            r#"
            paths:
              /do/not/care:
                post:
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            url: "http://do/not/care".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(request)
        );
    }

    #[test]
    fn reject_a_request_with_no_scheme_in_url() {
        let path_spec = indoc!(
            r#"
            paths:
              /do/not/care:
                post:
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            url: "test.com/do/not/care".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(request)
        );
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
        let validator = make_validator().unwrap();
        let request = Request {
            url: "http://test.com/ping".to_string(),
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
            url: "http://test.com/invalid/path".to_string(),
            operation: "get".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert_eq!(
            Err(()),
            make_validator_from_spec(path_spec).validate_request(request)
        );
    }
}

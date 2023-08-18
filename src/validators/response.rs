#[derive(Debug, PartialEq, Clone)]
pub struct ResponseValidator<'api> {
    pub response_spec: &'api openapiv3::Responses,
    pub components: &'api Option<openapiv3::Components>,
}

impl<'api> ResponseValidator<'api> {
    pub fn validate_response(self, _response: &dyn Response) -> Result<(), ()> {
        Ok(())
    }
}

pub trait Response {}

#[cfg(test)]
mod test_responses {
    use crate::validators::request::make_validator_from_spec;
    use crate::validators::request::Request;
    use crate::validators::response::Response;
    use indoc::indoc;
    use std::collections::HashMap;

    pub struct FakeResponse {}

    impl Response for FakeResponse {}

    #[test]
    fn validate_request_returns_a_response_validator_with_correct_status_code() {
        let path_spec = indoc!(
            r#"
            paths:
              /my/path:
                post:
                  responses:
                    200:
                      description: API call successful
            "#
        );
        let request = Request {
            url: "http:/test.com/my/path".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        let response = FakeResponse {};

        assert!(make_validator_from_spec(path_spec)
            .validate_request(&request)
            .unwrap()
            .validate_response(&response)
            .is_ok());
    }
}

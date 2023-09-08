#[derive(Debug, PartialEq, Clone)]
pub struct ResponseValidator<'api> {
    pub response_spec: &'api openapiv3::Responses,
    pub components: &'api Option<openapiv3::Components>,
}

impl<'api> ResponseValidator<'api> {
    pub fn validate_response(self, response: &dyn Response) -> Result<(), ()> {
        self.validate_status_code(response.status_code())
    }

    fn validate_status_code(self, status_code: u16) -> Result<(), ()> {
        if let Some(_response_spec) = self
            .response_spec
            .responses
            .get(&openapiv3::StatusCode::Code(status_code))
        {
            return Ok(());
        }

        Err(())
    }
}

pub trait Response {
    fn status_code(&self) -> u16;
}

#[cfg(test)]
mod test_responses {
    use crate::validators::request::test_helpers::*;
    use crate::validators::response::Response;
    use indoc::indoc;
    use std::collections::HashMap;

    pub struct FakeResponse {
        pub status_code: u16,
    }

    impl Response for FakeResponse {
        fn status_code(&self) -> u16 {
            self.status_code
        }
    }

    #[test]
    fn validate_a_response_with_valid_status_code() {
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
        let request = FakeRequest {
            url: "http:/test.com/my/path".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        let response = FakeResponse { status_code: 200 };

        assert!(make_validator_from_spec(path_spec)
            .validate_request(&request)
            .unwrap()
            .validate_response(&response)
            .is_ok());
    }

    #[test]
    fn reject_a_response_with_invalid_status_code() {
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
        let request = FakeRequest {
            url: "http:/test.com/my/path".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        let response = FakeResponse { status_code: 404 };

        assert!(make_validator_from_spec(path_spec)
            .validate_request(&request)
            .unwrap()
            .validate_response(&response)
            .is_err());
    }
}

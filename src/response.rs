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
        dbg!(self.response_spec);

        let responses = &self.response_spec.responses;

        responses
            .get(&openapiv3::StatusCode::Code(status_code))
            .or_else(|| responses.get(&Self::extract_range_from_status_code(status_code)))
            .ok_or(())
            .map(|_| ())
    }

    fn extract_range_from_status_code(status_code: u16) -> openapiv3::StatusCode {
        openapiv3::StatusCode::Range(match status_code {
            100..=199 => 1,
            200..=299 => 2,
            300..=399 => 3,
            400..=499 => 4,
            500..=599 => 5,
            _ => todo!(),
        })
    }
}

pub trait Response {
    fn status_code(&self) -> u16;
}

#[cfg(test)]
mod test_responses {
    use crate::request::test_helpers::*;
    use crate::response::Response;
    use indoc::indoc;
    use parameterized::parameterized;
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
    fn accept_a_response_with_valid_status_code() {
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

    #[parameterized(range={"1XX", "2XX", "3XX", "4XX", "5XX"}, response_code={150, 250, 350, 450, 550})]
    fn accept_a_response_with_a_status_code_within_range(range: &str, response_code: u16) {
        let path_spec = format!(
            indoc::indoc!(
                r#"
                paths:
                  /my/path:
                    post:
                      responses:
                        {}:
                          description: API call successful
                "#
            ),
            range
        );
        let request = FakeRequest {
            url: "http:/test.com/my/path".to_string(),
            operation: "post".to_string(),
            body: vec![],
            headers: HashMap::new(),
        };
        let response = FakeResponse {
            status_code: { response_code },
        };

        assert!(make_validator_from_spec(&path_spec)
            .validate_request(&request)
            .unwrap()
            .validate_response(&response)
            .is_ok());
    }
}

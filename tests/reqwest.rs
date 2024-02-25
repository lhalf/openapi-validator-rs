struct ReqwestWrapper(reqwest::Request);

impl validator::Request for ReqwestWrapper {
    fn url(&self) -> &str {
        self.0.url().as_str()
    }

    fn operation(&self) -> &str {
        self.0.method().as_str()
    }

    //make this option
    fn body(&self) -> &[u8] {
        self.0
            .body()
            .and_then(reqwest::Body::as_bytes)
            .unwrap_or(&[])
    }

    fn get_header(&self, key: &str) -> Option<String> {
        self.0.headers().get(key).and_then(|header_value| {
            header_value
                .to_str()
                .ok()
                .map(|header_value| header_value.to_string())
        })
    }
}

#[cfg(test)]
mod test_reqwest {
    #[test]
    fn test_reqwest_get() {}
}

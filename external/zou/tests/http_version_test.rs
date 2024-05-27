extern crate hyper;
extern crate libzou;

#[cfg(test)]
mod test_http_versions {
    use hyper::version::HttpVersion;
    use libzou::http_version::ValidateHttpVersion;

    #[test]
    fn version_09_failed() {
        assert!(!HttpVersion::Http09.greater_than_http_11())
    }

    #[test]
    fn version_10_failed() {
        assert!(!HttpVersion::Http10.greater_than_http_11())
    }

    #[test]
    fn version_11_succeeds() {
        assert!(HttpVersion::Http11.greater_than_http_11())
    }

    #[test]
    fn version_20_succeeds() {
        assert!(HttpVersion::Http20.greater_than_http_11())
    }
}

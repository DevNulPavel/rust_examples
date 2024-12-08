use tracing::instrument;

////////////////////////////////////////////////////////////////////////////////

#[cfg_attr(debug_assertions, instrument(level = "trace", skip_all))]
pub(super) fn matches_pattern(pattern: &str, path: &str) -> bool {
    if pattern == "*" {
        true
    } else if pattern.ends_with("*") {
        let prefix = &pattern[..pattern.len() - 1];
        path.starts_with(prefix)
    } else {
        pattern == path
    }
}

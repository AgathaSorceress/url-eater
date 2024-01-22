use super::*;

#[cfg(test)]
use pretty_assertions::assert_eq;

const PATTERNS: [&str; 2] = ["delete_me", "test@example.com"];

#[test]
fn query_parameters() -> Result<(), String> {
    for (url, expected) in [
        ("http://test.org/meow", "http://test.org/meow"),
        ("http://test.org/page?delete_me", "http://test.org/page"),
        (
            "http://test.org/page?delete_me&keep_me",
            "http://test.org/page?keep_me",
        ),
    ] {
        assert_eq!(
            clean_url(
                url.to_owned(),
                PATTERNS.into_iter().map(|v| v.into()).collect()
            )?,
            expected.to_owned()
        );
    }

    Ok(())
}

#[test]
fn url_fragments() -> Result<(), String> {
    for (url, expected) in [
        (
            "http://test.org/page?delete_me#fragment",
            "http://test.org/page#fragment",
        ),
        (
            "http://test.org/page#fragment",
            "http://test.org/page#fragment",
        ),
    ] {
        assert_eq!(
            clean_url(
                url.to_owned(),
                PATTERNS.into_iter().map(|v| v.into()).collect()
            )?,
            expected.to_owned()
        );
    }

    Ok(())
}

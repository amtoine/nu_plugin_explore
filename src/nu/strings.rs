/// TODO: documentation
#[derive(Debug, PartialEq)]
pub(crate) enum SpecialString {
    Url,
    Path,
}

impl std::fmt::Display for SpecialString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let repr = match self {
            Self::Url => "url".to_string(),
            Self::Path => "path".to_string(),
        };
        write!(f, "{}", repr)
    }
}

/// TODO: documentation
impl SpecialString {
    pub(crate) fn parse(input: &str) -> Option<Self> {
        if let Ok(url) = url::Url::parse(input) {
            if url.scheme() == "file" {
                Some(Self::Path)
            } else {
                Some(Self::Url)
            }
        } else if input.contains('\n') {
            None
        } else if input.contains('/') {
            Some(Self::Path)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod special_strings_tests {
    use super::SpecialString;

    #[test]
    fn parse_strings() {
        let cases = vec![
            ("foo", None),
            ("https://google.com", Some(SpecialString::Url)),
            ("file:///some/file", Some(SpecialString::Path)),
            ("/path/to/something", Some(SpecialString::Path)),
            ("relative/path/", Some(SpecialString::Path)),
            ("./relative/path/", Some(SpecialString::Path)),
            ("../../relative/path/", Some(SpecialString::Path)),
            ("file:", Some(SpecialString::Path)),
            ("normal string with a / inside", Some(SpecialString::Path)),
            ("normal string with \na / inside", None),
        ];

        for (input, expected) in cases {
            let actual = SpecialString::parse(input);
            assert_eq!(
                actual, expected,
                "expected {expected:#?} on input {input}, found {actual:#?}",
            );
        }
    }
}

use lazy_static::lazy_static;
use regex::Regex;

pub const REALM_TAG_NAME: &str = "palantir_realm";
pub const APPLICATION_TAG_NAME: &str = "palantir_application";
pub const APPLICATION_HASH_TAG_NAME: &str = "palantir_application_hash";
pub const ACTION_KIND_TAG_NAME: &str = "palantir_action_kind";
pub const ACTION_NAME_TAG_NAME: &str = "palantir_action_name";
pub const ACTION_SPAN_TAG_NAME: &str = "palantir_span";

pub const ACTION_METRIC_NAME: &str = "palantir_apm";
pub const UNTRACKED_ACTION_KIND_NAME: &str = "palantir_untracked";
pub const TOTAL_ACTION_KIND_NAME: &str = "palantir_total";

pub const EXTRA_LABEL_PREFIX: &str = "PALANTIR_LABEL_";
lazy_static! {
    pub static ref EXTRA_LABEL_REGEX: Regex = Regex::new("^[0-9a-zA-Z\\-_]+$").unwrap();
}

#[cfg(test)]
mod tests {
    use super::EXTRA_LABEL_REGEX;

    #[test]
    fn test_extra_label_regex() {
        let valid = vec!["some-valid-ai234s", "some_valid_0987123"];
        let invalid = vec![
            "dot.inside",
            "another;punctuation",
            "cyrillic-symbols-inside-quotes-'АААА'",
            "😀😃😄😅😘🤑",
        ];

        for entry in valid {
            assert!(EXTRA_LABEL_REGEX.is_match(entry));
        }
        for entry in invalid {
            assert!(!EXTRA_LABEL_REGEX.is_match(entry));
        }
    }
}

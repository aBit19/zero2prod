use unicode_segmentation::UnicodeSegmentation;

const FORBIDDEN_CHARACTERS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

#[derive(Debug)]
pub struct SubscriberName(String);
impl SubscriberName {
    pub fn parse(name: String) -> Result<Self, String> {
        let is_empty = name.trim().is_empty();
        if is_empty {
            return Err("Subscriber name cannot be empty.".to_string());
        }

        let is_too_long = name.graphemes(true).count() > 256;
        if is_too_long {
            return Err("Subscriber name cannot be longer than 256 characters.".to_string());
        }

        let contains_forbidden_characters = name
            .chars()
            .any(|character| FORBIDDEN_CHARACTERS.contains(&character));

        if contains_forbidden_characters {
            return Err(
                "Subscriber name cannot contain any of the following characters: /()\"<>\\. \\{\\}"
                    .to_string(),
            );
        }

        Ok(Self(name))
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{SubscriberName, FORBIDDEN_CHARACTERS};
    use claims::{assert_err, assert_ok};

    #[test]
    fn parse_given_256_characters_name_returns_error() {
        let name = "ё".repeat(257);
        let result = SubscriberName::parse(name);
        assert_err!(
            result,
            "Subscriber name cannot be longer than 256 characters."
        );
    }

    #[test]
    fn parse_given_empty_name_returns_error() {
        let name = "";
        let result = SubscriberName::parse(name.to_string());
        assert_err!(result, "Subscriber name cannot be empty.");
    }

    #[test]
    fn parse_given_name_with_forbidden_characters_returns_error() {
        let err_msg = String::from_iter(FORBIDDEN_CHARACTERS.iter());
        for char in FORBIDDEN_CHARACTERS {
            let name = format!("name{}", char);
            let result = SubscriberName::parse(name);
            assert_err!(
                result,
                "Subscriber name cannot contain any of the following characters: {err_msg}"
            );
        }
    }

    #[test]
    fn parse_given_valid_name_then_ok() {
        let name = "name".to_string();
        let result = SubscriberName::parse(name);
        assert_ok!(result, "Name should be valid.");
    }
}

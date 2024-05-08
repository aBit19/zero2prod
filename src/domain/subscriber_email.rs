use validator::ValidateEmail;

#[derive(Debug, Clone)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(email: String) -> Result<Self, String> {
        if email.validate_email() {
            Ok(Self(email))
        } else {
            Err("Invalid email.".to_string())
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;

    #[test]
    fn parse_given_empty_email_returns_error() {
        let email = "";
        let result = SubscriberEmail::parse(email.to_string());
        assert_err!(result, "Subscriber email cannot be empty.");
    }

    #[test]
    fn parse_given_email_without_at_symbol_then_error() {
        let email = "ursula.com";
        let result = SubscriberEmail::parse(email.to_string());
        assert_err!(result, "Subscriber email must contain an @ symbol.");
    }

    #[test]
    fn parse_given_email_without_host_then_error() {
        let email = "ewrwrw@";
        let result = SubscriberEmail::parse(email.to_string());
        assert_err!(result, "Subscriber email must contain a host.");
    }

    #[test]
    fn parse_given_email_without_name_then_error() {
        let email = "@gmail.com";
        let result = SubscriberEmail::parse(email.to_string());
        assert_err!(result, "Subscriber email must contain a name.");
    }

    #[test]
    fn parse_given_valid_email_returns_ok() {
        for _ in 0..100 {
            let email = SafeEmail().fake();
            let result = SubscriberEmail::parse(email);
            assert_ok!(result);
        }
    }
}

use crate::domain::SubscriberEmail;
use secrecy::{ExposeSecret, Secret};

#[derive(Debug)]
pub struct EmailClient {
    sender: SubscriberEmail,
    base_url: String,
    http_client: reqwest::Client,
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = reqwest::Client::builder().timeout(timeout).build().unwrap();
        Self {
            sender,
            base_url,
            http_client,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let send_email_request = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body,
            text_body,
        };
        self.http_client
            .post(&url)
            .header(
                "X-PostMark-Sever-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&send_email_request)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests {
    use crate::{domain::SubscriberEmail, email_client::EmailClient};
    use claims::{assert_err, assert_ok};
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use wiremock::{
        matchers::{any, header, header_exists, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    struct EmailBodyMatcher;

    impl wiremock::Match for EmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            if let Ok(payload) = serde_json::from_slice(request.body.as_slice()) {
                let payload: serde_json::Value = dbg!(payload);
                payload.get("From").is_some()
                    && payload.get("To").is_some()
                    && payload.get("Subject").is_some()
                    && payload.get("HtmlBody").is_some()
                    && payload.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn body() -> String {
        Paragraph(1..10).fake()
    }

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_uri: &str) -> EmailClient {
        EmailClient::new(
            base_uri.to_string(),
            email(),
            secrecy::Secret::new(Faker.fake()),
            std::time::Duration::from_millis(200),
        )
    }

    #[tokio::test]
    async fn send_email_fires_request_to_base_uri() {
        let server = MockServer::start().await;
        Mock::given(header_exists("X-PostMark-Sever-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(EmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let email_client = email_client(&server.uri());
        email_client
            .send_email(&email(), &subject(), &body(), &body())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn send_email_returns_ok_when_server_returns_200() {
        let server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let email_client = email_client(&server.uri());

        let send_email = email_client
            .send_email(&email(), &subject(), &body(), &body())
            .await;

        assert_ok!(send_email);
    }

    #[tokio::test]
    async fn send_email_given_500_then_return_err() {
        let server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&server)
            .await;

        let email_client = email_client(&server.uri());

        let send_email = email_client
            .send_email(&email(), &subject(), &body(), &body())
            .await;

        assert_err!(send_email);
    }

    #[tokio::test]
    async fn send_email_given_time_out_return_err() {
        let server = MockServer::start().await;
        let response_with_delay =
            ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response_with_delay)
            .expect(1)
            .mount(&server)
            .await;

        let email_client = email_client(&server.uri());

        let send_email = email_client
            .send_email(&email(), &subject(), &body(), &body())
            .await;

        assert_err!(send_email);
    }
}

use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::{spawn_app, TestApp};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;

    create_unconfirmed_subscribers(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter",
            "html": "Newsletter"
        }
    });

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletter", &app.address))
        .json(&newsletter)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status().as_u16(), 200);
}

async fn create_unconfirmed_subscribers(app: &TestApp) {
    let body = "name=le%20guin&email=test%40gmail.com";

    let _mock = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    let response = app.post_subscriptions(body).await.unwrap();
    assert_eq!(response.status().as_u16(), 200);
}

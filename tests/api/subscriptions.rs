use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_data() {
    // Arrange
    let test_app = spawn_app().await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    let send = test_app.post_subscriptions(body).await.unwrap();

    assert_eq!(200, send.status().as_u16());

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test_app.pool.clone())
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "pending_verification");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("name=&email=ursula_le_guin%40gmail.com", "empy name"),
        ("name=test_1&email=", "empty email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = test_app
            .post_subscriptions(invalid_body)
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email() {
    // Arrange
    let test_app = spawn_app().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    test_app.post_subscriptions(body).await.unwrap();
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_with_link() {
    // Arrange
    let test_app = spawn_app().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    test_app.post_subscriptions(body).await.unwrap();

    let received = &test_app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&received.body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<linkify::Link> = linkify::LinkFinder::new()
            .links(s)
            .filter(move |l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let html_link = get_link(body.get("HtmlBody").unwrap().as_str().unwrap());
    let text_link = get_link(body.get("TextBody").unwrap().as_str().unwrap());

    assert_eq!(html_link, text_link);
}

#[tokio::test]
async fn subscribe_inserts_a_subscription_token_to_db() {
    // Arrange
    let test_app = spawn_app().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    test_app.post_subscriptions(body).await.unwrap();

    let token = sqlx::query!("SELECT token FROM subscription_tokens",)
        .fetch_all(&test_app.pool.clone())
        .await
        .expect("Failed to fetch token from db.");

    assert_eq!(1, token.len());
}

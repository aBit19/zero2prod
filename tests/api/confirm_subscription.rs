use claims::assert_ok;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::*;

#[tokio::test]
async fn confirm_subscription_missing_token_return_bad_request() {
    let app = spawn_app().await;

    let post = reqwest::Client::new()
        .post(format!("{}/subscriptions/confirm", app.address))
        .send()
        .await;

    assert_eq!(post.unwrap().status().as_u16(), 400);
}

#[tokio::test]
async fn confirm_subscription_link_found_in_email_returns_200() {
    let app = spawn_app().await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(&body).await.unwrap();

    let body = &app.email_server.received_requests().await.unwrap()[0].body;
    let from_slice: serde_json::Value = serde_json::from_slice(body).unwrap();
    let get = from_slice.get("HtmlBody").unwrap().as_str().unwrap();
    let confirmation_link = linkify::LinkFinder::new()
        .links(get)
        .map(|l| l.as_str())
        .collect::<Vec<&str>>()[0];

    let send = reqwest::Client::new().post(confirmation_link).send().await;

    assert_ok!(send);
}

#[tokio::test]
async fn confirm_subscription_link_found_in_email_sets_the_status_of_subscription_to_active() {
    let app = spawn_app().await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(&body).await.unwrap();

    let body = &app.email_server.received_requests().await.unwrap()[0].body;
    let from_slice: serde_json::Value = serde_json::from_slice(body).unwrap();
    let get = from_slice.get("HtmlBody").unwrap().as_str().unwrap();
    let confirmation_link = linkify::LinkFinder::new()
        .links(get)
        .map(|l| l.as_str())
        .collect::<Vec<&str>>()[0];

    let send = reqwest::Client::new().post(confirmation_link).send().await;

    assert_ok!(send);

    let subscriptions = sqlx::query!("SELECT status FROM subscriptions")
        .fetch_one(&app.pool)
        .await
        .unwrap();

    assert_eq!(subscriptions.status, "confirmed");
}

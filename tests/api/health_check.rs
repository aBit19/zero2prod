use crate::helpers;

#[tokio::test]
async fn health_check_works() {
    let address = helpers::spawn_app().await.address;

    let response = helpers::get(&address, "health_check")
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

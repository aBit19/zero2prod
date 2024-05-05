use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use zero2prod::configuration::DatabaseSettings;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app().await.address;

    let response = get(&address, "health_check")
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_data() {
    // Arrange
    let test_app = spawn_app().await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let send = post(&test_app.address, "subscriptions")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, send.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.pool.clone())
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let test_app = spawn_app().await.address;

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = post(&test_app, "subscriptions")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
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

fn post(address: &str, path: &str) -> reqwest::RequestBuilder {
    client().post(&format!("{}/{}", address, path))
}

fn get(address: &str, path: &str) -> reqwest::RequestBuilder {
    client().get(&format!("{}/{}", address, path))
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

struct TestApp {
    address: String,
    pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let database_settings: DatabaseSettings = setup_database().await;

    let connection_string = database_settings.connection_string();
    let pool = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    let server = zero2prod::startup::run(listener, pool.clone()).expect("Failed to start server");
    let _ = tokio::spawn(server);
    let address = format!("http://127.0.0.1:{}", port);

    TestApp { address, pool }
}

async fn setup_database() -> DatabaseSettings {
    let database_settings = zero2prod::configuration::get_configuration()
        .expect("Failed to read configuration.")
        .database;

    let database_name = uuid::Uuid::new_v4().to_string();

    let mut connection = PgConnection::connect(&database_settings.connection_string_without_db())
        .await
        .expect("Unable connect to Postgres.");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, database_name).as_str())
        .await
        .expect("Failed to create database.");

    let database_settings = DatabaseSettings {
        database_name,
        ..database_settings
    };

    let mut connection = PgConnection::connect(&database_settings.connection_string())
        .await
        .expect("Unable connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&mut connection)
        .await
        .expect("Failed to run migrations.");

    database_settings
}

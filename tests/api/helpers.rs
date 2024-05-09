use std::net::TcpListener;

use once_cell::sync::Lazy;
use sqlx::{Executor, PgPool};
use zero2prod::{
    configuration::DatabaseSettings, db, domain::SubscriberEmail, email_client::EmailClient,
    telemetry,
};

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        telemetry::init("test", "debug", std::io::stdout);
    } else {
        telemetry::init("test", "debug", std::io::sink);
    };
});

pub fn post(address: &str, path: &str) -> reqwest::RequestBuilder {
    client().post(&format!("{}/{}", address, path))
}

pub fn get(address: &str, path: &str) -> reqwest::RequestBuilder {
    client().get(&format!("{}/{}", address, path))
}

pub fn client() -> reqwest::Client {
    reqwest::Client::new()
}

pub struct TestApp {
    pub address: String,
    pub pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let configuration = zero2prod::configuration::get_configuration();
    let database_settings = setup_database(configuration.database).await;

    let pool = db::get_pg_pool(&database_settings).await;
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        SubscriberEmail::parse(configuration.email_client.sender_email)
            .expect("Valid email for sender"),
        configuration.email_client.authorization_token,
        std::time::Duration::from_millis(configuration.email_client.timeout_millis),
    );

    let server = zero2prod::startup::run(listener, pool.clone(), email_client)
        .expect("Failed to start server");
    let _ = tokio::spawn(server);
    let address = format!("http://127.0.0.1:{}", port);

    TestApp { address, pool }
}

async fn setup_database(database_settings: DatabaseSettings) -> db::DatabaseSettings {
    Lazy::force(&TRACING);

    let database_name = uuid::Uuid::new_v4().to_string();

    let database_settings: db::DatabaseSettings = database_settings.into();
    let mut connection = db::get_pg_connection(&database_settings).await;

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, database_name).as_str())
        .await
        .expect("Failed to create database.");

    let database_settings = db::DatabaseSettings {
        database_name,
        ..database_settings
    };

    let mut connection = db::get_pg_connection(&database_settings).await;

    sqlx::migrate!("./migrations")
        .run(&mut connection)
        .await
        .expect("Failed to run migrations.");

    database_settings
}

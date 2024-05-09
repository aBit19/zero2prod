use once_cell::sync::Lazy;
use reqwest::Response;
use sqlx::{Executor, PgPool};
use zero2prod::{configuration::Settings, factory, startup::NewsletterApp, telemetry};

pub static TRACING: Lazy<()> = Lazy::new(|| {
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
    Lazy::force(&TRACING);

    // Ensure that we get a new database every time we run the tests to ensure isolation
    let configuration = {
        let mut config = zero2prod::configuration::get_configuration();
        config.application.port = 0;
        config
    };

    let configuration = setup_test_database(configuration).await;

    let pg_pool = factory::get_pool_with(&configuration.database).await;

    let build = NewsletterApp::build(configuration)
        .await
        .expect("Failed to build app");

    let port = build.port();

    let run = build.run().expect("Error running app");

    let _ = tokio::spawn(run);

    TestApp {
        address: format!("http://{}:{}", "127.0.0.1", port),
        pool: pg_pool,
    }
}

async fn setup_test_database(settings: Settings) -> Settings {
    let database_settings = {
        let mut database_settings = settings.database;
        database_settings.database_name = "".to_string();
        database_settings
    };

    let mut connection = factory::get_connection_with(&database_settings).await;

    let database_name = uuid::Uuid::new_v4().to_string();

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, database_name).as_str())
        .await
        .expect("Failed to create database.");

    let database_settings = {
        let mut database_settings = database_settings;
        database_settings.database_name = database_name;
        database_settings
    };

    let mut connection = factory::get_connection_with(&database_settings).await;

    sqlx::migrate!("./migrations")
        .run(&mut connection)
        .await
        .expect("Failed to run migrations.");

    Settings {
        database: database_settings,
        ..settings
    }
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: &str) -> Result<Response, reqwest::Error> {
        let response = post(&self.address, "subscriptions")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.to_string())
            .send()
            .await?;
        Ok(response)
    }
}

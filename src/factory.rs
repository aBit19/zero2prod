use crate::{
    configuration::{self, EmailClientSettings},
    db,
    domain::SubscriberEmail,
    email_client::EmailClient,
};
use sqlx::{Connection, Database, PgConnection, PgPool, Pool};

pub fn get_email_client(email_client: &EmailClientSettings) -> EmailClient {
    EmailClient::new(
        email_client.base_url.clone(),
        SubscriberEmail::parse(email_client.sender_email.clone()).expect("Valid email for sender"),
        email_client.authorization_token.clone(),
        std::time::Duration::from_millis(email_client.timeout_millis),
    )
}

pub async fn get_pool() -> Pool<impl Database> {
    let config = configuration::get_configuration();
    get_pool_with(&config.database).await
}

pub async fn get_connection() -> impl Connection {
    let config = configuration::get_configuration();
    get_connection_with(&config.database).await
}

pub async fn get_connection_with(config: &configuration::DatabaseSettings) -> PgConnection {
    db::get_connection(&config.into()).await
}

pub async fn get_pool_with(config: &configuration::DatabaseSettings) -> PgPool {
    db::get_pool(&config.into()).await
}

impl From<&configuration::DatabaseSettings> for db::DatabaseSettings {
    fn from(config: &configuration::DatabaseSettings) -> Self {
        db::DatabaseSettings {
            username: config.username.clone(),
            password: config.password.clone(),
            host: config.host.clone(),
            port: config.port,
            database_name: config.database_name.clone(),
            require_ssl: config.require_ssl,
        }
    }
}

use secrecy::{ExposeSecret, Secret};
use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    Connection, PgConnection, PgPool,
};

pub async fn get_pool(database_settings: &DatabaseSettings) -> PgPool {
    PgPool::connect_lazy_with(database_settings.into())
}

pub async fn get_pg_pool(database_settings: &DatabaseSettings) -> PgPool {
    PgPool::connect_lazy_with(database_settings.into())
}

pub async fn get_connection(database_settings: &DatabaseSettings) -> PgConnection {
    PgConnection::connect_with(&database_settings.into())
        .await
        .expect("Unable to connect to Postgres.")
}

pub async fn get_pg_connection(database_settings: &DatabaseSettings) -> PgConnection {
    PgConnection::connect_with(&database_settings.into())
        .await
        .expect("Unable to connect to Postgres.")
}

pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub require_ssl: bool,
}

impl From<&DatabaseSettings> for PgConnectOptions {
    fn from(value: &DatabaseSettings) -> Self {
        PgConnectOptions::new()
            .host(&value.host)
            .username(&value.username)
            .password(value.password.expose_secret())
            .port(value.port)
            .database(&value.database_name)
            .ssl_mode(if value.require_ssl {
                PgSslMode::Require
            } else {
                PgSslMode::Prefer
            })
    }
}

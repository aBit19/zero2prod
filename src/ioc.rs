use crate::{configuration, db};
use sqlx::{Connection, Database, Pool};

pub async fn get_pool() -> Pool<impl Database> {
    let config = configuration::get_configuration();
    get_pool_with(config.database).await
}

pub async fn get_connection() -> impl Connection {
    let config = configuration::get_configuration();
    get_connection_with(config.database).await
}

pub async fn get_connection_with(config: configuration::DatabaseSettings) -> impl Connection {
    db::get_connection(&config.into()).await
}

pub async fn get_pool_with(config: configuration::DatabaseSettings) -> Pool<impl Database> {
    db::get_pool(&config.into()).await
}

impl From<configuration::DatabaseSettings> for db::DatabaseSettings {
    fn from(config: configuration::DatabaseSettings) -> Self {
        db::DatabaseSettings {
            username: config.username,
            password: config.password,
            host: config.host,
            port: config.port,
            database_name: config.database_name,
            require_ssl: config.require_ssl,
        }
    }
}

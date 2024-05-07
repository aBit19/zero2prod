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
    db::get_connection(&get_db_settings(config)).await
}

pub async fn get_pool_with(config: configuration::DatabaseSettings) -> Pool<impl Database> {
    db::get_pool(&get_db_settings(config)).await
}

pub fn get_db_settings(config_db: configuration::DatabaseSettings) -> db::DatabaseSettings {
    db::DatabaseSettings {
        username: config_db.username,
        password: config_db.password,
        host: config_db.host,
        port: config_db.port,
        database_name: config_db.database_name,
        require_ssl: config_db.require_ssl,
    }
}

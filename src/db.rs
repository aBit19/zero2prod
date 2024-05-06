use secrecy::{ExposeSecret, Secret};
use sqlx::{Connection, Database, PgConnection, PgPool, Pool};

pub async fn get_pool(connection_string: &Secret<String>) -> Pool<impl Database> {
    PgPool::connect_lazy(connection_string.expose_secret()).expect("Unable to connect to Postgres.")
}

pub async fn get_connection(connection_string: &Secret<String>) -> impl Connection {
    PgConnection::connect(connection_string.expose_secret())
        .await
        .expect("Unable to connect to Postgres.")
}

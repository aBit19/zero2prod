use sqlx::{Connection, Database, PgConnection, PgPool, Pool};

pub async fn get_pool(connection_string: &str) -> Pool<impl Database> {
    PgPool::connect(connection_string)
        .await
        .expect("Unable to connect to Postgres.")
}

pub async fn get_connection(connection_string: &str) -> impl Connection {
    PgConnection::connect(connection_string)
        .await
        .expect("Unable to connect to Postgres.")
}

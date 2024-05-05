use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration;
use zero2prod::startup;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let conf = configuration::get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", conf.application_port);
    let pool = PgPool::connect(&conf.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    startup::run(TcpListener::bind(address)?, pool)?.await
}

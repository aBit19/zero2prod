use std::net::TcpListener;
use zero2prod::configuration;
use zero2prod::db;
use zero2prod::startup;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let conf = configuration::get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", conf.application_port);
    let pool = db::get_pool(&conf.database.connection_string())
        .await;
    startup::run(TcpListener::bind(address)?, pool)?.await
}

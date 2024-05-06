use std::net::TcpListener;
use zero2prod::configuration;
use zero2prod::db;
use zero2prod::startup;
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = telemetry::get_subscriber("zero2prod", "info", std::io::stdout);
    telemetry::init_subscriber(subscriber);
    let conf = configuration::get_configuration().expect("Failed to read configuration.");
    let address = format!("{}:{}", conf.application.host, conf.application.port);
    let pool = db::get_pool(&conf.database.connection_string()).await;
    startup::run(TcpListener::bind(address)?, pool)?.await
}

use std::net::TcpListener;
use zero2prod::configuration;
use zero2prod::factory;
use zero2prod::startup;
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    telemetry::init("zero2prod", "info", std::io::stdout);
    let conf = configuration::get_configuration();
    let address = format!("{}:{}", conf.application.host, conf.application.port);
    let pool = factory::get_pool().await;
    let email_client = factory::get_email_client();
    startup::run(TcpListener::bind(address)?, pool, email_client)?.await
}

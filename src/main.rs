use zero2prod::configuration;
use zero2prod::startup::NewsletterApp;
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    telemetry::init("zero2prod", "info", std::io::stdout);
    let build = NewsletterApp::build(configuration::get_configuration()).await?;
    build.run()?.await?;
    Ok(())
}

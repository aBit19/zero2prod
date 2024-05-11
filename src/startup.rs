use crate::configuration::Settings;
use crate::email_client::EmailClient;
use crate::factory;
use crate::routes::{confirm_subscription, health_check, subscribe, ApplicationBaseUrl};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use std::net::TcpListener;

pub struct NewsletterApp {
    port: u16,
    listener: TcpListener,
    pg_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
}

impl NewsletterApp {
    pub fn bind(configuration: &Settings) -> Result<TcpListener, std::io::Error> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;

        Ok(listener)
    }

    pub async fn build(configuration: Settings) -> Result<NewsletterApp, std::io::Error> {
        let listener = NewsletterApp::bind(&configuration)?;
        NewsletterApp::build_with(configuration, listener).await
    }

    pub async fn build_with(
        configuration: Settings,
        listener: TcpListener,
    ) -> Result<NewsletterApp, std::io::Error> {
        let pg_pool = factory::get_pool_with(&configuration.database).await;
        let email_client = factory::get_email_client(&configuration.email_client);
        let port = listener.local_addr().unwrap().port();
        Ok(NewsletterApp {
            listener,
            port,
            pg_pool,
            email_client,
            base_url: configuration.application.base_url,
        })
    }

    pub fn run(self) -> Result<Server, std::io::Error> {
        let pool = web::Data::new(self.pg_pool);
        let email_client = web::Data::new(self.email_client);
        let application_url = web::Data::new(ApplicationBaseUrl(self.base_url.clone()));
        let server = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .route("/health_check", web::get().to(health_check))
                .route("/subscriptions", web::post().to(subscribe))
                .route(
                    "/subscriptions/confirm",
                    web::post().to(confirm_subscription),
                )
                .app_data(pool.clone())
                .app_data(email_client.clone())
                .app_data(application_url.clone())
        })
        .listen(self.listener)?
        .run();
        Ok(server)
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

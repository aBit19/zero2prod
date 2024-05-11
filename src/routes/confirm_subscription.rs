use actix_web::{web, HttpResponse};
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct Token {
    _token: String,
}

#[tracing::instrument(name = "Confirming a subscription.", skip(_token, _pool))]
pub async fn confirm_subscription(
    _token: web::Query<Token>,
    _pool: web::Data<PgPool>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}

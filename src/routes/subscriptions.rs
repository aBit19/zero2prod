use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[tracing::instrument(name = "Adding a new subscriber.", skip(form, pool), fields(email=%form.email, name=%form.name))]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let name = match SubscriberName::parse(form.0.name) {
        Ok(name) => name,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    let email = match SubscriberEmail::parse(form.0.email) {
        Ok(email) => email,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    let subscriber: NewSubscriber = NewSubscriber { name, email };
    match insert_subscription(pool.get_ref(), &subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database.",
    skip(subscriber, pool)
)]
pub async fn insert_subscription(
    pool: &PgPool,
    subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, name, email, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        subscriber.name.as_ref(),
        subscriber.email.as_ref(),
        chrono::Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}.", e);
        e
    })?;

    Ok(())
}

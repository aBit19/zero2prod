use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub struct ApplicationBaseUrl(pub String);

#[tracing::instrument(name = "Adding a new subscriber.", skip(form, pool, email_client, base_url), fields(email=%form.email, name=%form.name))]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> HttpResponse {
    let subscriber: NewSubscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if insert_subscription(&pool, &subscriber).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    if send_welcome_email(&email_client, &subscriber, &base_url)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Sending welcome email to new subscriber.",
    skip(email_client, subscriber, base_url)
)]
async fn send_welcome_email(
    email_client: &EmailClient,
    subscriber: &NewSubscriber,
    base_url: &ApplicationBaseUrl,
) -> Result<(), ()> {
    let email_body = format!(
        "Welcome to our newsletter <a href=\"{}/subscriptions/confirm?token=erwrwrwe\">here</a>",
        base_url.0
    );

    email_client
        .send_email(&subscriber.email, "Welcome", &email_body, &email_body)
        .await
        .map_err(|_| ())?;

    Ok(())
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
        INSERT INTO subscriptions (id, name, email, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_verification')
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

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(NewSubscriber { name, email })
    }
}

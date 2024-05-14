use actix_web::{web, HttpResponse, ResponseError};
use rand::{thread_rng, Rng};
use reqwest::StatusCode;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

struct SubscriberId(Uuid);
impl SubscriberId {
    fn new() -> SubscriberId {
        SubscriberId(Uuid::new_v4())
    }
}

struct SubscriptionToken(String);
impl SubscriptionToken {
    fn new() -> SubscriptionToken {
        let mut rng = thread_rng();
        let token = std::iter::repeat_with(|| rng.sample(rand::distributions::Alphanumeric))
            .map(char::from)
            .take(25)
            .collect::<String>();

        SubscriptionToken(token)
    }
}

pub struct ApplicationBaseUrl(pub String);

#[tracing::instrument(
    name = "Adding a new subscriber.", 
    skip(form, pool, email_client, base_url),
    fields(email=%form.email, name=%form.name)
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscribeError> {
    let mut transaction = pool.begin().await?;
    let subscriber: NewSubscriber = form.0.try_into()?;
    let subscriber_id = insert_subscription(&mut transaction, &subscriber).await?;
    let subscription_token = insert_subscription_token(&mut transaction, &subscriber_id).await?;
    transaction.commit().await?;

    send_welcome_email(&email_client, &subscriber, &base_url, &subscription_token).await?;

    Ok(HttpResponse::Ok().finish())
}

async fn insert_subscription_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: &SubscriberId,
) -> Result<SubscriptionToken, sqlx::Error> {
    let subscription_token = SubscriptionToken::new();
    let query = sqlx::query!(
        r#"
        INSERT INTO subscription_tokens (subscription_id, token)
        VALUES ($1, $2)
        "#,
        subscriber_id.0,
        subscription_token.0,
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query");
        e
    })?;

    Ok(subscription_token)
}

#[tracing::instrument(
    name = "Sending welcome email to new subscriber.",
    skip(email_client, subscriber, base_url, subscription_token)
)]
async fn send_welcome_email(
    email_client: &EmailClient,
    subscriber: &NewSubscriber,
    base_url: &ApplicationBaseUrl,
    subscription_token: &SubscriptionToken,
) -> Result<(), reqwest::Error> {
    let email_body = format!(
        "Welcome to our newsletter <a href=\"{}/subscriptions/confirm?token={}\">here</a>",
        base_url.0, subscription_token.0
    );

    email_client
        .send_email(&subscriber.email, "Welcome", &email_body, &email_body)
        .await?;

    Ok(())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database.",
    skip(subscriber, transaction)
)]
pub async fn insert_subscription(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber: &NewSubscriber,
) -> Result<SubscriberId, sqlx::Error> {
    let subscriber_id = SubscriberId::new();
    let query = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, name, email, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_verification')
        "#,
        subscriber_id.0,
        subscriber.name.as_ref(),
        subscriber.email.as_ref(),
        chrono::Utc::now()
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query {:?}.", e);
        e
    })?;

    Ok(subscriber_id)
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(NewSubscriber { name, email })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SubscribeError {
    #[error("Failed to query.")]
    DatabaseError(#[from] sqlx::Error),
    #[error("{0}")]
    ValidationError(String),
    #[error("Error when sending a confirmation email")]
    ConfirmationError(#[from] reqwest::Error),
}

impl From<String> for SubscribeError {
    fn from(value: String) -> Self {
        SubscribeError::ValidationError(value)
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

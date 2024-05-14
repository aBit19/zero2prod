use actix_web::{web, HttpResponse};
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct Token {
    token: String,
}

#[derive(Debug)]
struct SubscriptionId(uuid::Uuid);
impl SubscriptionId {
    fn new(subscription_id: uuid::Uuid) -> Self {
        Self(subscription_id)
    }
}

#[tracing::instrument(name = "Confirming a subscription.", skip(token, pool))]
pub async fn confirm_subscription(
    token: web::Query<Token>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let subscription_id = match get_subscriber_id_from_token(&pool, &token.token).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let activate_subscription = activate_subscription(&pool, &subscription_id).await;

    match activate_subscription {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn get_subscriber_id_from_token(
    pool: &PgPool,
    token: &str,
) -> Result<SubscriptionId, sqlx::Error> {
    let fetch_one = sqlx::query!(
        r#"
        SELECT subscription_id from subscription_tokens
        WHERE token = $1
        "#,
        token
    )
    .fetch_one(pool)
    .await?;

    let subscription_id = fetch_one.subscription_id;

    Ok(SubscriptionId::new(subscription_id))
}

#[tracing::instrument(name = "Marking subscription as confirmed.", skip(pool))]
async fn activate_subscription(
    pool: &PgPool,
    subscription_id: &SubscriptionId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE subscriptions
        SET status = 'confirmed'
        WHERE id = $1
        "#,
        subscription_id.0
    )
    .execute(pool)
    .await?;

    Ok(())
}

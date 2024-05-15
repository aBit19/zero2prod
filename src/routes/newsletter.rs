use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct Newsletter {
    _title: String,
    _content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    _html: String,
    _text: String,
}

pub async fn publish_newsletter(_content: web::Json<Newsletter>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

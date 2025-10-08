use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct UrlRecord {
    pub id: String,
    pub url: String,
}

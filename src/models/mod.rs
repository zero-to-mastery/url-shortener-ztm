use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct UrlRecord {
    pub code: String,
    pub url: String,
}

#[derive(sqlx::FromRow)]
pub struct UpsertResult {
    pub id: i64,
    pub created: bool,
}

#[derive(sqlx::FromRow)]
pub struct Urls {
    pub id: i64,
    pub code: String,
}

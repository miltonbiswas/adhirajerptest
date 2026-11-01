use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ManufacturerQuery {
    pub search: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Manufacturer {
    pub id: Uuid,
    pub name: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct ManufacturerList {
    pub data: Vec<Manufacturer>,
    pub total: i64,
}

pub async fn list_manufacturers(
    State(pool): State<PgPool>,
    Query(query): Query<ManufacturerQuery>,
) -> Result<Json<ManufacturerList>, (StatusCode, String)> {
    let search = query.search.unwrap_or_default().trim().to_string();

    let limit = query.limit.unwrap_or(100).clamp(1, 500);

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM manufacturers
        WHERE is_active = TRUE
          AND ($1 = '' OR name ILIKE '%' || $1 || '%')
        "#,
    )
    .bind(&search)
    .fetch_one(&pool)
    .await
    .map_err(internal_error)?;

    let data = sqlx::query_as::<_, Manufacturer>(
        r#"
        SELECT id, name, is_active
        FROM manufacturers
        WHERE is_active = TRUE
          AND ($1 = '' OR name ILIKE '%' || $1 || '%')
        ORDER BY name ASC
        LIMIT $2
        "#,
    )
    .bind(&search)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(ManufacturerList { data, total }))
}

fn internal_error(error: sqlx::Error) -> (StatusCode, String) {
    tracing::error!("Manufacturer query failed: {error}");

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Unable to load manufacturers".to_string(),
    )
}

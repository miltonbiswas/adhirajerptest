use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub database: &'static str,
}

pub async fn health_check(
    State(pool): State<PgPool>,
) -> Result<Json<HealthResponse>, (StatusCode, Json<HealthResponse>)> {
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => Ok(Json(HealthResponse {
            status: "ok",
            database: "connected",
        })),
        Err(_) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: "error",
                database: "unavailable",
            }),
        )),
    }
}

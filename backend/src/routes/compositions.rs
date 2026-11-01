use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CompositionQuery {
    pub search: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Composition {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct CompositionList {
    pub data: Vec<Composition>,
    pub page: i64,
    pub limit: i64,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct CompositionInput {
    pub name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

fn validate_name(name: &str) -> Result<String, StatusCode> {
    let name = name.trim();

    if name.is_empty() || name.len() > 500 {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(name.to_string())
}

fn database_status(error: &sqlx::Error) -> StatusCode {
    if let sqlx::Error::Database(db_error) = error {
        match db_error.code().as_deref() {
            Some("23505") => return StatusCode::CONFLICT,
            Some("23503") => return StatusCode::CONFLICT,
            Some("23514") => return StatusCode::BAD_REQUEST,
            _ => {}
        }
    }

    tracing::error!("Composition database operation failed: {error}");
    StatusCode::INTERNAL_SERVER_ERROR
}

pub async fn list_compositions(
    State(pool): State<PgPool>,
    Query(query): Query<CompositionQuery>,
) -> Result<Json<CompositionList>, StatusCode> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = (page - 1) * limit;

    let search = query.search.unwrap_or_default().trim().to_string();
    let pattern = format!("%{}%", search);

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM compositions WHERE name ILIKE $1")
        .bind(&pattern)
        .fetch_one(&pool)
        .await
        .map_err(|error| database_status(&error))?;

    let data = sqlx::query_as::<_, Composition>(
        r#"
        SELECT id, name, description, is_active
        FROM compositions
        WHERE name ILIKE $1
        ORDER BY name ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(&pattern)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|error| database_status(&error))?;

    Ok(Json(CompositionList {
        data,
        page,
        limit,
        total,
    }))
}

pub async fn create_composition(
    State(pool): State<PgPool>,
    Json(input): Json<CompositionInput>,
) -> Result<(StatusCode, Json<Composition>), StatusCode> {
    let name = validate_name(&input.name)?;

    let composition = sqlx::query_as::<_, Composition>(
        r#"
        INSERT INTO compositions (name, description, is_active)
        VALUES ($1, $2, $3)
        RETURNING id, name, description, is_active
        "#,
    )
    .bind(name)
    .bind(input.description)
    .bind(input.is_active.unwrap_or(true))
    .fetch_one(&pool)
    .await
    .map_err(|error| database_status(&error))?;

    Ok((StatusCode::CREATED, Json(composition)))
}

pub async fn update_composition(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(input): Json<CompositionInput>,
) -> Result<Json<Composition>, StatusCode> {
    let name = validate_name(&input.name)?;

    let composition = sqlx::query_as::<_, Composition>(
        r#"
        UPDATE compositions
        SET
            name = $2,
            description = $3,
            is_active = $4,
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, name, description, is_active
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(input.description)
    .bind(input.is_active.unwrap_or(true))
    .fetch_optional(&pool)
    .await
    .map_err(|error| database_status(&error))?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(composition))
}

pub async fn delete_composition(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Soft delete: retain composition history and product mappings.
    let result = sqlx::query(
        r#"
        UPDATE compositions
        SET is_active = FALSE, updated_at = NOW()
        WHERE id = $1 AND is_active = TRUE
        "#,
    )
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|error| database_status(&error))?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

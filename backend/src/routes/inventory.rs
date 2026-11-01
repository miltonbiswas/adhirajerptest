use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/summary", get(get_inventory_summary))
        .route("/batches", get(list_inventory_batches))
        .route(
            "/adjustments",
            get(list_stock_adjustments).post(create_stock_adjustment),
        )
        .route("/adjustments/{id}", get(get_stock_adjustment))
}

// ============================================================
// INVENTORY SUMMARY
// ============================================================

#[derive(Debug, Serialize, FromRow)]
pub struct InventorySummary {
    pub total_products: i64,
    pub total_batches: i64,
    pub total_units: Decimal,
    pub low_stock: i64,
    pub out_of_stock: i64,
    pub expiring_soon: i64,
    pub expired: i64,
    pub stock_value: Decimal,
}

pub async fn get_inventory_summary(
    State(pool): State<PgPool>,
) -> Result<Json<InventorySummary>, ApiError> {
    let summary = sqlx::query_as::<_, InventorySummary>(
        r#"
        WITH product_stock AS (
            SELECT
                p.id AS product_id,

                COALESCE(
                    SUM(b.quantity_on_hand)
                    FILTER (WHERE b.is_active = TRUE),
                    0
                ) AS quantity,

                COALESCE(
                    SUM(b.reorder_level)
                    FILTER (WHERE b.is_active = TRUE),
                    0
                ) AS reorder_level

            FROM products p
            LEFT JOIN product_batches b
                ON b.product_id = p.id

            WHERE p.is_active = TRUE
            GROUP BY p.id
        ),

        batch_summary AS (
            SELECT
                COUNT(*) FILTER (
                    WHERE b.is_active = TRUE
                      AND b.quantity_on_hand > 0
                      AND b.expiry_date >= CURRENT_DATE
                      AND b.expiry_date <= CURRENT_DATE + 90
                )::BIGINT AS expiring_soon,

                COUNT(*) FILTER (
                    WHERE b.is_active = TRUE
                      AND b.quantity_on_hand > 0
                      AND b.expiry_date < CURRENT_DATE
                )::BIGINT AS expired,

                COUNT(*) FILTER (
                    WHERE b.is_active = TRUE
                )::BIGINT AS total_batches,

                COALESCE(
                    SUM(b.quantity_on_hand)
                    FILTER (WHERE b.is_active = TRUE),
                    0
                ) AS total_units,

                COALESCE(
                    SUM(b.quantity_on_hand * b.purchase_rate)
                    FILTER (WHERE b.is_active = TRUE),
                    0
                ) AS stock_value

            FROM product_batches b
            JOIN products p
                ON p.id = b.product_id

            WHERE p.is_active = TRUE
        )

        SELECT
            (SELECT COUNT(*) FROM product_stock)::BIGINT
                AS total_products,

            bs.total_batches,
            bs.total_units::NUMERIC AS total_units,

            (
                SELECT COUNT(*)
                FROM product_stock
                WHERE quantity > 0
                  AND quantity <= reorder_level
            )::BIGINT AS low_stock,

            (
                SELECT COUNT(*)
                FROM product_stock
                WHERE quantity = 0
            )::BIGINT AS out_of_stock,

            bs.expiring_soon,
            bs.expired,
            bs.stock_value::NUMERIC AS stock_value

        FROM batch_summary bs
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(ApiError::internal)?;

    Ok(Json(summary))
}

// ============================================================
// INVENTORY BATCH LIST
// ============================================================

#[derive(Debug, Deserialize)]
pub struct BatchQuery {
    pub search: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub stock_status: Option<String>,
    pub expiry_status: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct InventoryBatch {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub sku: String,
    pub brand_name: Option<String>,
    pub batch_number: String,
    pub manufacturing_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub quantity: Decimal,
    pub reorder_level: Decimal,
    pub unit: String,
    pub purchase_rate: Decimal,
    pub mrp: Decimal,
    pub selling_rate: Decimal,
}

#[derive(Debug, Serialize)]
pub struct BatchListResponse {
    pub data: Vec<InventoryBatch>,
    pub page: i64,
    pub limit: i64,
    pub total: i64,
}

pub async fn list_inventory_batches(
    State(pool): State<PgPool>,
    Query(params): Query<BatchQuery>,
) -> Result<Json<BatchListResponse>, ApiError> {
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(25).clamp(1, 100);
    let offset = (page - 1) * limit;

    let search = params.search.unwrap_or_default().trim().to_owned();
    let pattern = format!("%{search}%");

    let stock_status = params
        .stock_status
        .unwrap_or_else(|| "all".to_owned())
        .to_lowercase();

    let expiry_status = params
        .expiry_status
        .unwrap_or_else(|| "all".to_owned())
        .to_lowercase();

    let valid_stock_status = ["all", "in_stock", "low_stock", "out_of_stock"];
    let valid_expiry_status = ["all", "expired", "expiring_30", "expiring_90", "valid"];

    if !valid_stock_status.contains(&stock_status.as_str()) {
        return Err(ApiError::bad_request("Invalid stock_status filter"));
    }

    if !valid_expiry_status.contains(&expiry_status.as_str()) {
        return Err(ApiError::bad_request("Invalid expiry_status filter"));
    }

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM product_batches b
        JOIN products p ON p.id = b.product_id

        WHERE p.is_active = TRUE
          AND b.is_active = TRUE

          AND (
              $1 = ''
              OR p.name ILIKE $2
              OR p.sku ILIKE $2
              OR b.batch_number ILIKE $2
              OR COALESCE(p.brand_name, '') ILIKE $2
              OR COALESCE(p.generic_name, '') ILIKE $2
          )

          AND (
              $3 = 'all'
              OR ($3 = 'in_stock' AND b.quantity_on_hand > b.reorder_level)
              OR ($3 = 'low_stock'
                  AND b.quantity_on_hand > 0
                  AND b.quantity_on_hand <= b.reorder_level)
              OR ($3 = 'out_of_stock' AND b.quantity_on_hand = 0)
          )

          AND (
              $4 = 'all'
              OR ($4 = 'expired' AND b.expiry_date < CURRENT_DATE)
              OR ($4 = 'expiring_30'
                  AND b.expiry_date >= CURRENT_DATE
                  AND b.expiry_date <= CURRENT_DATE + 30)
              OR ($4 = 'expiring_90'
                  AND b.expiry_date >= CURRENT_DATE
                  AND b.expiry_date <= CURRENT_DATE + 90)
              OR ($4 = 'valid'
                  AND (b.expiry_date IS NULL OR b.expiry_date > CURRENT_DATE + 90))
          )
        "#,
    )
    .bind(&search)
    .bind(&pattern)
    .bind(&stock_status)
    .bind(&expiry_status)
    .fetch_one(&pool)
    .await
    .map_err(ApiError::internal)?;

    let data = sqlx::query_as::<_, InventoryBatch>(
        r#"
        SELECT
            b.id,
            p.id AS product_id,
            p.name AS product_name,
            p.sku,
            p.brand_name,
            b.batch_number,
            b.manufacturing_date,
            b.expiry_date,
            b.quantity_on_hand AS quantity,
            b.reorder_level,
            p.unit,
            b.purchase_rate,
            b.mrp,
            b.selling_rate

        FROM product_batches b
        JOIN products p
            ON p.id = b.product_id

        WHERE p.is_active = TRUE
          AND b.is_active = TRUE

          AND (
              $1 = ''
              OR p.name ILIKE $2
              OR p.sku ILIKE $2
              OR b.batch_number ILIKE $2
              OR COALESCE(p.brand_name, '') ILIKE $2
              OR COALESCE(p.generic_name, '') ILIKE $2
          )

          AND (
              $3 = 'all'
              OR ($3 = 'in_stock' AND b.quantity_on_hand > b.reorder_level)
              OR ($3 = 'low_stock'
                  AND b.quantity_on_hand > 0
                  AND b.quantity_on_hand <= b.reorder_level)
              OR ($3 = 'out_of_stock' AND b.quantity_on_hand = 0)
          )

          AND (
              $4 = 'all'
              OR ($4 = 'expired' AND b.expiry_date < CURRENT_DATE)
              OR ($4 = 'expiring_30'
                  AND b.expiry_date >= CURRENT_DATE
                  AND b.expiry_date <= CURRENT_DATE + 30)
              OR ($4 = 'expiring_90'
                  AND b.expiry_date >= CURRENT_DATE
                  AND b.expiry_date <= CURRENT_DATE + 90)
              OR ($4 = 'valid'
                  AND (b.expiry_date IS NULL OR b.expiry_date > CURRENT_DATE + 90))
          )

        ORDER BY
            b.expiry_date ASC NULLS LAST,
            p.name ASC,
            b.batch_number ASC

        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(&search)
    .bind(&pattern)
    .bind(&stock_status)
    .bind(&expiry_status)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(ApiError::internal)?;

    Ok(Json(BatchListResponse {
        data,
        page,
        limit,
        total,
    }))
}

// ============================================================
// STOCK ADJUSTMENTS
// ============================================================

#[derive(Debug, Deserialize)]
pub struct StockAdjustmentInput {
    pub product_id: Uuid,
    pub batch_id: Uuid,

    // IN, OUT, ADJUSTMENT
    pub movement_type: String,

    // IN / OUT: quantity to add or remove.
    // ADJUSTMENT: target quantity to set.
    pub quantity: Decimal,

    pub reason: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct StockMovement {
    pub id: Uuid,
    pub product_id: Uuid,
    pub batch_id: Uuid,
    pub movement_type: String,
    pub quantity_change: Decimal,
    pub quantity_before: Decimal,
    pub quantity_after: Decimal,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

pub async fn create_stock_adjustment(
    State(pool): State<PgPool>,
    Json(input): Json<StockAdjustmentInput>,
) -> Result<(StatusCode, Json<StockMovement>), ApiError> {
    let movement_type = input.movement_type.trim().to_uppercase();
    let reason = input.reason.trim().to_owned();

    if !["IN", "OUT", "ADJUSTMENT"].contains(&movement_type.as_str()) {
        return Err(ApiError::bad_request(
            "movement_type must be IN, OUT or ADJUSTMENT",
        ));
    }

    if input.quantity <= Decimal::ZERO {
        return Err(ApiError::bad_request("Quantity must be greater than zero"));
    }

    if reason.is_empty() {
        return Err(ApiError::bad_request("Reason is required"));
    }

    let mut tx = pool.begin().await.map_err(ApiError::internal)?;

    // Lock the selected batch so simultaneous adjustments
    // cannot overwrite one another.
    let batch = sqlx::query_as::<_, (Uuid, Decimal)>(
        r#"
        SELECT id, quantity_on_hand
        FROM product_batches
        WHERE id = $1
          AND product_id = $2
          AND is_active = TRUE
        FOR UPDATE
        "#,
    )
    .bind(input.batch_id)
    .bind(input.product_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(ApiError::internal)?;

    let (batch_id, before) = match batch {
        Some(batch) => batch,
        None => {
            return Err(ApiError::not_found(
                "Active batch not found for this product",
            ));
        }
    };

    let after = match movement_type.as_str() {
        "IN" => before + input.quantity,
        "OUT" => before - input.quantity,
        "ADJUSTMENT" => input.quantity,
        _ => unreachable!(),
    };

    if after < Decimal::ZERO {
        return Err(ApiError::conflict("Insufficient stock for this stock-out"));
    }

    let change = after - before;

    if change == Decimal::ZERO {
        return Err(ApiError::bad_request(
            "Adjustment does not change the current stock",
        ));
    }

    sqlx::query(
        r#"
        UPDATE product_batches
        SET quantity_on_hand = $1,
            updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(after)
    .bind(batch_id)
    .execute(&mut *tx)
    .await
    .map_err(ApiError::internal)?;

    let movement = sqlx::query_as::<_, StockMovement>(
        r#"
        INSERT INTO stock_movements (
            product_id,
            batch_id,
            movement_type,
            quantity_change,
            quantity_before,
            quantity_after,
            reason
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)

        RETURNING
            id,
            product_id,
            batch_id,
            movement_type,
            quantity_change,
            quantity_before,
            quantity_after,
            reason,
            created_at
        "#,
    )
    .bind(input.product_id)
    .bind(batch_id)
    .bind(&movement_type)
    .bind(change)
    .bind(before)
    .bind(after)
    .bind(reason)
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::internal)?;

    tx.commit().await.map_err(ApiError::internal)?;

    Ok((StatusCode::CREATED, Json(movement)))
}

#[derive(Debug, Deserialize)]
pub struct MovementQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub product_id: Option<Uuid>,
    pub batch_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct MovementListResponse {
    pub data: Vec<StockMovement>,
    pub page: i64,
    pub limit: i64,
    pub total: i64,
}

pub async fn list_stock_adjustments(
    State(pool): State<PgPool>,
    Query(params): Query<MovementQuery>,
) -> Result<Json<MovementListResponse>, ApiError> {
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(25).clamp(1, 100);
    let offset = (page - 1) * limit;

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM stock_movements
        WHERE ($1::UUID IS NULL OR product_id = $1)
          AND ($2::UUID IS NULL OR batch_id = $2)
        "#,
    )
    .bind(params.product_id)
    .bind(params.batch_id)
    .fetch_one(&pool)
    .await
    .map_err(ApiError::internal)?;

    let data = sqlx::query_as::<_, StockMovement>(
        r#"
        SELECT
            id,
            product_id,
            batch_id,
            movement_type,
            quantity_change,
            quantity_before,
            quantity_after,
            reason,
            created_at
        FROM stock_movements
        WHERE ($1::UUID IS NULL OR product_id = $1)
          AND ($2::UUID IS NULL OR batch_id = $2)
        ORDER BY created_at DESC, id DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(params.product_id)
    .bind(params.batch_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(ApiError::internal)?;

    Ok(Json(MovementListResponse {
        data,
        page,
        limit,
        total,
    }))
}

pub async fn get_stock_adjustment(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<StockMovement>, ApiError> {
    let movement = sqlx::query_as::<_, StockMovement>(
        r#"
        SELECT
            id,
            product_id,
            batch_id,
            movement_type,
            quantity_change,
            quantity_before,
            quantity_after,
            reason,
            created_at
        FROM stock_movements
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(ApiError::internal)?
    .ok_or_else(|| ApiError::not_found("Stock movement not found"))?;

    Ok(Json(movement))
}

// ============================================================
// API ERROR HANDLING
// ============================================================

pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.to_owned(),
        }
    }

    fn not_found(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.to_owned(),
        }
    }

    fn conflict(message: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.to_owned(),
        }
    }

    fn internal<E: std::fmt::Display>(error: E) -> Self {
        tracing::error!("Inventory API error: {error}");

        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Inventory operation failed. Check backend logs.".to_owned(),
        }
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(serde_json::json!({
                "error": self.message
            })),
        )
            .into_response()
    }
}

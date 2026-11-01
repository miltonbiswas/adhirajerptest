use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

// ============================================================
// REQUEST MODELS
// ============================================================

#[derive(Debug, Deserialize)]
pub struct ProductQuery {
    pub search: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ProductInput {
    pub sku: String,
    pub name: String,

    pub generic_name: Option<String>,
    pub brand_name: Option<String>,
    pub brand_id: Option<Uuid>,

    #[serde(default)]
    pub composition_ids: Option<Vec<Uuid>>,

    pub dosage_form: Option<String>,
    pub strength: Option<String>,
    pub pack_size: Option<String>,
    pub unit: Option<String>,
    pub hsn_code: Option<String>,

    // Retained for backward compatibility.
    // The server ignores this value and derives GST from HSN master.
    #[serde(default)]
    pub gst_rate: Option<Decimal>,

    pub barcode: Option<String>,
    pub is_prescription_required: Option<bool>,
    pub is_active: Option<bool>,

    pub schedule_class: Option<String>,
    pub is_narcotic: Option<bool>,
    pub units_per_pack: Option<Decimal>,
}

// ============================================================
// RESPONSE MODELS
// ============================================================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Product {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub generic_name: Option<String>,

    pub brand_id: Option<Uuid>,
    pub brand_name: Option<String>,

    pub composition_ids: Vec<Uuid>,
    pub composition_names: Vec<String>,

    pub dosage_form: Option<String>,
    pub strength: Option<String>,
    pub pack_size: Option<String>,
    pub unit: String,
    pub hsn_code: Option<String>,
    pub gst_rate: Decimal,
    pub barcode: Option<String>,
    pub is_prescription_required: bool,
    pub is_active: bool,

    pub schedule_class: String,
    pub is_narcotic: bool,
    pub units_per_pack: Decimal,
}

#[derive(Debug, Serialize)]
pub struct ProductList {
    pub data: Vec<Product>,
    pub page: i64,
    pub limit: i64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
}

// ============================================================
// SHARED PRODUCT SELECT
// ============================================================

// ============================================================
// GET /api/products
// ============================================================

pub async fn list_products(
    State(pool): State<PgPool>,
    Query(query): Query<ProductQuery>,
) -> Result<Json<ProductList>, (StatusCode, Json<ApiError>)> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = (page - 1) * limit;
    let search = query.search.unwrap_or_default().trim().to_owned();

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM public.products p
        LEFT JOIN public.brands b ON b.id = p.brand_id
        WHERE $1 = ''
           OR p.name ILIKE '%' || $1 || '%'
           OR p.sku ILIKE '%' || $1 || '%'
           OR COALESCE(p.generic_name, '') ILIKE '%' || $1 || '%'
           OR COALESCE(b.name, p.brand_name, '') ILIKE '%' || $1 || '%'
           OR EXISTS (
               SELECT 1
               FROM public.product_compositions pc
               JOIN public.compositions c
                   ON c.id = pc.composition_id
               WHERE pc.product_id = p.id
                 AND c.name ILIKE '%' || $1 || '%'
           )
        "#,
    )
    .bind(&search)
    .fetch_one(&pool)
    .await
    .map_err(internal_error)?;

    let data = sqlx::query_as::<_, Product>(
        r#"
    SELECT
        p.id,
        p.sku,
        p.name,
        p.generic_name,
        p.brand_id,
        COALESCE(b.name, p.brand_name) AS brand_name,

        COALESCE(
            (
                SELECT array_agg(pc.composition_id ORDER BY c.name)
                FROM public.product_compositions pc
                JOIN public.compositions c
                    ON c.id = pc.composition_id
                WHERE pc.product_id = p.id
            ),
            ARRAY[]::UUID[]
        ) AS composition_ids,

        COALESCE(
            (
                SELECT array_agg(c.name ORDER BY c.name)
                FROM public.product_compositions pc
                JOIN public.compositions c
                    ON c.id = pc.composition_id
                WHERE pc.product_id = p.id
            ),
            ARRAY[]::TEXT[]
        ) AS composition_names,

        p.dosage_form,
        p.strength,
        p.pack_size,
        p.unit,
        p.hsn_code,
        p.gst_rate,
        p.barcode,
        p.is_prescription_required,
        p.is_active,
        p.schedule_class,
        p.is_narcotic,
        p.units_per_pack

    FROM public.products p
    LEFT JOIN public.brands b ON b.id = p.brand_id

    WHERE $1 = ''
       OR p.name ILIKE '%' || $1 || '%'
       OR p.sku ILIKE '%' || $1 || '%'
       OR COALESCE(p.generic_name, '') ILIKE '%' || $1 || '%'
       OR COALESCE(b.name, p.brand_name, '') ILIKE '%' || $1 || '%'
       OR EXISTS (
           SELECT 1
           FROM public.product_compositions pc
           JOIN public.compositions c
               ON c.id = pc.composition_id
           WHERE pc.product_id = p.id
             AND c.name ILIKE '%' || $1 || '%'
       )

    ORDER BY p.name, p.sku
    LIMIT $2 OFFSET $3
    "#,
    )
    .bind(&search)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    let response = ProductList {
        data,
        page,
        limit,
        total,
    };

    Ok(Json(response))
}

// ============================================================
// GET /api/products/{id}
// ============================================================

pub async fn get_product(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Product>, (StatusCode, Json<ApiError>)> {
    let product = fetch_product(&pool, id).await?;
    Ok(Json(product))
}

async fn fetch_product(pool: &PgPool, id: Uuid) -> Result<Product, (StatusCode, Json<ApiError>)> {
    sqlx::query_as::<_, Product>(
        r#"
        SELECT
            p.id,
            p.sku,
            p.name,
            p.generic_name,
            p.brand_id,
            COALESCE(b.name, p.brand_name) AS brand_name,

            COALESCE(
                (
                    SELECT array_agg(pc.composition_id ORDER BY c.name)
                    FROM public.product_compositions pc
                    JOIN public.compositions c
                        ON c.id = pc.composition_id
                    WHERE pc.product_id = p.id
                ),
                ARRAY[]::UUID[]
            ) AS composition_ids,

            COALESCE(
                (
                    SELECT array_agg(c.name ORDER BY c.name)
                    FROM public.product_compositions pc
                    JOIN public.compositions c
                        ON c.id = pc.composition_id
                    WHERE pc.product_id = p.id
                ),
                ARRAY[]::TEXT[]
            ) AS composition_names,

            p.dosage_form,
            p.strength,
            p.pack_size,
            p.unit,
            p.hsn_code,
            p.gst_rate,
            p.barcode,
            p.is_prescription_required,
            p.is_active,
            p.schedule_class,
            p.is_narcotic,
            p.units_per_pack

        FROM public.products p
        LEFT JOIN public.brands b ON b.id = p.brand_id
        WHERE p.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(not_found)
}

// ============================================================
// HSN LOOKUP
// HSN MASTER IS THE SOURCE OF GST RATE
// ============================================================

async fn get_active_hsn_rate(
    tx: &mut Transaction<'_, Postgres>,
    hsn_code: &str,
) -> Result<Decimal, (StatusCode, Json<ApiError>)> {
    let code = hsn_code.trim();

    if code.is_empty() {
        return Err(bad_request("HSN code cannot be empty"));
    }

    let rate: Option<Decimal> = sqlx::query_scalar(
        r#"
        SELECT gst_rate
        FROM public.hsn_master
        WHERE hsn_code = $1
          AND is_active = TRUE
        FOR SHARE
        "#,
    )
    .bind(code)
    .fetch_optional(&mut **tx)
    .await
    .map_err(internal_error)?;

    rate.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: format!("HSN code '{code}' does not exist or is inactive"),
            }),
        )
    })
}

// ============================================================
// POST /api/products
// ============================================================

pub async fn create_product(
    State(pool): State<PgPool>,
    Json(input): Json<ProductInput>,
) -> Result<(StatusCode, Json<Product>), (StatusCode, Json<ApiError>)> {
    let sku = input.sku.trim();
    let name = input.name.trim();

    validate_product(
        sku,
        name,
        input.schedule_class.as_deref(),
        input.units_per_pack,
    )?;

    let schedule_class = normalized_schedule(input.schedule_class.as_deref());
    let units_per_pack = input.units_per_pack.unwrap_or(Decimal::ONE);
    let composition_ids = input.composition_ids.unwrap_or_default();

    validate_composition_ids(&composition_ids)?;

    let hsn_code = input
        .hsn_code
        .as_deref()
        .map(str::trim)
        .filter(|code| !code.is_empty())
        .map(str::to_owned);

    let mut tx = pool.begin().await.map_err(internal_error)?;

    let gst_rate = match hsn_code.as_deref() {
        Some(code) => get_active_hsn_rate(&mut tx, code).await?,
        None => Decimal::ZERO,
    };

    let id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO public.products (
            sku,
            name,
            generic_name,
            brand_name,
            brand_id,
            dosage_form,
            strength,
            pack_size,
            unit,
            hsn_code,
            gst_rate,
            barcode,
            is_prescription_required,
            is_active,
            schedule_class,
            is_narcotic,
            units_per_pack
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8,
            COALESCE($9, 'PCS'),
            $10,
            $11,
            $12,
            COALESCE($13, false),
            COALESCE($14, true),
            $15,
            COALESCE($16, false),
            $17
        )
        RETURNING id
        "#,
    )
    .bind(sku)
    .bind(name)
    .bind(input.generic_name)
    .bind(input.brand_name)
    .bind(input.brand_id)
    .bind(input.dosage_form)
    .bind(input.strength)
    .bind(input.pack_size)
    .bind(input.unit)
    .bind(&hsn_code)
    .bind(gst_rate)
    .bind(input.barcode)
    .bind(input.is_prescription_required)
    .bind(input.is_active)
    .bind(schedule_class)
    .bind(input.is_narcotic)
    .bind(units_per_pack)
    .fetch_one(&mut *tx)
    .await
    .map_err(write_error)?;

    replace_compositions(&mut tx, id, &composition_ids).await?;

    tx.commit().await.map_err(internal_error)?;

    let product = fetch_product(&pool, id).await?;

    Ok((StatusCode::CREATED, Json(product)))
}

// ============================================================
// PUT /api/products/{id}
//
// HSN behaviour:
// - HSN omitted: preserve existing HSN.
// - HSN supplied: validate and use its current GST rate.
// - Empty string: clear HSN and set GST to zero.
// ============================================================

pub async fn update_product(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(input): Json<ProductInput>,
) -> Result<Json<Product>, (StatusCode, Json<ApiError>)> {
    let sku = input.sku.trim();
    let name = input.name.trim();

    validate_product(
        sku,
        name,
        input.schedule_class.as_deref(),
        input.units_per_pack,
    )?;

    let schedule_class = normalized_schedule(input.schedule_class.as_deref());

    // Read current HSN without locking. We acquire HSN lock before
    // product lock to keep the same lock order as update_hsn().
    let current_hsn: Option<Option<String>> =
        sqlx::query_scalar("SELECT hsn_code FROM public.products WHERE id = $1")
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(internal_error)?;

    let current_hsn = current_hsn.ok_or_else(not_found)?;

    let requested_hsn = input
        .hsn_code
        .as_deref()
        .map(str::trim)
        .filter(|code| !code.is_empty())
        .map(str::to_owned);

    let target_hsn = match input.hsn_code.as_deref() {
        // Missing or null preserves existing mapping.
        None => current_hsn.clone(),
        // Empty string explicitly clears the mapping.
        Some(value) if value.trim().is_empty() => None,
        Some(_) => requested_hsn,
    };

    let mut tx = pool.begin().await.map_err(internal_error)?;

    // Lock the target HSN before locking the product row.
    let gst_rate = match target_hsn.as_deref() {
        Some(code) => get_active_hsn_rate(&mut tx, code).await?,
        None => Decimal::ZERO,
    };

    // Lock product row and verify that its mapping did not change
    // between the initial read and lock acquisition.
    let locked_hsn: Option<Option<String>> = sqlx::query_scalar(
        r#"
        SELECT hsn_code
        FROM public.products
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal_error)?;

    let locked_hsn = locked_hsn.ok_or_else(not_found)?;

    if locked_hsn != current_hsn {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiError {
                error: "Product HSN was changed by another request. Please reload and try again."
                    .to_owned(),
            }),
        ));
    }

    let result = sqlx::query(
        r#"
        UPDATE public.products
        SET
            sku = $2,
            name = $3,
            generic_name = $4,
            brand_name = $5,
            brand_id = COALESCE($6, brand_id),
            dosage_form = $7,
            strength = $8,
            pack_size = $9,
            unit = COALESCE($10, 'PCS'),
            hsn_code = $11,
            gst_rate = $12,
            barcode = $13,
            is_prescription_required = COALESCE($14, false),
            is_active = COALESCE($15, true),
            schedule_class = $16,
            is_narcotic = COALESCE($17, false),
            units_per_pack = COALESCE($18, 1),
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(sku)
    .bind(name)
    .bind(&input.generic_name)
    .bind(&input.brand_name)
    .bind(input.brand_id)
    .bind(&input.dosage_form)
    .bind(&input.strength)
    .bind(&input.pack_size)
    .bind(&input.unit)
    .bind(&target_hsn)
    .bind(gst_rate)
    .bind(&input.barcode)
    .bind(input.is_prescription_required)
    .bind(input.is_active)
    .bind(schedule_class)
    .bind(input.is_narcotic)
    .bind(input.units_per_pack)
    .execute(&mut *tx)
    .await
    .map_err(write_error)?;

    if result.rows_affected() == 0 {
        return Err(not_found());
    }

    if let Some(composition_ids) = input.composition_ids {
        validate_composition_ids(&composition_ids)?;
        replace_compositions(&mut tx, id, &composition_ids).await?;
    }

    tx.commit().await.map_err(internal_error)?;

    let product = fetch_product(&pool, id).await?;

    Ok(Json(product))
}

// ============================================================
// COMPOSITION HELPERS
// ============================================================

fn validate_composition_ids(ids: &[Uuid]) -> Result<(), (StatusCode, Json<ApiError>)> {
    let mut unique = std::collections::HashSet::new();

    for id in ids {
        if !unique.insert(*id) {
            return Err(bad_request("Duplicate composition IDs are not allowed"));
        }
    }

    Ok(())
}

async fn replace_compositions(
    tx: &mut Transaction<'_, Postgres>,
    product_id: Uuid,
    composition_ids: &[Uuid],
) -> Result<(), (StatusCode, Json<ApiError>)> {
    sqlx::query("DELETE FROM public.product_compositions WHERE product_id = $1")
        .bind(product_id)
        .execute(&mut **tx)
        .await
        .map_err(write_error)?;

    for composition_id in composition_ids {
        sqlx::query(
            r#"
            INSERT INTO public.product_compositions (
                product_id,
                composition_id
            )
            VALUES ($1, $2)
            "#,
        )
        .bind(product_id)
        .bind(composition_id)
        .execute(&mut **tx)
        .await
        .map_err(write_error)?;
    }

    Ok(())
}

// ============================================================
// DELETE /api/products/{id}
// ============================================================

pub async fn delete_product(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let result = sqlx::query("DELETE FROM public.products WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(write_error)?;

    if result.rows_affected() == 0 {
        return Err(not_found());
    }

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================
// VALIDATION
// ============================================================

fn normalized_schedule(value: Option<&str>) -> String {
    value.unwrap_or("OTC").trim().to_uppercase()
}

fn validate_product(
    sku: &str,
    name: &str,
    schedule_class: Option<&str>,
    units_per_pack: Option<Decimal>,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    if sku.is_empty() || name.is_empty() {
        return Err(bad_request("SKU and product name are required"));
    }

    if sku.len() > 80 || name.len() > 250 {
        return Err(bad_request(
            "SKU or product name exceeds the allowed length",
        ));
    }

    if let Some(class) = schedule_class {
        let class = class.trim().to_uppercase();

        if !["OTC", "H", "H1", "X"].contains(&class.as_str()) {
            return Err(bad_request("Schedule class must be OTC, H, H1 or X"));
        }
    }

    if let Some(units) = units_per_pack {
        if units <= Decimal::ZERO {
            return Err(bad_request("Units per pack must be greater than zero"));
        }
    }

    Ok(())
}

// ============================================================
// ERROR HELPERS
// ============================================================

fn bad_request(message: &str) -> (StatusCode, Json<ApiError>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiError {
            error: message.to_owned(),
        }),
    )
}

fn not_found() -> (StatusCode, Json<ApiError>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiError {
            error: "Product not found".to_owned(),
        }),
    )
}

fn internal_error(error: sqlx::Error) -> (StatusCode, Json<ApiError>) {
    tracing::error!("Product query failed: {error}");

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            error: "Database query failed".to_owned(),
        }),
    )
}

fn write_error(error: sqlx::Error) -> (StatusCode, Json<ApiError>) {
    if let sqlx::Error::Database(ref db_error) = error {
        if db_error.is_unique_violation() {
            return (
                StatusCode::CONFLICT,
                Json(ApiError {
                    error: "SKU or another unique field already exists".to_owned(),
                }),
            );
        }

        if db_error.is_foreign_key_violation() {
            return (
                StatusCode::CONFLICT,
                Json(ApiError {
                    error: "A referenced brand, composition, HSN or related record does not exist, or the product is referenced elsewhere".to_owned(),
                }),
            );
        }

        if db_error.is_check_violation() {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "One or more product values violate a database constraint".to_owned(),
                }),
            );
        }
    }

    tracing::error!("Product write failed: {error}");

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            error: "Product operation failed".to_owned(),
        }),
    )
}

use axum::{
    Json, Router,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// ============================================================
// MODELS
// ============================================================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Hsn {
    pub hsn_code: String,
    pub description: String,
    pub gst_rate: Decimal,
    pub cgst_rate: Decimal,
    pub sgst_rate: Decimal,
    pub igst_rate: Decimal,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct HsnList {
    pub data: Vec<Hsn>,
    pub page: i64,
    pub limit: i64,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct HsnQuery {
    pub search: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct HsnInput {
    pub hsn_code: String,
    pub description: Option<String>,
    pub gst_rate: Decimal,
    pub cgst_rate: Decimal,
    pub sgst_rate: Decimal,
    pub igst_rate: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct HsnStatusInput {
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
}

#[derive(Debug, Deserialize)]
struct HsnCsvRow {
    hsn_code: String,
    gst_rate: Decimal,
    cgst_rate: Decimal,
    sgst_rate: Decimal,
    igst_rate: Decimal,
}

#[derive(Debug, Serialize)]
pub struct HsnImportError {
    pub row: usize,
    pub hsn_code: String,
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct HsnImportResult {
    pub total_rows: usize,
    pub inserted: usize,
    pub skipped: usize,
    pub rejected: usize,
    pub errors: Vec<HsnImportError>,
}

// ============================================================
// ROUTER
// ============================================================

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", get(list_hsn).post(create_hsn))
        .route("/import", post(import_hsn_csv))
        .route("/{code}", get(get_hsn).put(update_hsn))
        .route("/{code}/status", patch(update_hsn_status))
}

// ============================================================
// LIST HSN
// ============================================================

pub async fn list_hsn(
    State(pool): State<PgPool>,
    Query(query): Query<HsnQuery>,
) -> Result<Json<HsnList>, (StatusCode, Json<ApiError>)> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = (page - 1) * limit;
    let search = query.search.unwrap_or_default().trim().to_owned();

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM public.hsn_master
        WHERE
            ($1 = '' OR hsn_code ILIKE '%' || $1 || '%'
             OR description ILIKE '%' || $1 || '%')
            AND ($2::BOOLEAN IS NULL OR is_active = $2)
        "#,
    )
    .bind(&search)
    .bind(query.active)
    .fetch_one(&pool)
    .await
    .map_err(internal_error)?;

    let data = sqlx::query_as::<_, Hsn>(
        r#"
        SELECT
            hsn_code, description, gst_rate,
            cgst_rate, sgst_rate, igst_rate,
            is_active, created_at, updated_at
        FROM public.hsn_master
        WHERE
            ($1 = '' OR hsn_code ILIKE '%' || $1 || '%'
             OR description ILIKE '%' || $1 || '%')
            AND ($2::BOOLEAN IS NULL OR is_active = $2)
        ORDER BY hsn_code
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(&search)
    .bind(query.active)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(HsnList {
        data,
        page,
        limit,
        total,
    }))
}

// ============================================================
// GET SINGLE HSN
// ============================================================

pub async fn get_hsn(
    State(pool): State<PgPool>,
    Path(code): Path<String>,
) -> Result<Json<Hsn>, (StatusCode, Json<ApiError>)> {
    let hsn = sqlx::query_as::<_, Hsn>(
        r#"
        SELECT
            hsn_code, description, gst_rate,
            cgst_rate, sgst_rate, igst_rate,
            is_active, created_at, updated_at
        FROM public.hsn_master
        WHERE hsn_code = $1
        "#,
    )
    .bind(code.trim())
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(not_found)?;

    Ok(Json(hsn))
}

// ============================================================
// CREATE HSN
// ============================================================

pub async fn create_hsn(
    State(pool): State<PgPool>,
    Json(input): Json<HsnInput>,
) -> Result<(StatusCode, Json<Hsn>), (StatusCode, Json<ApiError>)> {
    let code = input.hsn_code.trim();
    validate_hsn(code, &input)?;

    let description = input.description.as_deref().unwrap_or("").trim().to_owned();

    let hsn = sqlx::query_as::<_, Hsn>(
        r#"
        INSERT INTO public.hsn_master (
            hsn_code, description, gst_rate,
            cgst_rate, sgst_rate, igst_rate
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            hsn_code, description, gst_rate,
            cgst_rate, sgst_rate, igst_rate,
            is_active, created_at, updated_at
        "#,
    )
    .bind(code)
    .bind(description)
    .bind(input.gst_rate)
    .bind(input.cgst_rate)
    .bind(input.sgst_rate)
    .bind(input.igst_rate)
    .fetch_one(&pool)
    .await
    .map_err(write_error)?;

    Ok((StatusCode::CREATED, Json(hsn)))
}

// ============================================================
// UPDATE HSN + AUTOMATIC PRODUCT GST SYNCHRONIZATION
// ============================================================

pub async fn update_hsn(
    State(pool): State<PgPool>,
    Path(code): Path<String>,
    Json(input): Json<HsnInput>,
) -> Result<Json<Hsn>, (StatusCode, Json<ApiError>)> {
    let code = code.trim();

    if input.hsn_code.trim() != code {
        return Err(bad_request(
            "The HSN code in the URL and request body must match",
        ));
    }

    validate_hsn(code, &input)?;

    let description = input.description.as_deref().unwrap_or("").trim().to_owned();

    let mut tx = pool.begin().await.map_err(internal_error)?;

    // Lock the HSN row to serialize concurrent HSN changes
    // and product assignments.
    let exists: Option<String> = sqlx::query_scalar(
        r#"
        SELECT hsn_code
        FROM public.hsn_master
        WHERE hsn_code = $1
        FOR UPDATE
        "#,
    )
    .bind(code)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal_error)?;

    if exists.is_none() {
        return Err(not_found());
    }

    let hsn = sqlx::query_as::<_, Hsn>(
        r#"
        UPDATE public.hsn_master
        SET
            description = $2,
            gst_rate = $3,
            cgst_rate = $4,
            sgst_rate = $5,
            igst_rate = $6,
            updated_at = NOW()
        WHERE hsn_code = $1
        RETURNING
            hsn_code, description, gst_rate,
            cgst_rate, sgst_rate, igst_rate,
            is_active, created_at, updated_at
        "#,
    )
    .bind(code)
    .bind(description)
    .bind(input.gst_rate)
    .bind(input.cgst_rate)
    .bind(input.sgst_rate)
    .bind(input.igst_rate)
    .fetch_one(&mut *tx)
    .await
    .map_err(write_error)?;

    // Update all products linked to this HSN.
    // Historical purchase item tax snapshots are not modified.
    sqlx::query(
        r#"
        UPDATE public.products
        SET
            gst_rate = $2,
            updated_at = NOW()
        WHERE hsn_code = $1
        "#,
    )
    .bind(code)
    .bind(hsn.gst_rate)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    tx.commit().await.map_err(internal_error)?;

    Ok(Json(hsn))
}

// ============================================================
// ACTIVATE / DEACTIVATE HSN
// ============================================================

pub async fn update_hsn_status(
    State(pool): State<PgPool>,
    Path(code): Path<String>,
    Json(input): Json<HsnStatusInput>,
) -> Result<Json<Hsn>, (StatusCode, Json<ApiError>)> {
    let code = code.trim();

    let mut tx = pool.begin().await.map_err(internal_error)?;

    let exists: Option<String> = sqlx::query_scalar(
        r#"
        SELECT hsn_code
        FROM public.hsn_master
        WHERE hsn_code = $1
        FOR UPDATE
        "#,
    )
    .bind(code)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal_error)?;

    if exists.is_none() {
        return Err(not_found());
    }

    if !input.is_active {
        let linked_products: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM public.products
            WHERE hsn_code = $1
              AND is_active = TRUE
            "#,
        )
        .bind(code)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal_error)?;

        if linked_products > 0 {
            return Err((
                StatusCode::CONFLICT,
                Json(ApiError {
                    error: format!(
                        "Cannot deactivate this HSN because {linked_products} active products are linked to it. Reassign those products first."
                    ),
                }),
            ));
        }
    }

    let hsn = sqlx::query_as::<_, Hsn>(
        r#"
        UPDATE public.hsn_master
        SET
            is_active = $2,
            updated_at = NOW()
        WHERE hsn_code = $1
        RETURNING
            hsn_code, description, gst_rate,
            cgst_rate, sgst_rate, igst_rate,
            is_active, created_at, updated_at
        "#,
    )
    .bind(code)
    .bind(input.is_active)
    .fetch_one(&mut *tx)
    .await
    .map_err(write_error)?;

    tx.commit().await.map_err(internal_error)?;

    Ok(Json(hsn))
}

// ============================================================
// CSV BULK IMPORT
// Existing records are never overwritten.
// ============================================================

pub async fn import_hsn_csv(
    State(pool): State<PgPool>,
    mut multipart: Multipart,
) -> Result<Json<HsnImportResult>, (StatusCode, Json<ApiError>)> {
    let mut csv_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| bad_request("Could not read uploaded file"))?
    {
        if field.name() == Some("file") {
            let bytes = field
                .bytes()
                .await
                .map_err(|_| bad_request("Could not read CSV file"))?;

            if bytes.len() > 5 * 1024 * 1024 {
                return Err(bad_request("CSV file must be 5 MB or smaller"));
            }

            csv_bytes = Some(bytes.to_vec());
            break;
        }
    }

    let bytes =
        csv_bytes.ok_or_else(|| bad_request("Upload a CSV using the multipart field 'file'"))?;

    if bytes.is_empty() {
        return Err(bad_request("CSV file is empty"));
    }

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(bytes.as_slice());

    let headers = reader
        .headers()
        .map_err(|_| bad_request("Could not parse CSV headers"))?
        .clone();

    let required_headers = [
        "hsn_code",
        "gst_rate",
        "cgst_rate",
        "sgst_rate",
        "igst_rate",
    ];

    for required in required_headers {
        if !headers.iter().any(|header| header == required) {
            return Err(bad_request(&format!(
                "CSV is missing required column: {required}"
            )));
        }
    }

    let mut result = HsnImportResult {
        total_rows: 0,
        inserted: 0,
        skipped: 0,
        rejected: 0,
        errors: Vec::new(),
    };

    let mut tx = pool.begin().await.map_err(internal_error)?;

    for (index, record) in reader.deserialize::<HsnCsvRow>().enumerate() {
        result.total_rows += 1;
        let row_number = index + 2;

        let row = match record {
            Ok(row) => row,
            Err(error) => {
                result.rejected += 1;
                result.errors.push(HsnImportError {
                    row: row_number,
                    hsn_code: String::new(),
                    error: format!("Invalid CSV row: {error}"),
                });
                continue;
            }
        };

        let code = row.hsn_code.trim();

        if !is_valid_hsn_code(code) {
            result.rejected += 1;
            result.errors.push(HsnImportError {
                row: row_number,
                hsn_code: code.to_owned(),
                error: "HSN code must contain 1-8 ASCII letters, digits or dots".to_owned(),
            });
            continue;
        }

        let rates = [row.gst_rate, row.cgst_rate, row.sgst_rate, row.igst_rate];

        if rates
            .iter()
            .any(|rate| *rate < Decimal::ZERO || *rate > Decimal::from(100))
        {
            result.rejected += 1;
            result.errors.push(HsnImportError {
                row: row_number,
                hsn_code: code.to_owned(),
                error: "Tax rates must be between 0 and 100".to_owned(),
            });
            continue;
        }

        let insert_result = sqlx::query(
            r#"
            INSERT INTO public.hsn_master (
                hsn_code, description, gst_rate,
                cgst_rate, sgst_rate, igst_rate
            )
            VALUES ($1, '', $2, $3, $4, $5)
            ON CONFLICT (hsn_code) DO NOTHING
            "#,
        )
        .bind(code)
        .bind(row.gst_rate)
        .bind(row.cgst_rate)
        .bind(row.sgst_rate)
        .bind(row.igst_rate)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

        if insert_result.rows_affected() == 1 {
            result.inserted += 1;
        } else {
            result.skipped += 1;
        }
    }

    tx.commit().await.map_err(internal_error)?;

    Ok(Json(result))
}

// ============================================================
// VALIDATION
// ============================================================

fn is_valid_hsn_code(code: &str) -> bool {
    !code.is_empty()
        && code.len() <= 8
        && code
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'.')
}

fn validate_hsn(code: &str, input: &HsnInput) -> Result<(), (StatusCode, Json<ApiError>)> {
    if !is_valid_hsn_code(code) {
        return Err(bad_request(
            "HSN code must contain 1-8 ASCII letters, digits or dots",
        ));
    }

    let rates = [
        input.gst_rate,
        input.cgst_rate,
        input.sgst_rate,
        input.igst_rate,
    ];

    if rates
        .iter()
        .any(|rate| *rate < Decimal::ZERO || *rate > Decimal::from(100))
    {
        return Err(bad_request("Tax rates must be between 0 and 100"));
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
            error: "HSN code not found".to_owned(),
        }),
    )
}

fn internal_error(error: sqlx::Error) -> (StatusCode, Json<ApiError>) {
    tracing::error!("HSN database operation failed: {error}");

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            error: "Database operation failed".to_owned(),
        }),
    )
}

fn write_error(error: sqlx::Error) -> (StatusCode, Json<ApiError>) {
    if let sqlx::Error::Database(ref db_error) = error {
        if db_error.is_unique_violation() {
            return (
                StatusCode::CONFLICT,
                Json(ApiError {
                    error: "This HSN code already exists".to_owned(),
                }),
            );
        }

        if db_error.is_check_violation() {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "One or more tax rates are invalid".to_owned(),
                }),
            );
        }

        if db_error.is_foreign_key_violation() {
            return (
                StatusCode::CONFLICT,
                Json(ApiError {
                    error: "This HSN is referenced by existing products".to_owned(),
                }),
            );
        }
    }

    tracing::error!("HSN write failed: {error}");

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            error: "HSN operation failed".to_owned(),
        }),
    )
}

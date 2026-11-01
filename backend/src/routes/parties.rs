use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

// ============================================================
// REQUEST TYPES
// ============================================================

#[derive(Debug, Deserialize)]
pub struct PartyQuery {
    pub search: Option<String>,
    pub role: Option<String>,
    pub active: Option<bool>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PartyInput {
    pub party_code: String,
    pub name: String,
    pub roles: Vec<String>,

    pub contact_person: Option<String>,
    pub phone: Option<String>,
    pub alternate_phone: Option<String>,
    pub email: Option<String>,
    pub gstin: Option<String>,
    pub pan: Option<String>,
    pub drug_license_number: Option<String>,

    pub billing_address: Option<String>,
    pub shipping_address: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub state: Option<String>,
    pub pincode: Option<String>,
    pub country: Option<String>,

    pub opening_balance: Option<Decimal>,
    pub opening_balance_type: Option<String>,
    pub credit_limit: Option<Decimal>,
    pub credit_days: Option<i32>,

    pub notes: Option<String>,
    pub is_active: Option<bool>,
}

// ============================================================
// RESPONSE TYPES
// ============================================================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Party {
    pub id: Uuid,
    pub party_code: String,
    pub name: String,

    pub contact_person: Option<String>,
    pub phone: Option<String>,
    pub alternate_phone: Option<String>,
    pub email: Option<String>,
    pub gstin: Option<String>,
    pub pan: Option<String>,
    pub drug_license_number: Option<String>,

    pub billing_address: Option<String>,
    pub shipping_address: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub state: String,
    pub pincode: String,
    pub country: String,

    pub opening_balance: Decimal,
    pub opening_balance_type: String,
    pub credit_limit: Decimal,
    pub credit_days: i32,

    pub notes: Option<String>,
    pub is_active: bool,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub roles: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PartyList {
    pub data: Vec<Party>,
    pub page: i64,
    pub limit: i64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct ApiMessage {
    pub message: String,
}

// ============================================================
// ROUTER
// ============================================================

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", get(list_parties).post(create_party))
        .route(
            "/{id}",
            get(get_party).put(update_party).delete(deactivate_party),
        )
}

// ============================================================
// LIST PARTIES
// GET /api/parties
// GET /api/parties?role=SUPPLIER
// GET /api/parties?role=CUSTOMER
// GET /api/parties?search=abc&page=1&limit=50
// ============================================================

pub async fn list_parties(
    State(pool): State<PgPool>,
    Query(query): Query<PartyQuery>,
) -> Result<Json<PartyList>, (StatusCode, String)> {
    let search = query.search.unwrap_or_default().trim().to_string();

    let role = query.role.unwrap_or_default().trim().to_uppercase();

    if !role.is_empty() && !valid_role(&role) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Role must be SUPPLIER or CUSTOMER".to_string(),
        ));
    }

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1).saturating_mul(limit);
    let active = query.active.unwrap_or(true);

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM parties p
        WHERE
            p.is_active = $1
            AND (
                $2 = ''
                OR p.name ILIKE '%' || $2 || '%'
                OR p.party_code ILIKE '%' || $2 || '%'
                OR COALESCE(p.phone, '') ILIKE '%' || $2 || '%'
                OR COALESCE(p.gstin, '') ILIKE '%' || $2 || '%'
            )
            AND (
                $3 = ''
                OR EXISTS (
                    SELECT 1
                    FROM party_roles r
                    WHERE r.party_id = p.id
                      AND r.role = $3
                )
            )
        "#,
    )
    .bind(active)
    .bind(&search)
    .bind(&role)
    .fetch_one(&pool)
    .await
    .map_err(internal_error)?;

    let data = sqlx::query_as::<_, Party>(
        r#"
        SELECT
            p.id,
            p.party_code,
            p.name,
            p.contact_person,
            p.phone,
            p.alternate_phone,
            p.email,
            p.gstin,
            p.pan,
            p.drug_license_number,
            p.billing_address,
            p.shipping_address,
            p.city,
            p.district,
            p.state,
            p.pincode,
            p.country,
            p.opening_balance,
            p.opening_balance_type,
            p.credit_limit,
            p.credit_days,
            p.notes,
            p.is_active,
            p.created_at,
            p.updated_at,
            COALESCE(
                ARRAY_AGG(pr.role ORDER BY pr.role)
                    FILTER (WHERE pr.role IS NOT NULL),
                ARRAY[]::VARCHAR[]
            ) AS roles
        FROM parties p
        LEFT JOIN party_roles pr ON pr.party_id = p.id
        WHERE
            p.is_active = $1
            AND (
                $2 = ''
                OR p.name ILIKE '%' || $2 || '%'
                OR p.party_code ILIKE '%' || $2 || '%'
                OR COALESCE(p.phone, '') ILIKE '%' || $2 || '%'
                OR COALESCE(p.gstin, '') ILIKE '%' || $2 || '%'
            )
            AND (
                $3 = ''
                OR EXISTS (
                    SELECT 1
                    FROM party_roles r
                    WHERE r.party_id = p.id
                      AND r.role = $3
                )
            )
        GROUP BY p.id
        ORDER BY p.name ASC, p.id ASC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(active)
    .bind(&search)
    .bind(&role)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(PartyList {
        data,
        page,
        limit,
        total,
    }))
}

// ============================================================
// GET SINGLE PARTY
// GET /api/parties/{id}
// ============================================================

pub async fn get_party(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Party>, (StatusCode, String)> {
    let party = load_party(&pool, id).await?;

    Ok(Json(party))
}

// ============================================================
// CREATE PARTY
// POST /api/parties
// ============================================================

pub async fn create_party(
    State(pool): State<PgPool>,
    Json(input): Json<PartyInput>,
) -> Result<(StatusCode, Json<Party>), (StatusCode, String)> {
    let input = normalize_input(input)?;

    let mut tx = pool.begin().await.map_err(internal_error)?;

    let result = sqlx::query(
        r#"
        INSERT INTO parties (
            party_code,
            name,
            contact_person,
            phone,
            alternate_phone,
            email,
            gstin,
            pan,
            drug_license_number,
            billing_address,
            shipping_address,
            city,
            district,
            state,
            pincode,
            country,
            opening_balance,
            opening_balance_type,
            credit_limit,
            credit_days,
            notes,
            is_active
        )
        VALUES (
            $1, $2, $3, $4, $5, $6,
            $7, $8, $9, $10, $11, $12,
            $13, $14, $15, $16, $17, $18,
            $19, $20, $21, $22
        )
        "#,
    )
    .bind(&input.party_code)
    .bind(&input.name)
    .bind(&input.contact_person)
    .bind(&input.phone)
    .bind(&input.alternate_phone)
    .bind(&input.email)
    .bind(&input.gstin)
    .bind(&input.pan)
    .bind(&input.drug_license_number)
    .bind(&input.billing_address)
    .bind(&input.shipping_address)
    .bind(&input.city)
    .bind(&input.district)
    .bind(&input.state)
    .bind(&input.pincode)
    .bind(&input.country)
    .bind(input.opening_balance.unwrap_or(Decimal::ZERO))
    .bind(input.opening_balance_type.as_deref().unwrap_or("PAYABLE"))
    .bind(input.credit_limit.unwrap_or(Decimal::ZERO))
    .bind(input.credit_days.unwrap_or(0))
    .bind(&input.notes)
    .bind(input.is_active.unwrap_or(true))
    .execute(&mut *tx)
    .await;

    if let Err(error) = result {
        return Err(database_error(error));
    }

    let id: Uuid = sqlx::query_scalar("SELECT id FROM parties WHERE party_code = $1")
        .bind(&input.party_code)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal_error)?;

    insert_roles(&mut tx, id, &input.roles).await?;

    tx.commit().await.map_err(internal_error)?;

    let party = load_party(&pool, id).await?;

    Ok((StatusCode::CREATED, Json(party)))
}

// ============================================================
// UPDATE PARTY
// PUT /api/parties/{id}
// ============================================================

pub async fn update_party(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(input): Json<PartyInput>,
) -> Result<Json<Party>, (StatusCode, String)> {
    let input = normalize_input(input)?;

    let mut tx = pool.begin().await.map_err(internal_error)?;

    let result = sqlx::query(
        r#"
        UPDATE parties SET
            party_code = $2,
            name = $3,
            contact_person = $4,
            phone = $5,
            alternate_phone = $6,
            email = $7,
            gstin = $8,
            pan = $9,
            drug_license_number = $10,
            billing_address = $11,
            shipping_address = $12,
            city = $13,
            district = $14,
            state = $15,
            pincode = $16,
            country = $17,
            opening_balance = $18,
            opening_balance_type = $19,
            credit_limit = $20,
            credit_days = $21,
            notes = $22,
            is_active = $23,
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(&input.party_code)
    .bind(&input.name)
    .bind(&input.contact_person)
    .bind(&input.phone)
    .bind(&input.alternate_phone)
    .bind(&input.email)
    .bind(&input.gstin)
    .bind(&input.pan)
    .bind(&input.drug_license_number)
    .bind(&input.billing_address)
    .bind(&input.shipping_address)
    .bind(&input.city)
    .bind(&input.district)
    .bind(&input.state)
    .bind(&input.pincode)
    .bind(&input.country)
    .bind(input.opening_balance.unwrap_or(Decimal::ZERO))
    .bind(input.opening_balance_type.as_deref().unwrap_or("PAYABLE"))
    .bind(input.credit_limit.unwrap_or(Decimal::ZERO))
    .bind(input.credit_days.unwrap_or(0))
    .bind(&input.notes)
    .bind(input.is_active.unwrap_or(true))
    .execute(&mut *tx)
    .await;

    match result {
        Ok(result) if result.rows_affected() == 0 => {
            return Err((StatusCode::NOT_FOUND, "Party not found".to_string()));
        }
        Err(error) => return Err(database_error(error)),
        _ => {}
    }

    sqlx::query("DELETE FROM party_roles WHERE party_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    insert_roles(&mut tx, id, &input.roles).await?;

    tx.commit().await.map_err(internal_error)?;

    Ok(Json(load_party(&pool, id).await?))
}

// ============================================================
// DEACTIVATE PARTY
// DELETE /api/parties/{id}
// Soft delete: keeps historical references intact.
// ============================================================

pub async fn deactivate_party(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiMessage>, (StatusCode, String)> {
    let result = sqlx::query(
        r#"
        UPDATE parties
        SET is_active = FALSE, updated_at = NOW()
        WHERE id = $1 AND is_active = TRUE
        "#,
    )
    .bind(id)
    .execute(&pool)
    .await
    .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Active party not found".to_string()));
    }

    Ok(Json(ApiMessage {
        message: "Party deactivated successfully".to_string(),
    }))
}

// ============================================================
// SHARED PARTY LOADER
// ============================================================

async fn load_party(pool: &PgPool, id: Uuid) -> Result<Party, (StatusCode, String)> {
    sqlx::query_as::<_, Party>(
        r#"
        SELECT
            p.id,
            p.party_code,
            p.name,
            p.contact_person,
            p.phone,
            p.alternate_phone,
            p.email,
            p.gstin,
            p.pan,
            p.drug_license_number,
            p.billing_address,
            p.shipping_address,
            p.city,
            p.district,
            p.state,
            p.pincode,
            p.country,
            p.opening_balance,
            p.opening_balance_type,
            p.credit_limit,
            p.credit_days,
            p.notes,
            p.is_active,
            p.created_at,
            p.updated_at,
            COALESCE(
                ARRAY_AGG(pr.role ORDER BY pr.role)
                    FILTER (WHERE pr.role IS NOT NULL),
                ARRAY[]::VARCHAR[]
            ) AS roles
        FROM parties p
        LEFT JOIN party_roles pr ON pr.party_id = p.id
        WHERE p.id = $1
        GROUP BY p.id
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?
    .ok_or((StatusCode::NOT_FOUND, "Party not found".to_string()))
}

// ============================================================
// ROLE MANAGEMENT
// ============================================================

async fn insert_roles(
    tx: &mut Transaction<'_, Postgres>,
    party_id: Uuid,
    roles: &[String],
) -> Result<(), (StatusCode, String)> {
    for role in roles {
        sqlx::query("INSERT INTO party_roles (party_id, role) VALUES ($1, $2)")
            .bind(party_id)
            .bind(role)
            .execute(&mut **tx)
            .await
            .map_err(database_error)?;
    }

    Ok(())
}

// ============================================================
// INPUT VALIDATION
// ============================================================

fn normalize_input(mut input: PartyInput) -> Result<PartyInput, (StatusCode, String)> {
    input.party_code = input.party_code.trim().to_uppercase();
    input.name = input.name.trim().to_string();

    if input.party_code.is_empty() || input.name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Party code and name are required".to_string(),
        ));
    }

    if input.party_code.len() > 50 || input.name.len() > 250 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Party code or name exceeds the allowed length".to_string(),
        ));
    }

    input.roles = input
        .roles
        .iter()
        .map(|role| role.trim().to_uppercase())
        .collect();

    input.roles.sort();
    input.roles.dedup();

    if input.roles.is_empty() || input.roles.iter().any(|role| !valid_role(role)) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Select at least one valid role: SUPPLIER or CUSTOMER".to_string(),
        ));
    }

    let balance_type = input
        .opening_balance_type
        .as_deref()
        .unwrap_or("PAYABLE")
        .trim()
        .to_uppercase();

    if !matches!(balance_type.as_str(), "PAYABLE" | "RECEIVABLE" | "ADVANCE") {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid opening balance type".to_string(),
        ));
    }

    input.opening_balance_type = Some(balance_type);

    let opening = input.opening_balance.unwrap_or(Decimal::ZERO);
    let credit_limit = input.credit_limit.unwrap_or(Decimal::ZERO);
    let credit_days = input.credit_days.unwrap_or(0);

    if opening < Decimal::ZERO || credit_limit < Decimal::ZERO || credit_days < 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Balances, credit limit and credit days cannot be negative".to_string(),
        ));
    }

    input.opening_balance = Some(opening);
    input.credit_limit = Some(credit_limit);
    input.credit_days = Some(credit_days);
    input.is_active = Some(input.is_active.unwrap_or(true));

    input.state = Some(
        input
            .state
            .unwrap_or_else(|| "Rajasthan".to_string())
            .trim()
            .to_string(),
    );

    input.pincode = Some(input.pincode.unwrap_or_default().trim().to_string());

    input.country = Some(
        input
            .country
            .unwrap_or_else(|| "India".to_string())
            .trim()
            .to_string(),
    );

    normalize_optional(&mut input.contact_person);
    normalize_optional(&mut input.phone);
    normalize_optional(&mut input.alternate_phone);
    normalize_optional(&mut input.email);
    normalize_optional(&mut input.gstin);
    normalize_optional(&mut input.pan);
    normalize_optional(&mut input.drug_license_number);
    normalize_optional(&mut input.billing_address);
    normalize_optional(&mut input.shipping_address);
    normalize_optional(&mut input.city);
    normalize_optional(&mut input.district);
    normalize_optional(&mut input.notes);

    if let Some(email) = &input.email {
        if !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
            return Err((
                StatusCode::BAD_REQUEST,
                "Email address is invalid".to_string(),
            ));
        }
    }

    if let Some(gstin) = &input.gstin {
        if gstin.len() != 15 || !gstin.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err((
                StatusCode::BAD_REQUEST,
                "GSTIN must contain 15 alphanumeric characters".to_string(),
            ));
        }
    }

    if let Some(pan) = &input.pan {
        let chars: Vec<char> = pan.chars().collect();

        let valid = chars.len() == 10
            && chars[0..5].iter().all(|c| c.is_ascii_uppercase())
            && chars[5..9].iter().all(|c| c.is_ascii_digit())
            && chars[9].is_ascii_uppercase();

        if !valid {
            return Err((StatusCode::BAD_REQUEST, "PAN format is invalid".to_string()));
        }
    }

    if let Some(pincode) = &input.pincode {
        if !pincode.is_empty()
            && (pincode.len() != 6 || !pincode.chars().all(|c| c.is_ascii_digit()))
        {
            return Err((
                StatusCode::BAD_REQUEST,
                "Pincode must contain 6 digits".to_string(),
            ));
        }
    }

    Ok(input)
}

fn normalize_optional(value: &mut Option<String>) {
    if let Some(text) = value {
        let trimmed = text.trim().to_string();

        *value = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        };
    }
}

fn valid_role(role: &str) -> bool {
    matches!(role, "SUPPLIER" | "CUSTOMER")
}

// ============================================================
// ERROR HANDLING
// ============================================================

fn internal_error(error: sqlx::Error) -> (StatusCode, String) {
    tracing::error!("Party database operation failed: {error}");

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Unable to process party request".to_string(),
    )
}

fn database_error(error: sqlx::Error) -> (StatusCode, String) {
    if let sqlx::Error::Database(db_error) = &error {
        match db_error.code().as_deref() {
            Some("23505") => {
                return (
                    StatusCode::CONFLICT,
                    "Party code already exists".to_string(),
                );
            }
            Some("23503") => {
                return (
                    StatusCode::CONFLICT,
                    "Party is referenced by another record".to_string(),
                );
            }
            Some("23514") | Some("22P02") => {
                return (
                    StatusCode::BAD_REQUEST,
                    "Party data violates a database constraint".to_string(),
                );
            }
            _ => {}
        }
    }

    internal_error(error)
}

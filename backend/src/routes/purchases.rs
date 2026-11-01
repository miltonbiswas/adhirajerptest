use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, put},
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

// ============================================================
// ROUTES
// ============================================================

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", get(list_purchases).post(create_purchase))
        .route(
            "/{id}",
            get(get_purchase)
                .put(update_purchase)
                .delete(delete_purchase)
                .post(post_purchase),
        )
}

// ============================================================
// API ERROR
// ============================================================

#[derive(Debug, Serialize)]
pub struct ApiError {
    error: String,

    #[serde(skip)]
    status: StatusCode,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
            status: StatusCode::BAD_REQUEST,
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
            status: StatusCode::NOT_FOUND,
        }
    }

    fn conflict(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
            status: StatusCode::CONFLICT,
        }
    }

    fn internal(error: impl std::fmt::Display) -> Self {
        tracing::error!("Purchase API error: {error}");

        Self {
            error: "Purchase operation failed. Check backend logs.".to_owned(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self)).into_response()
    }
}

// ============================================================
// CONSTANTS
// ============================================================

const TAX_MODES: &[&str] = &["INCLUSIVE", "EXCLUSIVE"];
const TAX_TYPES: &[&str] = &["INTRA_STATE", "INTER_STATE"];

// ============================================================
// GST CALCULATION
// ============================================================

#[derive(Debug)]
struct LineCalculation {
    subtotal: Decimal,
    discount_amount: Decimal,
    taxable_amount: Decimal,
    cgst_amount: Decimal,
    sgst_amount: Decimal,
    igst_amount: Decimal,
    tax_amount: Decimal,
    total_amount: Decimal,
}

fn calculate_purchase_line(
    quantity: Decimal,
    purchase_rate: Decimal,
    discount_percent: Decimal,
    gst_rate: Decimal,
    tax_mode: &str,
    tax_type: &str,
) -> LineCalculation {
    let subtotal = (quantity * purchase_rate).round_dp(2);

    let discount_amount = (subtotal * discount_percent / Decimal::ONE_HUNDRED).round_dp(2);

    let amount_after_discount = subtotal - discount_amount;

    let (taxable_amount, tax_amount) = if tax_mode == "INCLUSIVE" {
        let divisor = Decimal::ONE + gst_rate / Decimal::ONE_HUNDRED;

        let taxable = (amount_after_discount / divisor).round_dp(2);
        let tax = (amount_after_discount - taxable).round_dp(2);

        (taxable, tax)
    } else {
        let taxable = amount_after_discount.round_dp(2);

        let tax = (taxable * gst_rate / Decimal::ONE_HUNDRED).round_dp(2);

        (taxable, tax)
    };

    let (cgst_amount, sgst_amount, igst_amount) = if tax_type == "INTRA_STATE" {
        let cgst = (tax_amount / Decimal::TWO).round_dp(2);
        let sgst = tax_amount - cgst;

        (cgst, sgst, Decimal::ZERO)
    } else {
        (Decimal::ZERO, Decimal::ZERO, tax_amount)
    };

    let total_amount = taxable_amount + tax_amount;

    LineCalculation {
        subtotal,
        discount_amount,
        taxable_amount,
        cgst_amount,
        sgst_amount,
        igst_amount,
        tax_amount,
        total_amount,
    }
}

// ============================================================
// PURCHASE LIST
// ============================================================

#[derive(Debug, Deserialize)]
pub struct PurchaseListQuery {
    pub search: Option<String>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PurchaseListItem {
    pub id: Uuid,
    pub purchase_number: String,
    pub supplier_id: Uuid,
    pub supplier_name: String,
    pub supplier_invoice_number: String,
    pub invoice_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub status: String,
    pub grand_total: Decimal,
    pub paid_amount: Decimal,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PurchaseListResponse {
    pub data: Vec<PurchaseListItem>,
    pub page: i64,
    pub limit: i64,
    pub total: i64,
}

pub async fn list_purchases(
    State(pool): State<PgPool>,
    Query(params): Query<PurchaseListQuery>,
) -> Result<Json<PurchaseListResponse>, ApiError> {
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(25).clamp(1, 100);
    let offset = (page - 1) * limit;

    let search = params.search.unwrap_or_default().trim().to_owned();
    let pattern = format!("%{search}%");

    let status = params
        .status
        .unwrap_or_else(|| "ALL".to_owned())
        .trim()
        .to_uppercase();

    if !["ALL", "DRAFT", "POSTED", "CANCELLED"].contains(&status.as_str()) {
        return Err(ApiError::bad_request("Invalid purchase status filter"));
    }

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM purchases p
        JOIN parties s ON s.id = p.supplier_id
        WHERE
            (
                $1 = ''
                OR p.purchase_number ILIKE $2
                OR p.supplier_invoice_number ILIKE $2
                OR s.name ILIKE $2
            )
            AND ($3 = 'ALL' OR p.status = $3)
        "#,
    )
    .bind(&search)
    .bind(&pattern)
    .bind(&status)
    .fetch_one(&pool)
    .await
    .map_err(ApiError::internal)?;

    let data = sqlx::query_as::<_, PurchaseListItem>(
        r#"
        SELECT
            p.id,
            p.purchase_number,
            p.supplier_id,
            s.name AS supplier_name,
            p.supplier_invoice_number,
            p.invoice_date,
            p.due_date,
            p.status,
            p.grand_total,
            p.paid_amount,
            p.created_at
        FROM purchases p
        JOIN parties s ON s.id = p.supplier_id
        WHERE
            (
                $1 = ''
                OR p.purchase_number ILIKE $2
                OR p.supplier_invoice_number ILIKE $2
                OR s.name ILIKE $2
            )
            AND ($3 = 'ALL' OR p.status = $3)
        ORDER BY p.invoice_date DESC, p.created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(&search)
    .bind(&pattern)
    .bind(&status)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(ApiError::internal)?;

    Ok(Json(PurchaseListResponse {
        data,
        page,
        limit,
        total,
    }))
}

// ============================================================
// CREATE PURCHASE REQUEST
// ============================================================

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseRequest {
    pub supplier_id: Uuid,
    pub supplier_invoice_number: String,
    pub invoice_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub notes: Option<String>,

    pub tax_mode: String,
    pub tax_type: String,

    pub items: Vec<CreatePurchaseItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseItemRequest {
    #[serde(default)]
    pub product_id: Option<Uuid>,

    #[serde(default)]
    pub new_product: Option<NewPurchaseProduct>,

    #[serde(default)]
    pub product_updates: Option<ExistingProductUpdates>,

    pub batch_number: String,
    pub manufacturing_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub quantity: Decimal,
    pub free_quantity: Option<Decimal>,
    pub purchase_rate: Decimal,
    pub mrp: Decimal,
    pub selling_rate: Option<Decimal>,
    pub discount_percent: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPurchaseProduct {
    pub sku: Option<String>,
    pub name: String,
    pub generic_name: Option<String>,
    pub brand_id: Option<Uuid>,
    pub brand_name: Option<String>,
    pub composition_ids: Option<Vec<Uuid>>,
    pub dosage_form: Option<String>,
    pub strength: Option<String>,
    pub pack_size: Option<String>,
    pub unit: Option<String>,
    pub hsn_code: Option<String>,
    pub barcode: Option<String>,
    pub is_prescription_required: Option<bool>,
    pub schedule_class: Option<String>,
    pub is_narcotic: Option<bool>,
    pub units_per_pack: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExistingProductUpdates {
    pub brand_id: Option<Uuid>,
    pub brand_name: Option<String>,
    pub dosage_form: Option<String>,
    pub hsn_code: Option<String>,
    pub schedule_class: Option<String>,
    pub is_narcotic: Option<bool>,
    pub is_prescription_required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
enum PurchaseProductPayload {
    Create(NewPurchaseProduct),
    Update(ExistingProductUpdates),
}

// ============================================================
// PURCHASE RESPONSE MODELS
// ============================================================

#[derive(Debug, Serialize, FromRow)]
pub struct PurchaseDetail {
    pub id: Uuid,
    pub purchase_number: String,
    pub supplier_id: Uuid,
    pub supplier_name: String,
    pub supplier_invoice_number: String,

    pub invoice_date: NaiveDate,
    pub due_date: Option<NaiveDate>,

    pub status: String,
    pub tax_mode: String,
    pub tax_type: String,

    pub subtotal: Decimal,
    pub discount_amount: Decimal,
    pub taxable_amount: Decimal,
    pub tax_amount: Decimal,

    pub other_charges: Decimal,
    pub round_off: Decimal,
    pub grand_total: Decimal,
    pub paid_amount: Decimal,

    pub notes: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PurchaseItemDetail {
    pub id: Uuid,
    pub line_number: i32,

    pub product_id: Option<Uuid>,
    pub product_name: String,
    pub sku: String,
    pub product_payload: Option<String>,

    pub batch_id: Option<Uuid>,
    pub batch_number: String,

    pub manufacturing_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,

    pub quantity: Decimal,
    pub free_quantity: Decimal,

    pub purchase_rate: Decimal,
    pub mrp: Decimal,
    pub selling_rate: Decimal,

    pub discount_percent: Decimal,
    pub discount_amount: Decimal,

    pub gst_rate: Decimal,
    pub taxable_amount: Decimal,

    pub cgst_amount: Decimal,
    pub sgst_amount: Decimal,
    pub igst_amount: Decimal,

    pub total_amount: Decimal,
}

#[derive(Debug, Serialize)]
pub struct PurchaseDetailResponse {
    pub purchase: PurchaseDetail,
    pub items: Vec<PurchaseItemDetail>,
}

// ============================================================
// SHARED PURCHASE DETAIL LOADER
// ============================================================

async fn load_purchase_detail(
    pool: &PgPool,
    purchase_id: Uuid,
) -> Result<PurchaseDetailResponse, ApiError> {
    let purchase = sqlx::query_as::<_, PurchaseDetail>(
        r#"
        SELECT
            p.id,
            p.purchase_number,
            p.supplier_id,
            s.name AS supplier_name,
            p.supplier_invoice_number,
            p.invoice_date,
            p.due_date,
            p.status,
            p.tax_mode,
            p.tax_type,
            p.subtotal,
            p.discount_amount,
            p.taxable_amount,
            p.tax_amount,
            p.other_charges,
            p.round_off,
            p.grand_total,
            p.paid_amount,
            p.notes,
            p.created_at
        FROM purchases p
        JOIN parties s ON s.id = p.supplier_id
        WHERE p.id = $1
        "#,
    )
    .bind(purchase_id)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::internal)?
    .ok_or_else(|| ApiError::not_found("Purchase not found"))?;

    let items = sqlx::query_as::<_, PurchaseItemDetail>(
        r#"
        SELECT
            pi.id,
            pi.line_number,
            pi.product_id,
            COALESCE(p.name, pi.product_payload #>> '{data,name}', '[New product]') AS product_name,
            COALESCE(p.sku, pi.product_payload #>> '{data,sku}', '[Generated on posting]') AS sku,
            pi.product_payload::TEXT AS product_payload,
            pi.batch_id,
            pi.batch_number,
            pi.manufacturing_date,
            pi.expiry_date,
            pi.quantity,
            pi.free_quantity,
            pi.purchase_rate,
            pi.mrp,
            pi.selling_rate,
            pi.discount_percent,
            pi.discount_amount,
            pi.gst_rate,
            pi.taxable_amount,
            pi.cgst_amount,
            pi.sgst_amount,
            pi.igst_amount,
            pi.total_amount
        FROM purchase_items pi
        LEFT JOIN products p ON p.id = pi.product_id
        WHERE pi.purchase_id = $1
        ORDER BY pi.line_number
        "#,
    )
    .bind(purchase_id)
    .fetch_all(pool)
    .await
    .map_err(ApiError::internal)?;

    Ok(PurchaseDetailResponse { purchase, items })
}

// ============================================================
// GET PURCHASE
// ============================================================

pub async fn get_purchase(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<PurchaseDetailResponse>, ApiError> {
    Ok(Json(load_purchase_detail(&pool, id).await?))
}

// ============================================================
// CREATE PURCHASE
// ============================================================

pub async fn create_purchase(
    State(pool): State<PgPool>,
    Json(request): Json<CreatePurchaseRequest>,
) -> Result<(StatusCode, Json<PurchaseDetailResponse>), ApiError> {
    let supplier_invoice_number = request.supplier_invoice_number.trim().to_owned();

    let tax_mode = request.tax_mode.trim().to_uppercase();
    let tax_type = request.tax_type.trim().to_uppercase();

    // Validate header.

    if supplier_invoice_number.is_empty() {
        return Err(ApiError::bad_request("Supplier invoice number is required"));
    }

    if !TAX_MODES.contains(&tax_mode.as_str()) {
        return Err(ApiError::bad_request(
            "Invalid tax mode. Use INCLUSIVE or EXCLUSIVE",
        ));
    }

    if !TAX_TYPES.contains(&tax_type.as_str()) {
        return Err(ApiError::bad_request(
            "Invalid tax type. Use INTRA_STATE or INTER_STATE",
        ));
    }

    if request.items.is_empty() {
        return Err(ApiError::bad_request(
            "At least one purchase item is required",
        ));
    }

    if request.items.len() > 500 {
        return Err(ApiError::bad_request(
            "A purchase cannot contain more than 500 lines",
        ));
    }

    // Validate all item values before opening the transaction.
    for item in &request.items {
        let has_existing = item.product_id.is_some();
        let has_new = item.new_product.is_some();

        if has_existing == has_new {
            return Err(ApiError::bad_request(
                "Each purchase line must contain either product_id or new_product, but not both",
            ));
        }

        if item.product_updates.is_some() && !has_existing {
            return Err(ApiError::bad_request(
                "product_updates can only be supplied for an existing product",
            ));
        }

        if item.batch_number.trim().is_empty() {
            return Err(ApiError::bad_request("Batch number is required"));
        }
        if item.quantity <= Decimal::ZERO {
            return Err(ApiError::bad_request(
                "Purchase quantity must be greater than zero",
            ));
        }
        if item.purchase_rate < Decimal::ZERO || item.mrp < Decimal::ZERO {
            return Err(ApiError::bad_request(
                "Purchase rate and MRP cannot be negative",
            ));
        }
        if item.selling_rate.is_some_and(|rate| rate < Decimal::ZERO) {
            return Err(ApiError::bad_request("Selling rate cannot be negative"));
        }

        let free_quantity = item.free_quantity.unwrap_or(Decimal::ZERO);
        let discount_percent = item.discount_percent.unwrap_or(Decimal::ZERO);
        if free_quantity < Decimal::ZERO {
            return Err(ApiError::bad_request("Free quantity cannot be negative"));
        }
        if !(Decimal::ZERO..=Decimal::ONE_HUNDRED).contains(&discount_percent) {
            return Err(ApiError::bad_request(
                "Discount percentage must be between 0 and 100",
            ));
        }
        if item
            .manufacturing_date
            .zip(item.expiry_date)
            .is_some_and(|(mfg, exp)| mfg > exp)
        {
            return Err(ApiError::bad_request(
                "Manufacturing date cannot be after expiry date",
            ));
        }

        if let Some(product) = &item.new_product {
            validate_new_product(product)?;
        }
        if let Some(updates) = &item.product_updates {
            validate_product_updates(updates)?;
        }
    }

    // Start transaction.

    let mut tx = pool.begin().await.map_err(ApiError::internal)?;

    // Validate supplier.

    let supplier_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM parties p
            JOIN party_roles pr ON pr.party_id = p.id
            WHERE p.id = $1
              AND p.is_active = TRUE
              AND pr.role = 'SUPPLIER'
        )
        "#,
    )
    .bind(request.supplier_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::internal)?;

    if !supplier_exists {
        return Err(ApiError::bad_request(
            "Supplier does not exist, is inactive, or lacks the SUPPLIER role",
        ));
    }

    // Check duplicate supplier invoice.

    let duplicate: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM purchases
            WHERE supplier_id = $1
              AND LOWER(BTRIM(supplier_invoice_number)) =
                  LOWER(BTRIM($2))
              AND status <> 'CANCELLED'
        )
        "#,
    )
    .bind(request.supplier_id)
    .bind(&supplier_invoice_number)
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::internal)?;

    if duplicate {
        return Err(ApiError::conflict(
            "This supplier invoice number already exists",
        ));
    }

    // Generate purchase number.

    let purchase_number: String = sqlx::query_scalar(
        r#"
        SELECT
            'PUR-' ||
            TO_CHAR(CURRENT_DATE, 'YYYYMMDD') ||
            '-' ||
            UPPER(
                SUBSTRING(
                    REPLACE(gen_random_uuid()::TEXT, '-', '')
                    FROM 1 FOR 8
                )
            )
        "#,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::internal)?;

    // Insert purchase header.

    let purchase_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO purchases (
            purchase_number,
            supplier_id,
            supplier_invoice_number,
            invoice_date,
            due_date,
            status,
            tax_mode,
            tax_type,
            notes
        )
        VALUES (
            $1, $2, $3, $4, $5,
            'DRAFT', $6, $7, $8
        )
        RETURNING id
        "#,
    )
    .bind(&purchase_number)
    .bind(request.supplier_id)
    .bind(&supplier_invoice_number)
    .bind(request.invoice_date)
    .bind(request.due_date)
    .bind(&tax_mode)
    .bind(&tax_type)
    .bind(request.notes.as_deref())
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::internal)?;

    let mut subtotal = Decimal::ZERO;
    let mut total_discount = Decimal::ZERO;
    let mut taxable_total = Decimal::ZERO;
    let mut tax_total = Decimal::ZERO;
    let mut grand_total = Decimal::ZERO;

    // Insert purchase lines. Product Master changes are staged only;
    // they are applied when the draft is posted.
    for (index, item) in request.items.iter().enumerate() {
        let free_quantity = item.free_quantity.unwrap_or(Decimal::ZERO);
        let discount_percent = item.discount_percent.unwrap_or(Decimal::ZERO);

        let (product_id, product_payload, gst_rate) = if let Some(product_id) = item.product_id {
            let current_hsn: Option<String> = sqlx::query_scalar(
                "SELECT hsn_code FROM public.products WHERE id = $1 AND is_active = TRUE",
            )
            .bind(product_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(ApiError::internal)?
            .ok_or_else(|| {
                ApiError::bad_request(format!(
                    "Product {product_id} does not exist or is inactive"
                ))
            })?;

            let target_hsn = match item
                .product_updates
                .as_ref()
                .and_then(|u| u.hsn_code.as_deref())
            {
                Some(value) if value.trim().is_empty() => None,
                Some(value) => Some(value.trim().to_owned()),
                None => current_hsn,
            };
            let rate = match target_hsn.as_deref() {
                Some(code) => get_active_hsn_rate(&mut tx, code).await?,
                None => Decimal::ZERO,
            };
            if let Some(updates) = &item.product_updates {
                validate_brand(&mut tx, updates.brand_id).await?;
            }
            let payload = match &item.product_updates {
                Some(updates) => Some(
                    serde_json::to_string(&PurchaseProductPayload::Update(updates.clone()))
                        .map_err(ApiError::internal)?,
                ),
                None => None,
            };
            (Some(product_id), payload, rate)
        } else {
            let new_product = item.new_product.as_ref().expect("validated new product");
            validate_brand(&mut tx, new_product.brand_id).await?;
            let target_hsn = new_product
                .hsn_code
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty());
            let rate = match target_hsn {
                Some(code) => get_active_hsn_rate(&mut tx, code).await?,
                None => Decimal::ZERO,
            };
            let payload =
                serde_json::to_string(&PurchaseProductPayload::Create(new_product.clone()))
                    .map_err(ApiError::internal)?;
            (None, Some(payload), rate)
        };

        let calculation = calculate_purchase_line(
            item.quantity,
            item.purchase_rate,
            discount_percent,
            gst_rate,
            &tax_mode,
            &tax_type,
        );

        subtotal += calculation.subtotal;
        total_discount += calculation.discount_amount;
        taxable_total += calculation.taxable_amount;
        tax_total += calculation.tax_amount;
        grand_total += calculation.total_amount;

        sqlx::query(
            r#"
            INSERT INTO purchase_items (
                purchase_id, line_number, product_id, product_payload,
                batch_number, manufacturing_date, expiry_date,
                quantity, free_quantity, purchase_rate, mrp, selling_rate,
                discount_percent, discount_amount, gst_rate, taxable_amount,
                cgst_amount, sgst_amount, igst_amount, total_amount
            )
            VALUES (
                $1, $2, $3, $4::jsonb, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20
            )
            "#,
        )
        .bind(purchase_id)
        .bind((index + 1) as i32)
        .bind(product_id)
        .bind(product_payload)
        .bind(item.batch_number.trim())
        .bind(item.manufacturing_date)
        .bind(item.expiry_date)
        .bind(item.quantity)
        .bind(free_quantity)
        .bind(item.purchase_rate)
        .bind(item.mrp)
        .bind(item.selling_rate.unwrap_or(Decimal::ZERO))
        .bind(discount_percent)
        .bind(calculation.discount_amount)
        .bind(gst_rate)
        .bind(calculation.taxable_amount)
        .bind(calculation.cgst_amount)
        .bind(calculation.sgst_amount)
        .bind(calculation.igst_amount)
        .bind(calculation.total_amount)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::internal)?;
    }

    // Update purchase totals.

    sqlx::query(
        r#"
        UPDATE purchases
        SET
            subtotal = $2,
            discount_amount = $3,
            taxable_amount = $4,
            tax_amount = $5,
            grand_total = $6,
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(purchase_id)
    .bind(subtotal.round_dp(2))
    .bind(total_discount.round_dp(2))
    .bind(taxable_total.round_dp(2))
    .bind(tax_total.round_dp(2))
    .bind(grand_total.round_dp(2))
    .execute(&mut *tx)
    .await
    .map_err(ApiError::internal)?;

    tx.commit().await.map_err(ApiError::internal)?;

    let response = load_purchase_detail(&pool, purchase_id).await?;

    Ok((StatusCode::CREATED, Json(response)))
}

// ============================================================
// UPDATE DRAFT PURCHASE
// PUT /api/purchases/{id}
// ============================================================

pub async fn update_purchase(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(request): Json<CreatePurchaseRequest>,
) -> Result<Json<PurchaseDetailResponse>, ApiError> {
    let supplier_invoice_number = request.supplier_invoice_number.trim().to_owned();
    let tax_mode = request.tax_mode.trim().to_uppercase();
    let tax_type = request.tax_type.trim().to_uppercase();

    if supplier_invoice_number.is_empty() {
        return Err(ApiError::bad_request("Supplier invoice number is required"));
    }
    if !TAX_MODES.contains(&tax_mode.as_str()) {
        return Err(ApiError::bad_request(
            "Invalid tax mode. Use INCLUSIVE or EXCLUSIVE",
        ));
    }
    if !TAX_TYPES.contains(&tax_type.as_str()) {
        return Err(ApiError::bad_request(
            "Invalid tax type. Use INTRA_STATE or INTER_STATE",
        ));
    }
    if request.items.is_empty() || request.items.len() > 500 {
        return Err(ApiError::bad_request(
            "A draft purchase must contain between 1 and 500 items",
        ));
    }

    for item in &request.items {
        let has_existing = item.product_id.is_some();
        let has_new = item.new_product.is_some();
        if has_existing == has_new {
            return Err(ApiError::bad_request(
                "Each purchase line must contain either product_id or new_product, but not both",
            ));
        }
        if item.product_updates.is_some() && !has_existing {
            return Err(ApiError::bad_request(
                "product_updates can only be supplied for an existing product",
            ));
        }
        if item.batch_number.trim().is_empty() {
            return Err(ApiError::bad_request("Batch number is required"));
        }
        if item.quantity <= Decimal::ZERO {
            return Err(ApiError::bad_request(
                "Purchase quantity must be greater than zero",
            ));
        }
        if item.purchase_rate < Decimal::ZERO || item.mrp < Decimal::ZERO {
            return Err(ApiError::bad_request(
                "Purchase rate and MRP cannot be negative",
            ));
        }
        if item.selling_rate.is_some_and(|v| v < Decimal::ZERO) {
            return Err(ApiError::bad_request("Selling rate cannot be negative"));
        }
        let free_quantity = item.free_quantity.unwrap_or(Decimal::ZERO);
        let discount_percent = item.discount_percent.unwrap_or(Decimal::ZERO);
        if free_quantity < Decimal::ZERO {
            return Err(ApiError::bad_request("Free quantity cannot be negative"));
        }
        if !(Decimal::ZERO..=Decimal::ONE_HUNDRED).contains(&discount_percent) {
            return Err(ApiError::bad_request(
                "Discount percentage must be between 0 and 100",
            ));
        }
        if item
            .manufacturing_date
            .zip(item.expiry_date)
            .is_some_and(|(mfg, exp)| mfg > exp)
        {
            return Err(ApiError::bad_request(
                "Manufacturing date cannot be after expiry date",
            ));
        }
        if let Some(product) = &item.new_product {
            validate_new_product(product)?;
        }
        if let Some(updates) = &item.product_updates {
            validate_product_updates(updates)?;
        }
    }

    let mut tx = pool.begin().await.map_err(ApiError::internal)?;

    // Lock first: a concurrent POST/DELETE cannot race this draft update.
    let existing = sqlx::query_as::<_, (String, Decimal, Decimal)>(
        "SELECT status, other_charges, round_off FROM purchases WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(ApiError::internal)?
    .ok_or_else(|| ApiError::not_found("Purchase not found"))?;

    if existing.0 != "DRAFT" {
        return Err(ApiError::conflict("Only draft purchases can be edited"));
    }

    let supplier_exists: bool = sqlx::query_scalar(
        r#"SELECT EXISTS (
            SELECT 1 FROM parties p
            JOIN party_roles pr ON pr.party_id = p.id
            WHERE p.id = $1 AND p.is_active = TRUE AND pr.role = 'SUPPLIER'
        )"#,
    )
    .bind(request.supplier_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::internal)?;
    if !supplier_exists {
        return Err(ApiError::bad_request(
            "Supplier does not exist, is inactive, or lacks the SUPPLIER role",
        ));
    }

    let duplicate: bool = sqlx::query_scalar(
        r#"SELECT EXISTS (
            SELECT 1 FROM purchases
            WHERE supplier_id = $1
              AND LOWER(BTRIM(supplier_invoice_number)) = LOWER(BTRIM($2))
              AND status <> 'CANCELLED'
              AND id <> $3
        )"#,
    )
    .bind(request.supplier_id)
    .bind(&supplier_invoice_number)
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::internal)?;
    if duplicate {
        return Err(ApiError::conflict(
            "This supplier invoice number already exists",
        ));
    }

    sqlx::query(
        r#"UPDATE purchases SET
            supplier_id = $2,
            supplier_invoice_number = $3,
            invoice_date = $4,
            due_date = $5,
            tax_mode = $6,
            tax_type = $7,
            notes = $8,
            updated_at = NOW()
        WHERE id = $1"#,
    )
    .bind(id)
    .bind(request.supplier_id)
    .bind(&supplier_invoice_number)
    .bind(request.invoice_date)
    .bind(request.due_date)
    .bind(&tax_mode)
    .bind(&tax_type)
    .bind(request.notes.as_deref())
    .execute(&mut *tx)
    .await
    .map_err(ApiError::internal)?;

    // Draft lines have no stock/batch/ledger side effects. Replace them atomically.
    sqlx::query("DELETE FROM purchase_items WHERE purchase_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::internal)?;

    let mut subtotal = Decimal::ZERO;
    let mut total_discount = Decimal::ZERO;
    let mut taxable_total = Decimal::ZERO;
    let mut tax_total = Decimal::ZERO;
    let mut lines_total = Decimal::ZERO;

    for (index, item) in request.items.iter().enumerate() {
        let free_quantity = item.free_quantity.unwrap_or(Decimal::ZERO);
        let discount_percent = item.discount_percent.unwrap_or(Decimal::ZERO);

        let (product_id, product_payload, gst_rate) = if let Some(product_id) = item.product_id {
            let current_hsn: Option<String> = sqlx::query_scalar(
                "SELECT hsn_code FROM public.products WHERE id = $1 AND is_active = TRUE",
            )
            .bind(product_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(ApiError::internal)?
            .ok_or_else(|| {
                ApiError::bad_request(format!(
                    "Product {product_id} does not exist or is inactive"
                ))
            })?;

            let target_hsn = match item
                .product_updates
                .as_ref()
                .and_then(|u| u.hsn_code.as_deref())
            {
                Some(value) if value.trim().is_empty() => None,
                Some(value) => Some(value.trim().to_owned()),
                None => current_hsn,
            };
            let rate = match target_hsn.as_deref() {
                Some(code) => get_active_hsn_rate(&mut tx, code).await?,
                None => Decimal::ZERO,
            };
            if let Some(updates) = &item.product_updates {
                validate_brand(&mut tx, updates.brand_id).await?;
            }
            let payload = match &item.product_updates {
                Some(updates) => Some(
                    serde_json::to_string(&PurchaseProductPayload::Update(updates.clone()))
                        .map_err(ApiError::internal)?,
                ),
                None => None,
            };
            (Some(product_id), payload, rate)
        } else {
            let product = item.new_product.as_ref().expect("validated new product");
            validate_brand(&mut tx, product.brand_id).await?;
            let target_hsn = product
                .hsn_code
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty());
            let rate = match target_hsn {
                Some(code) => get_active_hsn_rate(&mut tx, code).await?,
                None => Decimal::ZERO,
            };
            let payload = serde_json::to_string(&PurchaseProductPayload::Create(product.clone()))
                .map_err(ApiError::internal)?;
            (None, Some(payload), rate)
        };

        let calc = calculate_purchase_line(
            item.quantity,
            item.purchase_rate,
            discount_percent,
            gst_rate,
            &tax_mode,
            &tax_type,
        );
        subtotal += calc.subtotal;
        total_discount += calc.discount_amount;
        taxable_total += calc.taxable_amount;
        tax_total += calc.tax_amount;
        lines_total += calc.total_amount;

        sqlx::query(
            r#"INSERT INTO purchase_items (
                purchase_id, line_number, product_id, product_payload,
                batch_number, manufacturing_date, expiry_date,
                quantity, free_quantity, purchase_rate, mrp, selling_rate,
                discount_percent, discount_amount, gst_rate, taxable_amount,
                cgst_amount, sgst_amount, igst_amount, total_amount
            ) VALUES (
                $1, $2, $3, $4::jsonb, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20
            )"#,
        )
        .bind(id)
        .bind((index + 1) as i32)
        .bind(product_id)
        .bind(product_payload)
        .bind(item.batch_number.trim())
        .bind(item.manufacturing_date)
        .bind(item.expiry_date)
        .bind(item.quantity)
        .bind(free_quantity)
        .bind(item.purchase_rate)
        .bind(item.mrp)
        .bind(item.selling_rate.unwrap_or(Decimal::ZERO))
        .bind(discount_percent)
        .bind(calc.discount_amount)
        .bind(gst_rate)
        .bind(calc.taxable_amount)
        .bind(calc.cgst_amount)
        .bind(calc.sgst_amount)
        .bind(calc.igst_amount)
        .bind(calc.total_amount)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::internal)?;
    }

    let grand_total = (lines_total + existing.1 + existing.2).round_dp(2);
    sqlx::query(
        r#"UPDATE purchases SET
            subtotal = $2, discount_amount = $3, taxable_amount = $4,
            tax_amount = $5, grand_total = $6, updated_at = NOW()
        WHERE id = $1"#,
    )
    .bind(id)
    .bind(subtotal.round_dp(2))
    .bind(total_discount.round_dp(2))
    .bind(taxable_total.round_dp(2))
    .bind(tax_total.round_dp(2))
    .bind(grand_total)
    .execute(&mut *tx)
    .await
    .map_err(ApiError::internal)?;

    tx.commit().await.map_err(ApiError::internal)?;
    Ok(Json(load_purchase_detail(&pool, id).await?))
}

// ============================================================
// DELETE DRAFT PURCHASE
// DELETE /api/purchases/{id}
// ============================================================

pub async fn delete_purchase(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let mut tx = pool.begin().await.map_err(ApiError::internal)?;
    let status: Option<String> =
        sqlx::query_scalar("SELECT status FROM purchases WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(ApiError::internal)?;

    let status = status.ok_or_else(|| ApiError::not_found("Purchase not found"))?;
    if status != "DRAFT" {
        return Err(ApiError::conflict("Only draft purchases can be deleted"));
    }

    sqlx::query("DELETE FROM purchase_items WHERE purchase_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::internal)?;
    sqlx::query("DELETE FROM purchases WHERE id = $1 AND status = 'DRAFT'")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::internal)?;

    tx.commit().await.map_err(ApiError::internal)?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================
// PRODUCT STAGING / POSTING HELPERS
// ============================================================

fn validate_schedule_class(value: Option<&str>) -> Result<(), ApiError> {
    if let Some(value) = value {
        let value = value.trim().to_uppercase();
        if !["OTC", "H", "H1", "X"].contains(&value.as_str()) {
            return Err(ApiError::bad_request(
                "Regulatory class must be OTC, H, H1 or X",
            ));
        }
    }
    Ok(())
}

fn validate_new_product(product: &NewPurchaseProduct) -> Result<(), ApiError> {
    if product.name.trim().is_empty() {
        return Err(ApiError::bad_request("New product name is required"));
    }
    if product.name.trim().len() > 250 {
        return Err(ApiError::bad_request("Product name exceeds 250 characters"));
    }
    if product
        .sku
        .as_deref()
        .is_some_and(|sku| sku.trim().len() > 80)
    {
        return Err(ApiError::bad_request("SKU exceeds 80 characters"));
    }
    if product
        .units_per_pack
        .is_some_and(|units| units <= Decimal::ZERO)
    {
        return Err(ApiError::bad_request(
            "Units per pack must be greater than zero",
        ));
    }
    validate_schedule_class(product.schedule_class.as_deref())?;
    Ok(())
}

fn validate_product_updates(updates: &ExistingProductUpdates) -> Result<(), ApiError> {
    validate_schedule_class(updates.schedule_class.as_deref())
}

async fn get_active_hsn_rate(
    tx: &mut Transaction<'_, Postgres>,
    hsn_code: &str,
) -> Result<Decimal, ApiError> {
    let rate: Option<Decimal> = sqlx::query_scalar(
        "SELECT gst_rate FROM public.hsn_master WHERE hsn_code = $1 AND is_active = TRUE FOR SHARE",
    )
    .bind(hsn_code.trim())
    .fetch_optional(&mut **tx)
    .await
    .map_err(ApiError::internal)?;
    rate.ok_or_else(|| {
        ApiError::bad_request(format!(
            "HSN '{}' does not exist or is inactive",
            hsn_code.trim()
        ))
    })
}

async fn validate_brand(
    tx: &mut Transaction<'_, Postgres>,
    brand_id: Option<Uuid>,
) -> Result<(), ApiError> {
    if let Some(brand_id) = brand_id {
        let active: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM public.brands WHERE id = $1 AND is_active = TRUE)",
        )
        .bind(brand_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(ApiError::internal)?;
        if !active {
            return Err(ApiError::bad_request(
                "Selected brand does not exist or is inactive",
            ));
        }
    }
    Ok(())
}

async fn create_product_from_purchase(
    tx: &mut Transaction<'_, Postgres>,
    product: &NewPurchaseProduct,
) -> Result<Uuid, ApiError> {
    validate_new_product(product)?;
    validate_brand(tx, product.brand_id).await?;

    let hsn_code = product
        .hsn_code
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let gst_rate = match hsn_code {
        Some(code) => get_active_hsn_rate(tx, code).await?,
        None => Decimal::ZERO,
    };

    let sku = match product.sku.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        Some(sku) => sku.to_owned(),
        None => sqlx::query_scalar::<_, String>(
            "SELECT 'ADP-' || UPPER(SUBSTRING(REPLACE(gen_random_uuid()::TEXT, '-', '') FROM 1 FOR 12))",
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(ApiError::internal)?,
    };

    let schedule_class = product
        .schedule_class
        .as_deref()
        .unwrap_or("OTC")
        .trim()
        .to_uppercase();
    let units_per_pack = product.units_per_pack.unwrap_or(Decimal::ONE);

    let product_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO public.products (
            sku, name, generic_name, brand_name, brand_id,
            dosage_form, strength, pack_size, unit, hsn_code, gst_rate,
            barcode, is_prescription_required, is_active,
            schedule_class, is_narcotic, units_per_pack
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, COALESCE($9, 'PCS'),
            $10, $11, $12, COALESCE($13, FALSE), TRUE,
            $14, COALESCE($15, FALSE), $16
        )
        RETURNING id
        "#,
    )
    .bind(&sku)
    .bind(product.name.trim())
    .bind(product.generic_name.as_deref())
    .bind(product.brand_name.as_deref())
    .bind(product.brand_id)
    .bind(product.dosage_form.as_deref())
    .bind(product.strength.as_deref())
    .bind(product.pack_size.as_deref())
    .bind(product.unit.as_deref())
    .bind(hsn_code)
    .bind(gst_rate)
    .bind(product.barcode.as_deref())
    .bind(product.is_prescription_required)
    .bind(schedule_class)
    .bind(product.is_narcotic)
    .bind(units_per_pack)
    .fetch_one(&mut **tx)
    .await
    .map_err(|error| match error {
        sqlx::Error::Database(ref db_error) if db_error.is_unique_violation() => {
            ApiError::conflict(format!("Product SKU '{sku}' already exists"))
        }
        other => ApiError::internal(other),
    })?;

    if let Some(composition_ids) = &product.composition_ids {
        let mut unique = std::collections::HashSet::new();
        for composition_id in composition_ids {
            if !unique.insert(*composition_id) {
                return Err(ApiError::bad_request(
                    "Duplicate composition IDs are not allowed",
                ));
            }
            sqlx::query(
                "INSERT INTO public.product_compositions (product_id, composition_id) VALUES ($1, $2)",
            )
            .bind(product_id)
            .bind(composition_id)
            .execute(&mut **tx)
            .await
            .map_err(ApiError::internal)?;
        }
    }

    Ok(product_id)
}

async fn apply_existing_product_updates(
    tx: &mut Transaction<'_, Postgres>,
    product_id: Uuid,
    updates: &ExistingProductUpdates,
) -> Result<(), ApiError> {
    validate_product_updates(updates)?;
    validate_brand(tx, updates.brand_id).await?;

    // Read current HSN, then lock the target HSN before locking/updating product.
    let current_hsn: Option<String> =
        sqlx::query_scalar("SELECT hsn_code FROM public.products WHERE id = $1")
            .bind(product_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(ApiError::internal)?
            .ok_or_else(|| ApiError::not_found("Product not found during purchase posting"))?;

    let target_hsn = match updates.hsn_code.as_deref() {
        Some(value) if value.trim().is_empty() => None,
        Some(value) => Some(value.trim().to_owned()),
        None => current_hsn,
    };
    let gst_rate = match target_hsn.as_deref() {
        Some(code) => get_active_hsn_rate(tx, code).await?,
        None => Decimal::ZERO,
    };

    let brand_name = updates
        .brand_name
        .as_deref()
        .map(str::trim)
        .map(|v| if v.is_empty() { None } else { Some(v) });
    let dosage_form = updates
        .dosage_form
        .as_deref()
        .map(str::trim)
        .map(|v| if v.is_empty() { None } else { Some(v) });
    let schedule_class = updates
        .schedule_class
        .as_deref()
        .map(|v| v.trim().to_uppercase());

    sqlx::query(
        r#"
        UPDATE public.products
        SET
            brand_id = CASE WHEN $3 THEN $2 ELSE brand_id END,
            brand_name = CASE WHEN $4 THEN $5 ELSE brand_name END,
            dosage_form = CASE WHEN $6 THEN $7 ELSE dosage_form END,
            hsn_code = $8,
            gst_rate = $9,
            schedule_class = COALESCE($10, schedule_class),
            is_narcotic = COALESCE($11, is_narcotic),
            is_prescription_required = COALESCE($12, is_prescription_required),
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(product_id)
    .bind(updates.brand_id)
    .bind(updates.brand_id.is_some())
    .bind(updates.brand_name.is_some())
    .bind(brand_name.flatten())
    .bind(updates.dosage_form.is_some())
    .bind(dosage_form.flatten())
    .bind(&target_hsn)
    .bind(gst_rate)
    .bind(schedule_class)
    .bind(updates.is_narcotic)
    .bind(updates.is_prescription_required)
    .execute(&mut **tx)
    .await
    .map_err(ApiError::internal)?;

    Ok(())
}

// ============================================================
// PURCHASE POSTING MODELS
// ============================================================

#[derive(Debug, FromRow)]
struct PurchasePostingHeader {
    supplier_id: Uuid,
    status: String,
    grand_total: Decimal,
}

#[derive(Debug, FromRow)]
struct PurchasePostingItem {
    id: Uuid,
    product_id: Option<Uuid>,
    product_payload: Option<String>,
    batch_number: String,
    manufacturing_date: Option<NaiveDate>,
    expiry_date: Option<NaiveDate>,
    quantity: Decimal,
    free_quantity: Decimal,
    purchase_rate: Decimal,
    mrp: Decimal,
    selling_rate: Decimal,
}

#[derive(Debug, FromRow)]
struct UpdatedBatch {
    id: Uuid,
    quantity_before: Decimal,
    quantity_after: Decimal,
}

// ============================================================
// POST PURCHASE
// ============================================================

pub async fn post_purchase(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<PurchaseDetailResponse>, ApiError> {
    let mut tx = pool.begin().await.map_err(ApiError::internal)?;

    // Lock purchase row to prevent simultaneous posting.

    let purchase = sqlx::query_as::<_, PurchasePostingHeader>(
        r#"
        SELECT
            supplier_id,
            status,
            grand_total
        FROM purchases
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(ApiError::internal)?
    .ok_or_else(|| ApiError::not_found("Purchase not found"))?;

    if purchase.status != "DRAFT" {
        return Err(ApiError::conflict("Only draft purchases can be posted"));
    }

    // Load and lock purchase items.

    let items = sqlx::query_as::<_, PurchasePostingItem>(
        r#"
        SELECT
            id,
            product_id,
            product_payload::TEXT AS product_payload,
            batch_number,
            manufacturing_date,
            expiry_date,
            quantity,
            free_quantity,
            purchase_rate,
            mrp,
            selling_rate
        FROM purchase_items
        WHERE purchase_id = $1
        ORDER BY line_number
        FOR UPDATE
        "#,
    )
    .bind(id)
    .fetch_all(&mut *tx)
    .await
    .map_err(ApiError::internal)?;

    if items.is_empty() {
        return Err(ApiError::bad_request(
            "Cannot post a purchase without items",
        ));
    }

    // Process each purchase line. New products and staged master updates
    // are applied inside this same posting transaction.
    for item in items {
        if let Some(existing_product_id) = item.product_id {
            let product_is_active: bool = sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM public.products WHERE id = $1 AND is_active = TRUE)",
            )
            .bind(existing_product_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(ApiError::internal)?;

            if !product_is_active {
                return Err(ApiError::conflict(format!(
                    "Product {existing_product_id} became inactive before this purchase was posted"
                )));
            }
        }

        let product_id = match (item.product_id, item.product_payload.as_deref()) {
            (Some(product_id), Some(payload_text)) => {
                let payload: PurchaseProductPayload =
                    serde_json::from_str(payload_text).map_err(ApiError::internal)?;
                match payload {
                    PurchaseProductPayload::Update(updates) => {
                        apply_existing_product_updates(&mut tx, product_id, &updates).await?;
                    }
                    PurchaseProductPayload::Create(_) => {
                        return Err(ApiError::conflict(
                            "Invalid staged product action for an existing product",
                        ));
                    }
                }
                product_id
            }
            (Some(product_id), None) => product_id,
            (None, Some(payload_text)) => {
                let payload: PurchaseProductPayload =
                    serde_json::from_str(payload_text).map_err(ApiError::internal)?;
                match payload {
                    PurchaseProductPayload::Create(new_product) => {
                        create_product_from_purchase(&mut tx, &new_product).await?
                    }
                    PurchaseProductPayload::Update(_) => {
                        return Err(ApiError::conflict(
                            "New purchase line has an invalid product update payload",
                        ));
                    }
                }
            }
            (None, None) => {
                return Err(ApiError::conflict(
                    "Purchase line has no product or staged product details",
                ));
            }
        };

        // Attach newly-created products to the purchase line.
        sqlx::query("UPDATE purchase_items SET product_id = $2 WHERE id = $1")
            .bind(item.id)
            .bind(product_id)
            .execute(&mut *tx)
            .await
            .map_err(ApiError::internal)?;

        let stock_quantity = item.quantity + item.free_quantity;
        if stock_quantity <= Decimal::ZERO {
            return Err(ApiError::bad_request(
                "Purchase stock quantity must be greater than zero",
            ));
        }

        let batch = sqlx::query_as::<_, UpdatedBatch>(
            r#"
            INSERT INTO product_batches (
                product_id, batch_number, manufacturing_date, expiry_date,
                purchase_rate, mrp, selling_rate, quantity_on_hand,
                is_active, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, TRUE, NOW())
            ON CONFLICT (product_id, batch_number)
            DO UPDATE SET
                manufacturing_date = COALESCE(EXCLUDED.manufacturing_date, product_batches.manufacturing_date),
                expiry_date = COALESCE(EXCLUDED.expiry_date, product_batches.expiry_date),
                purchase_rate = EXCLUDED.purchase_rate,
                mrp = EXCLUDED.mrp,
                selling_rate = EXCLUDED.selling_rate,
                quantity_on_hand = product_batches.quantity_on_hand + EXCLUDED.quantity_on_hand,
                is_active = TRUE,
                updated_at = NOW()
            RETURNING
                id,
                quantity_on_hand - $8 AS quantity_before,
                quantity_on_hand AS quantity_after
            "#,
        )
        .bind(product_id)
        .bind(item.batch_number.trim())
        .bind(item.manufacturing_date)
        .bind(item.expiry_date)
        .bind(item.purchase_rate)
        .bind(item.mrp)
        .bind(item.selling_rate)
        .bind(stock_quantity)
        .fetch_one(&mut *tx)
        .await
        .map_err(ApiError::internal)?;

        sqlx::query("UPDATE purchase_items SET batch_id = $2 WHERE id = $1")
            .bind(item.id)
            .bind(batch.id)
            .execute(&mut *tx)
            .await
            .map_err(ApiError::internal)?;

        sqlx::query(
            r#"
            INSERT INTO stock_movements (
                product_id, batch_id, movement_type, quantity_change,
                quantity_before, quantity_after, reason
            )
            VALUES ($1, $2, 'IN', $3, $4, $5, $6)
            "#,
        )
        .bind(product_id)
        .bind(batch.id)
        .bind(stock_quantity)
        .bind(batch.quantity_before)
        .bind(batch.quantity_after)
        .bind(format!("Purchase posting: {}", id))
        .execute(&mut *tx)
        .await
        .map_err(ApiError::internal)?;
    }

    // Create supplier ledger entry.
    //
    // This uses the convention that a purchase debit increases
    // the supplier payable balance in this application's ledger.

    if purchase.grand_total > Decimal::ZERO {
        sqlx::query(
            r#"
            INSERT INTO supplier_ledger_entries (
                supplier_id,
                purchase_id,
                entry_type,
                debit_amount,
                credit_amount,
                reference_number,
                notes
            )
            VALUES (
                $1, $2, 'PURCHASE', $3, 0, $4, $5
            )
            "#,
        )
        .bind(purchase.supplier_id)
        .bind(id)
        .bind(purchase.grand_total)
        .bind(format!("PURCHASE-{}", id))
        .bind("Purchase invoice posted")
        .execute(&mut *tx)
        .await
        .map_err(ApiError::internal)?;
    }

    // Mark purchase as posted.

    let result = sqlx::query(
        r#"
        UPDATE purchases
        SET
            status = 'POSTED',
            posted_at = NOW(),
            updated_at = NOW()
        WHERE id = $1
          AND status = 'DRAFT'
        "#,
    )
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(ApiError::internal)?;

    if result.rows_affected() != 1 {
        return Err(ApiError::conflict("Purchase could not be posted"));
    }

    // Commit all inventory and accounting changes together.

    tx.commit().await.map_err(ApiError::internal)?;

    let response = load_purchase_detail(&pool, id).await?;

    Ok(Json(response))
}

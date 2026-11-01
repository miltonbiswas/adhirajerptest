use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BusinessProfile {
    pub business_name: String,
    pub legal_name: String,
    pub business_type: String,
    pub owner_name: String,

    pub phone: String,
    pub email: String,

    pub address_line1: String,
    pub address_line2: String,
    pub city: String,
    pub district: String,
    pub state: String,
    pub pincode: String,
    pub country: String,

    pub gstin: String,
    pub drug_license_retail: String,
    pub drug_license_wholesale: String,
    pub pharmacy_registration: String,
}

#[derive(Debug, Deserialize)]
pub struct BusinessProfileInput {
    #[serde(default)]
    pub business_name: String,
    #[serde(default)]
    pub legal_name: String,
    #[serde(default)]
    pub business_type: String,
    #[serde(default)]
    pub owner_name: String,

    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub email: String,

    #[serde(default)]
    pub address_line1: String,
    #[serde(default)]
    pub address_line2: String,
    #[serde(default)]
    pub city: String,
    #[serde(default)]
    pub district: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub pincode: String,
    #[serde(default)]
    pub country: String,

    #[serde(default)]
    pub gstin: String,
    #[serde(default)]
    pub drug_license_retail: String,
    #[serde(default)]
    pub drug_license_wholesale: String,
    #[serde(default)]
    pub pharmacy_registration: String,
}

fn normalize(value: &str) -> String {
    value.trim().to_string()
}

fn error_response(status: StatusCode, message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(serde_json::json!({
            "error": message
        })),
    )
}

async fn fetch_profile(pool: &PgPool) -> Result<Option<BusinessProfile>, sqlx::Error> {
    sqlx::query_as::<_, BusinessProfile>(
        r#"
        SELECT
            business_name,
            legal_name,
            business_type,
            owner_name,
            phone,
            email,
            address_line1,
            address_line2,
            city,
            district,
            state,
            pincode,
            country,
            gstin,
            drug_license_retail,
            drug_license_wholesale,
            pharmacy_registration
        FROM business_profile
        WHERE id = 1
        "#,
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_business_profile(
    axum::extract::State(pool): axum::extract::State<PgPool>,
) -> impl IntoResponse {
    match fetch_profile(&pool).await {
        Ok(Some(profile)) => (StatusCode::OK, Json(serde_json::json!(profile))).into_response(),

        Ok(None) => {
            let empty = BusinessProfile {
                business_name: String::new(),
                legal_name: String::new(),
                business_type: "Pharmacy / Medical Store".to_string(),
                owner_name: String::new(),
                phone: String::new(),
                email: String::new(),
                address_line1: String::new(),
                address_line2: String::new(),
                city: String::new(),
                district: String::new(),
                state: "Rajasthan".to_string(),
                pincode: String::new(),
                country: "India".to_string(),
                gstin: String::new(),
                drug_license_retail: String::new(),
                drug_license_wholesale: String::new(),
                pharmacy_registration: String::new(),
            };

            (StatusCode::OK, Json(serde_json::json!(empty))).into_response()
        }

        Err(error) => {
            tracing::error!(
                error = %error,
                "Failed to fetch business profile"
            );

            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Business profile fetch failed",
            )
            .into_response()
        }
    }
}

pub async fn save_business_profile(
    axum::extract::State(pool): axum::extract::State<PgPool>,
    Json(input): Json<BusinessProfileInput>,
) -> impl IntoResponse {
    let business_name = normalize(&input.business_name);

    if business_name.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Business name is required")
            .into_response();
    }

    if business_name.chars().count() > 200 {
        return error_response(
            StatusCode::BAD_REQUEST,
            "Business name must not exceed 200 characters",
        )
        .into_response();
    }

    let pincode = normalize(&input.pincode);

    if !pincode.is_empty() && (pincode.len() != 6 || !pincode.chars().all(|c| c.is_ascii_digit())) {
        return error_response(
            StatusCode::BAD_REQUEST,
            "PIN code must contain exactly 6 digits",
        )
        .into_response();
    }

    let gstin = normalize(&input.gstin).to_ascii_uppercase();

    if !gstin.is_empty() && (gstin.len() != 15 || !gstin.chars().all(|c| c.is_ascii_alphanumeric()))
    {
        return error_response(
            StatusCode::BAD_REQUEST,
            "GSTIN must contain 15 alphanumeric characters",
        )
        .into_response();
    }

    let legal_name = normalize(&input.legal_name);
    let business_type = normalize(&input.business_type);
    let owner_name = normalize(&input.owner_name);
    let phone = normalize(&input.phone);
    let email = normalize(&input.email);
    let address_line1 = normalize(&input.address_line1);
    let address_line2 = normalize(&input.address_line2);
    let city = normalize(&input.city);
    let district = normalize(&input.district);
    let state = normalize(&input.state);
    let country = normalize(&input.country);
    let drug_license_retail = normalize(&input.drug_license_retail);
    let drug_license_wholesale = normalize(&input.drug_license_wholesale);
    let pharmacy_registration = normalize(&input.pharmacy_registration);

    let result = sqlx::query(
        r#"
        INSERT INTO business_profile (
            id,
            business_name,
            legal_name,
            business_type,
            owner_name,
            phone,
            email,
            address_line1,
            address_line2,
            city,
            district,
            state,
            pincode,
            country,
            gstin,
            drug_license_retail,
            drug_license_wholesale,
            pharmacy_registration
        )
        VALUES (
            1, $1, $2, $3, $4, $5, $6, $7, $8, $9,
            $10, $11, $12, $13, $14, $15, $16, $17
        )
        ON CONFLICT (id)
        DO UPDATE SET
            business_name = EXCLUDED.business_name,
            legal_name = EXCLUDED.legal_name,
            business_type = EXCLUDED.business_type,
            owner_name = EXCLUDED.owner_name,
            phone = EXCLUDED.phone,
            email = EXCLUDED.email,
            address_line1 = EXCLUDED.address_line1,
            address_line2 = EXCLUDED.address_line2,
            city = EXCLUDED.city,
            district = EXCLUDED.district,
            state = EXCLUDED.state,
            pincode = EXCLUDED.pincode,
            country = EXCLUDED.country,
            gstin = EXCLUDED.gstin,
            drug_license_retail = EXCLUDED.drug_license_retail,
            drug_license_wholesale = EXCLUDED.drug_license_wholesale,
            pharmacy_registration = EXCLUDED.pharmacy_registration,
            updated_at = NOW()
        "#,
    )
    .bind(&business_name)
    .bind(&legal_name)
    .bind(&business_type)
    .bind(&owner_name)
    .bind(&phone)
    .bind(&email)
    .bind(&address_line1)
    .bind(&address_line2)
    .bind(&city)
    .bind(&district)
    .bind(&state)
    .bind(&pincode)
    .bind(&country)
    .bind(&gstin)
    .bind(&drug_license_retail)
    .bind(&drug_license_wholesale)
    .bind(&pharmacy_registration)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => match fetch_profile(&pool).await {
            Ok(Some(profile)) => (StatusCode::OK, Json(serde_json::json!(profile))).into_response(),

            Ok(None) => error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Profile saved but could not be retrieved",
            )
            .into_response(),

            Err(error) => {
                tracing::error!(
                    error = %error,
                    "Profile saved but fetch failed"
                );

                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Profile saved but could not be retrieved",
                )
                .into_response()
            }
        },

        Err(error) => {
            tracing::error!(
                error = %error,
                "Failed to save business profile"
            );

            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Business profile save failed",
            )
            .into_response()
        }
    }
}

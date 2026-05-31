//! Landlord / Schedule E routes — properties, tenants, leases, income,
//! expenses, mileage, maintenance, services log, and the per-year roll-up.
//!
//! Mounted as a sub-router under `/api/rental`. Auth uses the same
//! `AuthUser` extractor as the trading routes. SQL is inlined here in the
//! same style as `expense_routes`.
//!
//! Ownership is enforced at every endpoint: every read/write either filters
//! `user_id = $1` directly or joins through `rental_properties.user_id`. A
//! `Forbidden` response is returned when a property exists but belongs to a
//! different user; `NotFound` when no row matches.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, patch};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use traderview_expense::schedule_e::{
    roll_property, roll_report, ExpenseRow, IncomeKind as SeIncomeKind, IncomeRow, MileageRow,
    PropertyInput, PropertyType as SePropertyType, ScheduleECategory, ScheduleEReport,
};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        // properties
        .route("/properties", get(list_properties).post(create_property))
        .route("/properties/:id", patch(update_property).delete(delete_property))
        // tenants
        .route("/tenants", get(list_tenants).post(create_tenant))
        .route("/tenants/:id", patch(update_tenant).delete(delete_tenant))
        // leases
        .route("/properties/:property_id/leases", get(list_leases).post(create_lease))
        .route("/leases/:id", patch(update_lease).delete(delete_lease))
        // income
        .route("/properties/:property_id/income", get(list_income).post(create_income))
        .route("/income/:id", delete(delete_income))
        // expenses
        .route("/properties/:property_id/expenses", get(list_expenses).post(create_expense))
        .route("/expenses/:id", delete(delete_expense))
        // mileage
        .route("/properties/:property_id/mileage", get(list_mileage).post(create_mileage))
        .route("/mileage/:id", delete(delete_mileage))
        // maintenance
        .route("/properties/:property_id/maintenance", get(list_maintenance).post(create_maintenance))
        .route("/maintenance/:id", patch(update_maintenance).delete(delete_maintenance_row))
        // services log (QBI 250-hour tracker)
        .route("/properties/:property_id/services", get(list_services).post(create_service))
        .route("/services/:id", delete(delete_service))
        // categories
        .route("/categories", get(list_schedule_e_categories))
        // reports
        .route("/report/schedule_e", get(schedule_e_report))
        .route("/properties/:property_id/qbi-hours", get(qbi_hours_report))
        .route("/properties/:property_id/rent-roll", get(rent_roll))
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

async fn ensure_property_owner(s: &AppState, user_id: Uuid, pid: Uuid) -> Result<(), ApiError> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT user_id FROM rental_properties WHERE id = $1",
    )
    .bind(pid)
    .fetch_optional(&s.pool)
    .await?;
    match row {
        Some((owner,)) if owner == user_id => Ok(()),
        Some(_) => Err(ApiError::Forbidden),
        None => Err(ApiError::NotFound),
    }
}

async fn ensure_lease_owner(s: &AppState, user_id: Uuid, lease_id: Uuid) -> Result<Uuid, ApiError> {
    let row: Option<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT p.user_id, l.property_id
           FROM rental_leases l
           JOIN rental_properties p ON p.id = l.property_id
          WHERE l.id = $1",
    )
    .bind(lease_id)
    .fetch_optional(&s.pool)
    .await?;
    match row {
        Some((owner, pid)) if owner == user_id => Ok(pid),
        Some(_) => Err(ApiError::Forbidden),
        None => Err(ApiError::NotFound),
    }
}

fn parse_property_type(s: &str) -> Result<&'static str, ApiError> {
    Ok(match s {
        "single_family" => "single_family",
        "multi_family" => "multi_family",
        "vacation_short_term" => "vacation_short_term",
        "commercial" => "commercial",
        "land" => "land",
        "royalties" => "royalties",
        "self_rental" => "self_rental",
        "other" => "other",
        _ => return Err(ApiError::BadRequest(format!("invalid property_type: {s}"))),
    })
}

fn parse_lease_status(s: &str) -> Result<&'static str, ApiError> {
    Ok(match s {
        "draft" => "draft",
        "active" => "active",
        "expired" => "expired",
        "terminated_early" => "terminated_early",
        _ => return Err(ApiError::BadRequest(format!("invalid lease status: {s}"))),
    })
}

fn parse_income_kind(s: &str) -> Result<&'static str, ApiError> {
    Ok(match s {
        "rent" => "rent",
        "late_fee" => "late_fee",
        "deposit_forfeit" => "deposit_forfeit",
        "reimbursement" => "reimbursement",
        "royalty" => "royalty",
        "parking" => "parking",
        "laundry" => "laundry",
        "storage" => "storage",
        "other" => "other",
        _ => return Err(ApiError::BadRequest(format!("invalid income kind: {s}"))),
    })
}

fn parse_maint_status(s: &str) -> Result<&'static str, ApiError> {
    Ok(match s {
        "open" => "open",
        "in_progress" => "in_progress",
        "done" => "done",
        "cancelled" => "cancelled",
        _ => return Err(ApiError::BadRequest(format!("invalid maintenance status: {s}"))),
    })
}

fn parse_maint_priority(s: &str) -> Result<&'static str, ApiError> {
    Ok(match s {
        "low" => "low",
        "normal" => "normal",
        "high" => "high",
        "emergency" => "emergency",
        _ => return Err(ApiError::BadRequest(format!("invalid maintenance priority: {s}"))),
    })
}

fn property_type_enum(s: &str) -> SePropertyType {
    match s {
        "single_family"       => SePropertyType::SingleFamily,
        "multi_family"        => SePropertyType::MultiFamily,
        "vacation_short_term" => SePropertyType::VacationShortTerm,
        "commercial"          => SePropertyType::Commercial,
        "land"                => SePropertyType::Land,
        "royalties"           => SePropertyType::Royalties,
        "self_rental"         => SePropertyType::SelfRental,
        _                     => SePropertyType::Other,
    }
}

fn income_kind_enum(s: &str) -> SeIncomeKind {
    match s {
        "rent"     => SeIncomeKind::Rent,
        "royalty"  => SeIncomeKind::Royalty,
        _          => SeIncomeKind::Other,
    }
}

// ---------------------------------------------------------------------------
// properties
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct Property {
    id: Uuid,
    user_id: Uuid,
    nickname: String,
    property_type: String,
    status: String,
    address_line1: String,
    address_line2: String,
    city: String,
    state_region: String,
    postal_code: String,
    country: String,
    units: i32,
    purchased_at: Option<NaiveDate>,
    purchase_price: Option<Decimal>,
    land_value: Option<Decimal>,
    placed_in_service_at: Option<NaiveDate>,
    recovery_period_years: Decimal,
    fair_rental_days: i32,
    personal_use_days: i32,
    qjv_election: bool,
    qbi_safe_harbor: bool,
    sold_at: Option<NaiveDate>,
    sold_price: Option<Decimal>,
    notes: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct PropertyInputDto {
    nickname: String,
    property_type: String,
    status: Option<String>,
    address_line1: Option<String>,
    address_line2: Option<String>,
    city: Option<String>,
    state_region: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    units: Option<i32>,
    purchased_at: Option<NaiveDate>,
    purchase_price: Option<Decimal>,
    land_value: Option<Decimal>,
    placed_in_service_at: Option<NaiveDate>,
    recovery_period_years: Option<Decimal>,
    fair_rental_days: Option<i32>,
    personal_use_days: Option<i32>,
    qjv_election: Option<bool>,
    qbi_safe_harbor: Option<bool>,
    sold_at: Option<NaiveDate>,
    sold_price: Option<Decimal>,
    notes: Option<String>,
}

const PROPERTY_COLS: &str = "id, user_id, nickname, property_type::text, status::text,
    address_line1, address_line2, city, state_region, postal_code, country,
    units, purchased_at, purchase_price, land_value, placed_in_service_at,
    recovery_period_years, fair_rental_days, personal_use_days, qjv_election,
    qbi_safe_harbor, sold_at, sold_price, notes, created_at";

async fn list_properties(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<Vec<Property>>, ApiError> {
    Ok(Json(
        sqlx::query_as(&format!(
            "SELECT {PROPERTY_COLS} FROM rental_properties
              WHERE user_id = $1 ORDER BY status ASC, nickname ASC"
        ))
        .bind(u.id)
        .fetch_all(&s.pool)
        .await?,
    ))
}

async fn create_property(
    State(s): State<AppState>,
    u: AuthUser,
    Json(b): Json<PropertyInputDto>,
) -> Result<Json<Property>, ApiError> {
    if b.nickname.trim().is_empty() {
        return Err(ApiError::BadRequest("nickname required".into()));
    }
    let pt = parse_property_type(&b.property_type)?;
    let status = b.status.as_deref().unwrap_or("active");
    if !matches!(status, "active" | "vacant" | "sold" | "archived") {
        return Err(ApiError::BadRequest(format!("invalid status: {status}")));
    }
    let row = sqlx::query_as(&format!(
        "INSERT INTO rental_properties
           (user_id, nickname, property_type, status, address_line1, address_line2,
            city, state_region, postal_code, country, units, purchased_at,
            purchase_price, land_value, placed_in_service_at, recovery_period_years,
            fair_rental_days, personal_use_days, qjv_election, qbi_safe_harbor,
            sold_at, sold_price, notes)
         VALUES ($1, $2, $3::property_type_t, $4::property_status_t, $5, $6, $7, $8,
                 $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21,
                 $22, $23)
         RETURNING {PROPERTY_COLS}"
    ))
    .bind(u.id)
    .bind(&b.nickname)
    .bind(pt)
    .bind(status)
    .bind(b.address_line1.unwrap_or_default())
    .bind(b.address_line2.unwrap_or_default())
    .bind(b.city.unwrap_or_default())
    .bind(b.state_region.unwrap_or_default())
    .bind(b.postal_code.unwrap_or_default())
    .bind(b.country.unwrap_or_else(|| "US".into()))
    .bind(b.units.unwrap_or(1))
    .bind(b.purchased_at)
    .bind(b.purchase_price)
    .bind(b.land_value)
    .bind(b.placed_in_service_at)
    .bind(b.recovery_period_years.unwrap_or_else(|| Decimal::from_str("27.5").unwrap()))
    .bind(b.fair_rental_days.unwrap_or(0))
    .bind(b.personal_use_days.unwrap_or(0))
    .bind(b.qjv_election.unwrap_or(false))
    .bind(b.qbi_safe_harbor.unwrap_or(false))
    .bind(b.sold_at)
    .bind(b.sold_price)
    .bind(b.notes.unwrap_or_default())
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(row))
}

async fn update_property(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(b): Json<PropertyInputDto>,
) -> Result<Json<Property>, ApiError> {
    ensure_property_owner(&s, u.id, id).await?;
    let pt = parse_property_type(&b.property_type)?;
    let status = b.status.as_deref().unwrap_or("active");
    let row = sqlx::query_as(&format!(
        "UPDATE rental_properties SET
            nickname = $3,
            property_type = $4::property_type_t,
            status = $5::property_status_t,
            address_line1 = $6,
            address_line2 = $7,
            city = $8,
            state_region = $9,
            postal_code = $10,
            country = $11,
            units = $12,
            purchased_at = $13,
            purchase_price = $14,
            land_value = $15,
            placed_in_service_at = $16,
            recovery_period_years = $17,
            fair_rental_days = $18,
            personal_use_days = $19,
            qjv_election = $20,
            qbi_safe_harbor = $21,
            sold_at = $22,
            sold_price = $23,
            notes = $24
          WHERE id = $1 AND user_id = $2
          RETURNING {PROPERTY_COLS}"
    ))
    .bind(id)
    .bind(u.id)
    .bind(&b.nickname)
    .bind(pt)
    .bind(status)
    .bind(b.address_line1.unwrap_or_default())
    .bind(b.address_line2.unwrap_or_default())
    .bind(b.city.unwrap_or_default())
    .bind(b.state_region.unwrap_or_default())
    .bind(b.postal_code.unwrap_or_default())
    .bind(b.country.unwrap_or_else(|| "US".into()))
    .bind(b.units.unwrap_or(1))
    .bind(b.purchased_at)
    .bind(b.purchase_price)
    .bind(b.land_value)
    .bind(b.placed_in_service_at)
    .bind(b.recovery_period_years.unwrap_or_else(|| Decimal::from_str("27.5").unwrap()))
    .bind(b.fair_rental_days.unwrap_or(0))
    .bind(b.personal_use_days.unwrap_or(0))
    .bind(b.qjv_election.unwrap_or(false))
    .bind(b.qbi_safe_harbor.unwrap_or(false))
    .bind(b.sold_at)
    .bind(b.sold_price)
    .bind(b.notes.unwrap_or_default())
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(row))
}

async fn delete_property(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = sqlx::query("DELETE FROM rental_properties WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(u.id)
        .execute(&s.pool)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(Json(serde_json::json!({"deleted": true})))
}

// ---------------------------------------------------------------------------
// tenants
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct Tenant {
    id: Uuid,
    user_id: Uuid,
    display_name: String,
    email: String,
    phone: String,
    notes: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct TenantInput {
    display_name: String,
    email: Option<String>,
    phone: Option<String>,
    notes: Option<String>,
}

async fn list_tenants(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<Vec<Tenant>>, ApiError> {
    Ok(Json(
        sqlx::query_as(
            "SELECT id, user_id, display_name, email, phone, notes, created_at
               FROM rental_tenants WHERE user_id = $1 ORDER BY display_name",
        )
        .bind(u.id)
        .fetch_all(&s.pool)
        .await?,
    ))
}

async fn create_tenant(
    State(s): State<AppState>,
    u: AuthUser,
    Json(b): Json<TenantInput>,
) -> Result<Json<Tenant>, ApiError> {
    if b.display_name.trim().is_empty() {
        return Err(ApiError::BadRequest("display_name required".into()));
    }
    Ok(Json(
        sqlx::query_as(
            "INSERT INTO rental_tenants (user_id, display_name, email, phone, notes)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, user_id, display_name, email, phone, notes, created_at",
        )
        .bind(u.id)
        .bind(&b.display_name)
        .bind(b.email.unwrap_or_default())
        .bind(b.phone.unwrap_or_default())
        .bind(b.notes.unwrap_or_default())
        .fetch_one(&s.pool)
        .await?,
    ))
}

async fn update_tenant(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(b): Json<TenantInput>,
) -> Result<Json<Tenant>, ApiError> {
    Ok(Json(
        sqlx::query_as(
            "UPDATE rental_tenants SET display_name = $3, email = $4, phone = $5, notes = $6
              WHERE id = $1 AND user_id = $2
              RETURNING id, user_id, display_name, email, phone, notes, created_at",
        )
        .bind(id)
        .bind(u.id)
        .bind(&b.display_name)
        .bind(b.email.unwrap_or_default())
        .bind(b.phone.unwrap_or_default())
        .bind(b.notes.unwrap_or_default())
        .fetch_optional(&s.pool)
        .await?
        .ok_or(ApiError::NotFound)?,
    ))
}

async fn delete_tenant(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = sqlx::query("DELETE FROM rental_tenants WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(u.id)
        .execute(&s.pool)
        .await?
        .rows_affected();
    if n == 0 { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({"deleted": true})))
}

// ---------------------------------------------------------------------------
// leases
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct Lease {
    id: Uuid,
    property_id: Uuid,
    tenant_id: Uuid,
    unit_label: String,
    status: String,
    starts_on: NaiveDate,
    ends_on: Option<NaiveDate>,
    rent_amount: Decimal,
    rent_frequency: String,
    rent_due_day: i32,
    grace_days: i32,
    late_fee_fixed: Decimal,
    late_fee_pct: Decimal,
    security_deposit: Decimal,
    deposit_held_by: String,
    pet_deposit: Decimal,
    notes: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct LeaseInput {
    tenant_id: Uuid,
    unit_label: Option<String>,
    status: Option<String>,
    starts_on: NaiveDate,
    ends_on: Option<NaiveDate>,
    rent_amount: Decimal,
    rent_frequency: Option<String>,
    rent_due_day: Option<i32>,
    grace_days: Option<i32>,
    late_fee_fixed: Option<Decimal>,
    late_fee_pct: Option<Decimal>,
    security_deposit: Option<Decimal>,
    deposit_held_by: Option<String>,
    pet_deposit: Option<Decimal>,
    notes: Option<String>,
}

const LEASE_COLS: &str = "id, property_id, tenant_id, unit_label, status::text,
    starts_on, ends_on, rent_amount, rent_frequency, rent_due_day, grace_days,
    late_fee_fixed, late_fee_pct, security_deposit, deposit_held_by, pet_deposit,
    notes, created_at";

async fn list_leases(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
) -> Result<Json<Vec<Lease>>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    Ok(Json(
        sqlx::query_as(&format!(
            "SELECT {LEASE_COLS} FROM rental_leases
              WHERE property_id = $1 ORDER BY starts_on DESC"
        ))
        .bind(property_id)
        .fetch_all(&s.pool)
        .await?,
    ))
}

async fn create_lease(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Json(b): Json<LeaseInput>,
) -> Result<Json<Lease>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    if let Some(end) = b.ends_on {
        if end < b.starts_on {
            return Err(ApiError::BadRequest("ends_on must be >= starts_on".into()));
        }
    }
    let status = parse_lease_status(b.status.as_deref().unwrap_or("active"))?;
    let due_day = b.rent_due_day.unwrap_or(1);
    if !(1..=31).contains(&due_day) {
        return Err(ApiError::BadRequest("rent_due_day must be 1..31".into()));
    }
    Ok(Json(
        sqlx::query_as(&format!(
            "INSERT INTO rental_leases
               (property_id, tenant_id, unit_label, status, starts_on, ends_on,
                rent_amount, rent_frequency, rent_due_day, grace_days,
                late_fee_fixed, late_fee_pct, security_deposit, deposit_held_by,
                pet_deposit, notes)
             VALUES ($1, $2, $3, $4::lease_status_t, $5, $6, $7, $8, $9, $10,
                     $11, $12, $13, $14, $15, $16)
             RETURNING {LEASE_COLS}"
        ))
        .bind(property_id)
        .bind(b.tenant_id)
        .bind(b.unit_label.unwrap_or_default())
        .bind(status)
        .bind(b.starts_on)
        .bind(b.ends_on)
        .bind(b.rent_amount)
        .bind(b.rent_frequency.unwrap_or_else(|| "monthly".into()))
        .bind(due_day)
        .bind(b.grace_days.unwrap_or(5))
        .bind(b.late_fee_fixed.unwrap_or(Decimal::ZERO))
        .bind(b.late_fee_pct.unwrap_or(Decimal::ZERO))
        .bind(b.security_deposit.unwrap_or(Decimal::ZERO))
        .bind(b.deposit_held_by.unwrap_or_default())
        .bind(b.pet_deposit.unwrap_or(Decimal::ZERO))
        .bind(b.notes.unwrap_or_default())
        .fetch_one(&s.pool)
        .await?,
    ))
}

async fn update_lease(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(b): Json<LeaseInput>,
) -> Result<Json<Lease>, ApiError> {
    ensure_lease_owner(&s, u.id, id).await?;
    let status = parse_lease_status(b.status.as_deref().unwrap_or("active"))?;
    Ok(Json(
        sqlx::query_as(&format!(
            "UPDATE rental_leases SET
                tenant_id = $2,
                unit_label = $3,
                status = $4::lease_status_t,
                starts_on = $5,
                ends_on = $6,
                rent_amount = $7,
                rent_frequency = $8,
                rent_due_day = $9,
                grace_days = $10,
                late_fee_fixed = $11,
                late_fee_pct = $12,
                security_deposit = $13,
                deposit_held_by = $14,
                pet_deposit = $15,
                notes = $16
              WHERE id = $1
              RETURNING {LEASE_COLS}"
        ))
        .bind(id)
        .bind(b.tenant_id)
        .bind(b.unit_label.unwrap_or_default())
        .bind(status)
        .bind(b.starts_on)
        .bind(b.ends_on)
        .bind(b.rent_amount)
        .bind(b.rent_frequency.unwrap_or_else(|| "monthly".into()))
        .bind(b.rent_due_day.unwrap_or(1))
        .bind(b.grace_days.unwrap_or(5))
        .bind(b.late_fee_fixed.unwrap_or(Decimal::ZERO))
        .bind(b.late_fee_pct.unwrap_or(Decimal::ZERO))
        .bind(b.security_deposit.unwrap_or(Decimal::ZERO))
        .bind(b.deposit_held_by.unwrap_or_default())
        .bind(b.pet_deposit.unwrap_or(Decimal::ZERO))
        .bind(b.notes.unwrap_or_default())
        .fetch_one(&s.pool)
        .await?,
    ))
}

async fn delete_lease(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    ensure_lease_owner(&s, u.id, id).await?;
    sqlx::query("DELETE FROM rental_leases WHERE id = $1")
        .bind(id)
        .execute(&s.pool)
        .await?;
    Ok(Json(serde_json::json!({"deleted": true})))
}

// ---------------------------------------------------------------------------
// income
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct Income {
    id: Uuid,
    property_id: Uuid,
    lease_id: Option<Uuid>,
    posted_at: DateTime<Utc>,
    period_start: Option<NaiveDate>,
    period_end: Option<NaiveDate>,
    amount: Decimal,
    currency: String,
    kind: String,
    payer_raw: String,
    method: String,
    transaction_id: Option<Uuid>,
    notes: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct IncomeInput {
    lease_id: Option<Uuid>,
    posted_at: DateTime<Utc>,
    period_start: Option<NaiveDate>,
    period_end: Option<NaiveDate>,
    amount: Decimal,
    currency: Option<String>,
    kind: Option<String>,
    payer_raw: Option<String>,
    method: Option<String>,
    transaction_id: Option<Uuid>,
    notes: Option<String>,
}

const INCOME_COLS: &str = "id, property_id, lease_id, posted_at, period_start,
    period_end, amount, currency, kind::text, payer_raw, method, transaction_id,
    notes, created_at";

#[derive(Deserialize)]
struct IncomeQuery {
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    kind: Option<String>,
}

async fn list_income(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Query(q): Query<IncomeQuery>,
) -> Result<Json<Vec<Income>>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    let from = q.from.unwrap_or(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    let to = q.to.unwrap_or(NaiveDate::from_ymd_opt(9999, 12, 31).unwrap());
    let kind = q.kind.unwrap_or_default();
    let rows = sqlx::query_as(&format!(
        "SELECT {INCOME_COLS} FROM rental_income
          WHERE property_id = $1
            AND posted_at >= $2::date
            AND posted_at <  ($3::date + INTERVAL '1 day')
            AND ($4 = '' OR kind::text = $4)
          ORDER BY posted_at DESC"
    ))
    .bind(property_id)
    .bind(from)
    .bind(to)
    .bind(kind)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_income(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Json(b): Json<IncomeInput>,
) -> Result<Json<Income>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    let kind = parse_income_kind(b.kind.as_deref().unwrap_or("rent"))?;
    Ok(Json(
        sqlx::query_as(&format!(
            "INSERT INTO rental_income
               (property_id, lease_id, posted_at, period_start, period_end, amount,
                currency, kind, payer_raw, method, transaction_id, notes)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8::rental_income_kind_t,
                     $9, $10, $11, $12)
             RETURNING {INCOME_COLS}"
        ))
        .bind(property_id)
        .bind(b.lease_id)
        .bind(b.posted_at)
        .bind(b.period_start)
        .bind(b.period_end)
        .bind(b.amount)
        .bind(b.currency.unwrap_or_else(|| "USD".into()))
        .bind(kind)
        .bind(b.payer_raw.unwrap_or_default())
        .bind(b.method.unwrap_or_default())
        .bind(b.transaction_id)
        .bind(b.notes.unwrap_or_default())
        .fetch_one(&s.pool)
        .await?,
    ))
}

async fn delete_income(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = sqlx::query(
        "DELETE FROM rental_income
          WHERE id = $1
            AND property_id IN (SELECT id FROM rental_properties WHERE user_id = $2)",
    )
    .bind(id)
    .bind(u.id)
    .execute(&s.pool)
    .await?
    .rows_affected();
    if n == 0 { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({"deleted": true})))
}

// ---------------------------------------------------------------------------
// expenses
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct Expense {
    id: Uuid,
    property_id: Uuid,
    posted_at: DateTime<Utc>,
    amount: Decimal,
    currency: String,
    category_code: String,
    vendor_raw: String,
    vendor_normalized: String,
    description: String,
    is_capitalized: bool,
    capital_useful_life: Option<i32>,
    method: String,
    transaction_id: Option<Uuid>,
    notes: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct ExpenseInput {
    posted_at: DateTime<Utc>,
    amount: Decimal,
    currency: Option<String>,
    category_code: String,
    vendor_raw: Option<String>,
    description: Option<String>,
    is_capitalized: Option<bool>,
    capital_useful_life: Option<i32>,
    method: Option<String>,
    transaction_id: Option<Uuid>,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct ExpenseQuery {
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    category: Option<String>,
    capitalized: Option<bool>,
}

fn normalize_vendor(raw: &str) -> String {
    raw.trim().to_uppercase()
}

async fn list_expenses(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Query(q): Query<ExpenseQuery>,
) -> Result<Json<Vec<Expense>>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    let from = q.from.unwrap_or(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    let to = q.to.unwrap_or(NaiveDate::from_ymd_opt(9999, 12, 31).unwrap());
    let cat = q.category.unwrap_or_default();
    let capitalized = q.capitalized; // None = either
    Ok(Json(
        sqlx::query_as(
            "SELECT id, property_id, posted_at, amount, currency, category_code,
                    vendor_raw, vendor_normalized, description, is_capitalized,
                    capital_useful_life, method, transaction_id, notes, created_at
               FROM rental_expenses
              WHERE property_id = $1
                AND posted_at >= $2::date
                AND posted_at <  ($3::date + INTERVAL '1 day')
                AND ($4 = '' OR category_code = $4)
                AND ($5::boolean IS NULL OR is_capitalized = $5)
              ORDER BY posted_at DESC",
        )
        .bind(property_id)
        .bind(from)
        .bind(to)
        .bind(cat)
        .bind(capitalized)
        .fetch_all(&s.pool)
        .await?,
    ))
}

async fn create_expense(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Json(b): Json<ExpenseInput>,
) -> Result<Json<Expense>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    let vendor = b.vendor_raw.unwrap_or_default();
    let normalized = normalize_vendor(&vendor);
    Ok(Json(
        sqlx::query_as(
            "INSERT INTO rental_expenses
               (property_id, posted_at, amount, currency, category_code, vendor_raw,
                vendor_normalized, description, is_capitalized, capital_useful_life,
                method, transaction_id, notes)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             RETURNING id, property_id, posted_at, amount, currency, category_code,
                       vendor_raw, vendor_normalized, description, is_capitalized,
                       capital_useful_life, method, transaction_id, notes, created_at",
        )
        .bind(property_id)
        .bind(b.posted_at)
        .bind(b.amount)
        .bind(b.currency.unwrap_or_else(|| "USD".into()))
        .bind(&b.category_code)
        .bind(&vendor)
        .bind(normalized)
        .bind(b.description.unwrap_or_default())
        .bind(b.is_capitalized.unwrap_or(false))
        .bind(b.capital_useful_life)
        .bind(b.method.unwrap_or_default())
        .bind(b.transaction_id)
        .bind(b.notes.unwrap_or_default())
        .fetch_one(&s.pool)
        .await?,
    ))
}

async fn delete_expense(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = sqlx::query(
        "DELETE FROM rental_expenses
          WHERE id = $1
            AND property_id IN (SELECT id FROM rental_properties WHERE user_id = $2)",
    )
    .bind(id)
    .bind(u.id)
    .execute(&s.pool)
    .await?
    .rows_affected();
    if n == 0 { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({"deleted": true})))
}

// ---------------------------------------------------------------------------
// mileage
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct Mileage {
    id: Uuid,
    property_id: Uuid,
    drove_on: NaiveDate,
    miles: Decimal,
    rate_per_mile: Decimal,
    purpose: String,
    odometer_start: Option<Decimal>,
    odometer_end: Option<Decimal>,
    notes: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct MileageInput {
    drove_on: NaiveDate,
    miles: Decimal,
    rate_per_mile: Decimal,
    purpose: Option<String>,
    odometer_start: Option<Decimal>,
    odometer_end: Option<Decimal>,
    notes: Option<String>,
}

async fn list_mileage(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
) -> Result<Json<Vec<Mileage>>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    Ok(Json(
        sqlx::query_as(
            "SELECT id, property_id, drove_on, miles, rate_per_mile, purpose,
                    odometer_start, odometer_end, notes, created_at
               FROM rental_mileage
              WHERE property_id = $1
              ORDER BY drove_on DESC",
        )
        .bind(property_id)
        .fetch_all(&s.pool)
        .await?,
    ))
}

async fn create_mileage(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Json(b): Json<MileageInput>,
) -> Result<Json<Mileage>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    if b.miles <= Decimal::ZERO {
        return Err(ApiError::BadRequest("miles must be > 0".into()));
    }
    Ok(Json(
        sqlx::query_as(
            "INSERT INTO rental_mileage
               (property_id, drove_on, miles, rate_per_mile, purpose,
                odometer_start, odometer_end, notes)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, property_id, drove_on, miles, rate_per_mile, purpose,
                       odometer_start, odometer_end, notes, created_at",
        )
        .bind(property_id)
        .bind(b.drove_on)
        .bind(b.miles)
        .bind(b.rate_per_mile)
        .bind(b.purpose.unwrap_or_default())
        .bind(b.odometer_start)
        .bind(b.odometer_end)
        .bind(b.notes.unwrap_or_default())
        .fetch_one(&s.pool)
        .await?,
    ))
}

async fn delete_mileage(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = sqlx::query(
        "DELETE FROM rental_mileage
          WHERE id = $1
            AND property_id IN (SELECT id FROM rental_properties WHERE user_id = $2)",
    )
    .bind(id)
    .bind(u.id)
    .execute(&s.pool)
    .await?
    .rows_affected();
    if n == 0 { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({"deleted": true})))
}

// ---------------------------------------------------------------------------
// maintenance
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct Maintenance {
    id: Uuid,
    property_id: Uuid,
    lease_id: Option<Uuid>,
    title: String,
    description: String,
    status: String,
    priority: String,
    reported_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    vendor: String,
    expense_id: Option<Uuid>,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct MaintenanceInput {
    lease_id: Option<Uuid>,
    title: String,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    vendor: Option<String>,
    expense_id: Option<Uuid>,
}

const MAINT_COLS: &str = "id, property_id, lease_id, title, description,
    status::text, priority::text, reported_at, started_at, completed_at,
    vendor, expense_id, created_at";

async fn list_maintenance(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
) -> Result<Json<Vec<Maintenance>>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    Ok(Json(
        sqlx::query_as(&format!(
            "SELECT {MAINT_COLS} FROM rental_maintenance
              WHERE property_id = $1
              ORDER BY status = 'done' ASC, reported_at DESC"
        ))
        .bind(property_id)
        .fetch_all(&s.pool)
        .await?,
    ))
}

async fn create_maintenance(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Json(b): Json<MaintenanceInput>,
) -> Result<Json<Maintenance>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    if b.title.trim().is_empty() {
        return Err(ApiError::BadRequest("title required".into()));
    }
    let status = parse_maint_status(b.status.as_deref().unwrap_or("open"))?;
    let priority = parse_maint_priority(b.priority.as_deref().unwrap_or("normal"))?;
    Ok(Json(
        sqlx::query_as(&format!(
            "INSERT INTO rental_maintenance
               (property_id, lease_id, title, description, status, priority,
                started_at, completed_at, vendor, expense_id)
             VALUES ($1, $2, $3, $4, $5::maintenance_status_t,
                     $6::maintenance_priority_t, $7, $8, $9, $10)
             RETURNING {MAINT_COLS}"
        ))
        .bind(property_id)
        .bind(b.lease_id)
        .bind(&b.title)
        .bind(b.description.unwrap_or_default())
        .bind(status)
        .bind(priority)
        .bind(b.started_at)
        .bind(b.completed_at)
        .bind(b.vendor.unwrap_or_default())
        .bind(b.expense_id)
        .fetch_one(&s.pool)
        .await?,
    ))
}

async fn update_maintenance(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
    Json(b): Json<MaintenanceInput>,
) -> Result<Json<Maintenance>, ApiError> {
    // ownership check via subquery in WHERE
    let status = parse_maint_status(b.status.as_deref().unwrap_or("open"))?;
    let priority = parse_maint_priority(b.priority.as_deref().unwrap_or("normal"))?;
    let row = sqlx::query_as(&format!(
        "UPDATE rental_maintenance SET
            lease_id = $2, title = $3, description = $4,
            status = $5::maintenance_status_t,
            priority = $6::maintenance_priority_t,
            started_at = $7, completed_at = $8, vendor = $9, expense_id = $10
          WHERE id = $1
            AND property_id IN (SELECT id FROM rental_properties WHERE user_id = $11)
          RETURNING {MAINT_COLS}"
    ))
    .bind(id)
    .bind(b.lease_id)
    .bind(&b.title)
    .bind(b.description.unwrap_or_default())
    .bind(status)
    .bind(priority)
    .bind(b.started_at)
    .bind(b.completed_at)
    .bind(b.vendor.unwrap_or_default())
    .bind(b.expense_id)
    .bind(u.id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    Ok(Json(row))
}

async fn delete_maintenance_row(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = sqlx::query(
        "DELETE FROM rental_maintenance
          WHERE id = $1
            AND property_id IN (SELECT id FROM rental_properties WHERE user_id = $2)",
    )
    .bind(id)
    .bind(u.id)
    .execute(&s.pool)
    .await?
    .rows_affected();
    if n == 0 { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({"deleted": true})))
}

// ---------------------------------------------------------------------------
// services log (QBI 250-hour safe harbor tracker)
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct ServiceLog {
    id: Uuid,
    property_id: Uuid,
    performed_on: NaiveDate,
    hours: Decimal,
    activity: String,
    performer: String,
    notes: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct ServiceInput {
    performed_on: NaiveDate,
    hours: Decimal,
    activity: String,
    performer: Option<String>,
    notes: Option<String>,
}

async fn list_services(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
) -> Result<Json<Vec<ServiceLog>>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    Ok(Json(
        sqlx::query_as(
            "SELECT id, property_id, performed_on, hours, activity, performer,
                    notes, created_at
               FROM rental_services_log
              WHERE property_id = $1
              ORDER BY performed_on DESC",
        )
        .bind(property_id)
        .fetch_all(&s.pool)
        .await?,
    ))
}

async fn create_service(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Json(b): Json<ServiceInput>,
) -> Result<Json<ServiceLog>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    if b.hours <= Decimal::ZERO {
        return Err(ApiError::BadRequest("hours must be > 0".into()));
    }
    if b.activity.trim().is_empty() {
        return Err(ApiError::BadRequest("activity required".into()));
    }
    Ok(Json(
        sqlx::query_as(
            "INSERT INTO rental_services_log
               (property_id, performed_on, hours, activity, performer, notes)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, property_id, performed_on, hours, activity, performer,
                       notes, created_at",
        )
        .bind(property_id)
        .bind(b.performed_on)
        .bind(b.hours)
        .bind(&b.activity)
        .bind(b.performer.unwrap_or_else(|| "self".into()))
        .bind(b.notes.unwrap_or_default())
        .fetch_one(&s.pool)
        .await?,
    ))
}

async fn delete_service(
    State(s): State<AppState>,
    u: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let n = sqlx::query(
        "DELETE FROM rental_services_log
          WHERE id = $1
            AND property_id IN (SELECT id FROM rental_properties WHERE user_id = $2)",
    )
    .bind(id)
    .bind(u.id)
    .execute(&s.pool)
    .await?
    .rows_affected();
    if n == 0 { return Err(ApiError::NotFound); }
    Ok(Json(serde_json::json!({"deleted": true})))
}

// ---------------------------------------------------------------------------
// categories — seeded read-only list
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct Category {
    code: String,
    schedule_e_line: String,
    label: String,
    deduction_pct: Decimal,
    sort_order: i32,
}

async fn list_schedule_e_categories(
    State(s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<Vec<Category>>, ApiError> {
    Ok(Json(
        sqlx::query_as(
            "SELECT code, schedule_e_line, label, deduction_pct, sort_order
               FROM schedule_e_categories ORDER BY sort_order",
        )
        .fetch_all(&s.pool)
        .await?,
    ))
}

// ---------------------------------------------------------------------------
// Schedule E roll-up report
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ReportQuery {
    year: i32,
}

async fn schedule_e_report(
    State(s): State<AppState>,
    u: AuthUser,
    Query(q): Query<ReportQuery>,
) -> Result<Json<ScheduleEReport>, ApiError> {
    let start = NaiveDate::from_ymd_opt(q.year, 1, 1)
        .ok_or_else(|| ApiError::BadRequest("invalid year".into()))?;
    let end = NaiveDate::from_ymd_opt(q.year + 1, 1, 1)
        .ok_or_else(|| ApiError::BadRequest("invalid year".into()))?;

    let props: Vec<(Uuid, String, i32, i32)> = sqlx::query_as(
        "SELECT id, property_type::text, fair_rental_days, personal_use_days
           FROM rental_properties WHERE user_id = $1 AND status != 'archived'",
    )
    .bind(u.id)
    .fetch_all(&s.pool)
    .await?;

    let mut lines = Vec::with_capacity(props.len());
    for (pid, ptype, frd, pud) in props {
        let income_rows: Vec<(String, Decimal)> = sqlx::query_as(
            "SELECT kind::text, amount FROM rental_income
              WHERE property_id = $1 AND posted_at >= $2 AND posted_at < $3",
        )
        .bind(pid)
        .bind(start)
        .bind(end)
        .fetch_all(&s.pool)
        .await?;

        let income: Vec<IncomeRow> = income_rows
            .iter()
            .map(|(k, a)| IncomeRow { kind: income_kind_enum(k), amount: *a })
            .collect();

        let expense_rows: Vec<(String, Decimal, bool)> = sqlx::query_as(
            "SELECT category_code, amount, is_capitalized FROM rental_expenses
              WHERE property_id = $1 AND posted_at >= $2 AND posted_at < $3",
        )
        .bind(pid)
        .bind(start)
        .bind(end)
        .fetch_all(&s.pool)
        .await?;

        let expenses: Vec<ExpenseRow> = expense_rows
            .iter()
            .filter_map(|(code, amt, cap)| {
                ScheduleECategory::from_code(code).map(|cat| ExpenseRow {
                    category: cat,
                    amount: *amt,
                    is_capitalized: *cap,
                })
            })
            .collect();

        let mileage_rows: Vec<(Decimal, Decimal)> = sqlx::query_as(
            "SELECT miles, rate_per_mile FROM rental_mileage
              WHERE property_id = $1 AND drove_on >= $2 AND drove_on < $3",
        )
        .bind(pid)
        .bind(start)
        .bind(end)
        .fetch_all(&s.pool)
        .await?;

        let mileage: Vec<MileageRow> = mileage_rows
            .iter()
            .map(|(m, r)| MileageRow { miles: *m, rate_per_mile: *r })
            .collect();

        let pid_str = pid.to_string();
        let input = PropertyInput {
            property_id: &pid_str,
            property_type: property_type_enum(&ptype),
            fair_rental_days: frd as u32,
            personal_use_days: pud as u32,
            income: &income,
            expenses: &expenses,
            mileage: &mileage,
            // Depreciation passes through depreciation.rs in a follow-up
            // iteration. Zero today.
            depreciation_for_year: Decimal::ZERO,
        };
        lines.push(roll_property(&input));
    }

    Ok(Json(roll_report(lines)))
}

// ---------------------------------------------------------------------------
// QBI 250-hour safe-harbor report
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct QbiHoursReport {
    year: i32,
    hours_logged: Decimal,
    hours_required: Decimal,
    hours_remaining: Decimal,
    qualifies: bool,
}

async fn qbi_hours_report(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Query(q): Query<ReportQuery>,
) -> Result<Json<QbiHoursReport>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    let start = NaiveDate::from_ymd_opt(q.year, 1, 1)
        .ok_or_else(|| ApiError::BadRequest("invalid year".into()))?;
    let end = NaiveDate::from_ymd_opt(q.year + 1, 1, 1)
        .ok_or_else(|| ApiError::BadRequest("invalid year".into()))?;
    let (total,): (Option<Decimal>,) = sqlx::query_as(
        "SELECT SUM(hours) FROM rental_services_log
          WHERE property_id = $1 AND performed_on >= $2 AND performed_on < $3",
    )
    .bind(property_id)
    .bind(start)
    .bind(end)
    .fetch_one(&s.pool)
    .await?;
    let logged = total.unwrap_or(Decimal::ZERO);
    let req = Decimal::from(250);
    let remaining = if logged >= req { Decimal::ZERO } else { req - logged };
    Ok(Json(QbiHoursReport {
        year: q.year,
        hours_logged: logged,
        hours_required: req,
        hours_remaining: remaining,
        qualifies: logged >= req,
    }))
}

// ---------------------------------------------------------------------------
// Rent roll: per-lease expected vs collected for a month window
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct RentRollRow {
    lease_id: Uuid,
    tenant_name: String,
    unit_label: String,
    rent_amount: Decimal,
    rent_due_day: i32,
    grace_days: i32,
    expected: Decimal,
    collected: Decimal,
    balance: Decimal,
    status: String, // paid | partial | due | late
}

#[derive(Deserialize)]
struct RentRollQuery {
    year: i32,
    month: u32,
}

async fn rent_roll(
    State(s): State<AppState>,
    u: AuthUser,
    Path(property_id): Path<Uuid>,
    Query(q): Query<RentRollQuery>,
) -> Result<Json<Vec<RentRollRow>>, ApiError> {
    ensure_property_owner(&s, u.id, property_id).await?;
    if !(1..=12).contains(&q.month) {
        return Err(ApiError::BadRequest("month must be 1..12".into()));
    }
    let start = NaiveDate::from_ymd_opt(q.year, q.month, 1)
        .ok_or_else(|| ApiError::BadRequest("invalid date".into()))?;
    let (next_y, next_m) = if q.month == 12 { (q.year + 1, 1) } else { (q.year, q.month + 1) };
    let end = NaiveDate::from_ymd_opt(next_y, next_m, 1)
        .ok_or_else(|| ApiError::BadRequest("invalid date".into()))?;

    // Active leases overlapping the window.
    let leases: Vec<(Uuid, String, String, Decimal, i32, i32, NaiveDate, Option<NaiveDate>)> =
        sqlx::query_as(
            "SELECT l.id, COALESCE(t.display_name, ''), l.unit_label, l.rent_amount,
                    l.rent_due_day, l.grace_days, l.starts_on, l.ends_on
               FROM rental_leases l
               LEFT JOIN rental_tenants t ON t.id = l.tenant_id
              WHERE l.property_id = $1
                AND l.status = 'active'
                AND l.starts_on < $3
                AND (l.ends_on IS NULL OR l.ends_on >= $2)",
        )
        .bind(property_id)
        .bind(start)
        .bind(end)
        .fetch_all(&s.pool)
        .await?;

    let mut rows = Vec::with_capacity(leases.len());
    for (lid, tname, unit, rent, due_day, grace, _starts, _ends) in leases {
        // Collected = rent-kind income posted in window for this lease.
        let (col,): (Option<Decimal>,) = sqlx::query_as(
            "SELECT SUM(amount) FROM rental_income
              WHERE lease_id = $1 AND kind = 'rent'
                AND posted_at >= $2 AND posted_at < $3",
        )
        .bind(lid)
        .bind(start)
        .bind(end)
        .fetch_one(&s.pool)
        .await?;
        let collected = col.unwrap_or(Decimal::ZERO);
        let expected = rent;
        let balance = expected - collected;
        let today = Utc::now().date_naive();
        let due_date = NaiveDate::from_ymd_opt(q.year, q.month, due_day.min(28) as u32)
            .unwrap_or(start);
        let late_threshold = due_date + chrono::Duration::days(grace as i64);
        let status = if collected >= expected {
            "paid"
        } else if collected > Decimal::ZERO {
            if today > late_threshold { "late" } else { "partial" }
        } else if today > late_threshold {
            "late"
        } else {
            "due"
        }
        .to_string();
        rows.push(RentRollRow {
            lease_id: lid,
            tenant_name: tname,
            unit_label: unit,
            rent_amount: rent,
            rent_due_day: due_day,
            grace_days: grace,
            expected,
            collected,
            balance,
            status,
        });
    }
    Ok(Json(rows))
}

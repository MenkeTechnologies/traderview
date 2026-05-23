use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
    Short,
    Cover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeSide {
    Long,
    Short,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: Option<String>,
    pub display_name: String,
    pub is_local: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub user_id: Uuid,
    pub broker: String,
    pub name: String,
    pub base_currency: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    pub id: Uuid,
    pub account_id: Uuid,
    pub symbol: String,
    pub side: Side,
    pub qty: Decimal,
    pub price: Decimal,
    pub fee: Decimal,
    pub executed_at: DateTime<Utc>,
    pub broker_order_id: Option<String>,
    #[serde(default)]
    pub raw: serde_json::Value,
    pub import_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub account_id: Uuid,
    pub symbol: String,
    pub side: TradeSide,
    pub status: TradeStatus,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub qty: Decimal,
    pub entry_avg: Decimal,
    pub exit_avg: Option<Decimal>,
    pub gross_pnl: Option<Decimal>,
    pub fees: Decimal,
    pub net_pnl: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub trade_id: Option<Uuid>,
    pub day: Option<NaiveDate>,
    pub body_md: String,
    pub mood: Option<i16>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub id: Uuid,
    pub account_id: Uuid,
    pub source: String,
    pub filename: String,
    pub sha256: String,
    pub row_count: i32,
    pub imported_at: DateTime<Utc>,
}

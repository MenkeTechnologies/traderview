//! Domain types — kept in sync with `migrations/0001..0008`.
//!
//! All money columns use [`rust_decimal::Decimal`] — no floats anywhere.
//! All instants are [`chrono::DateTime<Utc>`] — `TIMESTAMPTZ` columns.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ===========================================================================
// Enums
// ===========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
    Short,
    Cover,
}

impl Side {
    /// `true` when this side increases an open long (`Buy`) or open short (`Short`).
    pub fn opens(self) -> bool {
        matches!(self, Side::Buy | Side::Short)
    }
    pub fn closes(self) -> bool {
        matches!(self, Side::Sell | Side::Cover)
    }
    pub fn is_long_side(self) -> bool {
        matches!(self, Side::Buy | Side::Sell)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeSide {
    Long,
    Short,
}

impl TradeSide {
    pub fn sign(self) -> Decimal {
        match self {
            TradeSide::Long => Decimal::ONE,
            TradeSide::Short => -Decimal::ONE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetClass {
    Stock,
    Option,
    Future,
    Forex,
}

impl Default for AssetClass {
    fn default() -> Self {
        AssetClass::Stock
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OptionType {
    Call,
    Put,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarInterval {
    #[serde(rename = "1m")]
    M1,
    #[serde(rename = "5m")]
    M5,
    #[serde(rename = "15m")]
    M15,
    #[serde(rename = "1h")]
    H1,
    #[serde(rename = "1d")]
    D1,
    #[serde(rename = "1w")]
    W1,
}

impl BarInterval {
    pub fn seconds(self) -> i64 {
        match self {
            BarInterval::M1 => 60,
            BarInterval::M5 => 300,
            BarInterval::M15 => 900,
            BarInterval::H1 => 3600,
            BarInterval::D1 => 86_400,
            BarInterval::W1 => 604_800,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            BarInterval::M1 => "1m",
            BarInterval::M5 => "5m",
            BarInterval::M15 => "15m",
            BarInterval::H1 => "1h",
            BarInterval::D1 => "1d",
            BarInterval::W1 => "1w",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MentorshipStatus {
    Pending,
    Active,
    Revoked,
}

// ===========================================================================
// Users / accounts / settings
// ===========================================================================

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
pub struct UserSettings {
    pub user_id: Uuid,
    pub default_account_id: Option<Uuid>,
    pub base_currency: String,
    pub timezone: String,
    pub theme: String,
    pub starting_cash: Decimal,
    pub dashboard_layout: serde_json::Value,
    #[serde(default)]
    pub commission_per_share: Decimal,
    #[serde(default)]
    pub commission_per_contract: Decimal,
    #[serde(default = "true_default")]
    pub auto_flatten: bool,
    #[serde(default)]
    pub require_account_tag: bool,
    pub updated_at: DateTime<Utc>,
}

fn true_default() -> bool { true }

impl Default for UserSettings {
    fn default() -> Self {
        UserSettings {
            user_id: Uuid::nil(),
            default_account_id: None,
            base_currency: "USD".into(),
            timezone: "America/New_York".into(),
            theme: "cyberpunk".into(),
            starting_cash: Decimal::ZERO,
            dashboard_layout: serde_json::json!({}),
            commission_per_share: Decimal::ZERO,
            commission_per_contract: Decimal::ZERO,
            auto_flatten: true,
            require_account_tag: false,
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteTemplate {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub scope: String, // "trade" | "journal"
    pub body_md: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ===========================================================================
// Executions (atoms) and trades (FIFO-derived materialization)
// ===========================================================================

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
    // multi-asset
    #[serde(default)]
    pub asset_class: AssetClass,
    pub option_type: Option<OptionType>,
    pub strike: Option<Decimal>,
    pub expiration: Option<NaiveDate>,
    #[serde(default = "one_dec")]
    pub multiplier: Decimal,
    pub tick_size: Option<Decimal>,
    pub tick_value: Option<Decimal>,
    pub base_ccy: Option<String>,
    pub quote_ccy: Option<String>,
    pub pip_size: Option<Decimal>,
}

fn one_dec() -> Decimal {
    Decimal::ONE
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
    #[serde(default)]
    pub asset_class: AssetClass,
    pub option_type: Option<OptionType>,
    pub strike: Option<Decimal>,
    pub expiration: Option<NaiveDate>,
    #[serde(default = "one_dec")]
    pub multiplier: Decimal,
    pub tick_size: Option<Decimal>,
    pub tick_value: Option<Decimal>,
    pub base_ccy: Option<String>,
    pub quote_ccy: Option<String>,
    pub pip_size: Option<Decimal>,
    // risk / planning / excursion
    pub stop_loss: Option<Decimal>,
    pub risk_amount: Option<Decimal>,
    pub initial_target: Option<Decimal>,
    pub mfe: Option<Decimal>,
    pub mae: Option<Decimal>,
    pub best_exit_pnl: Option<Decimal>,
    pub exit_efficiency: Option<Decimal>,
}

impl Trade {
    /// R-multiple = net_pnl / risk_amount. None if either is missing.
    pub fn r_multiple(&self) -> Option<Decimal> {
        let r = self.risk_amount?;
        if r.is_zero() {
            return None;
        }
        Some(self.net_pnl? / r)
    }

    /// Hold time in seconds. None if the trade is still open.
    pub fn hold_seconds(&self) -> Option<i64> {
        Some((self.closed_at? - self.opened_at).num_seconds())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExecutionLink {
    pub trade_id: Uuid,
    pub execution_id: Uuid,
    pub qty_used: Decimal,
}

// ===========================================================================
// Tags / journal / screenshots
// ===========================================================================

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
pub struct Screenshot {
    pub id: Uuid,
    pub user_id: Uuid,
    pub trade_id: Option<Uuid>,
    pub journal_id: Option<Uuid>,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub caption: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}

// ===========================================================================
// Imports
// ===========================================================================

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

// ===========================================================================
// Mentorship + sharing + comments
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mentorship {
    pub id: Uuid,
    pub mentor_id: Uuid,
    pub mentee_id: Uuid,
    pub status: MentorshipStatus,
    pub scope: String,
    pub created_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeShare {
    pub id: Uuid,
    pub trade_id: Uuid,
    pub owner_id: Uuid,
    pub slug: String,
    pub is_public: bool,
    pub show_notes: bool,
    pub show_screenshots: bool,
    pub view_count: i64,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub share_id: Uuid,
    pub author_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub body_md: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ===========================================================================
// Forum
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumCategory {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub position: i32,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumThread {
    pub id: Uuid,
    pub category_id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub slug: String,
    pub is_pinned: bool,
    pub is_locked: bool,
    pub view_count: i64,
    pub post_count: i32,
    pub last_post_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumPost {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub author_id: Uuid,
    pub body_md: String,
    pub is_op: bool,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolActivity {
    pub symbol: String,
    pub user_id: Uuid,
    pub trade_count: i32,
    pub last_trade_at: DateTime<Utc>,
}

// ===========================================================================
// Price bars + symbol metadata
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceBar {
    pub symbol: String,
    pub interval: BarInterval,
    pub bar_time: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolMeta {
    pub symbol: String,
    pub name: Option<String>,
    pub exchange: Option<String>,
    pub asset_class: AssetClass,
    pub currency: String,
    pub multiplier: Decimal,
    pub tick_size: Option<Decimal>,
    pub tick_value: Option<Decimal>,
    pub last_refreshed: DateTime<Utc>,
}

// ===========================================================================
// Trade plans + filter sets + FX
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradePlan {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub symbol: String,
    pub asset_class: AssetClass,
    pub side: TradeSide,
    pub intended_qty: Decimal,
    pub intended_entry: Decimal,
    pub stop_loss: Option<Decimal>,
    pub initial_target: Option<Decimal>,
    pub setup_notes: String,
    pub plan_status: String,
    pub linked_trade_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub filled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterSet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub payload: serde_json::Value,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FxRate {
    pub base: String,
    pub quote: String,
    pub day: NaiveDate,
    pub rate: Decimal,
    pub source: String,
}

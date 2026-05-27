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
#[derive(Default)]
pub enum AssetClass {
    #[default]
    Stock,
    Option,
    Future,
    Forex,
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
    #[serde(default)]
    pub daily_profit_goal: Decimal,
    #[serde(default)]
    pub daily_max_loss: Decimal,
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
            daily_profit_goal: Decimal::ZERO,
            daily_max_loss: Decimal::ZERO,
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // ─── Side enum methods ────────────────────────────────────────────────

    #[test]
    fn side_opens_only_for_buy_and_short() {
        assert!(Side::Buy.opens());
        assert!(Side::Short.opens());
        assert!(!Side::Sell.opens());
        assert!(!Side::Cover.opens());
    }

    #[test]
    fn side_closes_only_for_sell_and_cover() {
        assert!(Side::Sell.closes());
        assert!(Side::Cover.closes());
        assert!(!Side::Buy.closes());
        assert!(!Side::Short.closes());
    }

    #[test]
    fn side_is_long_side_only_for_buy_and_sell() {
        assert!(Side::Buy.is_long_side());
        assert!(Side::Sell.is_long_side());
        assert!(!Side::Short.is_long_side());
        assert!(!Side::Cover.is_long_side());
    }

    #[test]
    fn side_opens_and_closes_are_mutually_exclusive() {
        for s in [Side::Buy, Side::Sell, Side::Short, Side::Cover] {
            assert!(s.opens() ^ s.closes(),
                "{:?}: every side must be either open or close, never both", s);
        }
    }

    #[test]
    fn side_serde_lowercase_roundtrip() {
        // Side must serialize as the lowercase string the DB enum expects.
        for (variant, expected) in [
            (Side::Buy,   r#""buy""#),
            (Side::Sell,  r#""sell""#),
            (Side::Short, r#""short""#),
            (Side::Cover, r#""cover""#),
        ] {
            assert_eq!(serde_json::to_string(&variant).unwrap(), expected);
            let back: Side = serde_json::from_str(expected).unwrap();
            assert_eq!(back, variant);
        }
    }

    // ─── TradeSide ────────────────────────────────────────────────────────

    #[test]
    fn trade_side_sign_is_signed_one() {
        assert_eq!(TradeSide::Long.sign(), Decimal::ONE);
        assert_eq!(TradeSide::Short.sign(), -Decimal::ONE);
    }

    // ─── AssetClass default ───────────────────────────────────────────────

    #[test]
    fn asset_class_defaults_to_stock() {
        // Trade rows created from sources that don't carry asset_class
        // (older imports, basic CSV) must land as Stock.
        let a: AssetClass = Default::default();
        assert_eq!(a, AssetClass::Stock);
    }

    // ─── BarInterval ──────────────────────────────────────────────────────

    #[test]
    fn bar_interval_seconds_match_real_durations() {
        assert_eq!(BarInterval::M1.seconds(),  60);
        assert_eq!(BarInterval::M5.seconds(),  300);
        assert_eq!(BarInterval::M15.seconds(), 900);
        assert_eq!(BarInterval::H1.seconds(),  3_600);
        assert_eq!(BarInterval::D1.seconds(),  86_400);
        assert_eq!(BarInterval::W1.seconds(),  604_800);
    }

    #[test]
    fn bar_interval_serde_uses_short_label() {
        let one_min: BarInterval = serde_json::from_str(r#""1m""#).unwrap();
        assert_eq!(one_min, BarInterval::M1);
        assert_eq!(serde_json::to_string(&BarInterval::D1).unwrap(), r#""1d""#);
    }

    // ─── Trade::r_multiple / hold_seconds ─────────────────────────────────

    fn closed_trade(net_pnl: Option<Decimal>, risk_amount: Option<Decimal>) -> Trade {
        Trade {
            id: Uuid::nil(),
            account_id: Uuid::nil(),
            symbol: "TEST".into(),
            side: TradeSide::Long,
            status: TradeStatus::Closed,
            opened_at: Utc.with_ymd_and_hms(2026, 1, 1, 9, 30, 0).unwrap(),
            closed_at: Some(Utc.with_ymd_and_hms(2026, 1, 1, 15, 30, 0).unwrap()),
            qty: Decimal::from(100),
            entry_avg: Decimal::from(50),
            exit_avg: Some(Decimal::from(52)),
            gross_pnl: net_pnl,
            fees: Decimal::ZERO,
            net_pnl,
            asset_class: AssetClass::Stock,
            option_type: None, strike: None, expiration: None,
            multiplier: Decimal::ONE,
            tick_size: None, tick_value: None,
            base_ccy: None, quote_ccy: None, pip_size: None,
            stop_loss: None, risk_amount, initial_target: None,
            mfe: None, mae: None, best_exit_pnl: None, exit_efficiency: None,
        }
    }

    #[test]
    fn r_multiple_returns_none_when_risk_amount_missing() {
        let t = closed_trade(Some(Decimal::from(200)), None);
        assert!(t.r_multiple().is_none());
    }

    #[test]
    fn r_multiple_returns_none_when_risk_amount_zero() {
        // Divide-by-zero guard — must not panic.
        let t = closed_trade(Some(Decimal::from(200)), Some(Decimal::ZERO));
        assert!(t.r_multiple().is_none());
    }

    #[test]
    fn r_multiple_computes_pnl_over_risk() {
        // $300 P&L on $100 R = 3R.
        let t = closed_trade(Some(Decimal::from(300)), Some(Decimal::from(100)));
        assert_eq!(t.r_multiple(), Some(Decimal::from(3)));
    }

    #[test]
    fn r_multiple_handles_negative_pnl() {
        // -$150 on $100 R = -1.5R.
        let t = closed_trade(Some(Decimal::from(-150)), Some(Decimal::from(100)));
        assert_eq!(t.r_multiple(), Some(Decimal::new(-15, 1)));
    }

    #[test]
    fn r_multiple_returns_none_when_net_pnl_missing() {
        let t = closed_trade(None, Some(Decimal::from(100)));
        assert!(t.r_multiple().is_none());
    }

    #[test]
    fn hold_seconds_returns_difference_between_open_and_close() {
        let t = closed_trade(Some(Decimal::from(100)), None);
        // 09:30 → 15:30 = 6 hours = 21600 seconds.
        assert_eq!(t.hold_seconds(), Some(21_600));
    }

    #[test]
    fn hold_seconds_returns_none_when_open() {
        let mut t = closed_trade(Some(Decimal::from(100)), None);
        t.closed_at = None;
        t.status = TradeStatus::Open;
        assert!(t.hold_seconds().is_none());
    }
}

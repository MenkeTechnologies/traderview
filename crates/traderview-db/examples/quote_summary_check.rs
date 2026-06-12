//! One-shot: fetch a Yahoo quoteSummary module through the cookie+crumb
//! auth path. Companion diagnostic to `options_check` for the 401
//! "Invalid Crumb" failure mode.
//!
//! Run:
//!   cargo run -p traderview-db --example quote_summary_check -- AAPL defaultKeyStatistics

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let symbol = std::env::args().nth(1).unwrap_or_else(|| "AAPL".into());
    let module = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "defaultKeyStatistics".into());
    let v = traderview_db::market_data::quote_summary(&symbol, &[&module]).await?;
    let keys: Vec<&str> = v[&module]
        .as_object()
        .map(|m| m.keys().map(String::as_str).collect())
        .unwrap_or_default();
    println!("{symbol} {module}: {} fields ({})", keys.len(), keys.join(", "));
    Ok(())
}

//! One-shot: run the market gamma regime snapshot pipeline (SPY chain
//! fetch → per-strike GEX → regime classification) and print the result.
//! Diagnostic for the "MARKET GAMMA REGIME · no data" failure mode.
//!
//! Run:
//!   cargo run -p traderview-db --example gamma_check

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match traderview_db::market_gamma_regime::fetch_snapshot().await {
        Some(s) => println!(
            "ok: total_gex=${:.0}M spot={} regime={} expirations={:?}",
            s.total_gex_usd / 1e6,
            s.spot,
            s.regime.as_str(),
            s.expirations_used,
        ),
        None => println!("fetch_snapshot returned None"),
    }
    Ok(())
}

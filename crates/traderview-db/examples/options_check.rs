//! One-shot: fetch a Yahoo options chain through the cookie+crumb auth
//! path and print what came back. Diagnostic for the 401 "Invalid
//! Crumb" failure mode that silently emptied every chain-based view.
//!
//! Run:
//!   cargo run -p traderview-db --example options_check -- SPY

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let symbol = std::env::args().nth(1).unwrap_or_else(|| "SPY".into());
    let chain = traderview_db::options::chain(&symbol, None).await?;
    println!(
        "{}: spot={} expirations={} calls={} puts={} (nearest exp {})",
        chain.symbol,
        chain.spot,
        chain.expirations.len(),
        chain.calls.len(),
        chain.puts.len(),
        chain.expiration,
    );
    Ok(())
}

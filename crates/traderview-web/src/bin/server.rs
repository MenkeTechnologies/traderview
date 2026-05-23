//! Standalone web server. External Postgres, multi-user.
//!
//! Env vars:
//!   DATABASE_URL              postgres://user:pass@host/db   (required)
//!   TRADERVIEW_JWT_SECRET     hex-encoded, >= 32 bytes       (required)
//!   TRADERVIEW_BIND           default 0.0.0.0:8080
//!   TRADERVIEW_STATIC_DIR     default ../../frontend         (relative to bin)

use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::services::ServeDir;
use traderview_web::{router, AppMode, AppState};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    #[arg(long, env = "TRADERVIEW_JWT_SECRET")]
    jwt_secret: String,

    #[arg(long, env = "TRADERVIEW_BIND", default_value = "0.0.0.0:8080")]
    bind: SocketAddr,

    #[arg(long, env = "TRADERVIEW_STATIC_DIR", default_value = "frontend")]
    static_dir: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "traderview_web=info,tower_http=info".into()),
        )
        .init();

    let args = Args::parse();
    let jwt_secret = hex::decode(&args.jwt_secret)
        .map_err(|e| anyhow::anyhow!("TRADERVIEW_JWT_SECRET must be hex: {e}"))?;
    if jwt_secret.len() < 32 {
        anyhow::bail!("TRADERVIEW_JWT_SECRET must decode to >= 32 bytes");
    }

    let pool = traderview_db::connect_external(&args.database_url).await?;
    traderview_db::migrate(&pool).await?;

    let state = AppState::new(pool, AppMode::Web, jwt_secret);
    let api = router(state);

    let static_service = ServeDir::new(&args.static_dir).append_index_html_on_directories(true);
    let app = api.fallback_service(static_service);

    tracing::info!(%args.bind, static_dir = %args.static_dir.display(), "traderview-web listening");
    let listener = tokio::net::TcpListener::bind(args.bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

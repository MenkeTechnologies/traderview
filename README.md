# traderview

TraderVue-style trading journal. Rust workspace, Postgres backend, vanilla JS + uPlot frontend.
Ships as a Tauri v2 desktop app (with embedded Postgres) **and** as a multi-user axum web service
from the same crates.

Created by [MenkeTechnologies](https://github.com/MenkeTechnologies).

## Architecture

```
                    ┌──────────────────────────────────┐
                    │  frontend/  (vanilla JS + uPlot) │
                    └──────────────┬───────────────────┘
                                   │ HTTP /api/*
                  ┌────────────────┴────────────────┐
                  │                                 │
        ┌─────────▼──────────┐         ┌────────────▼─────────────┐
        │  src-tauri         │         │  traderview-web (axum)   │
        │  desktop wrapper   │         │  multi-user web server   │
        │  + embedded PG     │         │  + external Postgres     │
        └─────────┬──────────┘         └────────────┬─────────────┘
                  │                                 │
                  └────────────────┬────────────────┘
                                   │
                  ┌────────────────▼────────────────┐
                  │  traderview-{core,db,import}    │
                  │  (shared library crates)        │
                  └─────────────────────────────────┘
```

### Crates

| Crate                 | Purpose                                                              |
|-----------------------|----------------------------------------------------------------------|
| `traderview-core`     | Domain models, FIFO trade roll-up, statistics                        |
| `traderview-db`       | `sqlx` pool factory, migrations, embedded-Postgres lifecycle         |
| `traderview-import`   | Broker file parsers (Webull first, mapping wizard later)             |
| `traderview-web`      | axum router + JWT auth + `server` binary for web deploy              |
| `src-tauri`           | Tauri v2 shell that spawns embedded Postgres + axum on localhost     |

## Running

### Desktop (Tauri, embedded Postgres)

```sh
# First-time vendor uPlot into frontend/lib/
./scripts/vendor-uplot.sh

# Dev (downloads postgresql binary on first launch, ~80MB, cached in ~/.theseus)
cargo tauri dev
```

The desktop app:
- Downloads + extracts a portable PostgreSQL on first launch (via `postgresql_embedded`).
- Stores data under `$APP_DATA_DIR/traderview/pg/`.
- Auto-creates a local user and auto-logs in.
- Starts axum on a random localhost port; the WebView talks to it via `fetch`.

### Web (external Postgres, multi-user)

```sh
docker compose up -d postgres
export DATABASE_URL=postgres://traderview:traderview@localhost:5432/traderview
export TRADERVIEW_JWT_SECRET=$(openssl rand -hex 32)
cargo run -p traderview-web --bin server
```

Open <http://localhost:8080>. Register, then log in.

## Importing trades

Webull `Account Statement → Orders` CSV. Parser awaits a redacted real sample —
**do not** infer columns from documentation. Drop the file in via the UI; the
importer dedupes by `(broker_order_id, executed_at)`.

## Status

| Phase | Item                                              | Status |
|-------|---------------------------------------------------|--------|
| 1     | Workspace + crate scaffold                        | done   |
| 2     | Webull importer (needs real sample)               | todo   |
| 3     | FIFO trade roll-up                                | todo   |
| 4     | Trades UI + filters + tags                        | todo   |
| 5     | Equity curve + summary stats (uPlot)              | todo   |
| 6     | Journal (markdown, per-trade + per-day)           | todo   |

## License

MIT

# traderview

TraderVue-style trading journal **plus** Schedule C business-expense tracker. Rust workspace,
Postgres backend, vanilla JS + uPlot frontend. Ships as a Tauri v2 desktop app (with embedded
Postgres) **and** as a multi-user axum web service from the same crates.

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

| Crate                  | Purpose                                                              |
|------------------------|----------------------------------------------------------------------|
| `traderview-core`      | Domain models, FIFO trade roll-up, statistics                        |
| `traderview-db`        | `sqlx` pool factory, migrations, embedded-Postgres lifecycle         |
| `traderview-import`    | Broker file parsers (Webull first, mapping wizard later)             |
| `traderview-expense`   | Business-expense CSV parsers, merchant→category rules, transfer dedup |
| `traderview-ocr`       | Pure-Rust receipt OCR (PaddleOCR via `tract-onnx`) + PDF text extract |
| `traderview-web`       | axum router + JWT auth + `server` binary for web deploy              |
| `src-tauri`            | Tauri v2 shell that spawns embedded Postgres + axum on localhost     |

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

## Business expenses (Schedule C)

The Expenses tab is a separate flow from trade ingestion. It tracks
business-deductible spending so the year-end Schedule C is one click away.

### What goes in

| Source            | Formats supported  | Account kind   | Notes                                                |
|-------------------|--------------------|----------------|------------------------------------------------------|
| Amazon orders     | CSV, XLSX          | marketplace    | Header-less position schema (23 cols). Total at col 7. |
| Bank of America   | CSV, XLSX          | bank           | Two-section export; parser skips the summary block.    |
| Chase             | CSV                | credit_card    | Header-based; respects Chase's signed `Amount`.        |
| Apple Card        | PDF                | credit_card    | Born-digital monthly statement; PDF text layer.        |

Format detection is automatic — drop in whatever the export tool gave you.
ZIP-magic bytes route through `calamine` (xlsx/ods/xls), `%PDF` routes
through `lopdf`, everything else is treated as CSV. No format flag needed.

### Categorization

Transactions are tagged with one of 23 IRS Schedule C lines (8 through 27a:
Advertising, Car & truck, Supplies, Travel, Meals, Utilities, etc.). The
merchant→category mapping is a learned rule table — first time you see
`STAPLES.COM` you tag it `office`, every future row from that merchant
auto-categorizes. Default seed covers ~70 common US merchants (AWS, Adobe,
Uber/Lyft, Chevron/Shell, Starbucks, etc.).

The `meals_50` category has a 0.5 deduction percentage baked in so the
year-end report applies the IRS 50% rule without you doing math.

### Transfer dedup

When a credit-card payment shows up in **both** your bank statement (money
out) and the credit-card statement (money in), the importer detects the pair
by amount + date proximity + account kind and marks both as
`is_transfer = true`. Schedule C report excludes them.

### Receipts (image / PDF)

Drop a JPG, PNG, WebP, or PDF onto the Expenses tab. OCR runs in the
background using pure-Rust PaddleOCR (DBNet + SVTR via `tract-onnx`) — no
Tesseract, no system libraries, no C dependencies. PDF receipts use the
text-layer fast path via `lopdf`; scanned PDFs prompt you to re-upload as an
image.

OCR models are not bundled in the repo (size). On first OCR call, drop
PaddleOCR English mobile model files into
`$APP_DATA_DIR/traderview/models/paddleocr/` as:

| File             | Source model                                      |
|------------------|---------------------------------------------------|
| `det.onnx`       | DBNet text detection (e.g. `en_PP-OCRv4_det`)     |
| `rec.onnx`       | SVTR text recognition (e.g. `en_PP-OCRv4_rec`)    |
| `line_ori.onnx`  | Text-line orientation classifier (per-line skew)  |
| `doc_ori.onnx`   | Document orientation classifier (page rotation)   |
| `dict.txt`       | Character dictionary for the recognition model    |

The two orientation models matter for phone-camera receipts — PaddleOCR's
pipeline corrects line and page rotation natively, so you don't have to
hold the camera straight to get a clean OCR.

To compile in the OCR engine itself, build with the `ocr-engine` feature.
The flag propagates from the binary down through `traderview-web` to
`traderview-ocr`:

```sh
# Web server with OCR enabled
cargo run -p traderview-web --features ocr-engine --bin server

# Desktop app with OCR enabled
cargo tauri dev --features ocr-engine
```

Heavy first-time compile (~5 min cold) because `tract-onnx` + `ndarray`
get pulled in — leave the flag off until you actually need receipt OCR.
PDF text-layer extraction (born-digital receipts and Apple Card
statements) works without the flag.

After OCR, the receipt is matched against your last week of transactions
using amount + date + merchant-token Jaccard scoring. You confirm the best
match and the receipt is permanently attached to the transaction row.

### Schedule C report

`GET /api/expense/report/schedule_c?year=2026` (or the UI button) returns
per-line totals for the calendar year, applying each category's deduction
percentage. The report also surfaces:

- uncategorized business expenses that aren't rolled in yet
- excluded transfers + excluded personal rows (transparency on what was
  filtered out)

## Status

| Phase | Item                                              | Status |
|-------|---------------------------------------------------|--------|
| 1     | Workspace + crate scaffold                        | done   |
| 2     | Webull importer (needs real sample)               | todo   |
| 3     | FIFO trade roll-up                                | todo   |
| 4     | Trades UI + filters + tags                        | todo   |
| 5     | Equity curve + summary stats (uPlot)              | todo   |
| 6     | Journal (markdown, per-trade + per-day)           | todo   |
| 7     | Expense schema + Schedule C categories            | done   |
| 8     | Expense parsers (Amazon/BoA/Chase/Apple)          | done   |
| 9     | Merchant→category rule engine + seed              | done   |
| 10    | Cross-account transfer dedup                      | done   |
| 11    | Receipt upload + pure-Rust OCR + match            | done   |
| 12    | Schedule C report (year, per-line, meals 50%)     | done   |
| 13    | XLSX / ODS / PDF format support (calamine, lopdf) | done   |
| 14    | Bundle/lazy-download PaddleOCR model files        | todo   |
| 15    | Apple Card CSV path (Wallet export)               | todo   |

## License

MIT

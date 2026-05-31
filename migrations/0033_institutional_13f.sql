-- traderview SEC Form 13F institutional holdings surface.
--
-- 13F-HR: quarterly long position report filed by institutional managers
-- with $100M+ AUM (Section 13(f) of the Securities Exchange Act). Public
-- via SEC EDGAR — CIK + accession number uniquely identifies a filing,
-- each filing carries an `informationTable` XML with one row per holding.
-- The 45-day filing window means data is stale by ~6 weeks but is the
-- canonical "smart money" signal that QuiverQuant / WhaleWisdom /
-- 13F.info expose for $30+/month each.
--
-- This migration is purely additive: no ALTER on existing tables. We
-- model managers, filings, and individual holdings, then derive
-- position-change deltas via a SQL view that joins the latest two
-- filings per manager.

-- ---------------------------------------------------------------------------
-- institutional_managers — one row per CIK (Central Index Key)
-- ---------------------------------------------------------------------------
-- CIK is the SEC's stable 10-digit identifier (e.g. Berkshire Hathaway =
-- 0001067983). Manager type lets the UI segment hedge fund vs RIA vs
-- pension vs sovereign-wealth-fund.
CREATE TYPE manager_type_t AS ENUM (
    'hedge_fund', 'rita', 'pension', 'sovereign', 'insurance', 'bank', 'other'
);

CREATE TABLE institutional_managers (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cik             TEXT NOT NULL UNIQUE,                       -- '0001067983'
    name            TEXT NOT NULL,                              -- 'BERKSHIRE HATHAWAY INC'
    manager_type    manager_type_t NOT NULL DEFAULT 'other',
    state           TEXT,                                       -- HQ state per CRS Form ADV
    aliases         TEXT[] NOT NULL DEFAULT '{}',               -- 'Berkshire', 'BRK', etc.
    notable         BOOLEAN NOT NULL DEFAULT FALSE,             -- featured-list flag for UI
    first_seen_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX institutional_managers_name_idx     ON institutional_managers(name);
CREATE INDEX institutional_managers_notable_idx  ON institutional_managers(notable) WHERE notable = TRUE;

-- ---------------------------------------------------------------------------
-- institutional_13f_filings — one row per accession (quarterly)
-- ---------------------------------------------------------------------------
-- `accession_number` is EDGAR's filing primary key. `quarter_end` is the
-- calendar quarter the holdings reflect (NOT when it was filed).
CREATE TABLE institutional_13f_filings (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    manager_id          UUID NOT NULL REFERENCES institutional_managers(id) ON DELETE CASCADE,
    accession_number    TEXT NOT NULL,                          -- '0001067983-24-000010'
    form_type           TEXT NOT NULL DEFAULT '13F-HR',         -- 13F-HR / 13F-HR/A (amendment)
    quarter_end         DATE NOT NULL,                          -- '2024-09-30'
    filed_at            TIMESTAMPTZ NOT NULL,
    detected_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    total_value_usd     NUMERIC(28, 2),                         -- sum of holding values_usd
    holdings_count      INTEGER NOT NULL DEFAULT 0,
    source_url          TEXT,
    raw_meta            JSONB NOT NULL DEFAULT '{}'::jsonb,
    UNIQUE (manager_id, accession_number)
);
CREATE INDEX i13f_filings_manager_idx ON institutional_13f_filings(manager_id, quarter_end DESC);
CREATE INDEX i13f_filings_quarter_idx ON institutional_13f_filings(quarter_end DESC);
CREATE INDEX i13f_filings_filed_idx   ON institutional_13f_filings(filed_at DESC);

-- ---------------------------------------------------------------------------
-- institutional_holdings — one row per position in a filing
-- ---------------------------------------------------------------------------
-- CUSIP is the 9-character identifier on the 13F XML. Symbol is filled
-- post-ingest via CUSIP -> ticker lookup. `put_call` distinguishes
-- the (rare) option positions: 'PUT' / 'CALL' / NULL for common.
CREATE TABLE institutional_holdings (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    filing_id           UUID NOT NULL REFERENCES institutional_13f_filings(id) ON DELETE CASCADE,
    cusip               TEXT NOT NULL,                          -- '037833100' (AAPL)
    symbol              TEXT,                                   -- 'AAPL' after CUSIP lookup
    issuer_name         TEXT NOT NULL,                          -- 'APPLE INC'
    issuer_class        TEXT,                                   -- 'COM' / 'PUT' / 'CALL' / 'ADR'
    put_call            TEXT,                                   -- 'PUT' / 'CALL' / NULL
    shares              NUMERIC(28, 4) NOT NULL,
    value_usd           NUMERIC(28, 2) NOT NULL,
    sole_voting         NUMERIC(28, 4),
    shared_voting       NUMERIC(28, 4),
    none_voting         NUMERIC(28, 4)
);
-- Unique on (filing, cusip, put_call) — common-stock vs put vs call vs null
-- coexist on the same CUSIP. Expressions in UNIQUE constraints require
-- a unique INDEX, not a table-level constraint.
CREATE UNIQUE INDEX inst_holdings_dedupe_idx
    ON institutional_holdings(filing_id, cusip, COALESCE(put_call, ''));
CREATE INDEX inst_holdings_symbol_idx  ON institutional_holdings(symbol);
CREATE INDEX inst_holdings_cusip_idx   ON institutional_holdings(cusip);
CREATE INDEX inst_holdings_filing_idx  ON institutional_holdings(filing_id);
-- Top-N queries over the whole table: largest positions across all filings.
CREATE INDEX inst_holdings_value_idx   ON institutional_holdings(value_usd DESC);

-- ---------------------------------------------------------------------------
-- View: latest-filing per manager
-- ---------------------------------------------------------------------------
-- Lets the UI ask "what does this manager own RIGHT NOW (most recent
-- filing)" without a window function in every query.
CREATE VIEW institutional_latest_filings AS
    SELECT DISTINCT ON (manager_id)
        id            AS filing_id,
        manager_id,
        accession_number,
        form_type,
        quarter_end,
        filed_at,
        total_value_usd,
        holdings_count
      FROM institutional_13f_filings
     ORDER BY manager_id, quarter_end DESC, filed_at DESC;

-- ---------------------------------------------------------------------------
-- View: 13F position changes (latest vs prior quarter per manager+CUSIP)
-- ---------------------------------------------------------------------------
-- Materialized lazily by the API endpoint, but the view shape is stable
-- and indexed enough for ad-hoc joins.
-- delta_shares > 0 = added, < 0 = sold, = 0 = held (rare for institutions
-- with rebalancing).
CREATE VIEW institutional_position_changes AS
    WITH ranked AS (
        SELECT
            f.manager_id,
            f.id          AS filing_id,
            f.quarter_end,
            h.cusip,
            h.symbol,
            h.issuer_name,
            h.shares,
            h.value_usd,
            ROW_NUMBER() OVER (
                PARTITION BY f.manager_id, h.cusip, COALESCE(h.put_call, '')
                ORDER BY f.quarter_end DESC, f.filed_at DESC
            ) AS rn
          FROM institutional_13f_filings f
          JOIN institutional_holdings h ON h.filing_id = f.id
    )
    SELECT
        cur.manager_id,
        cur.cusip,
        cur.symbol,
        cur.issuer_name,
        cur.quarter_end                            AS current_quarter,
        prv.quarter_end                            AS prior_quarter,
        cur.shares                                 AS shares_now,
        COALESCE(prv.shares, 0)                    AS shares_prior,
        cur.value_usd                              AS value_now,
        COALESCE(prv.value_usd, 0)                 AS value_prior,
        (cur.shares - COALESCE(prv.shares, 0))     AS delta_shares,
        (cur.value_usd - COALESCE(prv.value_usd, 0)) AS delta_value,
        CASE
            WHEN prv.shares IS NULL                  THEN 'new'
            WHEN cur.shares > prv.shares             THEN 'increased'
            WHEN cur.shares < prv.shares             THEN 'decreased'
            ELSE                                          'held'
        END                                        AS change_type
      FROM ranked cur
      LEFT JOIN ranked prv
        ON prv.manager_id = cur.manager_id
       AND prv.cusip      = cur.cusip
       AND prv.rn         = cur.rn + 1
     WHERE cur.rn = 1;

-- ---------------------------------------------------------------------------
-- Rollback (manual):
--   DROP VIEW institutional_position_changes;
--   DROP VIEW institutional_latest_filings;
--   DROP TABLE institutional_holdings, institutional_13f_filings,
--              institutional_managers CASCADE;
--   DROP TYPE manager_type_t;
-- ---------------------------------------------------------------------------

-- 0013 — sub-30-second insider + congressional disclosure pipeline.
--
-- SEC EDGAR Form 4 atom feed:
--   https://www.sec.gov/cgi-bin/browse-edgar?action=getcurrent&type=4&output=atom
-- Senate STOCK Act:
--   https://efdsearch.senate.gov/search/  (HTML — scraped, no clean API)
-- House STOCK Act:
--   https://disclosures-clerk.house.gov/PublicDisclosure/FinancialDisclosure
--
-- We poll every 20s. Each new filing fires user-configured push subscriptions
-- whose watcher rules (symbol / person / threshold) match.

CREATE TYPE disclosure_kind_t AS ENUM ('insider_form4', 'senate_stock', 'house_stock');

CREATE TABLE disclosures (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kind            disclosure_kind_t NOT NULL,
    -- External primary key (EDGAR accession, Senate filing id, House doc id).
    external_id     TEXT NOT NULL,
    symbol          TEXT,                                  -- ticker if resolved
    -- Person filing
    filer_name      TEXT NOT NULL,
    filer_role      TEXT,                                  -- 'CEO', 'Director', 'Sen.', 'Rep.', etc.
    -- Transaction
    txn_type        TEXT,                                  -- 'P' (purchase) / 'S' (sale) / 'A' (award) for Form 4
    shares          NUMERIC(28, 8),
    price           NUMERIC(20, 8),
    amount_usd      NUMERIC(28, 2),                        -- shares*price OR Congress reported amount range midpoint
    amount_range    TEXT,                                  -- '$15001 - $50000' for Congress
    txn_date        DATE,
    filed_at        TIMESTAMPTZ NOT NULL,                  -- when the form hit the wire
    detected_at     TIMESTAMPTZ NOT NULL DEFAULT now(),    -- when we ingested it
    source_url      TEXT,
    raw             JSONB NOT NULL DEFAULT '{}'::jsonb,
    UNIQUE (kind, external_id)
);
CREATE INDEX disclosures_kind_filed_idx   ON disclosures(kind, filed_at DESC);
CREATE INDEX disclosures_symbol_filed_idx ON disclosures(symbol, filed_at DESC);
CREATE INDEX disclosures_filer_idx        ON disclosures(filer_name, filed_at DESC);
CREATE INDEX disclosures_detected_idx     ON disclosures(detected_at DESC);

-- ---------------------------------------------------------------------------
-- User-configured watchers — fire when a disclosure matches.
-- ---------------------------------------------------------------------------
CREATE TABLE disclosure_watchers (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    kinds           TEXT[] NOT NULL DEFAULT ARRAY['insider_form4','senate_stock','house_stock'],
    symbols         TEXT[],                                -- match any of these (null = any)
    filers          TEXT[],                                -- match any of these (null = any)
    min_amount_usd  NUMERIC(28, 2),                        -- filter by transaction size
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    sound           TEXT NOT NULL DEFAULT 'bell',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX disclosure_watchers_user_idx ON disclosure_watchers(user_id, enabled);

-- ---------------------------------------------------------------------------
-- Browser push subscriptions (RFC 8030 / VAPID).
-- ---------------------------------------------------------------------------
CREATE TABLE push_subscriptions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    endpoint        TEXT NOT NULL,
    p256dh          TEXT NOT NULL,                         -- VAPID public key for this subscription
    auth            TEXT NOT NULL,
    ua              TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, endpoint)
);

-- ---------------------------------------------------------------------------
-- Audit: which watchers fired for which disclosures (for de-dup).
-- ---------------------------------------------------------------------------
CREATE TABLE watcher_firings (
    watcher_id      UUID NOT NULL REFERENCES disclosure_watchers(id) ON DELETE CASCADE,
    disclosure_id   UUID NOT NULL REFERENCES disclosures(id) ON DELETE CASCADE,
    fired_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (watcher_id, disclosure_id)
);

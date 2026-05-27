-- 0021 — Sentiment-tagged news history with full-text search.
--
-- Each headline polled from Yahoo lands here, deduplicated by Yahoo uuid
-- (most rows have one) plus a (symbol, title) fallback for items missing
-- uuid. `sentiment` is a normalized -1..+1 score from the same lexicon
-- used by the WSB/StockTwits sentiment module so news contributes to the
-- same axis.
--
-- search_tsv = title || publisher; English unaccent + stemming via the
-- 'english' regconfig (no pg_trgm needed for headline-length text).

CREATE TABLE news_items (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol          TEXT NOT NULL,
    uuid            TEXT,                              -- Yahoo's uuid; nullable
    title           TEXT NOT NULL,
    publisher       TEXT,
    link            TEXT,
    thumbnail       TEXT,
    sentiment       REAL,                              -- -1..+1
    published_at    TIMESTAMPTZ,
    fetched_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    search_tsv      tsvector GENERATED ALWAYS AS (
        setweight(to_tsvector('english', coalesce(title, '')),    'A') ||
        setweight(to_tsvector('english', coalesce(publisher, '')), 'B') ||
        setweight(to_tsvector('english', coalesce(symbol, '')),    'C')
    ) STORED
);

CREATE INDEX news_items_search_idx   ON news_items USING gin(search_tsv);
CREATE INDEX news_items_symbol_idx   ON news_items(symbol, published_at DESC);
CREATE INDEX news_items_published_idx ON news_items(published_at DESC);

-- Dedupe: Yahoo uuid wins when present; otherwise (symbol, title) within
-- a 24h window — same headline crossposted to multiple symbols is allowed.
CREATE UNIQUE INDEX news_items_uuid_uq ON news_items(uuid) WHERE uuid IS NOT NULL;
CREATE UNIQUE INDEX news_items_symtitle_uq
    ON news_items(symbol, title) WHERE uuid IS NULL;

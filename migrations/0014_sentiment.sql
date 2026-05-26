-- 0014 — Sentiment-as-a-feed: per-mention rows + per-window aggregates.
--
-- Sources: 'wsb' (Reddit r/wallstreetbets), 'stocktwits', 'x' (Twitter/X — auth-gated).
-- Sentiment is a Decimal in [-1.0, +1.0], from the lexicon scorer in
-- traderview-core::sentiment.

CREATE TYPE sentiment_source_t AS ENUM ('wsb', 'stocktwits', 'x');

CREATE TABLE mentions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source          sentiment_source_t NOT NULL,
    external_id     TEXT NOT NULL,
    symbol          TEXT NOT NULL,
    sentiment       NUMERIC(6, 4) NOT NULL,   -- [-1.0, +1.0]
    snippet         TEXT NOT NULL,            -- first ~280 chars of the post
    author          TEXT,
    url             TEXT,
    posted_at       TIMESTAMPTZ NOT NULL,
    fetched_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (source, external_id, symbol)
);
CREATE INDEX mentions_symbol_posted_idx ON mentions(symbol, posted_at DESC);
CREATE INDEX mentions_source_posted_idx ON mentions(source, posted_at DESC);
CREATE INDEX mentions_posted_idx        ON mentions(posted_at DESC);

-- Hourly rollup, kept hot for the "ranked-by-delta" query.
CREATE TABLE sentiment_snapshots (
    symbol          TEXT NOT NULL,
    bucket_hour     TIMESTAMPTZ NOT NULL,     -- floor(posted_at, '1 hour')
    source          sentiment_source_t NOT NULL,
    mention_count   INTEGER NOT NULL,
    avg_sentiment   NUMERIC(6, 4) NOT NULL,
    PRIMARY KEY (symbol, bucket_hour, source)
);
CREATE INDEX sentiment_snapshots_bucket_idx ON sentiment_snapshots(bucket_hour DESC);

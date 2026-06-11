-- Tag algo orders as entry vs exit. The engine knows which is which at
-- submit time but never recorded it, so per-day entry counting (the
-- overtrading gate) had no reliable data. Backfill heuristic: entries
-- have always been submitted as order_class 'bracket' and closes as
-- 'simple' (algo_engine submit sites) — labeled accordingly.
ALTER TABLE algo_orders
    ADD COLUMN IF NOT EXISTS kind TEXT NOT NULL DEFAULT 'entry';

UPDATE algo_orders
   SET kind = CASE WHEN order_class = 'bracket' THEN 'entry' ELSE 'exit' END;

CREATE INDEX IF NOT EXISTS algo_orders_kind_idx
    ON algo_orders (strategy_id, kind, submitted_at DESC);

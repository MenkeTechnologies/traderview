-- Trailing stops in the paper engine. A 'trailing' order rests as
-- 'pending'; the ticker ratchets trail_extreme (high-water for
-- sell/short, low-water for buy/cover) and fills when price retraces
-- trail_value ($ or fraction when trail_is_pct) from the extreme.
ALTER TYPE paper_order_type_t ADD VALUE IF NOT EXISTS 'trailing';

ALTER TABLE paper_orders
    ADD COLUMN IF NOT EXISTS trail_value   NUMERIC,
    ADD COLUMN IF NOT EXISTS trail_is_pct  BOOLEAN,
    ADD COLUMN IF NOT EXISTS trail_extreme NUMERIC;

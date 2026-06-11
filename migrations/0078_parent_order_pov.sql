-- POV (percent-of-volume) pacing for paper parent orders. A 'pov'
-- parent sizes each child as participation_rate x the cumulative-volume
-- delta observed between ticks; last_market_volume is the baseline.
-- status gains 'capped' (child cap hit with quantity still unfilled).
ALTER TABLE paper_parent_orders
    ADD COLUMN IF NOT EXISTS participation_rate NUMERIC,
    ADD COLUMN IF NOT EXISTS last_market_volume BIGINT;

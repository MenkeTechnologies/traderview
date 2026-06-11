-- Execution style for paper parent orders: 'twap' (equal slices) or
-- 'vwap' (slices weighted by the stylized intraday volume U-curve).
ALTER TABLE paper_parent_orders
    ADD COLUMN IF NOT EXISTS style TEXT NOT NULL DEFAULT 'twap';

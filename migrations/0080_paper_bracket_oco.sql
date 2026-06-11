-- Bracket / OCO orders in the paper engine. A bracket is an entry
-- order plus two exit legs (stop-loss + take-profit) sharing an
-- oco_group: legs rest as 'held' until parent_order_id fills, then
-- promote to 'pending'; the first leg to fill cancels its sibling.
ALTER TYPE paper_order_status_t ADD VALUE IF NOT EXISTS 'held';

ALTER TABLE paper_orders
    ADD COLUMN IF NOT EXISTS oco_group       UUID,
    ADD COLUMN IF NOT EXISTS parent_order_id UUID;

CREATE INDEX IF NOT EXISTS idx_paper_orders_held_parent
    ON paper_orders (parent_order_id) WHERE parent_order_id IS NOT NULL;

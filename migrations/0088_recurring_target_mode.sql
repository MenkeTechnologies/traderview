-- Cash-flow rebalancing: a recurring buy may point at a rebalance
-- TARGET instead of one fixed symbol — each run buys the most
-- underweight asset in the target, reducing drift with deposits
-- instead of sells. symbol becomes nullable (exactly one of
-- symbol / target_id is set; enforced in code with a readable error).
ALTER TABLE paper_recurring_orders
    ALTER COLUMN symbol DROP NOT NULL,
    ADD COLUMN IF NOT EXISTS target_id UUID
        REFERENCES paper_rebalance_targets(id) ON DELETE SET NULL;

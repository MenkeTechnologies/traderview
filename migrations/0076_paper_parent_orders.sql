-- TWAP parent orders for the paper engine: a parent slices into equal
-- child market orders submitted by the background ticker at a fixed
-- interval. Child fills go through paper::submit's existing friction
-- model — this table only owns the slicing schedule and progress.
CREATE TABLE IF NOT EXISTS paper_parent_orders (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID        NOT NULL,
    account_id       UUID        NOT NULL REFERENCES paper_accounts(id) ON DELETE CASCADE,
    symbol           TEXT        NOT NULL,
    side             TEXT        NOT NULL, -- buy | sell | short | cover
    total_qty        NUMERIC     NOT NULL,
    slices           INT         NOT NULL,
    interval_seconds INT         NOT NULL,
    slices_filled    INT         NOT NULL DEFAULT 0,
    qty_filled       NUMERIC     NOT NULL DEFAULT 0,
    status           TEXT        NOT NULL DEFAULT 'working', -- working | done | cancelled | error
    last_error       TEXT,
    next_slice_at    TIMESTAMPTZ NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_paper_parent_orders_due
    ON paper_parent_orders (status, next_slice_at);
CREATE INDEX IF NOT EXISTS idx_paper_parent_orders_user
    ON paper_parent_orders (user_id, created_at DESC);

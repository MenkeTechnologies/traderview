-- Brokers that don't supply an order_id (Webull's Orders Records CSV, the
-- "Manual / Other" source, several others) were leaving broker_order_id NULL,
-- which made `executions_dedupe_idx` a no-op for those rows (it's partial:
-- `WHERE broker_order_id IS NOT NULL`). Re-uploading the same CSV — including
-- the retry that follows a mid-import failure — inserted every row a second
-- time and inflated P&L proportionally.
--
-- This migration:
--   1) Deletes duplicates already in the table for rows with NULL
--      broker_order_id, keeping the earliest-inserted row per content tuple.
--      CASCADE on trade_executions handles the join-table cleanup. Trades
--      themselves are not deleted here; users run rollup_account to rebuild
--      them against the deduped executions.
--   2) Adds a content-based partial unique index covering the NULL case so
--      future re-uploads are caught by ON CONFLICT.

-- 1) Deduplicate existing rows.
WITH dups AS (
    SELECT id,
           ROW_NUMBER() OVER (
               PARTITION BY account_id, executed_at, symbol, side, qty, price
               ORDER BY created_at, id
           ) AS rn
      FROM executions
     WHERE broker_order_id IS NULL
)
DELETE FROM executions
 WHERE id IN (SELECT id FROM dups WHERE rn > 1);

-- 2) Prevent the duplicates from coming back.
CREATE UNIQUE INDEX IF NOT EXISTS executions_dedupe_no_order_id_idx
    ON executions(account_id, executed_at, symbol, side, qty, price)
    WHERE broker_order_id IS NULL;

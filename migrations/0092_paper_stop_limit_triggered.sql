-- stop_limit was in paper_order_type_t from day one (0011) but the
-- engine never implemented it. Once the stop crosses, the order is
-- PERMANENTLY a limit order — that state must survive process
-- restarts, hence a column, not memory.
ALTER TABLE paper_orders ADD COLUMN stop_triggered BOOLEAN NOT NULL DEFAULT FALSE;

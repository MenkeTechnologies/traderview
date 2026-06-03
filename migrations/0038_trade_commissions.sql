-- 0038 — Split broker commissions out of the combined `fees` column.
--
-- Until now `trades.fees` held everything a broker took: per-share/per-trade
-- commissions plus regulatory/exchange/clearing fees (SEC, FINRA TAF, NSCC,
-- OCC, etc.). The Tradervue parity audit flagged that "Total Commissions"
-- and "Total Fees" are reported separately on the Detailed page, so we
-- separate them here.
--
-- Backwards compat: existing rows keep `fees` as the combined number and
-- get `commissions = 0`. Importers that have per-execution commission
-- breakdowns should populate the new column going forward; the SQL
-- aggregator sums whichever fields exist, so totals stay correct.

ALTER TABLE trades
    ADD COLUMN commissions NUMERIC(20, 8) NOT NULL DEFAULT 0;

ALTER TABLE executions
    ADD COLUMN commissions NUMERIC(20, 8) NOT NULL DEFAULT 0;

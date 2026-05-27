-- Pre-trade risk-gate rules. The engine itself lives in
-- traderview-core::risk_gate; this table just persists user-defined rule
-- sets so the route can load them and evaluate against a proposed trade.
--
-- `rule` column is JSONB matching the `RiskRule` serde-tagged enum
-- (`{ "type": "max_loss_per_trade_pct", "pct": "1.0" }` etc). Storing
-- as JSONB means adding new rule variants is a backend-only change —
-- no schema migration each time.

CREATE TABLE risk_rules (
    id         UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- NULL = applies to every account. Set to scope to a single account
    -- (e.g. tighter rules on a small live account vs paper).
    account_id UUID         REFERENCES accounts(id) ON DELETE CASCADE,
    rule       JSONB        NOT NULL,
    enabled    BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX risk_rules_user_idx    ON risk_rules (user_id, enabled);
CREATE INDEX risk_rules_account_idx ON risk_rules (account_id) WHERE account_id IS NOT NULL;

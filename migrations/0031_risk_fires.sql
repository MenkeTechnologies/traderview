-- Audit log for every Risk Gate decision. One row per evaluate() call
-- where at least one rule fired (warnings count too — they reveal what
-- almost stopped the user). The view surfaces this so the user can see
-- "Risk Gate saved me 3 times today" and tune accordingly.

CREATE TABLE risk_fires (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_id  UUID         REFERENCES accounts(id) ON DELETE CASCADE,
    symbol      TEXT         NOT NULL,
    -- The full GateDecision serde-encoded — keeps every rule + severity
    -- + message verbatim for later audit. JSONB lets us GROUP BY rule
    -- name later for "which rule fires most" analytics.
    decision    JSONB        NOT NULL,
    blocked     BOOLEAN      NOT NULL,
    fired_at    TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX risk_fires_user_fired_idx ON risk_fires (user_id, fired_at DESC);
CREATE INDEX risk_fires_blocked_idx    ON risk_fires (user_id, blocked, fired_at DESC);

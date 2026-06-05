-- 0045: Tax-filing wizard storage.
--
-- The `#file-taxes` wizard is a multi-step TurboTax/Cash-App-Taxes-style
-- guided interview. Three tables back it:
--
--   tax_returns               — one row per (user, tax_year). The wizard's
--                               state lives in `data` (JSONB) so adding new
--                               fields (e.g. a new credit) doesn't need
--                               another migration. `status` tracks where in
--                               the wizard the user is, for the dashboard
--                               widget's progress bar.
--
--   tax_return_revisions      — append-only autosave log so the user can
--                               recover from accidental wipes. Trimmed to
--                               the last 50 per return by a background
--                               sweeper; no FK cascade because we want
--                               revisions to survive return-deletes for
--                               audit purposes (manual cleanup).
--
--   w2_imports                — one row per W-2 / 1099 the user uploads.
--                               Box values land here, the wizard reads
--                               them into the Income section. Links back
--                               to a receipt_id so the user can re-open
--                               the source image; `kind` discriminates
--                               between W-2 / 1099-NEC / 1099-MISC /
--                               1099-INT / 1099-DIV / 1099-K.

CREATE TABLE IF NOT EXISTS tax_returns (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tax_year     INTEGER      NOT NULL,
    -- Wizard step the user is on. Free text rather than an enum so we
    -- can rename steps without a migration. UI canonical values:
    --   'personal' | 'income' | 'adjustments' | 'deductions' |
    --   'credits' | 'other_taxes' | 'review' | 'filed'
    status       TEXT         NOT NULL DEFAULT 'personal',
    -- Wizard state. All collected answers, including auto-populated
    -- Schedule C / E rollups. Shape mirrors `TaxReturnData` in
    -- traderview-tax.
    data         JSONB        NOT NULL DEFAULT '{}'::jsonb,
    -- Computed totals — recomputed on every PATCH so the dashboard
    -- widget can show "Estimated refund: $X" without re-running the
    -- engine. Snapshot, not authoritative.
    refund_due   NUMERIC(14, 2),
    tax_owed     NUMERIC(14, 2),
    agi          NUMERIC(14, 2),
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, tax_year)
);

CREATE INDEX IF NOT EXISTS idx_tax_returns_user_year
    ON tax_returns(user_id, tax_year DESC);

-- ------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS tax_return_revisions (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    tax_return_id UUID        NOT NULL,
    -- Snapshot of `tax_returns.data` at the time of the change.
    data          JSONB       NOT NULL,
    -- What changed. Free-form label set by the endpoint
    -- ("personal.dob", "income.w2.added", "deductions.itemize_selected").
    change_label  TEXT        NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tax_return_revisions_lookup
    ON tax_return_revisions(tax_return_id, created_at DESC);

-- ------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS w2_imports (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id        UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tax_year       INTEGER     NOT NULL,
    -- 'w2' | '1099_nec' | '1099_misc' | '1099_int' | '1099_div' | '1099_k'
    kind           TEXT        NOT NULL,
    -- Receipt the bytes were stored against — the OCR pipeline reuses
    -- the existing receipts table for blob storage so we don't
    -- duplicate the file path / dedup logic.
    receipt_id     UUID        REFERENCES receipts(id) ON DELETE SET NULL,
    -- Box-by-box payload. Schema varies per `kind`. For W-2 the keys
    -- are box_1, box_2, …, box_20 plus employer_name, employer_ein,
    -- employee_ssn, employee_name. Storing as JSONB keeps each kind
    -- self-describing in one column.
    payload        JSONB       NOT NULL DEFAULT '{}'::jsonb,
    -- Confidence the parser is in this extraction (0..1). Below 0.7
    -- the wizard prompts the user to verify each box manually.
    confidence     REAL        NOT NULL DEFAULT 0,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_w2_imports_user_year
    ON w2_imports(user_id, tax_year DESC);

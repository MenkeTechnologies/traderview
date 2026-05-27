-- traderview business-expense tracking.
-- Purely additive: no ALTER on existing trading tables. Safe to roll back by
-- dropping the new objects (see bottom of file for the inverse).

-- ---------------------------------------------------------------------------
-- financial_accounts (bank, credit card, marketplace — non-broker sources)
-- ---------------------------------------------------------------------------
CREATE TYPE financial_account_kind_t AS ENUM ('bank', 'credit_card', 'marketplace');

CREATE TABLE financial_accounts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind            financial_account_kind_t NOT NULL,
    source          TEXT NOT NULL,                          -- 'bofa', 'chase', 'apple_card', 'amazon', 'manual'
    name            TEXT NOT NULL,                          -- user-visible label
    base_currency   TEXT NOT NULL DEFAULT 'USD',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX financial_accounts_user_id_idx ON financial_accounts(user_id);

-- ---------------------------------------------------------------------------
-- expense_categories (IRS Schedule C lines 8-27a, seeded below)
-- ---------------------------------------------------------------------------
-- `code` is the stable key referenced by transactions + rules + reports. Never
-- rename a code — add new ones and migrate existing rows if Schedule C changes.
CREATE TABLE expense_categories (
    code                TEXT PRIMARY KEY,
    schedule_c_line     TEXT NOT NULL,                      -- '8', '9', '24a', '24b', etc.
    label               TEXT NOT NULL,
    deduction_pct       NUMERIC(5, 4) NOT NULL DEFAULT 1.0, -- meals = 0.5000
    sort_order          INTEGER NOT NULL
);

INSERT INTO expense_categories (code, schedule_c_line, label, deduction_pct, sort_order) VALUES
    ('advertising',     '8',   'Advertising',                          1.0,    10),
    ('car_truck',       '9',   'Car and truck expenses',               1.0,    20),
    ('commissions',     '10',  'Commissions and fees',                 1.0,    30),
    ('contract_labor',  '11',  'Contract labor',                       1.0,    40),
    ('depletion',       '12',  'Depletion',                            1.0,    50),
    ('depreciation',    '13',  'Depreciation and section 179',         1.0,    60),
    ('employee_benefit','14',  'Employee benefit programs',            1.0,    70),
    ('insurance',       '15',  'Insurance (other than health)',        1.0,    80),
    ('interest_mort',   '16a', 'Mortgage interest (to banks)',         1.0,    90),
    ('interest_other',  '16b', 'Other interest',                       1.0,   100),
    ('legal',           '17',  'Legal and professional services',      1.0,   110),
    ('office',          '18',  'Office expense',                       1.0,   120),
    ('pension',         '19',  'Pension and profit-sharing plans',     1.0,   130),
    ('rent_vehicle',    '20a', 'Rent or lease: vehicles, machinery',   1.0,   140),
    ('rent_other',      '20b', 'Rent or lease: other business prop',   1.0,   150),
    ('repairs',         '21',  'Repairs and maintenance',              1.0,   160),
    ('supplies',        '22',  'Supplies',                             1.0,   170),
    ('taxes',           '23',  'Taxes and licenses',                   1.0,   180),
    ('travel',          '24a', 'Travel',                               1.0,   190),
    ('meals_50',        '24b', 'Deductible meals (50%)',               0.5,   200),
    ('utilities',       '25',  'Utilities',                            1.0,   210),
    ('wages',           '26',  'Wages (less employment credits)',      1.0,   220),
    ('other',           '27a', 'Other expenses',                       1.0,   230)
ON CONFLICT (code) DO NOTHING;

-- ---------------------------------------------------------------------------
-- transactions (one row per imported line; signed amount)
-- ---------------------------------------------------------------------------
-- Sign convention: negative = money out (expense), positive = money in (refund,
-- income, credit-card payment received). Mirrors how the source CSVs typically
-- encode amounts after normalization.
CREATE TABLE transactions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID NOT NULL REFERENCES financial_accounts(id) ON DELETE CASCADE,
    posted_at           TIMESTAMPTZ NOT NULL,
    amount              NUMERIC(20, 2) NOT NULL,
    currency            TEXT NOT NULL DEFAULT 'USD',
    merchant_raw        TEXT NOT NULL,
    merchant_normalized TEXT NOT NULL,
    description         TEXT NOT NULL DEFAULT '',
    category_code       TEXT REFERENCES expense_categories(code),
    is_business         BOOLEAN NOT NULL DEFAULT TRUE,
    is_transfer         BOOLEAN NOT NULL DEFAULT FALSE,
    transfer_peer_id    UUID REFERENCES transactions(id) ON DELETE SET NULL,
    notes               TEXT NOT NULL DEFAULT '',
    import_id           UUID,
    raw                 JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX transactions_account_posted_idx ON transactions(account_id, posted_at DESC);
CREATE INDEX transactions_category_idx       ON transactions(category_code);
CREATE INDEX transactions_merchant_norm_idx  ON transactions(merchant_normalized);
-- Dedupe within an account: identical date+amount+merchant collapses repeated imports.
CREATE UNIQUE INDEX transactions_dedupe_idx
    ON transactions(account_id, posted_at, amount, merchant_raw);

-- ---------------------------------------------------------------------------
-- merchant_rules (user-trained merchant -> category map)
-- ---------------------------------------------------------------------------
CREATE TYPE merchant_pattern_kind_t AS ENUM ('substring', 'regex');

CREATE TABLE merchant_rules (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    pattern         TEXT NOT NULL,
    pattern_kind    merchant_pattern_kind_t NOT NULL DEFAULT 'substring',
    category_code   TEXT NOT NULL REFERENCES expense_categories(code),
    is_business     BOOLEAN NOT NULL DEFAULT TRUE,
    priority        INTEGER NOT NULL DEFAULT 100,           -- lower = matched first
    hit_count       BIGINT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX merchant_rules_user_idx ON merchant_rules(user_id, priority);

-- ---------------------------------------------------------------------------
-- receipts (image/PDF blobs stored on disk, metadata + OCR result here)
-- ---------------------------------------------------------------------------
CREATE TYPE ocr_status_t AS ENUM ('pending', 'done', 'failed', 'needs_image');

CREATE TABLE receipts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    transaction_id  UUID REFERENCES transactions(id) ON DELETE SET NULL,
    filename        TEXT NOT NULL,
    sha256          TEXT NOT NULL,
    mime            TEXT NOT NULL,
    bytes_len       BIGINT NOT NULL,
    storage_path    TEXT NOT NULL,                          -- relative to $APP_DATA_DIR/traderview/receipts/
    ocr_status      ocr_status_t NOT NULL DEFAULT 'pending',
    ocr_text        TEXT,
    ocr_merchant    TEXT,
    ocr_total       NUMERIC(20, 2),
    ocr_date        DATE,
    ocr_confidence  REAL,
    match_score     REAL,
    error_message   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, sha256)
);
CREATE INDEX receipts_user_idx        ON receipts(user_id, created_at DESC);
CREATE INDEX receipts_transaction_idx ON receipts(transaction_id);
CREATE INDEX receipts_ocr_status_idx  ON receipts(ocr_status);

-- ---------------------------------------------------------------------------
-- expense_imports (audit; mirrors `imports` for the broker side)
-- ---------------------------------------------------------------------------
CREATE TABLE expense_imports (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID NOT NULL REFERENCES financial_accounts(id) ON DELETE CASCADE,
    source          TEXT NOT NULL,                          -- parser id: 'bofa', 'chase', 'amazon', 'apple_card'
    filename        TEXT NOT NULL,
    sha256          TEXT NOT NULL,
    row_count       INTEGER NOT NULL DEFAULT 0,
    inserted_count  INTEGER NOT NULL DEFAULT 0,
    imported_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (account_id, sha256)
);

ALTER TABLE transactions
    ADD CONSTRAINT transactions_import_fk
    FOREIGN KEY (import_id) REFERENCES expense_imports(id) ON DELETE SET NULL;

-- ---------------------------------------------------------------------------
-- Rollback (manual):
--   DROP TABLE expense_imports, receipts, merchant_rules, transactions,
--              expense_categories, financial_accounts CASCADE;
--   DROP TYPE  ocr_status_t, merchant_pattern_kind_t,
--              financial_account_kind_t;
-- ---------------------------------------------------------------------------

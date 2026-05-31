-- traderview landlord / rental-property tracking (IRS Form 1040 Schedule E Part I).
--
-- Purely additive: no ALTER on existing tables. Mirrors the discipline of the
-- Schedule C surface in 0029_expenses.sql:
--   * stable string `code` keys for line items (never renamed; new codes only).
--   * one row per atomic event (rent receipt, expense payment, mileage log).
--   * dedupe index on (property_id, posted_at, amount, payer_or_payee_raw).
--   * receipts.rental_expense_id back-link so OCR can attach to either side.
--
-- Cross-reference: Schedule E is Form 1040's rental income/loss form; Part I
-- covers real estate rentals. The 17 expense line items below (Lines 5-19)
-- match the form 1:1 so the Schedule E export can render without remapping.

-- ---------------------------------------------------------------------------
-- property_type_t — IRS Schedule E codes 1-8 (Form 1040 instructions, 2024)
-- ---------------------------------------------------------------------------
CREATE TYPE property_type_t AS ENUM (
    'single_family',        -- IRS code 1
    'multi_family',         -- IRS code 2
    'vacation_short_term',  -- IRS code 3 (Vrbo, Airbnb)
    'commercial',           -- IRS code 4
    'land',                 -- IRS code 5
    'royalties',            -- IRS code 6 (oil, gas, mineral, copyright, patent)
    'self_rental',          -- IRS code 7
    'other'                 -- IRS code 8
);

CREATE TYPE property_status_t AS ENUM ('active', 'vacant', 'sold', 'archived');

-- ---------------------------------------------------------------------------
-- properties — one row per real-estate unit being rented
-- ---------------------------------------------------------------------------
CREATE TABLE rental_properties (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                 UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    nickname                TEXT NOT NULL,                          -- "Maple St duplex"
    property_type           property_type_t NOT NULL,
    status                  property_status_t NOT NULL DEFAULT 'active',
    address_line1           TEXT NOT NULL DEFAULT '',
    address_line2           TEXT NOT NULL DEFAULT '',
    city                    TEXT NOT NULL DEFAULT '',
    state_region            TEXT NOT NULL DEFAULT '',
    postal_code             TEXT NOT NULL DEFAULT '',
    country                 TEXT NOT NULL DEFAULT 'US',
    units                   INTEGER NOT NULL DEFAULT 1,             -- multi-family unit count
    purchased_at            DATE,
    purchase_price          NUMERIC(20, 2),
    land_value              NUMERIC(20, 2),                         -- excluded from depreciation basis
    placed_in_service_at    DATE,                                   -- depreciation start
    recovery_period_years   NUMERIC(5, 2) NOT NULL DEFAULT 27.5,    -- residential = 27.5, commercial = 39
    fair_rental_days        INTEGER NOT NULL DEFAULT 0,             -- Schedule E line 2
    personal_use_days       INTEGER NOT NULL DEFAULT 0,             -- Schedule E line 2
    qjv_election            BOOLEAN NOT NULL DEFAULT FALSE,         -- Qualified Joint Venture (spouse co-owner)
    qbi_safe_harbor         BOOLEAN NOT NULL DEFAULT FALSE,         -- Rev. Proc. 2019-38: 250 hours/yr
    sold_at                 DATE,
    sold_price              NUMERIC(20, 2),
    notes                   TEXT NOT NULL DEFAULT '',
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX rental_properties_user_idx   ON rental_properties(user_id);
CREATE INDEX rental_properties_status_idx ON rental_properties(user_id, status);

-- ---------------------------------------------------------------------------
-- tenants — people / entities renting a property
-- ---------------------------------------------------------------------------
CREATE TABLE rental_tenants (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    display_name    TEXT NOT NULL,
    email           TEXT NOT NULL DEFAULT '',
    phone           TEXT NOT NULL DEFAULT '',
    notes           TEXT NOT NULL DEFAULT '',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX rental_tenants_user_idx ON rental_tenants(user_id);

-- ---------------------------------------------------------------------------
-- leases — a tenancy term on a property unit
-- ---------------------------------------------------------------------------
CREATE TYPE lease_status_t AS ENUM ('draft', 'active', 'expired', 'terminated_early');

CREATE TABLE rental_leases (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id         UUID NOT NULL REFERENCES rental_properties(id) ON DELETE CASCADE,
    tenant_id           UUID NOT NULL REFERENCES rental_tenants(id) ON DELETE RESTRICT,
    unit_label          TEXT NOT NULL DEFAULT '',                   -- "1A" for multi-family
    status              lease_status_t NOT NULL DEFAULT 'active',
    starts_on           DATE NOT NULL,
    ends_on             DATE,                                       -- NULL = month-to-month
    rent_amount         NUMERIC(20, 2) NOT NULL,
    rent_frequency      TEXT NOT NULL DEFAULT 'monthly',            -- monthly/weekly/yearly
    rent_due_day        INTEGER NOT NULL DEFAULT 1,                 -- 1..31 day-of-month
    grace_days          INTEGER NOT NULL DEFAULT 5,
    late_fee_fixed      NUMERIC(20, 2) NOT NULL DEFAULT 0,
    late_fee_pct        NUMERIC(5, 4) NOT NULL DEFAULT 0,
    security_deposit    NUMERIC(20, 2) NOT NULL DEFAULT 0,
    deposit_held_by     TEXT NOT NULL DEFAULT '',                   -- bank/escrow holder name
    pet_deposit         NUMERIC(20, 2) NOT NULL DEFAULT 0,
    notes               TEXT NOT NULL DEFAULT '',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX rental_leases_property_idx ON rental_leases(property_id);
CREATE INDEX rental_leases_tenant_idx   ON rental_leases(tenant_id);
CREATE INDEX rental_leases_active_idx   ON rental_leases(property_id, status, starts_on DESC);

-- ---------------------------------------------------------------------------
-- rental_income — receipts (rent, late fees, deposit forfeiture, parking)
-- ---------------------------------------------------------------------------
CREATE TYPE rental_income_kind_t AS ENUM (
    'rent',
    'late_fee',
    'deposit_forfeit',
    'reimbursement',        -- tenant pays back damage / utility shortfall
    'royalty',
    'parking',
    'laundry',
    'storage',
    'other'
);

CREATE TABLE rental_income (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id         UUID NOT NULL REFERENCES rental_properties(id) ON DELETE CASCADE,
    lease_id            UUID REFERENCES rental_leases(id) ON DELETE SET NULL,
    posted_at           TIMESTAMPTZ NOT NULL,
    period_start        DATE,                                       -- rent for which month
    period_end          DATE,
    amount              NUMERIC(20, 2) NOT NULL,                    -- positive
    currency            TEXT NOT NULL DEFAULT 'USD',
    kind                rental_income_kind_t NOT NULL DEFAULT 'rent',
    payer_raw           TEXT NOT NULL DEFAULT '',
    method              TEXT NOT NULL DEFAULT '',                   -- ach/check/cash/zelle/cashapp/venmo
    transaction_id      UUID REFERENCES transactions(id) ON DELETE SET NULL,
    notes               TEXT NOT NULL DEFAULT '',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (property_id, posted_at, amount, payer_raw, kind)
);
CREATE INDEX rental_income_property_idx ON rental_income(property_id, posted_at DESC);
CREATE INDEX rental_income_lease_idx    ON rental_income(lease_id);
CREATE INDEX rental_income_kind_idx     ON rental_income(kind);

-- ---------------------------------------------------------------------------
-- schedule_e_categories — Form 1040 Schedule E Part I, Lines 5-19
-- ---------------------------------------------------------------------------
-- Like expense_categories: stable `code` keys, ordered by Schedule E line.
CREATE TABLE schedule_e_categories (
    code                TEXT PRIMARY KEY,
    schedule_e_line     TEXT NOT NULL,
    label               TEXT NOT NULL,
    deduction_pct       NUMERIC(5, 4) NOT NULL DEFAULT 1.0,
    sort_order          INTEGER NOT NULL
);

INSERT INTO schedule_e_categories (code, schedule_e_line, label, deduction_pct, sort_order) VALUES
    ('e_advertising',       '5',   'Advertising',                          1.0,    10),
    ('e_auto_travel',       '6',   'Auto and travel',                      1.0,    20),
    ('e_cleaning_maint',    '7',   'Cleaning and maintenance',             1.0,    30),
    ('e_commissions',       '8',   'Commissions',                          1.0,    40),
    ('e_insurance',         '9',   'Insurance',                            1.0,    50),
    ('e_legal_prof',        '10',  'Legal and other professional fees',    1.0,    60),
    ('e_mgmt_fees',         '11',  'Management fees',                      1.0,    70),
    ('e_mortgage_interest', '12',  'Mortgage interest paid to banks',      1.0,    80),
    ('e_other_interest',    '13',  'Other interest',                       1.0,    90),
    ('e_repairs',           '14',  'Repairs',                              1.0,   100),
    ('e_supplies',          '15',  'Supplies',                             1.0,   110),
    ('e_taxes',             '16',  'Taxes (property + local)',             1.0,   120),
    ('e_utilities',         '17',  'Utilities',                            1.0,   130),
    ('e_depreciation',      '18',  'Depreciation expense or depletion',    1.0,   140),
    ('e_other',             '19',  'Other (list separately)',              1.0,   150),
    ('e_hoa',               '19',  'HOA dues',                             1.0,   151),
    ('e_landscaping',       '19',  'Landscaping / lawn',                   1.0,   152),
    ('e_pest_control',      '19',  'Pest control',                         1.0,   153),
    ('e_permit_license',    '19',  'Permits and licenses',                 1.0,   154),
    ('e_appliance',         '19',  'Appliance replacement',                1.0,   155),
    ('e_software',          '19',  'Property-management software',         1.0,   156),
    ('e_bank_fee',          '19',  'Bank / wire fees',                     1.0,   157),
    ('e_eviction',          '19',  'Eviction filing / sheriff fees',       1.0,   158),
    ('e_security',          '19',  'Security / alarm monitoring',          1.0,   159)
ON CONFLICT (code) DO NOTHING;

-- ---------------------------------------------------------------------------
-- rental_expenses — outflows attributable to a property
-- ---------------------------------------------------------------------------
CREATE TABLE rental_expenses (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id         UUID NOT NULL REFERENCES rental_properties(id) ON DELETE CASCADE,
    posted_at           TIMESTAMPTZ NOT NULL,
    amount              NUMERIC(20, 2) NOT NULL,                    -- positive (sign neutralized at this layer)
    currency            TEXT NOT NULL DEFAULT 'USD',
    category_code       TEXT NOT NULL REFERENCES schedule_e_categories(code),
    vendor_raw          TEXT NOT NULL DEFAULT '',
    vendor_normalized   TEXT NOT NULL DEFAULT '',
    description         TEXT NOT NULL DEFAULT '',
    is_capitalized      BOOLEAN NOT NULL DEFAULT FALSE,             -- improvement vs repair (Reg. 1.263(a)-3)
    capital_useful_life INTEGER,                                    -- years if capitalized
    method              TEXT NOT NULL DEFAULT '',                   -- ach/check/card/cash
    transaction_id      UUID REFERENCES transactions(id) ON DELETE SET NULL,
    notes               TEXT NOT NULL DEFAULT '',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (property_id, posted_at, amount, vendor_raw, category_code)
);
CREATE INDEX rental_expenses_property_idx ON rental_expenses(property_id, posted_at DESC);
CREATE INDEX rental_expenses_category_idx ON rental_expenses(category_code);
CREATE INDEX rental_expenses_vendor_idx   ON rental_expenses(vendor_normalized);
CREATE INDEX rental_expenses_capital_idx  ON rental_expenses(property_id, is_capitalized) WHERE is_capitalized = TRUE;

-- ---------------------------------------------------------------------------
-- rental_mileage — auto/travel log for Schedule E line 6
-- ---------------------------------------------------------------------------
-- Standard mileage rate is set by the IRS annually (e.g. 67.0¢ for 2024).
-- We persist the rate at log time so historical entries don't re-rate.
CREATE TABLE rental_mileage (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id     UUID NOT NULL REFERENCES rental_properties(id) ON DELETE CASCADE,
    drove_on        DATE NOT NULL,
    miles           NUMERIC(10, 2) NOT NULL,
    rate_per_mile   NUMERIC(10, 4) NOT NULL,                        -- 0.6700 = 67.0¢
    purpose         TEXT NOT NULL DEFAULT '',
    odometer_start  NUMERIC(12, 1),
    odometer_end    NUMERIC(12, 1),
    notes           TEXT NOT NULL DEFAULT '',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX rental_mileage_property_idx ON rental_mileage(property_id, drove_on DESC);

-- ---------------------------------------------------------------------------
-- rental_maintenance — work orders / repair tickets
-- ---------------------------------------------------------------------------
CREATE TYPE maintenance_status_t AS ENUM ('open', 'in_progress', 'done', 'cancelled');
CREATE TYPE maintenance_priority_t AS ENUM ('low', 'normal', 'high', 'emergency');

CREATE TABLE rental_maintenance (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id     UUID NOT NULL REFERENCES rental_properties(id) ON DELETE CASCADE,
    lease_id        UUID REFERENCES rental_leases(id) ON DELETE SET NULL,
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    status          maintenance_status_t NOT NULL DEFAULT 'open',
    priority        maintenance_priority_t NOT NULL DEFAULT 'normal',
    reported_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    vendor          TEXT NOT NULL DEFAULT '',
    expense_id      UUID REFERENCES rental_expenses(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX rental_maintenance_property_idx ON rental_maintenance(property_id, reported_at DESC);
CREATE INDEX rental_maintenance_status_idx   ON rental_maintenance(status) WHERE status IN ('open', 'in_progress');

-- ---------------------------------------------------------------------------
-- rental_services_log — QBI safe-harbor 250-hour tracker (Rev. Proc. 2019-38)
-- ---------------------------------------------------------------------------
-- Section 199A QBI deduction for rental real estate requires 250+ hours of
-- rental services per year (advertising, screening, collecting, repairs,
-- management). Aggregated by tax year.
CREATE TABLE rental_services_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id     UUID NOT NULL REFERENCES rental_properties(id) ON DELETE CASCADE,
    performed_on    DATE NOT NULL,
    hours           NUMERIC(6, 2) NOT NULL,
    activity        TEXT NOT NULL,                                  -- "tenant screening", "snow removal", ...
    performer       TEXT NOT NULL DEFAULT 'self',                   -- self / employee / contractor
    notes           TEXT NOT NULL DEFAULT '',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX rental_services_property_idx ON rental_services_log(property_id, performed_on DESC);

-- ---------------------------------------------------------------------------
-- Wire receipts back into the rental side so an OCR'd PDF can attach to a
-- rental expense (in addition to the existing transactions back-link).
-- ---------------------------------------------------------------------------
ALTER TABLE receipts ADD COLUMN rental_expense_id UUID
    REFERENCES rental_expenses(id) ON DELETE SET NULL;
CREATE INDEX receipts_rental_expense_idx ON receipts(rental_expense_id);

-- ---------------------------------------------------------------------------
-- Rollback (manual):
--   ALTER TABLE receipts DROP COLUMN rental_expense_id;
--   DROP TABLE rental_services_log, rental_maintenance, rental_mileage,
--              rental_expenses, schedule_e_categories, rental_income,
--              rental_leases, rental_tenants, rental_properties CASCADE;
--   DROP TYPE  maintenance_priority_t, maintenance_status_t,
--              rental_income_kind_t, lease_status_t,
--              property_status_t, property_type_t;
-- ---------------------------------------------------------------------------

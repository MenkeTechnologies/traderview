-- Per-receipt business tagging. The previous business-tagging work
-- (0047_businesses.sql) added business_id to items inside the
-- `ocr_extracted` JSONB blob — fine for per-item granularity but
-- forces every analytics endpoint to walk items in Rust just to know
-- which receipts belong to which business.
--
-- This migration adds a receipt-level `business_id`. Semantics:
--   * NULL              → unassigned (mixed / personal / aggregated)
--   * UUID              → entire receipt belongs to this business
--
-- When set on the receipt, all items inherit it as a default — useful
-- for the upload flow where the active business context is known.
-- Item-level `business_id` in JSONB still overrides for the per-item
-- case (e.g. a Costco run that mixes Business A and Personal items).

ALTER TABLE receipts
    ADD COLUMN business_id UUID REFERENCES businesses(id) ON DELETE SET NULL;
CREATE INDEX receipts_business_idx
    ON receipts(business_id)
    WHERE business_id IS NOT NULL;
-- Covering index for the common analytics query
-- ("everything for a business in a date window").
CREATE INDEX receipts_user_business_date_idx
    ON receipts(user_id, business_id, ocr_date)
    WHERE business_id IS NOT NULL;

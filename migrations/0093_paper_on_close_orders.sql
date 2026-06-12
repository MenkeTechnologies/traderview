-- Market-on-close / limit-on-close. Both rest until the next 16:00 US
-- Eastern close (holiday/DST aware via day_order_expiry, stamped into
-- trigger_at at submit). At the first tick at-or-after the close: MOC
-- fills at last; LOC fills only at limit-or-better and CANCELS
-- otherwise (an unfilled LOC does not survive to the next session).
ALTER TYPE paper_order_type_t ADD VALUE IF NOT EXISTS 'moc';
ALTER TYPE paper_order_type_t ADD VALUE IF NOT EXISTS 'loc';

ALTER TABLE paper_orders ADD COLUMN IF NOT EXISTS trigger_at TIMESTAMPTZ;

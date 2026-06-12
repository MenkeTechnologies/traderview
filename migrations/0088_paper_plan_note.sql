-- Trade plan notes on paper orders. The RequirePlanBeforeTrade risk
-- rule was unsatisfiable at the manual ticket (has_attached_plan was
-- hardcoded false); a non-empty plan_note now satisfies it and the
-- note persists with the order as the audit trail.
ALTER TABLE paper_orders
    ADD COLUMN IF NOT EXISTS plan_note TEXT;

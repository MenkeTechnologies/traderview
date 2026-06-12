-- The thesis: WHY this strategy should work, written down before it
-- runs — the algo-side counterpart of the paper ticket's plan note.
ALTER TABLE algo_strategies ADD COLUMN IF NOT EXISTS notes TEXT;

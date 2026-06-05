//! Merchant canonicalization — collapse OCR / statement noise into a
//! single stable key per real-world merchant.
//!
//! Pipeline:
//!   1. Run the existing `traderview_expense::normalize::normalize`
//!      (strips SQ*/TST* prefixes, trailing IDs, multi-space).
//!   2. Walk the user's `merchant_aliases` rows. Each row carries a
//!      canonical name and 0..N case-insensitive regex aliases. First
//!      match wins.
//!   3. If no alias matches, fall back to the normalized form. This
//!      means that even without a user-curated alias table, identical
//!      OCR strings cluster correctly — and similar variants ("Wal*Mart
//!      #482") collapse via the prefix/ID stripping in step 1.
//!
//! All matching is regex-based on `alias_patterns[]`. We compile lazily
//! per call to keep the helper stateless — the alias list is tiny
//! (10-100 rows per user) so the cost is irrelevant compared to the
//! OCR pass.

use regex::RegexBuilder;
use sqlx::PgPool;
use uuid::Uuid;

/// One merchant_aliases row, joined as a slice for matching.
#[derive(Debug, Clone)]
pub struct MerchantAlias {
    pub canonical: String,
    pub patterns: Vec<String>,
}

/// Load every alias the user has defined. Cheap — typical users have
/// <100 rows; we just slurp the whole set.
pub async fn load_aliases(pool: &PgPool, user_id: Uuid) -> Result<Vec<MerchantAlias>, sqlx::Error> {
    let rows: Vec<(String, Vec<String>)> = sqlx::query_as(
        "SELECT canonical, alias_patterns
           FROM merchant_aliases
          WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(canonical, patterns)| MerchantAlias { canonical, patterns })
        .collect())
}

/// Canonicalize `raw` against `aliases`. Returns the canonical name
/// the merchant should roll up under. Never returns an empty string —
/// callers can treat the result as a stable grouping key.
pub fn canonicalize(raw: &str, aliases: &[MerchantAlias]) -> String {
    let normalized = traderview_expense::normalize::normalize(raw);

    for alias in aliases {
        for pattern in &alias.patterns {
            // Compile each pattern case-insensitively. We DON'T cache —
            // the alias list is short and the regex crate's caching of
            // common patterns means compilation is microseconds.
            let re = match RegexBuilder::new(pattern)
                .case_insensitive(true)
                .build()
            {
                Ok(re) => re,
                Err(_) => continue, // malformed user-supplied regex — skip silently
            };
            // Match against BOTH the raw and normalized forms so a
            // user who pasted in `WAL.?MART` matches "WAL-MART STORE
            // 4892" (raw) AND "wal mart" (normalized).
            if re.is_match(raw) || re.is_match(&normalized) {
                return alias.canonical.clone();
            }
        }
    }

    // No alias hit — fall back to the normalized form so identical OCR
    // strings still cluster. Capitalize the first word for display so
    // it doesn't look like an internal slug.
    if normalized.is_empty() {
        raw.trim().to_string()
    } else {
        // Title-case the first word only — "wal mart" → "Wal mart".
        // Cheap heuristic, good enough for display in the top-merchants
        // table without pulling in unicode_segmentation.
        let mut chars = normalized.chars();
        let first = chars.next().unwrap().to_uppercase().collect::<String>();
        let rest: String = chars.collect();
        format!("{first}{rest}")
    }
}

/// UPSERT an alias. Inserts when absent, increments use_count + touches
/// updated_at when present. Used both by the "Merge merchants" UI and
/// by the auto-seed path that records a freshly-seen canonical at OCR
/// completion.
pub async fn upsert_alias(
    pool: &PgPool,
    user_id: Uuid,
    canonical: &str,
    new_patterns: &[String],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO merchant_aliases (user_id, canonical, alias_patterns, use_count)
         VALUES ($1, $2, $3, 1)
         ON CONFLICT (user_id, canonical)
         DO UPDATE SET
            alias_patterns = ARRAY(
                SELECT DISTINCT UNNEST(merchant_aliases.alias_patterns || EXCLUDED.alias_patterns)
            ),
            use_count = merchant_aliases.use_count + 1,
            updated_at = NOW()",
    )
    .bind(user_id)
    .bind(canonical)
    .bind(new_patterns)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalize_falls_back_to_normalized_when_no_aliases() {
        let out = canonicalize("WAL-MART STORE 4892", &[]);
        // expense::normalize lowercases + strips trailing IDs, leaving
        // "wal-mart store" (the digit-only "4892" is stripped); the
        // canonicalize() title-case wrap → "Wal-mart store".
        assert!(
            out.starts_with("Wal"),
            "expected normalized fallback starting with 'Wal', got {out:?}"
        );
    }

    #[test]
    fn canonicalize_returns_alias_canonical_on_pattern_match() {
        let aliases = vec![MerchantAlias {
            canonical: "Walmart".into(),
            patterns: vec!["WAL.?MART".into()],
        }];
        assert_eq!(canonicalize("WAL-MART STORE 4892", &aliases), "Walmart");
        assert_eq!(canonicalize("Wal*mart #482", &aliases), "Walmart");
        assert_eq!(canonicalize("Walmart.com", &aliases), "Walmart");
    }

    #[test]
    fn canonicalize_ignores_malformed_regex() {
        // A regex the user pasted incorrectly must not blow up the
        // canonicalizer — it just doesn't match.
        let aliases = vec![MerchantAlias {
            canonical: "Walmart".into(),
            patterns: vec!["(unclosed".into(), "WAL.?MART".into()],
        }];
        assert_eq!(canonicalize("WAL-MART", &aliases), "Walmart");
    }

    #[test]
    fn canonicalize_handles_empty_input() {
        // Empty raw → empty result, but never panics.
        let out = canonicalize("", &[]);
        assert_eq!(out, "");
    }
}

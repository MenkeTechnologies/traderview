//! Webull execution parser — STUB.
//!
//! Awaiting a real redacted CSV sample from the user. Webull has multiple
//! export shapes (Account Statement → Orders, History → Orders, monthly
//! statement PDFs, mobile-app email exports). Each has different column
//! names, dt formats, and side encodings. Inferring columns from documentation
//! is exactly the kind of guesswork CLAUDE.md forbids — the parser body stays
//! empty until a sample exists.

use crate::{ImportError, ParsedExecution, Parser};

pub struct WebullParser;

impl Parser for WebullParser {
    fn source(&self) -> &'static str {
        "webull"
    }

    fn parse(&self, _bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError> {
        Err(ImportError::Unsupported(
            "webull parser not yet implemented — upload a real CSV sample".into(),
        ))
    }
}

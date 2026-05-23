//! FIFO trade roll-up.
//!
//! Algorithm (per (account_id, symbol)):
//!   1. Sort executions by executed_at ASC, then by created_at as tiebreaker.
//!   2. Maintain an open-lot queue (per direction).
//!   3. Opens add to the queue.
//!   4. Closes consume from the queue FIFO, emitting per-leg P&L.
//!   5. A trade starts when the queue transitions 0 -> nonzero in a direction,
//!      and closes when it returns to 0.
//!
//! This module currently exposes a placeholder API. Implementation comes in
//! Phase 3 after real Webull executions exist to verify against.

use crate::models::{Execution, Trade};

#[derive(Debug, Clone, Copy)]
pub enum LotMethod {
    Fifo,
    Lifo,
}

#[derive(Debug, thiserror::Error)]
pub enum RollupError {
    #[error("not yet implemented")]
    Todo,
}

pub fn rollup(_executions: &[Execution], _method: LotMethod) -> Result<Vec<Trade>, RollupError> {
    Err(RollupError::Todo)
}

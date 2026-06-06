//! traderview-tax: US federal income-tax compute engine.
//!
//! Scope: **2025 tax year** (returns filed in calendar year 2026).
//! All constants come from IRS Rev. Proc. 2024-40 (the official 2025
//! inflation adjustments) and the Social Security Administration's
//! 2025 wage-base announcement. Sources are cited inline next to each
//! constant so a future-year update is mechanical.
//!
//! Modules:
//!   * [`brackets`] — ordinary income tax brackets per filing status.
//!   * [`se_tax`] — self-employment tax (Schedule SE), 15.3% on 92.35%
//!     of net SE earnings, SS portion capped at the 2025 wage base.
//!   * [`qbi`] — § 199A qualified business income deduction.
//!   * [`credits`] — Child Tax Credit + Other Dependent Credit + EITC.
//!   * [`engine`] — `TaxReturn` → `TaxResult` orchestrator.
//!   * [`safe_harbor`] — IRC § 6654 quarterly estimated-tax safe harbor.
//!   * [`what_if`] — scenario delta engine for refund/owed planning.

pub mod brackets;
pub mod capital_gains;
pub mod credits;
pub mod engine;
pub mod niit;
pub mod qbi;
pub mod safe_harbor;
pub mod se_tax;
pub mod what_if;

pub use capital_gains::{compute as compute_qdcgtw, QdcgtwInput, QdcgtwResult};
pub use engine::{compute, FilingStatus, TaxResult, TaxReturn};
pub use niit::{compute as compute_niit, NiitInput, NiitResult};
pub use safe_harbor::{compute as compute_safe_harbor, SafeHarborInput, SafeHarborResult};
pub use what_if::{compute_what_if, Scenario, WhatIfResult};

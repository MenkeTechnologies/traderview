//! Stock scanners — Warrior Trading / Zendoo preset filters over daily bars.
//!
//! Each preset answers a yes/no per symbol: "does this symbol match the
//! pattern today?". The scanner-routes layer iterates over a universe of
//! symbols and returns hits with the key stats that justify the match.

use crate::indicators::{closes, highs, lows, sma, volumes};
use crate::models::PriceBar;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ScanHit {
    pub symbol: String,
    pub matched: Vec<&'static str>, // preset names matched
    pub price: f64,
    pub gap_pct: f64,    // open vs prior close
    pub change_pct: f64, // close vs prior close
    pub day_pct: f64,    // close vs open
    pub volume: f64,
    pub rel_volume: f64,    // today vs 20-day avg
    pub hod_dist_pct: f64,  // (close - day_high) / day_high
    pub lod_dist_pct: f64,  // (close - day_low) / day_low
    pub year_high_pct: f64, // close vs 52w high
    pub year_low_pct: f64,  // close vs 52w low
}

impl ScanHit {
    pub fn empty(symbol: &str) -> Self {
        ScanHit {
            symbol: symbol.into(),
            matched: Vec::new(),
            price: 0.0,
            gap_pct: 0.0,
            change_pct: 0.0,
            day_pct: 0.0,
            volume: 0.0,
            rel_volume: 0.0,
            hod_dist_pct: 0.0,
            lod_dist_pct: 0.0,
            year_high_pct: 0.0,
            year_low_pct: 0.0,
        }
    }
}

/// Compute the raw stats for a single symbol given its daily bars (most
/// recent last). Returns None if there's not enough data.
pub fn stats_for(symbol: &str, bars: &[PriceBar]) -> Option<ScanHit> {
    let n = bars.len();
    if n < 2 {
        return None;
    }
    let last = &bars[n - 1];
    let prev = &bars[n - 2];
    let opens: Vec<f64> = bars.iter().map(|b| dec(b.open)).collect();
    let cs = closes(bars);
    let hi = highs(bars);
    let lo = lows(bars);
    let vol = volumes(bars);
    let price = cs[n - 1];
    let open = opens[n - 1];
    let prev_close = cs[n - 2];
    let day_high = hi[n - 1];
    let day_low = lo[n - 1];
    let vol_today = vol[n - 1];
    let avg_vol = sma(&vol, 20.min(n))
        .last()
        .and_then(|x| *x)
        .unwrap_or(vol_today.max(1.0));
    let year_high = hi[n.saturating_sub(252)..]
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let year_low = lo[n.saturating_sub(252)..]
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);

    let pct = |a: f64, b: f64| if b > 0.0 { (a - b) / b * 100.0 } else { 0.0 };
    let _ = last;
    let _ = prev; // future use
    Some(ScanHit {
        symbol: symbol.into(),
        matched: Vec::new(),
        price,
        gap_pct: pct(open, prev_close),
        change_pct: pct(price, prev_close),
        day_pct: pct(price, open),
        volume: vol_today,
        rel_volume: if avg_vol > 0.0 {
            vol_today / avg_vol
        } else {
            0.0
        },
        hod_dist_pct: if day_high > 0.0 {
            (price - day_high) / day_high * 100.0
        } else {
            0.0
        },
        lod_dist_pct: if day_low > 0.0 {
            (price - day_low) / day_low * 100.0
        } else {
            0.0
        },
        year_high_pct: pct(price, year_high),
        year_low_pct: pct(price, year_low),
    })
}

fn dec(d: rust_decimal::Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    PremarketGappers, // gap_pct >= 5% (up or down)
    MomentumMovers,   // change_pct >= 5% and rel_volume >= 2.0
    HighOfDay,        // |hod_dist_pct| < 0.5%
    LowFloatRunners,  // change_pct >= 10% and rel_volume >= 5.0 (proxy for low float)
    Pct52wHigh,       // within 1% of 52w high
    Pct52wLow,        // within 1% of 52w low
    VolumeSurge,      // rel_volume >= 3.0
    Breakdown,        // change_pct <= -5%
    Breakout,         // close above 20-day high
    OversoldBounce,   // close > yesterday close AND yesterday was -5% or worse
    // === Batch added presets ===
    GapAndGo,         // gap up >= 3% AND close above open AND closed near HOD (+volume)
    GapAndFade,       // gap up >= 3% BUT close < open (fade) AND closed near LOD
    InsideDayLow,     // close <= day_low + 0.5% — could break down tomorrow
    InsideDayHigh,    // close near HOD AND barely off prev close (coiling at extreme)
    RangeContractionDay, // tight range vs avg: day_pct + gap_pct both near zero, rel_vol low
    DistributionDay,  // close down >= 2% on rel_volume >= 1.5x
    AccumulationDay,  // close up >= 2% on rel_volume >= 1.5x
    NearYearHighLowVol, // within 1% of 52w high BUT rel_volume < 1 (no real buying interest)
    InsideDaySqueeze,   // tight day + low volume + close not near either extreme (compression)
    LowVolSqueeze,      // very quiet bar: rel_volume < 0.5 AND |day_pct| < 1
    CoilingSqueeze,     // change near zero (|change_pct| < 1) AND quiet volume (< 0.7×) AND narrow gap
    MidRangeSqueeze,    // far from both 52w extremes (year_high_pct < -10 AND year_low_pct > 10) AND quiet day
    BracketSqueeze,     // very tight day_pct (<0.5) AND narrow distance to HOD/LOD (<0.5) — coiled spring
    DojiSqueeze,        // change_pct near zero AND tight day AND no gap — perfect equilibrium bar
    GapFillSqueeze,     // gap opened ≥1% AND day_pct collapsed (<0.5) AND quiet volume — failed continuation
    EndOfRangeSqueeze,  // close hugging mid of intraday range (HOD≈LOD distance) AND tight day_pct
    PreBreakoutSqueeze, // near 52w high (year_high_pct ≥ -3) AND very tight day_pct AND low rel_volume
    PreBreakdownSqueeze,// near 52w low (year_low_pct ≤ 3) AND very tight day_pct AND low rel_volume
    SymmetricSqueeze,   // identical HOD/LOD distances AND change near zero AND gap near zero AND quiet
    OpenCloseSqueeze,   // |day_pct| < 0.3 (close near open) AND quiet vol — open/close almost equal
    TightHodSqueeze,    // pressed to HOD (<0.3%) but change < 1% AND quiet vol — wound spring at high
    TightLodSqueeze,    // pressed to LOD (<0.3%) but change > -1% AND quiet vol — wound spring at low
    NoGapNoChangeSqueeze, // gap≈0 + change≈0 + tight day + quiet vol — overnight + intraday total stall
    QuietTickSqueeze,    // rel_volume < 0.3 AND day_pct.abs() < 0.5 — extreme apathy bar
    NarrowGapPostMomentum, // |gap|<0.3 after |change|>=3 prior — post-trend rest day (proxy: gap & change opposite signs near zero)
    DistantExtremesSqueeze, // far from BOTH 52w high AND 52w low (>=20% each side) AND quiet
    BalancedDriftSqueeze,   // tiny gap + slow drift (|change|<0.5) + close mid-day + quiet
    PennyMoveSqueeze,       // |change| < 0.05 (sub-tick close) — frozen tape
    DryUpSqueeze,           // rel_volume < 0.4 AND |change| < 1.5 AND |gap| < 0.5 — supply/demand exhaustion
    UpperRangeSqueeze,      // close in upper third (lod_dist > 2× hod_dist) AND tight day AND quiet
    LowerRangeSqueeze,      // close in lower third (hod_dist > 2× lod_dist) AND tight day AND quiet
    GapReversalSqueeze,     // gap and change opposite signs AND |day_pct| < 0.5 AND quiet — fade trapped
    Pct52wMidSqueeze,       // year_high_pct between -15 and -5 AND year_low_pct between 5 and 15 AND tight day — true mid-range coiling
    DeepDiscountSqueeze,    // year_high_pct ≤ -30 AND |day_pct| < 1 AND quiet — basing far below highs
    FlatRangeQuietSqueeze,  // |day_pct| < 0.2 AND |gap| < 0.2 AND |change| < 0.5 AND rel_volume < 0.5 — total stall
    NearAthQuietSqueeze,    // ≤ -1% from 52w high AND rel_volume < 0.6 AND |day_pct| < 1 — quiet at the top
    NearAtlQuietSqueeze,    // ≤ 1% from 52w low  AND rel_volume < 0.6 AND |day_pct| < 1 — quiet at the bottom
    SilentBreakoutSetup,    // hod_dist < 0.5 AND day_pct < 0.5 AND rel_volume < 0.7 AND year_high_pct >= -5 — quiet edge of multi-month resistance
    SilentBreakdownSetup,   // lod_dist < 0.5 AND day_pct < 0.5 AND rel_volume < 0.7 AND year_low_pct <= 5  — quiet edge of multi-month support
    GapDownNoFollowSqueeze, // gap_pct <= -1 AND change_pct >= -0.5 AND day_pct.abs() < 0.5 — gap down failing to extend
    GapUpNoFollowSqueeze,   // gap_pct >=  1 AND change_pct <=  0.5 AND day_pct.abs() < 0.5 — gap up failing to extend
    UnchVolDryUpSqueeze,    // |change| < 0.1 AND rel_volume < 0.5 — unchanged on dried-up volume
    NarrowAfterTrendSqueeze,// |day_pct| < 0.5 AND |change| >= 5 — narrow day after a big prior move
    DeadCenterSqueeze,      // hod_dist AND lod_dist within 0.4 of each other AND |change| < 0.3 AND rel_vol < 0.7
    AnchorDriftSqueeze,     // |day_pct| < 1 AND |change| < 1 AND |gap| < 1 AND rel_volume < 0.8 — generic light-day setup
    PostGapFillSqueeze,     // gap and change opposite signs AND |change| < |gap| / 2 AND day_pct.abs() < 0.5 — gap getting filled then stalling
    PostSpikeQuietSqueeze,  // |change_pct| > 2 AND day_pct.abs() < 0.3 AND rel_volume < 0.6 — quiet day after a spike
    HighSqueezeBracket,     // tight HOD distance (<1) AND tight LOD distance (<1) AND year_high_pct >= -3 — both ends near top
    LowSqueezeBracket,      // tight HOD distance (<1) AND tight LOD distance (<1) AND year_low_pct <= 3  — both ends near bottom
    HighRelVolStallSqueeze, // rel_volume >= 1.5 AND |change_pct| < 0.3 AND |day_pct| < 0.5 — busy volume but no price move
    SlightLeanLongSqueeze,  // change_pct between 0.2 and 1 AND rel_vol < 0.8 AND day_pct.abs() < 0.6 — quiet drift higher
    SlightLeanShortSqueeze, // change_pct between -1 and -0.2 AND rel_vol < 0.8 AND day_pct.abs() < 0.6 — quiet drift lower
    GapWithChangeMatchSqueeze, // gap and change SAME sign small magnitude (<0.5) + tight day + quiet — small gap held flat
    SlackBetweenExtremesSqueeze, // hod_dist + lod_dist > 4 (close mid of a wide-ish range) AND |change| < 0.5 — wide range but no decision
    PivotPinSqueeze,             // day_pct.abs() < 0.3 AND hod_dist.abs() < 1 AND lod_dist.abs() < 1 — close pinned to open with tight extremes
    EvenSidesSqueeze,            // gap and change opposite sign AND each |x| < 1 AND tight day AND quiet — overnight + intraday cancel out
    InsideQuarterDaySqueeze,     // day_pct.abs() < 0.25 AND change_pct.abs() < 1 AND rel_volume < 0.8 — barely-moving inside bar
    EvenVolumeQuietSqueeze,      // rel_volume between 0.9 and 1.1 AND |day_pct| < 0.3 AND |change_pct| < 0.5 — average vol but no move
    TightCoilHighSqueeze,        // year_high_pct >= -2 AND day_pct.abs() < 0.5 AND |change_pct| < 0.8 AND hod_dist.abs() < 1 — coiled near all-time/52w high
    TightCoilLowSqueeze,         // year_low_pct <= 2 AND day_pct.abs() < 0.5 AND |change_pct| < 0.8 AND lod_dist.abs() < 1 — coiled near 52w low
    EvenWidthSqueeze,            // hod_dist between 1 and 2 AND lod_dist between 1 and 2 AND |change_pct| < 0.5 AND rel_volume < 0.9 — evenly distributed range
    SmallGapNoFollowSqueeze,     // |gap_pct| between 0.3 and 0.8 AND |change_pct| < 0.3 AND quiet — small gap that fades to flat
    HoldingHighsSqueeze,         // change_pct >= 0 AND change_pct < 1 AND hod_dist.abs() < 0.5 AND rel_volume < 1.2 AND year_high_pct >= -5 — closing at HOD without explosion
    HoldingLowsSqueeze,          // change_pct <= 0 AND change_pct > -1 AND lod_dist.abs() < 0.5 AND rel_volume < 1.2 AND year_low_pct <= 5 — closing at LOD without panic
    StableMidSqueeze,            // 30% < (1 - year_high_pct.abs() / (year_high_pct.abs() + year_low_pct.abs())) < 70% — true 30-70 from top, tight day
    LeanGapMatchSqueeze,         // gap & change same sign + each between 0.5 and 1.5 + tight day + quiet — modest opening gap held
    LongShadowQuietSqueeze,      // (hod_dist.abs() + lod_dist.abs()) > 6 AND day_pct.abs() < 1 AND rel_volume < 0.9 — long-shadow doji on quiet vol
    ChangeNoDayPctSqueeze,       // |change_pct| >= 1 AND |day_pct| < 0.2 AND quiet — overnight move with no intraday follow-through
    DayPctNoChangeSqueeze,       // |day_pct| >= 1 AND |change_pct| < 0.2 AND quiet — intraday wiggle but closes near prior close
    HotDryUpSqueeze,             // year_high_pct >= -1 AND rel_volume < 0.5 AND |day_pct| < 0.5 — at 52w high with dried up volume
    ColdDryUpSqueeze,            // year_low_pct <= 1 AND rel_volume < 0.5 AND |day_pct| < 0.5 — at 52w low with dried up volume
    HighVolGapFadeSqueeze,       // |gap| >= 1 AND change_pct opposite to gap AND |day_pct| < 0.5 AND rel_volume >= 1.2 — high-vol gap reversal
    NearZeroChangeQuietSqueeze,  // |change_pct| < 0.5 AND |gap_pct| < 0.5 AND |day_pct| < 1 AND rel_volume < 0.7 — chop-and-rest quiet bar
    SilentInsideSqueeze,         // day_pct.abs() < 0.4 AND change_pct.abs() < 0.4 AND gap_pct.abs() < 0.4 AND rel_volume < 0.7 — true tri-quiet inside bar
    HighVolNoMoveSqueeze,        // rel_volume >= 2 AND |change_pct| < 0.5 AND |day_pct| < 0.7 — heavy distribution-without-move bar
    ChangeButLodNearbySqueeze,   // change_pct >= 1 AND lod_dist.abs() < 1 — up day but close is back near LOD (failed)
    ChangeButHodNearbySqueeze,   // change_pct <= -1 AND hod_dist.abs() < 1 — down day but close is back near HOD (bullish reversal)
    GapAndCloseAtHodSqueeze,     // gap >= 0.5 AND hod_dist.abs() < 0.5 AND day_pct >= 0 — gap up + close at HOD (consolidation high)
    GapAndCloseAtLodSqueeze,     // gap <= -0.5 AND lod_dist.abs() < 0.5 AND day_pct <= 0 — gap down + close at LOD (consolidation low)
    LongInsideQuietSqueeze,      // hod_dist + lod_dist between 2 and 4 AND day_pct.abs() < 0.5 AND rel_volume < 0.8 — wider-than-tight inside bar
    TripleZeroSqueeze,           // gap_pct.abs() < 0.1 AND change_pct.abs() < 0.1 AND day_pct.abs() < 0.1 — gap, change, AND day all near zero
    Pct52wQuarterFromHighSqueeze, // year_high_pct between -25 and -15 AND day_pct.abs() < 0.7 AND rel_volume < 0.9 — quarter-from-high resting
    Pct52wQuarterFromLowSqueeze,  // year_low_pct between 15 and 25 AND day_pct.abs() < 0.7 AND rel_volume < 0.9 — quarter-from-low resting
    NoExtremeAndQuietSqueeze,    // year_high_pct <= -5 AND year_low_pct >= 5 AND rel_volume < 0.7 AND |day_pct| < 0.8 — away from extremes + quiet
    SmallChangeNarrowGapSqueeze, // change_pct between 0.5 and 1 AND |gap_pct| < 0.3 AND |day_pct| < 0.5 AND rel_volume < 0.9 — modest move + tight day + quiet
    BigRangeNoCommitSqueeze,     // hod_dist + lod_dist > 6 AND change_pct.abs() < 0.5 AND rel_volume < 1.5 — wide range, no commit (battle bar)
    EvenSwingSqueeze,            // hod_dist.abs() between 1 and 3 AND lod_dist.abs() between 1 and 3 AND day_pct.abs() < 1 — small balanced swing
    NoMoveAtMidSqueeze,          // |change_pct| < 0.2 AND hod_dist.abs() > 1 AND lod_dist.abs() > 1 — close pinned mid with both extremes far
    BarelyMovingHighSqueeze,     // year_high_pct >= -8 AND |day_pct| < 0.3 AND change_pct.abs() < 0.5 AND rel_volume < 0.9 — near top quiet drift
    BarelyMovingLowSqueeze,      // year_low_pct <= 8 AND |day_pct| < 0.3 AND change_pct.abs() < 0.5 AND rel_volume < 0.9 — near bottom quiet drift
    MicroRangeSqueeze,           // hod_dist.abs() < 0.2 AND lod_dist.abs() < 0.2 — close pinned to BOTH HOD and LOD (zero range)
    LowVolGapHoldSqueeze,        // |gap_pct| >= 0.5 AND |change_pct| < |gap_pct| / 4 AND rel_volume < 0.8 — gap holds with quiet volume
    HighVolGapHoldSqueeze,       // |gap_pct| >= 0.5 AND |change_pct| < |gap_pct| / 4 AND rel_volume >= 1.5 — gap holds with high volume (institutional accumulation)
    UpsideAttemptedSqueeze,      // hod_dist.abs() >= 1 AND lod_dist.abs() < 0.5 AND change_pct < 0 — tried up, settled at lows (bear control)
    DownsideAttemptedSqueeze,    // lod_dist.abs() >= 1 AND hod_dist.abs() < 0.5 AND change_pct > 0 — tried down, settled at highs (bull control)
    TightGapSmallChangeSqueeze,  // |gap_pct| < 0.2 AND change_pct between -2 and 2 AND day_pct.abs() < 0.5 — slow drift with no overnight surprise
    Pct52wMidWideRangeSqueeze,   // year_high_pct between -10 and -5 AND year_low_pct between 5 and 10 AND (hod_dist+lod_dist)>3 — at exact 52w mid with wide intraday
    InsideAndCoiledSqueeze,      // day_pct.abs() < 0.6 AND hod_dist.abs() < 0.6 AND lod_dist.abs() < 0.6 AND rel_volume < 0.8 — all 3 constraints tight + quiet
    Pct52wHighBreathSqueeze,     // year_high_pct >= -1 AND day_pct.abs() < 0.4 AND |change_pct| < 0.4 — taking a breath right at 52w high
    Pct52wLowBreathSqueeze,      // year_low_pct <= 1 AND day_pct.abs() < 0.4 AND |change_pct| < 0.4 — taking a breath right at 52w low
    GapAroundCloseSqueeze,       // |gap_pct| < 0.4 AND change_pct.abs() < 0.4 AND day_pct.abs() < 1.5 AND rel_volume < 1.0 — slow drift with gap & change both small
    TightCloseSplitSqueeze,      // hod_dist between 0.5 and 1.5 AND lod_dist between 0.5 and 1.5 AND day_pct.abs() < 0.5 — close exactly middle of small range
    HiVolNoExtremeSqueeze,       // rel_volume >= 2 AND hod_dist > 1 AND lod_dist > 1 AND change_pct.abs() < 1 — heavy vol but no breakout (rotation)
    TinyMoveWithGapSqueeze,      // |gap_pct| between 0.5 and 1.5 AND |change_pct| < 0.5 AND day_pct.abs() < 0.5 — gap held but no further move
    LowVolatilityGreenSqueeze,   // change_pct > 0 AND change_pct < 1 AND day_pct.abs() < 0.5 AND rel_volume < 0.5 — quiet up day with dry volume
    LowVolatilityRedSqueeze,     // change_pct < 0 AND change_pct > -1 AND day_pct.abs() < 0.5 AND rel_volume < 0.5 — quiet down day with dry volume
    GapAlignsChangeSqueeze,      // gap_pct.signum() == change_pct.signum() AND (|gap| + |change|) < 1.5 AND day_pct.abs() < 0.5 — small aligned move
    UnaffectedGapSqueeze,        // |gap_pct| >= 0.3 AND |gap_pct| <= 1 AND day_pct.abs() < 0.3 AND |change_pct - gap_pct| < 0.3 — gap simply transferred without intraday motion
    StackedClosesSqueeze,        // hod_dist.abs() < 0.5 AND lod_dist.abs() < 0.5 AND |day_pct| < 0.5 AND |change_pct| < 1 — close, HOD, LOD, open all stacked
    PullbackToMidSqueeze,        // change_pct between -2 and -0.5 AND hod_dist.abs() > 1.5 AND year_high_pct >= -10 — orderly pullback from highs
    BounceFromMidSqueeze,        // change_pct between 0.5 and 2 AND lod_dist.abs() > 1.5 AND year_low_pct <= 10 — orderly bounce from lows
    NarrowGapHotCloseSqueeze,    // |gap_pct| < 0.2 AND year_high_pct >= -2 AND day_pct.abs() < 0.5 — no-gap close at 52w high
    NarrowGapColdCloseSqueeze,   // |gap_pct| < 0.2 AND year_low_pct <= 2 AND day_pct.abs() < 0.5 — no-gap close at 52w low
    AbsorptionUpSqueeze,         // change_pct 0.5-2 AND rel_volume >= 2 AND day_pct.abs() < 1 — heavy buying absorbed without breakout
    AbsorptionDownSqueeze,       // change_pct -2 to -0.5 AND rel_volume >= 2 AND day_pct.abs() < 1 — heavy selling absorbed without breakdown
    StallAtMidSqueeze,           // year_high_pct -40 to -60 (mid of 52w range) AND |change_pct| < 0.5 AND rel_volume < 0.8 — deep mid-range stall
    NoCloseDecisionSqueeze,      // hod_dist & lod_dist 0.4-0.6 AND day_pct.abs() < 0.3 — close exactly equidistant from extremes
    GapInsideRangeSqueeze,       // |gap_pct| < 0.5 AND hod_dist.abs() < 1 AND lod_dist.abs() < 1 AND rel_volume < 0.8 — small gap fully contained inside narrow range
    SubpointMoveSqueeze,         // |change_pct| < 0.05 AND |day_pct| < 0.05 — essentially zero net move
    NoVolNoMoveSqueeze,          // rel_volume < 0.3 AND change_pct.abs() < 0.3 AND day_pct.abs() < 0.3 — both vol and price asleep
    VolWithoutChangeSqueeze,     // rel_volume >= 1.5 AND change_pct.abs() < 0.2 AND day_pct.abs() < 0.5 — vol arrives but price doesn't move
    TickInsideOpenSqueeze,       // |day_pct| < 0.15 AND |change_pct| < 0.5 AND |gap_pct| < 0.3 — closing within a tick of open
    Pct52wExactHalfSqueeze,      // year_high_pct -55 to -45 AND year_low_pct 45 to 55 AND |change_pct| < 0.5 — sitting at exact 52w midpoint
    UnchangedOnVolumeSqueeze,    // |change_pct| < 0.1 AND rel_volume >= 1 — totally unchanged on at-or-above-average volume
    WideHodNarrowLodSqueeze,     // hod_dist.abs() >= 2 AND lod_dist.abs() < 0.5 AND change_pct < 0 — high failed, close pinned to low
    NarrowHodWideLodSqueeze,     // hod_dist.abs() < 0.5 AND lod_dist.abs() >= 2 AND change_pct > 0 — low failed, close pinned to high
    PerfectBalanceSqueeze,       // |hod_dist - lod_dist| < 0.1 AND |change_pct| < 0.3 — mathematically balanced bar
    LowVolHotZoneSqueeze,        // year_high_pct >= -5 AND rel_volume < 0.4 — at 52w high zone with very low vol (institutions absent)
    LowVolColdZoneSqueeze,       // year_low_pct <= 5 AND rel_volume < 0.4 — at 52w low zone with very low vol (no panic, no buying)
    DriftHigherSqueeze,          // change_pct > 0 AND change_pct < 2 AND day_pct > 0 AND day_pct < 1 AND rel_volume < 0.9 — slow grinding-up day
    DriftLowerSqueeze,           // change_pct < 0 AND change_pct > -2 AND day_pct < 0 AND day_pct > -1 AND rel_volume < 0.9 — slow grinding-down day
    ExtremeQuietSqueeze,         // rel_volume < 0.2 AND change_pct.abs() < 0.5 AND day_pct.abs() < 0.5 AND gap_pct.abs() < 0.2 — extreme quiet on all axes
    PinnedToOpenSqueeze,         // day_pct.abs() < 0.05 AND hod_dist.abs() < 1 AND lod_dist.abs() < 1 — close ≈ open + narrow range
    BigGapSmallDaySqueeze,       // |gap_pct| >= 2 AND day_pct.abs() < 0.5 AND |change_pct| < 0.5 — big overnight gap fully absorbed
    PostCrashSqueeze,            // change_pct <= -3 AND day_pct.abs() < 0.5 AND rel_volume < 1 — quiet stabilization after a crash
    PostSpikeStabilizeSqueeze,   // change_pct >= 3 AND day_pct.abs() < 0.5 AND rel_volume < 1 — quiet stabilization after a spike
    TightWithSmallGapSqueeze,    // |gap_pct| < 0.5 AND |change_pct| between 0.3 and 0.8 AND |day_pct| < 0.4 — modest move on small gap, tight day
    BigVolWithTinyChangeSqueeze, // rel_volume >= 3 AND change_pct.abs() < 0.1 — heavy volume but virtually zero change
    QuietExpansionSqueeze,       // hod_dist + lod_dist 2-4 AND change_pct.abs() < 0.2 AND rel_volume < 0.7 — modest range, no net move, quiet
    InsideBarHighSqueeze,        // hod_dist.abs() < 1.5 AND lod_dist.abs() < 1.5 AND year_high_pct >= -2 — narrow inside bar at 52w high
    InsideBarLowSqueeze,         // hod_dist.abs() < 1.5 AND lod_dist.abs() < 1.5 AND year_low_pct <= 2 — narrow inside bar at 52w low
    FlatGapInsideRangeSqueeze,   // gap_pct.abs() < 0.1 AND hod_dist + lod_dist < 2 — no gap and very narrow intraday range
    Pct52wEdgeDryUp,             // (year_high_pct >= -2 OR year_low_pct <= 2) AND rel_volume < 0.3 — at 52w extreme with extremely dried-up volume
    NarrowCenterSqueeze,         // hod_dist between 0.5 and 1 AND lod_dist between 0.5 and 1 AND day_pct.abs() < 0.5 — close centered in a slim range
    LopsidedQuietSqueeze,        // hod_dist.abs() < 0.5 OR lod_dist.abs() < 0.5 (close pinned to one side) AND rel_volume < 0.5 AND |day_pct| < 0.5 — extreme-pin with quiet vol
    SilentLeaderSqueeze,         // year_high_pct >= -3 AND year_low_pct >= 50 AND |day_pct| < 0.5 — leader at top of 52w range, taking a quiet day
    SilentLaggardSqueeze,        // year_low_pct <= 3 AND year_high_pct <= -50 AND |day_pct| < 0.5 — laggard at bottom of 52w range, taking a quiet day
    NearVwapQuietSqueeze,        // |day_pct| < 0.3 AND |change_pct| < 0.3 AND rel_volume < 0.8 — close near VWAP-ish (open) on quiet vol
    BarelyMovingMidSqueeze,      // year_high_pct -50 to -30 AND |day_pct| < 0.3 AND rel_volume < 0.8 — quiet stall in middle of 52w range
    Pct52wThirdFromHighSqueeze,  // year_high_pct -33 to -20 AND |day_pct| < 0.5 AND rel_volume < 0.9 — one-third off 52w high
    Pct52wThirdFromLowSqueeze,   // year_low_pct 20 to 33 AND |day_pct| < 0.5 AND rel_volume < 0.9 — one-third off 52w low
    HighRangeNoChangeSqueeze,    // hod_dist + lod_dist > 5 AND |change_pct| < 0.5 AND rel_volume >= 1 — wide range, zero net change on average volume
    LowRangeNoChangeSqueeze,     // hod_dist + lod_dist < 1 AND |change_pct| < 0.5 — very tight range with no net change
    LowVolumeUpDaySqueeze,       // change_pct 1-3 AND rel_volume < 0.5 — modest up day on dry volume (no participation)
    LowVolumeDownDaySqueeze,     // change_pct -3 to -1 AND rel_volume < 0.5 — modest down day on dry volume (no panic)
    HighVolumeUpDayNoExtreme,    // change_pct 1-2 AND rel_volume >= 2 AND hod_dist.abs() > 0.5 — up day on heavy volume but didn't push HOD
    HighVolumeDownDayNoExtreme,  // change_pct -2 to -1 AND rel_volume >= 2 AND lod_dist.abs() > 0.5 — down day on heavy volume but didn't push LOD
    GapUpFadeToFlat,             // gap_pct > 2 AND change_pct.abs() < 0.5 — gapped up overnight but unchanged on the day (full fade)
    GapDownReclaimToFlat,        // gap_pct < -2 AND change_pct.abs() < 0.5 — gapped down overnight but unchanged on the day (full reclaim)
    GapUpHeldGreen,              // gap_pct > 2 AND change_pct > gap_pct AND rel_volume >= 1 — gap held + extended on participation (continuation squeeze)
    GapDownHeldRed,              // gap_pct < -2 AND change_pct < gap_pct AND rel_volume >= 1 — gap-down extended lower on participation (continuation squeeze)
    GapUpHalfFade,               // gap_pct > 2 AND change_pct between 0 and gap_pct*0.5 — gap-up faded to half its overnight move
    GapDownHalfReclaim,          // gap_pct < -2 AND change_pct between gap_pct*0.5 and 0 — gap-down reclaimed half its overnight move
    GapAndGoXl,                  // gap_pct > 3 AND change_pct > 5 AND rel_volume >= 2 — extra-large gap-and-go (strong gap + strong day + heavy vol)
    GapAndCrashXl,               // gap_pct < -3 AND change_pct < -5 AND rel_volume >= 2 — extra-large gap-and-crash (strong gap-down + strong red day + heavy vol)
    GapUpButDayRed,              // gap_pct > 1 AND change_pct < -1 — gapped up overnight but day closed red (failed open)
    GapDownButDayGreen,          // gap_pct < -1 AND change_pct > 1 — gapped down overnight but day closed green (reversal)
    GapUpFlushOnVolume,          // gap_pct > 2 AND change_pct < -2 AND rel_volume >= 2 — failed open with heavy participation (distribution)
    GapDownReversalOnVolume,     // gap_pct < -2 AND change_pct > 2 AND rel_volume >= 2 — gap-down reversal with heavy participation (accumulation)
    Pct52wTopDecileHotVol,       // year_high_pct > -10 AND rel_volume >= 2 — within 10% of 52w high on heavy volume (breakout candidate)
    Pct52wBottomDecileHotVol,    // year_low_pct < 10 AND rel_volume >= 2 — within 10% of 52w low on heavy volume (capitulation candidate)
    Pct52wTopDecileDryVol,       // year_high_pct > -10 AND rel_volume < 0.5 — within 10% of 52w high but dry volume (no demand at the highs)
    Pct52wBottomDecileDryVol,    // year_low_pct < 10 AND rel_volume < 0.5 — within 10% of 52w low but dry volume (no panic at the lows)
    NewHighGreenDay,             // year_high_pct >= 0 AND change_pct >= 1 — printed a new 52w high and closed green (continuation)
    NewLowRedDay,                // year_low_pct <= 0 AND change_pct <= -1 — printed a new 52w low and closed red (continuation)
    NewHighRedDay,               // year_high_pct >= 0 AND change_pct <= -1 — printed a new 52w high then reversed red (failed breakout)
    NewLowGreenDay,              // year_low_pct <= 0 AND change_pct >= 1 — printed a new 52w low then reversed green (failed breakdown)
    NewHighOnHotVol,             // year_high_pct >= 0 AND rel_volume >= 3 — new 52w high on >=3× volume (institutional accumulation)
    NewLowOnHotVol,              // year_low_pct <= 0 AND rel_volume >= 3 — new 52w low on >=3× volume (institutional distribution)
    QuietNearTheTop,             // year_high_pct > -3 AND hod_dist + lod_dist < 1.5 AND rel_volume < 1 — very tight range near 52w high on light vol (coiled spring up)
    QuietNearTheBottom,          // year_low_pct < 3 AND hod_dist + lod_dist < 1.5 AND rel_volume < 1 — very tight range near 52w low on light vol (coiled spring down)
    NoisyNearTheTop,             // year_high_pct > -3 AND hod_dist + lod_dist > 4 AND rel_volume >= 2 — wide range near 52w high on heavy vol (battle for the top)
    NoisyNearTheBottom,          // year_low_pct < 3 AND hod_dist + lod_dist > 4 AND rel_volume >= 2 — wide range near 52w low on heavy vol (battle for the bottom)
    MidRangeChopHotVol,          // hod_dist between 1-3 AND lod_dist between 1-3 AND rel_volume >= 2 — equidistant mid-range on heavy vol (indecision squeeze)
    MidRangeChopDryVol,          // hod_dist between 1-3 AND lod_dist between 1-3 AND rel_volume < 0.5 — equidistant mid-range on dry vol (range-bound digestion)
    CloseNearHodNoBreakout,      // hod_dist.abs() < 0.5 AND change_pct < 1 — closed within 0.5% of HOD but change_pct < 1 (failed thrust)
    CloseNearLodNoBreakdown,     // lod_dist.abs() < 0.5 AND change_pct > -1 — closed within 0.5% of LOD but change_pct > -1 (failed flush)
    CloseNearHodStrongDay,       // hod_dist.abs() < 0.5 AND change_pct > 3 — closed at HOD AND day up >3% (full-send breakout)
    CloseNearLodWeakDay,         // lod_dist.abs() < 0.5 AND change_pct < -3 — closed at LOD AND day down >3% (full-send breakdown)
    InsideRangeNoVolume,         // hod_dist + lod_dist < 2 AND rel_volume < 0.5 — tight inside range on dry vol (NR7-style squeeze)
    OutsideRangeOnVolume,        // hod_dist + lod_dist > 6 AND rel_volume >= 2 — wide outside range on heavy vol (volatility expansion)
    UpDayLowerHigh,              // change_pct > 1 AND hod_dist.abs() > 1 AND lod_dist.abs() < 1 — green day but failed to make HOD (capped advance)
    DownDayHigherLow,            // change_pct < -1 AND lod_dist.abs() > 1 AND hod_dist.abs() < 1 — red day but failed to make LOD (cushioned decline)
    StrongDayBalancedRange,      // change_pct > 3 AND hod_dist.abs() < 1 AND lod_dist.abs() < 1 — strong day with both ends touched (impulsive breakout)
    WeakDayBalancedRange,        // change_pct < -3 AND hod_dist.abs() < 1 AND lod_dist.abs() < 1 — weak day with both ends touched (impulsive breakdown)
    ChannelRideUp,               // change_pct > 1 AND day_pct > 0 AND hod_dist.abs() < 0.5 AND lod_dist.abs() > 3 — close at HOD with LOD far away (one-side day up)
    ChannelRideDown,             // change_pct < -1 AND day_pct < 0 AND lod_dist.abs() < 0.5 AND hod_dist.abs() > 3 — close at LOD with HOD far away (one-side day down)
    PullbackInUptrend,           // year_high_pct > -15 AND change_pct between -3 and 0 AND rel_volume < 1 — minor pullback in uptrend on light vol (textbook continuation buy zone)
    BounceInDowntrend,           // year_low_pct < 15 AND change_pct between 0 and 3 AND rel_volume < 1 — minor bounce in downtrend on light vol (textbook continuation short zone)
    DeepPullbackInUptrend,       // year_high_pct > -25 AND change_pct between -10 and -3 AND rel_volume >= 2 — deep pullback in uptrend on heavy vol (capitulation entry zone)
    DeepBounceInDowntrend,       // year_low_pct < 25 AND change_pct between 3 and 10 AND rel_volume >= 2 — deep bounce in downtrend on heavy vol (rip-and-reject entry zone)
    TightAboveMidStrong,         // hod_dist.abs() + lod_dist.abs() < 1.5 AND day_pct > 0 AND change_pct > 0.5 — tight range above mid (coiled spring; bullish bias)
    TightBelowMidWeak,           // hod_dist.abs() + lod_dist.abs() < 1.5 AND day_pct < 0 AND change_pct < -0.5 — tight range below mid (coiled spring; bearish bias)
    HotVolNoMoveAtHigh,          // year_high_pct > -5 AND change_pct.abs() < 0.5 AND rel_volume >= 2 — heavy churn at 52w high with no net move (distribution candidate)
    HotVolNoMoveAtLow,           // year_low_pct < 5 AND change_pct.abs() < 0.5 AND rel_volume >= 2 — heavy churn at 52w low with no net move (accumulation candidate)
    BigUpGapInsideDay,           // gap_pct > 3 AND hod_dist.abs() < 0.5 AND lod_dist.abs() < 0.5 AND change_pct between 1 and 3 — big gap-up but contained inside-day (consolidation after thrust)
    BigDownGapInsideDay,         // gap_pct < -3 AND hod_dist.abs() < 0.5 AND lod_dist.abs() < 0.5 AND change_pct between -3 and -1 — big gap-down but contained inside-day (consolidation after thrust)
    SteadyUpDryVol,              // change_pct between 0.5 and 2 AND day_pct > 0 AND rel_volume < 0.7 — steady-up day on light volume (low-conviction drift up)
    SteadyDownDryVol,            // change_pct between -2 and -0.5 AND day_pct < 0 AND rel_volume < 0.7 — steady-down day on light volume (low-conviction drift down)
    ImpulsiveUpHotVol,           // change_pct between 2 and 5 AND day_pct > 0 AND rel_volume >= 1.5 — impulsive up day on heavy vol (initiative buying)
    ImpulsiveDownHotVol,         // change_pct between -5 and -2 AND day_pct < 0 AND rel_volume >= 1.5 — impulsive down day on heavy vol (initiative selling)
    ParabolicUp,                 // change_pct > 10 AND rel_volume >= 3 AND hod_dist.abs() < 0.5 — parabolic up: >10% on >=3× vol closing at HOD (capitulation buy / blow-off candidate)
    ParabolicDown,               // change_pct < -10 AND rel_volume >= 3 AND lod_dist.abs() < 0.5 — parabolic down: <-10% on >=3× vol closing at LOD (capitulation sell / panic candidate)
    BlowOffTop,                  // change_pct > 5 AND rel_volume >= 5 AND year_high_pct > -2 — extreme volume + extreme move at the highs (climactic top candidate)
    SellingClimaxBottom,         // change_pct < -5 AND rel_volume >= 5 AND year_low_pct < 2 — extreme volume + extreme move at the lows (selling climax candidate)
    UpDayGapOnlyMove,            // change_pct between 1 and 3 AND change_pct.sub(gap_pct).abs() < 0.3 AND rel_volume < 1 — entire change came from overnight gap; flat day after (gap-and-fade indecision)
    DownDayGapOnlyMove,          // change_pct between -3 and -1 AND change_pct.sub(gap_pct).abs() < 0.3 AND rel_volume < 1 — entire decline came from overnight gap-down; flat day after
    IntradayOnlyGreenDay,        // change_pct > 1 AND gap_pct.abs() < 0.3 AND rel_volume >= 1 — flat open, intraday all-the-work green day (initiative buying continuation)
    IntradayOnlyRedDay,          // change_pct < -1 AND gap_pct.abs() < 0.3 AND rel_volume >= 1 — flat open, intraday all-the-work red day (initiative selling continuation)
    ReversalUpFromOpen,          // gap_pct < -1 AND change_pct > 0 AND rel_volume >= 1.5 AND hod_dist.abs() < 0.5 — gap-down reversed and closed at HOD on heavy vol (powerful reclaim)
    ReversalDownFromOpen,        // gap_pct > 1 AND change_pct < 0 AND rel_volume >= 1.5 AND lod_dist.abs() < 0.5 — gap-up reversed and closed at LOD on heavy vol (powerful failure)
    TrendDayUp,                  // change_pct > 2 AND day_pct > 1 AND rel_volume >= 1.2 AND hod_dist.abs() < 0.5 AND lod_dist.abs() > 2 — trend-day-up: opened low, closed near HOD on heavy vol
    TrendDayDown,                // change_pct < -2 AND day_pct < -1 AND rel_volume >= 1.2 AND lod_dist.abs() < 0.5 AND hod_dist.abs() > 2 — trend-day-down: opened high, closed near LOD on heavy vol
    DoubleBottomCandidate,       // year_low_pct < 5 AND hod_dist.abs() > 1 AND lod_dist.abs() < 0.5 AND change_pct > 0 — touched 52w-low zone but closed higher (double-bottom candidate)
    DoubleTopCandidate,          // year_high_pct > -5 AND lod_dist.abs() > 1 AND hod_dist.abs() < 0.5 AND change_pct < 0 — touched 52w-high zone but closed lower (double-top candidate)
    Pct52wMidZone,               // year_high_pct < -40 AND year_high_pct > -60 AND year_low_pct > 40 AND year_low_pct < 60 — mid-zone of 52w range (no extreme positioning; chop bias)
    Pct52wRangeBreakoutTriggered, // year_high_pct >= 0 AND change_pct > 2 AND rel_volume >= 2 — broke above 52w high range on heavy vol (range-breakout trigger)
    Pct52wRangeBreakdownTriggered, // year_low_pct <= 0 AND change_pct < -2 AND rel_volume >= 2 — broke below 52w low range on heavy vol (range-breakdown trigger)
    Pct52wTightCoil,             // year_high_pct between -10 and -5 AND year_low_pct between 5 and 10 AND hod_dist + lod_dist < 2 — coiled mid-high zone on tight range (decision-zone setup)
    SymmetricTriangle,           // hod_dist + lod_dist < 3 AND change_pct.abs() < 0.5 AND rel_volume between 0.7 and 1.3 — balanced tight range with average vol (symmetric triangle wait)
    NarrowingRangeOnFlat,        // hod_dist between 0.5 and 2 AND lod_dist between 0.5 and 2 AND change_pct.abs() < 0.3 — both wicks small, no net move (narrowing-range setup)
    GapTooFarBigPullback,        // gap_pct > 4 AND change_pct < gap_pct - 3 — gapped up but pulled back >3% from the gap (over-extended fade)
    GapTooFarBigBounce,          // gap_pct < -4 AND change_pct > gap_pct + 3 — gapped down but bounced >3% off the gap (over-extended bounce)
    ChainBreakoutLevel,          // hod_dist.abs() < 0.3 AND lod_dist.abs() > 2 AND change_pct > 1 — close at HOD with broad day-range; breakout above prior level
    ChainBreakdownLevel,         // lod_dist.abs() < 0.3 AND hod_dist.abs() > 2 AND change_pct < -1 — close at LOD with broad day-range; breakdown below prior level
    Pct52wRangePosTop,           // year_high_pct > -20 AND year_low_pct > 30 — position in top half of 52w range (bullish positioning)
    Pct52wRangePosBottom,        // year_high_pct < -50 AND year_low_pct < 30 — position in bottom half of 52w range (bearish positioning)
    HighRangeHighVolStrong,      // hod_dist + lod_dist > 4 AND change_pct > 3 AND rel_volume >= 1.5 — wide-range strong up day on heavy vol (initiative buying day)
    HighRangeHighVolWeak,        // hod_dist + lod_dist > 4 AND change_pct < -3 AND rel_volume >= 1.5 — wide-range weak day on heavy vol (initiative selling day)
    LowRangeLowVolNeutral,       // hod_dist + lod_dist < 1.5 AND change_pct.abs() < 0.5 AND rel_volume < 0.7 — quiet, tight, flat day (balance / observation day)
    AvgRangeAvgVolNeutral,       // hod_dist + lod_dist between 2 and 4 AND change_pct.abs() < 0.5 AND rel_volume between 0.8 and 1.2 — average range/vol with no net move (no-edge day)
    FailedBreakoutHighReclaim,   // year_high_pct > -1 AND hod_dist.abs() > 1 AND change_pct < -1 AND rel_volume >= 1.5 — touched/exceeded 52w high then closed -1% lower on heavy vol (failed breakout)
    FailedBreakdownLowReclaim,   // year_low_pct < 1 AND lod_dist.abs() > 1 AND change_pct > 1 AND rel_volume >= 1.5 — touched/exceeded 52w low then closed +1% higher on heavy vol (failed breakdown)
    HotVolHotGap,                // gap_pct.abs() > 2 AND rel_volume >= 2 — heavy-volume gap day (institutional positioning before open)
    DryVolDryGap,                // gap_pct.abs() < 0.5 AND rel_volume < 0.5 — flat gap on dry volume (no overnight positioning; no day participation)
    OuterEdgePushUp,             // year_high_pct > -10 AND change_pct > 5 AND rel_volume >= 2 — strong push into the top decile on heavy vol (continuation buyers)
    OuterEdgePushDown,           // year_low_pct < 10 AND change_pct < -5 AND rel_volume >= 2 — strong push into the bottom decile on heavy vol (continuation sellers)
    MiddleZoneUpDrift,           // year_high_pct between -50 and -20 AND year_low_pct between 20 and 50 AND change_pct > 0.5 AND rel_volume < 1 — mid-zone drift up on light vol (no-conviction continuation up)
    MiddleZoneDownDrift,         // year_high_pct between -50 and -20 AND year_low_pct between 20 and 50 AND change_pct < -0.5 AND rel_volume < 1 — mid-zone drift down on light vol (no-conviction continuation down)
    MiddleZoneHotVolBreakoutHigh, // year_high_pct between -50 and -20 AND change_pct > 2 AND rel_volume >= 2 — mid-zone breakout up on heavy vol (range-exit conviction)
    MiddleZoneHotVolBreakoutLow, // year_low_pct between 20 and 50 AND change_pct < -2 AND rel_volume >= 2 — mid-zone breakdown on heavy vol (range-exit conviction)
    GapUpSmallButHotVol,         // gap_pct between 0.5 and 1.5 AND rel_volume >= 2 — small overnight gap-up on heavy vol (early-positioning signal)
    GapDownSmallButHotVol,       // gap_pct between -1.5 and -0.5 AND rel_volume >= 2 — small overnight gap-down on heavy vol (early-positioning signal)
    GapUpMediumNeutral,          // gap_pct between 1.5 and 3 AND change_pct between -0.5 and 0.5 AND rel_volume < 1 — medium gap-up but flat day on light vol (gap-and-stall)
    GapDownMediumNeutral,        // gap_pct between -3 and -1.5 AND change_pct between -0.5 and 0.5 AND rel_volume < 1 — medium gap-down but flat day on light vol (gap-and-stall)
    HodReclaimAfterFlush,        // change_pct > 0 AND hod_dist.abs() < 0.5 AND lod_dist.abs() > 2 AND rel_volume >= 1.5 — closed at HOD after touching deep LOD on heavy vol (V-bottom intraday)
    LodFailAfterPush,            // change_pct < 0 AND lod_dist.abs() < 0.5 AND hod_dist.abs() > 2 AND rel_volume >= 1.5 — closed at LOD after touching distant HOD on heavy vol (failed-push intraday)
    HodReclaimFromFlatGap,       // gap_pct.abs() < 0.5 AND hod_dist.abs() < 0.5 AND change_pct > 1 — flat open then closed at HOD with positive change (organic up-day climb)
    LodFailFromFlatGap,          // gap_pct.abs() < 0.5 AND lod_dist.abs() < 0.5 AND change_pct < -1 — flat open then closed at LOD with negative change (organic down-day slide)
    Pct52wTopBoundaryReject,     // year_high_pct between -1 and 0 AND change_pct < -0.5 — touched 52w high boundary but closed lower (rejection from top boundary)
    Pct52wBottomBoundaryReject,  // year_low_pct between 0 and 1 AND change_pct > 0.5 — touched 52w low boundary but closed higher (rejection from bottom boundary)
    Pct52wTopBoundaryAccept,     // year_high_pct between -1 and 0 AND change_pct > 0.5 — touched 52w high boundary and closed higher (acceptance above prior high)
    Pct52wBottomBoundaryAccept,  // year_low_pct between 0 and 1 AND change_pct < -0.5 — touched 52w low boundary and closed lower (acceptance below prior low)
    UpFromBottomSpring,          // year_low_pct < 10 AND change_pct > 5 AND rel_volume >= 2 — strong rally up from the 52w lows on heavy vol (spring reversal)
    DownFromTopUpthrust,         // year_high_pct > -10 AND change_pct < -5 AND rel_volume >= 2 — strong sell-off from the 52w highs on heavy vol (upthrust reversal)
    UpThrustBarReject,           // hod_dist.abs() > 3 AND lod_dist.abs() < 0.5 AND change_pct < -1 AND rel_volume >= 2 — wick high then closed near LOD on heavy vol (textbook upthrust bar)
    DownThrustBarReject,         // lod_dist.abs() > 3 AND hod_dist.abs() < 0.5 AND change_pct > 1 AND rel_volume >= 2 — wick low then closed near HOD on heavy vol (textbook spring bar)
    ExhaustionTopWideRange,      // year_high_pct > -5 AND hod_dist.abs() > 5 AND lod_dist.abs() < 0.5 AND change_pct > 5 AND rel_volume >= 3 — extreme range close-at-HOD into prior high (exhaustion top candidate)
    ExhaustionBottomWideRange,   // year_low_pct < 5 AND lod_dist.abs() > 5 AND hod_dist.abs() < 0.5 AND change_pct < -5 AND rel_volume >= 3 — extreme range close-at-LOD into prior low (exhaustion bottom candidate)
    UpTrendDayWideRange,         // hod_dist.abs() < 0.3 AND lod_dist.abs() > 5 AND change_pct > 3 AND rel_volume >= 2 — strong trend up with wide range; close at HOD on heavy vol (continuation buyers)
    DownTrendDayWideRange,       // lod_dist.abs() < 0.3 AND hod_dist.abs() > 5 AND change_pct < -3 AND rel_volume >= 2 — strong trend down with wide range; close at LOD on heavy vol (continuation sellers)
    SilentSpringNear52wLow,      // year_low_pct < 5 AND change_pct.abs() < 0.5 AND rel_volume < 0.5 AND hod_dist + lod_dist < 1.5 — flat tight bar at 52w low on dry vol (silent spring waiting)
    SilentUpThrustNear52wHigh,   // year_high_pct > -5 AND change_pct.abs() < 0.5 AND rel_volume < 0.5 AND hod_dist + lod_dist < 1.5 — flat tight bar at 52w high on dry vol (silent upthrust waiting)
    GapStrongDayOpenPivot,       // gap_pct >= 1 AND gap_pct <= 3 AND change_pct > 4 AND rel_volume >= 2 — small-to-medium gap + strong day on heavy vol (gap held + accelerated)
    GapWeakDayOpenPivot,         // gap_pct <= -1 AND gap_pct >= -3 AND change_pct < -4 AND rel_volume >= 2 — small-to-medium gap-down + weak day on heavy vol (gap held + accelerated)
    ConvictionBreakoutCombo,     // year_high_pct >= 0 AND hod_dist.abs() < 0.5 AND change_pct > 3 AND rel_volume >= 2.5 — 52w-high broken + closed at HOD + strong day + heavy vol (highest-conviction breakout)
    ConvictionBreakdownCombo,    // year_low_pct <= 0 AND lod_dist.abs() < 0.5 AND change_pct < -3 AND rel_volume >= 2.5 — 52w-low broken + closed at LOD + weak day + heavy vol (highest-conviction breakdown)
    PullbackInsideTrendUp,       // year_high_pct between -20 and -5 AND change_pct between -1 and -0.2 AND rel_volume >= 0.7 AND rel_volume <= 1.3 — small pullback inside an uptrend with avg vol (orderly continuation entry)
    PullbackInsideTrendDown,     // year_low_pct between 5 and 20 AND change_pct between 0.2 and 1 AND rel_volume >= 0.7 AND rel_volume <= 1.3 — small bounce inside a downtrend with avg vol (orderly continuation entry)
    RangeContractionSqueezeHigh, // year_high_pct > -5 AND hod_dist + lod_dist < 1 AND rel_volume < 0.5 AND change_pct.abs() < 0.3 — extreme range contraction at 52w high (mega-squeeze coil)
    RangeContractionSqueezeLow,  // year_low_pct < 5 AND hod_dist + lod_dist < 1 AND rel_volume < 0.5 AND change_pct.abs() < 0.3 — extreme range contraction at 52w low (mega-squeeze coil)
    RangeExpansionAtTopOnVol,    // year_high_pct > -5 AND hod_dist + lod_dist > 6 AND rel_volume >= 2 AND change_pct.abs() < 0.5 — wide-range churn at 52w high with no net move on heavy vol (distribution at top)
    RangeExpansionAtBottomOnVol, // year_low_pct < 5 AND hod_dist + lod_dist > 6 AND rel_volume >= 2 AND change_pct.abs() < 0.5 — wide-range churn at 52w low with no net move on heavy vol (accumulation at bottom)
    GapInsideRangeBalanced,      // gap_pct.abs() < 1 AND hod_dist + lod_dist < 2 AND change_pct.abs() < 0.5 — flat gap + tight range + flat day (multi-day balance candidate)
    GapInsideRangeImpulse,       // gap_pct.abs() < 1 AND hod_dist + lod_dist > 4 AND change_pct.abs() > 2 AND rel_volume >= 1.5 — flat gap but wide impulsive day on heavy vol (intraday breakout from balance)
    OneWickCloseAtMid,           // hod_dist.abs() > 2 AND lod_dist.abs() < 0.5 AND change_pct.abs() < 0.5 — long upper wick but closed near LOD flat (rejection from upper extreme)
    OneWickCloseAtMidDown,       // lod_dist.abs() > 2 AND hod_dist.abs() < 0.5 AND change_pct.abs() < 0.5 — long lower wick but closed near HOD flat (rejection from lower extreme)
    UpperWickGreenDayConfirm,    // hod_dist.abs() > 2 AND lod_dist.abs() < 0.5 AND change_pct > 1 — long upper wick + closed green day (failed reversal; trend continuation)
    LowerWickRedDayConfirm,      // lod_dist.abs() > 2 AND hod_dist.abs() < 0.5 AND change_pct < -1 — long lower wick + closed red day (failed reversal; trend continuation)
    InsideBarTightAtMid,         // hod_dist + lod_dist < 1 AND change_pct.abs() < 0.2 AND rel_volume < 0.8 — extremely tight inside bar at mid (NR4 / NR7 silent compression)
    OutsideBarVolumeBoth,        // hod_dist.abs() > 3 AND lod_dist.abs() > 3 AND rel_volume >= 2 AND change_pct.abs() < 0.5 — outside-bar both extremes touched on heavy vol with no net move (battle for direction)
    LeadingUpDayLightVol,        // change_pct > 2 AND rel_volume < 0.7 AND hod_dist.abs() < 0.5 — strong up close on light volume (leadership without participation; suspect quality)
    LeadingDownDayLightVol,      // change_pct < -2 AND rel_volume < 0.7 AND lod_dist.abs() < 0.5 — strong down close on light volume (leadership without participation; suspect quality)
    SmallChangeOnVolNearHigh,    // year_high_pct > -3 AND change_pct.abs() between 0.5 and 1.5 AND rel_volume >= 1.5 — modest move at 52w high on above-avg vol (top consolidation / distribution prep)
    SmallChangeOnVolNearLow,     // year_low_pct < 3 AND change_pct.abs() between 0.5 and 1.5 AND rel_volume >= 1.5 — modest move at 52w low on above-avg vol (bottom consolidation / accumulation prep)
    BigGapBigVolBigDay,          // gap_pct.abs() > 3 AND change_pct.abs() > 5 AND rel_volume >= 3 — extreme gap + extreme day + extreme vol (all-in conviction trade)
    BigGapNoFollowThrough,       // gap_pct.abs() > 3 AND change_pct.abs() < 1 AND rel_volume < 1 — extreme gap + flat day + dry vol (failed positioning; no follow-through)
    ConfluenceLongSetup,         // gap_pct between -0.5 and 0.5 AND year_low_pct between 5 and 15 AND change_pct between 0.5 and 1.5 AND rel_volume >= 1.2 — flat-open + above 52w low + minor green move + above-avg vol (confluence long setup)
    ConfluenceShortSetup,        // gap_pct between -0.5 and 0.5 AND year_high_pct between -15 and -5 AND change_pct between -1.5 and -0.5 AND rel_volume >= 1.2 — flat-open + below 52w high + minor red move + above-avg vol (confluence short setup)
    NoExtremeDay,                // year_high_pct < -10 AND year_high_pct > -40 AND year_low_pct > 10 AND year_low_pct < 40 AND change_pct.abs() < 0.5 — middle-of-range flat day (no extreme positioning; no edge)
    AcceleratingUpTrend,         // change_pct > 1 AND day_pct > 0 AND year_high_pct > -5 AND rel_volume > 1 — pushing harder + at 52w high + above-avg vol (accelerating uptrend)
    AcceleratingDownTrend,       // change_pct < -1 AND day_pct < 0 AND year_low_pct < 5 AND rel_volume > 1 — pushing harder + at 52w low + above-avg vol (accelerating downtrend)
    DivergencePushFromTop,       // year_high_pct > -3 AND change_pct < -1 AND rel_volume < 0.8 — touched 52w high but closed red on light vol (divergence reject from top)
    DivergencePushFromBottom,    // year_low_pct < 3 AND change_pct > 1 AND rel_volume < 0.8 — touched 52w low but closed green on light vol (divergence reject from bottom)
    PriceFlatVolHotAboveMid,     // hod_dist + lod_dist < 2 AND day_pct > 0 AND rel_volume >= 1.5 AND change_pct.abs() < 0.3 — flat price + above-mid + heavy vol (silent accumulation distribution)
    PriceFlatVolHotBelowMid,     // hod_dist + lod_dist < 2 AND day_pct < 0 AND rel_volume >= 1.5 AND change_pct.abs() < 0.3 — flat price + below-mid + heavy vol (silent distribution)
    SmallChangeOnVolMid,         // year_high_pct between -50 and -20 AND year_low_pct between 20 and 50 AND change_pct.abs() between 0.5 and 1.5 AND rel_volume >= 1.5 — modest move at mid range on heavy vol (mid-range positioning shift)
    HotRollingVolGap,            // gap_pct.abs() > 1.5 AND change_pct.abs() > 1.5 AND rel_volume >= 2 — material gap + day move + heavy vol (institutional positioning + execution)
    SilentDriftGap,              // gap_pct.abs() > 1 AND change_pct.abs() < 0.3 AND rel_volume < 0.7 — material gap but flat day on dry vol (silent overnight repositioning; no daytime conviction)
    UpDayOnDryVolNear52wHigh,    // year_high_pct > -10 AND change_pct > 1 AND rel_volume < 0.7 — push near 52w high on dry vol (suspect breakout candidate)
    DownDayOnDryVolNear52wLow,   // year_low_pct < 10 AND change_pct < -1 AND rel_volume < 0.7 — push near 52w low on dry vol (suspect breakdown candidate)
    UpDayOnHotVolNear52wHigh,    // year_high_pct > -10 AND change_pct > 1 AND rel_volume >= 2 — push near 52w high on heavy vol (high-quality breakout candidate)
    DownDayOnHotVolNear52wLow,   // year_low_pct < 10 AND change_pct < -1 AND rel_volume >= 2 — push near 52w low on heavy vol (high-quality breakdown candidate)
    NarrowDayDryVolMid,          // hod_dist + lod_dist < 1.5 AND change_pct.abs() < 0.3 AND rel_volume < 0.5 AND year_high_pct between -30 and -15 AND year_low_pct between 15 and 30 — silent narrow day in mid-zone (coiled rest day)
    WideDayHotVolMid,            // hod_dist + lod_dist > 5 AND rel_volume >= 2 AND year_high_pct between -30 and -15 AND year_low_pct between 15 and 30 — wide-range churn in mid-zone on heavy vol (rotation breakout candidate)
    HotVolAtMidNoMove,           // year_high_pct between -50 and -20 AND year_low_pct between 20 and 50 AND change_pct.abs() < 0.3 AND rel_volume >= 1.5 — heavy vol at mid range with no net move (silent positioning shift)
    DryVolAtMidNoMove,           // year_high_pct between -50 and -20 AND year_low_pct between 20 and 50 AND change_pct.abs() < 0.3 AND rel_volume < 0.5 — dry vol at mid range with no net move (true equilibrium)
    BigChangeTinyRangeUp,        // change_pct > 2 AND hod_dist + lod_dist < 1 AND rel_volume >= 1.5 — strong up close on tight range (all-the-way trend bar; impressive efficiency)
    BigChangeTinyRangeDown,      // change_pct < -2 AND hod_dist + lod_dist < 1 AND rel_volume >= 1.5 — strong down close on tight range (all-the-way trend bar; impressive efficiency)
    TinyChangeWideRangeOnVol,    // hod_dist + lod_dist > 5 AND change_pct.abs() < 0.5 AND rel_volume >= 2 — wide-range no-net-move on heavy vol (battle bar; reversal candidate)
    TinyChangeWideRangeOnDryVol, // hod_dist + lod_dist > 5 AND change_pct.abs() < 0.5 AND rel_volume < 0.7 — wide-range no-net-move on light vol (failed setup; both sides absent)
    LargeGapModerateMoveHotVol,  // gap_pct.abs() > 3 AND change_pct.abs() between 1.5 and 3 AND rel_volume >= 2 — large gap + moderate day on heavy vol (institutional execution post-gap)
    SmallGapBigMoveHotVol,       // gap_pct.abs() < 0.5 AND change_pct.abs() > 3 AND rel_volume >= 2 — flat gap + big day on heavy vol (intraday-driven trend; no overnight positioning)
    NoVolTrendUp,                // change_pct > 1 AND day_pct > 0 AND rel_volume < 0.4 AND hod_dist.abs() < 0.5 — up close on extreme dry vol (trend without participation; vacuum risk)
    NoVolTrendDown,              // change_pct < -1 AND day_pct < 0 AND rel_volume < 0.4 AND lod_dist.abs() < 0.5 — down close on extreme dry vol (trend without participation; vacuum risk)
    ChurnAtTopDryVol,            // year_high_pct > -3 AND change_pct.abs() < 0.3 AND rel_volume < 0.6 — pinned near 52w high on dry vol (consolidation at highs; bullish setup)
    ChurnAtBottomDryVol,         // year_low_pct < 3 AND change_pct.abs() < 0.3 AND rel_volume < 0.6 — pinned near 52w low on dry vol (consolidation at lows; potential capitulation done)
    HugeGapFlatChange,           // gap_pct.abs() > 5 AND change_pct.abs() < 0.5 AND rel_volume < 1 — huge overnight gap but flat close on avg vol (frozen open; gap-fill candidate)
    NoGapHugeChange,             // gap_pct.abs() < 0.3 AND change_pct.abs() > 5 AND rel_volume >= 1.5 — no overnight gap but huge intraday change on hot vol (intraday-only catalyst)
    ExtremeVolFlatGapFlatDay,    // rel_volume >= 3 AND gap_pct.abs() < 0.3 AND change_pct.abs() < 1 — extreme vol with flat gap and small move (heavy churn no direction; potential trap)
    IlliquidBigGapFlatDay,       // rel_volume < 0.4 AND gap_pct.abs() > 3 AND change_pct.abs() < 1 — big gap on dry vol with flat day (illiquid gap; gap-fill risk)
    OrganicUpDayCloseAtHod,      // gap_pct.abs() < 0.5 AND hod_dist.abs() < 0.3 AND day_pct > 2 AND rel_volume between 0.7 and 1.3 — flat open + close at HOD + strong day on avg vol (organic up-day; no overnight noise)
    OrganicDownDayCloseAtLod,    // gap_pct.abs() < 0.5 AND lod_dist.abs() < 0.3 AND day_pct < -2 AND rel_volume between 0.7 and 1.3 — flat open + close at LOD + weak day on avg vol (organic down-day)
    StrongDayDryVolUp,           // change_pct > 3 AND day_pct > 2 AND rel_volume < 0.5 — strong up day on dry vol (no participation; suspect-quality rally)
    StrongDayDryVolDown,         // change_pct < -3 AND day_pct < -2 AND rel_volume < 0.5 — strong down day on dry vol (no participation; suspect-quality flush)
    TightCoilAtMidRange,         // hod_dist 0.5-1.5 AND lod_dist 0.5-1.5 AND change_pct.abs() < 0.5 AND rel_volume < 0.7 — tight coil at center of intraday range on dry vol (pre-breakout setup)
    WideOutsideRangeDryVol,      // hod_dist + lod_dist > 6 AND rel_volume < 0.6 — wide outside range on dry vol (one-sided liquidation; no follow-through)
    GapHeldAndExtendedUp,        // gap_pct > 1 AND day_pct > 1 AND rel_volume >= 1.5 — gap up + held + extended intraday on vol (continuation buyers)
    GapHeldAndExtendedDown,      // gap_pct < -1 AND day_pct < -1 AND rel_volume >= 1.5 — gap down + held + extended intraday on vol (continuation sellers)
    Pct52wHighBreakoutCloseAtHod,  // year_high_pct > 0 AND day_pct > 1 AND hod_dist.abs() < 0.5 AND rel_volume >= 2 — broke above 52w high + close at HOD + hot vol (textbook breakout)
    Pct52wLowBreakdownCloseAtLod,  // year_low_pct < 0 AND day_pct < -1 AND lod_dist.abs() < 0.5 AND rel_volume >= 2 — broke below 52w low + close at LOD + hot vol (textbook breakdown)
    Pct52wMidHotVolFlat,           // year_high_pct between -55 and -35 AND year_low_pct between 35 and 55 AND change_pct.abs() < 1 AND rel_volume >= 2 — middle of 52w range with hot vol but flat change (decision-point churn)
    Pct52wMidDryVolFlat,           // year_high_pct between -55 and -35 AND year_low_pct between 35 and 55 AND change_pct.abs() < 1 AND rel_volume < 0.5 — middle of 52w range with dry vol (forgotten consolidation)
    VolSpikeNoTrend,               // rel_volume >= 5 AND change_pct.abs() < 0.5 — massive 5×+ vol spike with no net change (climax/exhaustion candidate; true churn)
    VolSpikeOnTrend,               // rel_volume >= 5 AND change_pct.abs() > 3 — massive 5×+ vol spike with big change (institutional execution; trend day)
    TightCoilAtHighDryVol,         // hod_dist.abs() < 0.3 AND lod_dist.abs() < 1 AND change_pct.abs() < 0.5 AND rel_volume < 0.7 AND year_high_pct > -5 — closed at HOD on tight range + dry vol + near 52w high (coil at highs)
    TightCoilAtLowDryVol,          // lod_dist.abs() < 0.3 AND hod_dist.abs() < 1 AND change_pct.abs() < 0.5 AND rel_volume < 0.7 AND year_low_pct < 5 — closed at LOD on tight range + dry vol + near 52w low (coil at lows)
    OrderlyTrendAtHighs,           // change_pct > 0 AND day_pct > 0 AND year_high_pct > -1 AND rel_volume between 0.7 and 1.5 — at 52w high with green intraday + green day on normal vol (orderly trend at highs)
    OrderlyTrendAtLows,            // change_pct < 0 AND day_pct < 0 AND year_low_pct < 1 AND rel_volume between 0.7 and 1.5 — at 52w low with red intraday + red day on normal vol (orderly downtrend at lows)
    HotVolMidRangeChurn,           // rel_volume >= 3 AND hod_dist.abs() > 0.5 AND lod_dist.abs() > 0.5 — hot vol but close not at HOD or LOD (mid-range churn; no commitment)
    DryVolAtExtremeClose,          // rel_volume < 0.4 AND (hod_dist.abs() < 0.3 OR lod_dist.abs() < 0.3) — dry vol but close at one extreme (unconfirmed extreme; thin tape edge)
    DayChangeMismatch,             // change_pct * day_pct < 0 AND change_pct.abs() > 1 AND day_pct.abs() > 1 — change_pct and day_pct opposite signs both >1 (full intraday reversal vs overnight)
    DayChangeAlignedBig,           // change_pct * day_pct > 0 AND change_pct.abs() > 3 AND day_pct.abs() > 3 AND rel_volume >= 1.5 — change_pct and day_pct same-sign big on hot vol (full trend day)
    HugeRangeHotVol,               // hod_dist + lod_dist > 8 AND rel_volume >= 3 — massive intraday range on hot vol (volatility expansion; chaos)
    HugeRangeDryVol,               // hod_dist + lod_dist > 8 AND rel_volume < 0.5 — massive intraday range on dry vol (illiquid swing; one-sided liquidation)
    Pct52wLowHotVolUp,             // year_low_pct < 10 AND change_pct > 3 AND rel_volume >= 2 — near 52w low + big up move + hot vol (basing-to-up; bounce / accumulation candidate)
    Pct52wHighHotVolDown,          // year_high_pct > -10 AND change_pct < -3 AND rel_volume >= 2 — near 52w high + big down move + hot vol (distribution / topping candidate)
    GapHeldNoExtension,            // gap_pct.abs() > 1 AND day_pct.abs() < 0.3 AND rel_volume between 0.7 and 1.5 — gap (up or down) + close ≈ open + avg vol (held the gap, no extension)
    GapPartialFade,                // gap_pct.abs() > 2 AND change_pct * gap_pct > 0 AND change_pct.abs() < gap_pct.abs()/2 AND rel_volume >= 1.2 — gap kept direction but faded > half its move (partial fade)
    YearHighIntradayWeak,          // year_high_pct > -1 AND day_pct < -1 AND rel_volume >= 1.5 — at 52w high but intraday weak (close < open) on hot vol (rejection from high; failed continuation)
    YearLowIntradayStrong,         // year_low_pct < 1 AND day_pct > 1 AND rel_volume >= 1.5 — at 52w low but intraday strong (close > open) on hot vol (reclaim from low; failed continuation down)
    WeakHandsAtHighs,              // year_high_pct > -2 AND change_pct < -0.5 AND day_pct < -0.3 AND rel_volume between 1 and 2 — at 52w high but red day on slightly elevated vol (early weakness; weak hands)
    StrongHandsAtLows,              // year_low_pct < 2 AND change_pct > 0.5 AND day_pct > 0.3 AND rel_volume between 1 and 2 — at 52w low but green day on slightly elevated vol (early strength; strong hands)
    NarrowRangeHotVolSqueeze,       // hod_dist + lod_dist < 1 AND rel_volume >= 3 — narrow range + extreme vol (heavy absorption inside tight range; coiling under pressure)
    WideRangeDryVolDrift,           // hod_dist + lod_dist > 4 AND rel_volume < 0.5 AND change_pct.abs() < 1 — wide range + dry vol + flat change (low-participation swing; hidden fade)
    LeadershipTrendDay,             // year_high_pct > -5 AND change_pct > 2 AND day_pct > 1 AND hod_dist.abs() < 1 AND rel_volume >= 1.5 — near 52w high + big up + green intraday + close near HOD on vol (leadership trend day)
    WorstActorFlushDay,             // year_low_pct < 5 AND change_pct < -2 AND day_pct < -1 AND lod_dist.abs() < 1 AND rel_volume >= 1.5 — near 52w low + big down + red intraday + close near LOD on vol (worst-actor flush)
    GapUpAtYearLow,                 // gap_pct > 2 AND year_low_pct < 5 AND rel_volume >= 1.5 — gap up while still near 52w low on vol (oversold squeeze; mean-reversion buy candidate)
    GapDownAtYearHigh,              // gap_pct < -2 AND year_high_pct > -5 AND rel_volume >= 1.5 — gap down while still near 52w high on vol (sudden distribution; risk-off topping candidate)
    BigUpMidRangeClose,             // change_pct > 3 AND hod_dist.abs() > 1.5 AND lod_dist.abs() > 1.5 AND rel_volume >= 1.5 — big up move but close mid-range on vol (failed to hold extremes; topping action)
    BigDownMidRangeClose,           // change_pct < -3 AND hod_dist.abs() > 1.5 AND lod_dist.abs() > 1.5 AND rel_volume >= 1.5 — big down move but close mid-range on vol (failed flush; basing action)
    HodCloseHotVolFlat,             // hod_dist.abs() < 0.5 AND rel_volume >= 2 AND change_pct.abs() < 0.5 — close at HOD on hot vol but flat change (absorption at highs; climax buy candidate)
    LodCloseHotVolFlat,             // lod_dist.abs() < 0.5 AND rel_volume >= 2 AND change_pct.abs() < 0.5 — close at LOD on hot vol but flat change (absorption at lows; climax sell candidate)
    RisingWedgeCoil,                // hod_dist + lod_dist < 2 AND change_pct > 0 AND day_pct > 0 AND rel_volume < 0.8 — narrow range + green change + green intraday + dry vol (rising wedge consolidation)
    FallingWedgeCoil,               // hod_dist + lod_dist < 2 AND change_pct < 0 AND day_pct < 0 AND rel_volume < 0.8 — narrow range + red change + red intraday + dry vol (falling wedge consolidation)
    BigGapAndExtend,                // gap_pct.abs() > 3 AND change_pct * gap_pct > 0 AND change_pct.abs() > gap_pct.abs() AND rel_volume >= 1.5 — big gap + intraday extends beyond gap on vol (gap-and-extend)
    BigGapAndReverse,               // gap_pct.abs() > 3 AND change_pct * gap_pct < 0 AND change_pct.abs() > gap_pct.abs() AND rel_volume >= 1.5 — big gap + intraday reverses + close past prior close on vol (full gap fade and reverse)
    EfficientMoverHotVol,           // rel_volume >= 1.5 AND change_pct.abs() >= rel_volume * 1.5 — change-per-vol elevated (efficient mover; clean trend day)
    InefficientChurnHotVol,         // rel_volume >= 2 AND change_pct.abs() < rel_volume * 0.3 — hot vol but tiny change relative to vol (inefficient churn; failed trend or absorption)
    GapUpAtMidRange,                // gap_pct > 1 AND year_high_pct between -50 and -20 AND year_low_pct between 20 and 50 AND rel_volume >= 1.5 — gap up while in middle of 52w range on vol (breakout from consolidation)
    GapDownAtMidRange,              // gap_pct < -1 AND year_high_pct between -50 and -20 AND year_low_pct between 20 and 50 AND rel_volume >= 1.5 — gap down while in middle of 52w range on vol (breakdown from consolidation)
    BattleBarHotVol,                // hod_dist + lod_dist > 3 AND change_pct.abs() < 0.3 AND rel_volume >= 2.5 — wide intraday range + flat change + hot vol (battle bar with extreme participation)
    IlliquidSwingDryVol,            // hod_dist + lod_dist > 3 AND change_pct.abs() < 0.3 AND rel_volume < 0.5 — wide intraday range + flat change + dry vol (illiquid swing both directions)
    GapDownIntradayReclaimUp,       // gap_pct < -1 AND hod_dist.abs() < 0.5 AND change_pct > 0.5 — gap down + close at HOD + close above prior (full intraday reclaim + extension)
    GapUpIntradayRejectDown,        // gap_pct > 1 AND lod_dist.abs() < 0.5 AND change_pct < -0.5 — gap up + close at LOD + close below prior (full intraday rejection)
    HotVolModerateChangeFlatDay,    // rel_volume >= 2 AND change_pct.abs() between 1 and 2 AND day_pct.abs() < 0.5 — hot vol + moderate change + flat intraday (gap-driven move, no continuation)
    DryVolModerateChangeFlatDay,    // rel_volume < 0.6 AND change_pct.abs() between 1 and 2 AND day_pct.abs() < 0.5 — dry vol + moderate change + flat intraday (sleepy gap-held move)
    WideRangeAtYearHigh,            // hod_dist + lod_dist > 5 AND year_high_pct > -5 AND rel_volume >= 1.5 — wide intraday range while at 52w high on vol (volatility at the top; potential top)
    WideRangeAtYearLow,             // hod_dist + lod_dist > 5 AND year_low_pct < 5 AND rel_volume >= 1.5 — wide intraday range while at 52w low on vol (volatility at the bottom; potential bottom)
    HotVolGapHeldAndExtended,       // rel_volume >= 2 AND gap_pct.abs() > 2 AND change_pct * gap_pct > 0 AND change_pct.abs() >= gap_pct.abs() * 0.8 — hot vol + big gap + held + ≥80% extension (institutional gap-hold-and-go)
    HotVolGapFadedDeep,             // rel_volume >= 2 AND gap_pct.abs() > 2 AND change_pct * gap_pct < 0 AND change_pct.abs() >= gap_pct.abs() * 0.5 — hot vol + big gap + reversed ≥50% of gap (institutional gap-fade with conviction)
    TightRangeAtYearHigh,           // hod_dist + lod_dist < 1 AND year_high_pct > -3 AND rel_volume between 0.7 and 1.3 — tight range at 52w high on normal vol (consolidation at the top; bullish base)
    TightRangeAtYearLow,            // hod_dist + lod_dist < 1 AND year_low_pct < 3 AND rel_volume between 0.7 and 1.3 — tight range at 52w low on normal vol (basing at the bottom; potential reversal)
    BalancedMidWickHotVol,          // hod_dist between 0.3 and 1.5 AND lod_dist between 0.3 and 1.5 AND rel_volume >= 1.5 — balanced wicks in middle on hot vol (mid-range churn with participation)
    BalancedMidWickDryVol,          // hod_dist between 0.3 and 1.5 AND lod_dist between 0.3 and 1.5 AND rel_volume < 0.6 — balanced wicks in middle on dry vol (sleepy mid-range)
    GapUpHodCloseControlled,        // gap_pct > 0.5 AND hod_dist.abs() < 0.5 AND change_pct > 0 AND rel_volume between 0.7 and 1.5 — gap up + close at HOD + green close + normal vol (gap-and-hold; controlled trend)
    GapDownLodCloseControlled,      // gap_pct < -0.5 AND lod_dist.abs() < 0.5 AND change_pct < 0 AND rel_volume between 0.7 and 1.5 — gap down + close at LOD + red close + normal vol (gap-and-hold; controlled decline)
    AllGreenTightDay,               // change_pct > 0 AND day_pct > 0 AND gap_pct > 0 AND hod_dist + lod_dist < 2 — all green directions + tight range (strong-hands tight up day)
    AllRedTightDay,                 // change_pct < 0 AND day_pct < 0 AND gap_pct < 0 AND hod_dist + lod_dist < 2 — all red directions + tight range (strong-sellers tight down day)
    MicroRangeAtYearHigh,           // hod_dist.abs() < 0.3 AND lod_dist.abs() < 0.3 AND year_high_pct > -3 — micro range at 52w high (zero-range pin at top; topping or pre-breakout)
    MicroRangeAtYearLow,            // hod_dist.abs() < 0.3 AND lod_dist.abs() < 0.3 AND year_low_pct < 3 — micro range at 52w low (zero-range pin at bottom; basing or pre-breakdown)
    ConsolidationBreakUp,           // hod_dist + lod_dist > 3 AND year_high_pct between -20 and -5 AND change_pct > 1 AND rel_volume >= 2 — wide range + mid-upper 52w + green + hot vol (consolidation breaking up)
    ConsolidationBreakDown,         // hod_dist + lod_dist > 3 AND year_low_pct between 5 and 20 AND change_pct < -1 AND rel_volume >= 2 — wide range + mid-lower 52w + red + hot vol (consolidation breaking down)
    HotVolGapHeldFlatChange,        // rel_volume >= 2 AND gap_pct.abs() > 1 AND change_pct.abs() < 0.5 — hot vol + gap + flat change (gap held + heavy participation absorbing both sides)
    DryVolGapHeldFlatChange,        // rel_volume < 0.5 AND gap_pct.abs() > 1 AND change_pct.abs() < 0.5 — dry vol + gap + flat change (gap held + no participation; thin tape)
    AllDirectionsAlignedHotVolUp,   // rel_volume >= 3 AND change_pct > 0 AND day_pct > 0 AND gap_pct > 0 — extreme vol + all up directions (full directional bull bar; conviction long)
    AllDirectionsAlignedHotVolDown, // rel_volume >= 3 AND change_pct < 0 AND day_pct < 0 AND gap_pct < 0 — extreme vol + all down directions (full directional bear bar; conviction short)
    IntradayRecoveryFromGapDown,    // year_high_pct between -30 and -10 AND day_pct > 1 AND gap_pct < -0.5 AND rel_volume >= 1.5 — mid-range from top + green intraday + gap-down + hot vol (intraday recovery from gap-down)
    IntradayRejectionFromGapUp,     // year_low_pct between 10 and 30 AND day_pct < -1 AND gap_pct > 0.5 AND rel_volume >= 1.5 — mid-range from bottom + red intraday + gap-up + hot vol (intraday rejection from gap-up)
    Pct52wMidUpperHotVolDown,       // year_high_pct between -30 and -10 AND change_pct < -3 AND rel_volume >= 2 — mid-upper 52w + big down + hot vol (correction in uptrend)
    Pct52wMidLowerHotVolUp,         // year_low_pct between 10 and 30 AND change_pct > 3 AND rel_volume >= 2 — mid-lower 52w + big up + hot vol (rally in downtrend)
    OrderlyMidRangeRally,           // year_high_pct between -25 and -15 AND year_low_pct between 15 and 25 AND change_pct > 1 AND rel_volume between 0.8 and 1.5 — symmetrically mid + up move + normal vol (orderly rally in middle)
    OrderlyMidRangePullback,        // year_high_pct between -25 and -15 AND year_low_pct between 15 and 25 AND change_pct < -1 AND rel_volume between 0.8 and 1.5 — symmetrically mid + down move + normal vol (orderly pullback in middle)
    StrongBreakoutDay,              // year_high_pct > -5 AND gap_pct > 2 AND change_pct > 2 AND rel_volume >= 2 — near 52w high + gap up + green close + hot vol (strong breakout day)
    StrongBreakdownDay,             // year_low_pct < 5 AND gap_pct < -2 AND change_pct < -2 AND rel_volume >= 2 — near 52w low + gap down + red close + hot vol (strong breakdown day)
    VolSpikeBigGapBigChange,        // rel_volume >= 3 AND gap_pct.abs() > 3 AND change_pct.abs() > 3 — extreme vol + big gap + big change (institutional gap-on-news)
    VolSpikeTinyGapBigChange,       // rel_volume >= 3 AND gap_pct.abs() < 0.3 AND change_pct.abs() > 3 — extreme vol + flat gap + big change (mid-session catalyst; intraday-only news)
    StrongCloseAtHodHotVol,         // hod_dist.abs() < 0.3 AND day_pct > 1 AND rel_volume >= 2 — close at HOD + green intraday + hot vol (strong close; buyers in control)
    WeakCloseAtLodHotVol,           // lod_dist.abs() < 0.3 AND day_pct < -1 AND rel_volume >= 2 — close at LOD + red intraday + hot vol (weak close; sellers in control)
    Pct52wHighDryVolFlat,           // year_high_pct > -3 AND rel_volume < 0.5 AND change_pct.abs() < 0.5 — near 52w high on dry vol with no move (forgotten leadership; consolidation at highs)
    Pct52wLowDryVolFlat,            // year_low_pct < 3 AND rel_volume < 0.5 AND change_pct.abs() < 0.5 — near 52w low on dry vol with no move (forgotten weakness; basing at lows)
    OvernightReversalRepositioning, // change_pct * gap_pct < 0 AND change_pct.abs() > 0.5 AND gap_pct.abs() > 0.5 AND day_pct.abs() < 0.5 — gap and change opposite + flat intraday (overnight repositioning; reversal of overnight bias)
    OrganicIntradayTrendDay,        // gap_pct.abs() < 0.3 AND change_pct.abs() > 2 AND day_pct.abs() > 1 AND rel_volume between 0.7 and 1.3 — no gap + significant change + clear intraday on normal vol (organic intraday trend day)
    TightRangeFlatDayHotVol,        // hod_dist + lod_dist < 1 AND day_pct.abs() < 0.2 AND rel_volume >= 2 — tight range + flat intraday + hot vol (extreme absorption coil)
    TightRangeFlatDayDryVol,        // hod_dist + lod_dist < 1 AND day_pct.abs() < 0.2 AND rel_volume < 0.5 — tight range + flat intraday + dry vol (deep sleep; no participation)
    HodHotVolMicroRange,            // hod_dist_pct.abs() < 0.2 AND 0.5 <= lod_dist_pct.abs() <= 2 AND rel_volume >= 2 — close pinned at HOD with wider intraday range on hot vol (controlled mark-up)
    LodHotVolMicroRange,            // lod_dist_pct.abs() < 0.2 AND 0.5 <= hod_dist_pct.abs() <= 2 AND rel_volume >= 2 — close pinned at LOD with wider intraday range on hot vol (controlled mark-down)
    GapAndGoStrongClose,            // gap_pct > 1 AND change_pct > gap_pct AND hod_dist_pct.abs() < 0.5 — gap up + extended past gap + closing at HOD (gap-and-go follow-through)
    GapAndFadeWeakClose,            // gap_pct > 1 AND change_pct < 0 AND lod_dist_pct.abs() < 0.5 — gap up + reversed below flat + closing at LOD (gap fade / failed breakout)
    InsideDayDryVolCoiled,          // hod_dist + lod_dist < 1.5 AND day_pct.abs() < 0.5 AND rel_volume < 0.7 AND gap_pct.abs() < 0.3 — narrow range + flat + dry vol + no gap (inside-day coil; pre-expansion drift)
    OutsideDayHotVolExpansion,      // hod_dist + lod_dist > 4 AND change_pct.abs() > 2 AND rel_volume >= 1.8 — wide range + significant move + hot vol (outside-day expansion / volatility breakout)
    MidRangeDryVolNoConviction,     // 1 <= hod_dist <= 3 AND 1 <= lod_dist <= 3 AND day_pct.abs() < 0.5 AND rel_volume < 0.7 — closed mid-range + flat day + dry vol (indecision; no conviction either way)
    LowOfYearHotVolPanic,           // year_low_pct < 1 AND rel_volume >= 2 AND change_pct < -2 — close near 52w low + hot vol + red day (capitulation panic candle)
    HighOfYearHotVolEuphoria,       // year_high_pct < 1 AND rel_volume >= 2 AND change_pct > 2 — close near 52w high + hot vol + green day (euphoric breakout candle)
    WideRangeFlatCloseHeavyChurn,   // hod_dist + lod_dist > 4 AND day_pct.abs() < 0.5 AND rel_volume >= 1.8 — wide intraday range + flat close + hot vol (heavy churning; both sides absorbing)
    RangeExpansionDryVol,           // hod_dist + lod_dist > 3 AND rel_volume < 0.7 — wide intraday range on dry volume (thin-liquidity swing; algorithmic or illiquid; small participants creating wide range)
    YearHighGapDownHotVol,          // year_high_pct < 5 AND gap_pct < -2 AND rel_volume >= 1.5 — near 52w high but gapping down on hot vol (distribution from top; topping pattern signal)
    YearLowGapUpHotVol,             // year_low_pct < 5 AND gap_pct > 2 AND rel_volume >= 1.5 — near 52w low but gapping up on hot vol (relief gap; reversal off bottom signal)
    IntradayFakeoutTopReject,       // hod_dist_pct.abs() > 2 AND day_pct < -0.5 AND rel_volume >= 1.5 — significant pullback from HOD + red day + hot vol (intraday failed breakout; top rejection)
    IntradayFakeoutBottomReject,    // lod_dist_pct.abs() > 2 AND day_pct > 0.5 AND rel_volume >= 1.5 — significant bounce from LOD + green day + hot vol (intraday failed breakdown; bottom rejection)
    RangeContractionAfterMove,      // hod_dist + lod_dist < 0.8 AND day_pct.abs() < 0.3 AND change_pct.abs() > 1 AND rel_volume between 0.8 and 1.3 — narrow range + flat close + meaningful change + normal vol (continuation pause / range contraction after move)
    RelativeStrengthBuild,          // change_pct > 1.5 AND rel_volume between 1.0 and 1.5 AND gap_pct.abs() < 0.5 — meaningful gain + slightly elevated vol + no big gap (organic relative-strength build; no catalyst-driven gap)
    RelativeWeaknessBuild,          // change_pct < -1.5 AND rel_volume between 1.0 and 1.5 AND gap_pct.abs() < 0.5 — meaningful drop + slightly elevated vol + no big gap (organic relative-weakness build; no catalyst-driven gap)
    HighVolAbsorbingChange,         // rel_volume >= 2 AND hod_dist + lod_dist < 1.5 AND change_pct.abs() > 1 — hot vol + tight intraday range + meaningful change (volume absorbing in tight range; strong directional pressure being absorbed for likely follow-through)
    LowVolWideRangeAccumulator,     // rel_volume < 0.5 AND hod_dist + lod_dist > 2 AND change_pct.abs() > 1 — dry vol + wider intraday range + meaningful change (low-participation but wide spread; accumulator working orders quietly)
    BullishEngulfingHotVol,         // gap_pct < -0.5 AND change_pct > 1.5 AND day_pct > 1 AND rel_volume >= 1.5 — gap down + reversed strong + closed positive intraday + hot vol (bullish engulfing with volume confirmation)
    BearishEngulfingHotVol,         // gap_pct > 0.5 AND change_pct < -1.5 AND day_pct < -1 AND rel_volume >= 1.5 — gap up + reversed weak + closed negative intraday + hot vol (bearish engulfing with volume confirmation)
    DoubleBottomRetest,             // year_low_pct < 3 AND rel_volume >= 1.2 AND change_pct >= 0 AND day_pct >= 0 — near 52w low + decent vol + non-negative day (potential double-bottom retest forming)
    DoubleTopRetest,                // year_high_pct < 3 AND rel_volume >= 1.2 AND change_pct <= 0 AND day_pct <= 0 — near 52w high + decent vol + non-positive day (potential double-top retest forming)
    LiquiditySweepBothSides,        // hod_dist + lod_dist > 3 AND hod_dist > 1 AND lod_dist > 1 AND day_pct.abs() < 0.3 AND rel_volume >= 1.5 — both extremes visited + flat close + hot vol (raid-both-sides liquidity sweep before closing flat)
    SteadyGrinderNoVolPickup,       // change_pct between 0.5 and 2 AND rel_volume between 0.8 and 1.0 AND day_pct between 0.3 and 1.5 AND gap_pct.abs() < 0.2 — small steady gain + below-average vol + steady intraday + no gap (low-vol grinder; quiet uptrend continuation)
    SteadyDeclinerNoVolPickup,      // change_pct between -2 and -0.5 AND rel_volume between 0.8 and 1.0 AND day_pct between -1.5 and -0.3 AND gap_pct.abs() < 0.2 — small steady decline + below-average vol + steady intraday + no gap (low-vol decliner; quiet downtrend continuation)
    HighVolStallNearHighOfYear,     // year_high_pct < 2 AND rel_volume >= 2 AND day_pct.abs() < 0.5 — near 52w high + hot vol + flat day (high-vol stall at the top; supply meeting demand at resistance)
    HighVolStallNearLowOfYear,      // year_low_pct < 2 AND rel_volume >= 2 AND day_pct.abs() < 0.5 — near 52w low + hot vol + flat day (high-vol stall at the bottom; demand meeting supply at floor)
    OutlierSessionBigMoveBigVol,    // change_pct.abs() > 3 AND rel_volume >= 3 AND hod_dist + lod_dist > 2.5 — really big move + really hot vol + wide range (outlier session; momentum/news/squeeze event)
    EodParabolicAccelerationUp,     // change_pct > 2 AND day_pct > change_pct * 0.7 AND hod_dist_pct.abs() < 0.3 AND rel_volume >= 1.5 — most of move happened intraday + closing at HOD + hot vol (EOD parabolic acceleration up; possible MOC short-covering finale)
    EodParabolicAccelerationDown,   // change_pct < -2 AND day_pct < change_pct * 0.7 AND lod_dist_pct.abs() < 0.3 AND rel_volume >= 1.5 — most of move happened intraday + closing at LOD + hot vol (EOD parabolic acceleration down; possible MOC long-liquidation finale)
    FullSpectrumDayUp,              // change_pct > 1 AND day_pct > 0 AND hod_dist_pct.abs() < 0.5 AND lod_dist_pct.abs() > 1 AND rel_volume >= 1.2 — closed at HOD + visited LOD intraday + green + decent vol (volatile session that traded full range and went up)
    FullSpectrumDayDown,            // change_pct < -1 AND day_pct < 0 AND lod_dist_pct.abs() < 0.5 AND hod_dist_pct.abs() > 1 AND rel_volume >= 1.2 — closed at LOD + visited HOD intraday + red + decent vol (volatile session that traded full range and went down)
    GreenStreakAccumulator,         // change_pct > 0.5 AND day_pct > 0 AND rel_volume >= 1.2 AND gap_pct >= 0 — modest gain + green intraday + decent vol + non-negative gap (steady accumulation day; multi-day green-streak candidate)
    RedStreakDistributor,           // change_pct < -0.5 AND day_pct < 0 AND rel_volume >= 1.2 AND gap_pct <= 0 — modest loss + red intraday + decent vol + non-positive gap (steady distribution day; multi-day red-streak candidate)
    GapDownReclaim,                 // gap_pct < -1 AND change_pct >= 0 AND day_pct > -gap_pct * 0.8 — gap down + reclaimed positive + intraday recovered most of gap (full intraday rotation; gap-down reclaim)
    GapUpFailReclaimed,             // gap_pct > 1 AND change_pct <= 0 AND day_pct < -gap_pct * 0.8 — gap up + faded negative + intraday gave up most of gap (full intraday rotation; gap-up fail)
    MidYearRangeConsolidation,      // year_low_pct > 20 AND year_high_pct > 20 AND change_pct.abs() < 0.5 AND rel_volume < 1 — clearly mid-52w range + flat + sub-avg vol (consolidation; nowhere on the chart)
    AtYearExtremeVolatilityExpansion, // (year_high_pct < 3 OR year_low_pct < 3) AND hod_dist + lod_dist > 3 AND rel_volume >= 1.5 — at either 52w extreme + wide range + decent vol (at-extreme volatility expansion; testing structural level)
    BreakoutFromMidLevels,          // year_high_pct < 10 AND year_low_pct >= 20 AND change_pct > 1 AND rel_volume >= 1.5 — within 10% of 52w high coming from mid-range + decent move + hot vol (breakout candidate from mid-range to upper-zone)
    BreakdownFromMidLevels,         // year_low_pct < 10 AND year_high_pct >= 20 AND change_pct < -1 AND rel_volume >= 1.5 — within 10% of 52w low coming from mid-range + decent drop + hot vol (breakdown candidate from mid-range to lower-zone)
    IntradayStrongerThanGap,        // gap_pct.abs() < 1 AND day_pct.abs() > 1.5 AND change_pct.abs() > 1 AND rel_volume >= 1.2 — small gap + big intraday + decent change + decent vol (intraday energy > overnight; all action during regular session)
    OvernightStrongerThanIntraday,  // gap_pct.abs() > 1.5 AND day_pct.abs() < 0.5 AND change_pct.abs() > 1 — big gap + flat intraday + decent change (overnight dominated; market accepted gap without intraday expansion)
    EfficientMoveLowEffort,         // change_pct.abs() > 1 AND rel_volume < 0.7 AND hod_dist + lod_dist < 1.5 — meaningful change + dry vol + narrow range (efficient move; few prints needed; sleeper trade)
    SignalVsNoiseChurn,             // change_pct.abs() < 0.2 AND rel_volume >= 2 AND hod_dist + lod_dist > 2 — tiny net change + hot vol + wide range visited (signal-vs-noise: lots of activity, no net move; pure noise day)
    GreenCloseRedIntraday,          // change_pct > 0 AND day_pct < 0 AND rel_volume >= 1.5 — green close vs prior + red intraday + hot vol (gap held positive despite intraday erosion; close-of-day mark-up)
    RedCloseGreenIntraday,          // change_pct < 0 AND day_pct > 0 AND rel_volume >= 1.5 — red close vs prior + green intraday + hot vol (gap held negative despite intraday recovery; close-of-day mark-down)
    FullConvictionUpDay,            // gap_pct > 0 AND change_pct > 1 AND day_pct > 0.5 AND hod_dist_pct.abs() < 0.5 AND rel_volume >= 2 — every directional signal aligned up + HOD close + hot vol (full-conviction up day; institutional commitment across session)
    FullConvictionDownDay,          // gap_pct < 0 AND change_pct < -1 AND day_pct < -0.5 AND lod_dist_pct.abs() < 0.5 AND rel_volume >= 2 — every directional signal aligned down + LOD close + hot vol (full-conviction down day; institutional commitment across session)
    YearLowProximityRallyAttempt,   // year_low_pct < 5 AND gap_pct >= 0 AND day_pct > 1 AND change_pct > 0.5 AND rel_volume >= 1 — close near 52w low but rallied intraday with green close + decent vol (rally attempt off the lows; potential trend reversal seedling)
    YearHighProximityFailAttempt,   // year_high_pct < 5 AND gap_pct <= 0 AND day_pct < -1 AND change_pct < -0.5 AND rel_volume >= 1 — close near 52w high but faded intraday with red close + decent vol (failed move at high; potential top forming)
    OpenGapFilledNetFlat,           // gap_pct.abs() > 1.5 AND change_pct.abs() < 0.5 — significant gap opened the day but ended near flat (full gap absorption / round-trip; market rejected the overnight move)
    CompressedRangeVolatileSession, // year_high_pct < 15 AND year_low_pct < 25 AND hod_dist + lod_dist > 3 AND rel_volume >= 1.5 — small 52w range with volatile session (structurally compressed asset moving today; multi-month coil breaking out intraday)
    OrderlyNewHighContinuation,     // year_high_pct < 1 AND change_pct between 0.5 and 1.5 AND rel_volume >= 1 — fresh 52w high with moderate (not parabolic) move + decent vol (orderly breakout; quality trend continuation)
    OrderlyNewLowContinuation,      // year_low_pct < 1 AND change_pct between -1.5 and -0.5 AND rel_volume >= 1 — fresh 52w low with moderate (not crash) drop + decent vol (orderly breakdown; quality downtrend continuation)
    DryVolGapUpFade,                // gap_pct > 1 AND change_pct < 0 AND rel_volume < 0.7 — gap up + faded to red + dry vol (gap-up fade without participation; orderly absorption of overnight optimism)
    DryVolGapDownReclaim,           // gap_pct < -1 AND change_pct > 0 AND rel_volume < 0.7 — gap down + recovered to green + dry vol (gap-down reclaim without panic vol; orderly absorption of overnight pessimism)
    InstitutionalChurnDay,          // rel_volume >= 3 AND change_pct.abs() < 0.3 — very hot vol + flat net change (3x avg vol with no directional outcome; institutional rebalance / churn day)
    ExtremeTailEvent,               // rel_volume >= 3 AND change_pct.abs() > 5 — very hot vol + huge change (extreme tail event; major news / earnings / squeeze)
    Year52HighRetestStrongClose,    // year_high_pct < 5 AND change_pct > 0 AND hod_dist_pct.abs() < 0.3 AND rel_volume >= 1.5 — near 52w high + green close at HOD + hot vol (retest of 52w high with strong close confirming the level)
    Year52LowRetestWeakClose,       // year_low_pct < 5 AND change_pct < 0 AND lod_dist_pct.abs() < 0.3 AND rel_volume >= 1.5 — near 52w low + red close at LOD + hot vol (retest of 52w low with weak close confirming the level)
    DivergentGapVsIntraday,         // gap_pct * day_pct < 0 AND gap_pct.abs() > 0.5 AND day_pct.abs() > 0.5 — gap and intraday point opposite directions, both meaningful (clear overnight-vs-intraday divergence; market disagreed with the gap)
    CongruentGapAndIntradaySameDir, // gap_pct * day_pct > 0 AND gap_pct.abs() > 0.5 AND day_pct.abs() > 0.5 — gap and intraday same-direction, both meaningful (gap extended by intraday; same-direction follow-through)
    DeepMidRangeQuietSiesta,        // year_high_pct > 30 AND year_low_pct > 30 AND rel_volume < 0.5 AND change_pct.abs() < 0.5 — deeply mid-52w-range + very quiet vol + flat change (structurally calm asset siesta; total disinterest)
    DeepMidRangeActiveOutlier,      // year_high_pct > 30 AND year_low_pct > 30 AND rel_volume >= 2 AND change_pct.abs() > 1 — deeply mid-52w-range + hot vol + meaningful change (mid-range action; out-of-character active day; potential trend genesis)
    IntradayDirectionExceedsChange, // day_pct * change_pct > 0 AND day_pct.abs() > change_pct.abs() * 1.5 AND change_pct.abs() > 0.5 — same direction but intraday dominates by 1.5x (intraday late-session continuation outweighs gap)
    ChangeExceedsIntradayMagnitude, // change_pct.abs() > day_pct.abs() * 2 AND change_pct.abs() > 1 — change dominated by overnight component (gap-dominant move; intraday small relative to total)
    JustOffYearLowBouncingUp,       // year_low_pct between 5 and 15 AND change_pct > 1 AND rel_volume >= 1.5 — slightly off 52w lows + green move + hot vol (early bounce off lows; momentum picking up before fully reclaiming)
    JustOffYearHighFadingDown,      // year_high_pct between 5 and 15 AND change_pct < -1 AND rel_volume >= 1.5 — slightly off 52w highs + red move + hot vol (early fade from highs; distribution starting before fully breaking)
    OverextendedHighPullbackHealthy,// year_high_pct < 3 AND hod_dist_pct.abs() > 1.5 AND day_pct < -0.5 AND change_pct >= 0 — at 52w high + pulled back from HOD + red intraday + still positive day (healthy pullback retrace while in trend)
    OverextendedLowBounceHealthy,   // year_low_pct < 3 AND lod_dist_pct.abs() > 1.5 AND day_pct > 0.5 AND change_pct <= 0 — at 52w low + bounced from LOD + green intraday + still negative day (healthy bounce retrace while in downtrend)
    CleanTrendDayUp,                // change_pct > 0 AND day_pct > 0 AND gap_pct > 0 AND hod_dist_pct.abs() < 1 AND rel_volume >= 1 — every signal positive + close near HOD + decent vol (clean trend day up; everything aligned without extremity)
    CleanTrendDayDown,              // change_pct < 0 AND day_pct < 0 AND gap_pct < 0 AND lod_dist_pct.abs() < 1 AND rel_volume >= 1 — every signal negative + close near LOD + decent vol (clean trend day down; everything aligned without extremity)
    ClimaxRedBouncedFromLod,        // change_pct < -3 AND day_pct > 1 AND rel_volume >= 2 AND lod_dist_pct.abs() > 1.5 — big red overall + significant green intraday + hot vol + bounced from LOD (climax low; capitulation followed by intraday bounce)
    ClimaxGreenFadedFromHod,        // change_pct > 3 AND day_pct < -1 AND rel_volume >= 2 AND hod_dist_pct.abs() > 1.5 — big green overall + significant red intraday + hot vol + pulled from HOD (climax high; euphoria followed by intraday fade)
    WideRangeChopMixedVol,          // hod_dist + lod_dist > 3 AND change_pct.abs() < 0.5 AND rel_volume between 0.7 and 1.5 — wide range + small change + normal vol (range exploration without conviction; vol-of-vol; mixed signal)
    NarrowRangeBigChangeNoIntraday, // hod_dist + lod_dist < 1 AND change_pct.abs() > 1.5 AND day_pct.abs() < 0.3 — narrow range + big change + flat intraday (all change happened overnight; no intraday move; pure gap day)
    EveryAxisExtreme,               // gap_pct.abs() > 1 AND day_pct.abs() > 1 AND change_pct.abs() > 2 AND rel_volume >= 2 AND hod_dist + lod_dist > 2 — every measurable signal axis extreme simultaneously (multi-axis breakout; outlier across the full feature space)
    EveryAxisFlat,                  // gap_pct.abs() < 0.2 AND day_pct.abs() < 0.2 AND change_pct.abs() < 0.2 AND rel_volume < 0.7 AND hod_dist + lod_dist < 1 — every measurable signal axis tiny simultaneously (silent day; total absence of activity across the full feature space)
    Year52HighRejectedToLod,        // year_high_pct < 5 AND lod_dist_pct.abs() < 0.5 AND rel_volume >= 1.5 AND change_pct < 0 — close at LOD even though near 52w high + hot vol + red day (sharp rejection at the highs; topping signal)
    Year52LowReclaimedToHod,        // year_low_pct < 5 AND hod_dist_pct.abs() < 0.5 AND rel_volume >= 1.5 AND change_pct > 0 — close at HOD even though near 52w low + hot vol + green day (sharp reclaim from the lows; bottoming signal)
    HighVolNoGapModerateChange,     // gap_pct.abs() < 0.3 AND rel_volume >= 2 AND change_pct.abs() < 1 — hot vol + no gap + modest change (institutional accumulation/distribution without overnight catalyst; quiet-price big-flow day)
    LowVolWithLargeGap,             // gap_pct.abs() > 2 AND rel_volume < 0.7 AND change_pct.abs() > 1 — big gap + dry vol + meaningful change (gap held quietly; minimal participation needed to absorb the overnight move)
    GapErasedByIntradayFlat,        // gap_pct.abs() > 1 AND change_pct.abs() < 0.3 AND gap_pct * day_pct < 0 — gap + opposite-direction intraday + flat close (intraday completely erased the gap; full counter-move absorption)
    BothSidesTaggedFlatBalance,     // hod_dist_pct.abs() > 1 AND lod_dist_pct.abs() > 1 AND change_pct.abs() < 0.3 AND rel_volume >= 1.2 — both extremes well-distant from close + flat change + decent vol (range-bound balance; close mid-range after exploring both sides)
    OutsideDayWideBalanceHotVol,    // hod_dist + lod_dist > 5 AND change_pct.abs() < 0.3 AND rel_volume >= 1.5 — really wide range + flat change + hot vol (outside-day balance; both extremes visited then closed flat on heavy participation)
    InsideDayBigChangeBigVol,       // hod_dist + lod_dist < 1.5 AND change_pct.abs() > 2 AND rel_volume >= 2 — narrow range + big change + hot vol (inside-day big move; gap-driven change but with massive participation at flat-after-gap level)
    LongCandleUpTrendDay,           // change_pct > 2 AND day_pct > 1 AND hod_dist + lod_dist > 2 AND rel_volume >= 1.5 AND hod_dist_pct.abs() < 0.5 — big green day + significant intraday + wide range + hot vol + HOD close (long candle up; broad trend day with HOD finish)
    LongCandleDownTrendDay,         // change_pct < -2 AND day_pct < -1 AND hod_dist + lod_dist > 2 AND rel_volume >= 1.5 AND lod_dist_pct.abs() < 0.5 — big red day + significant intraday + wide range + hot vol + LOD close (long candle down; broad trend day with LOD finish)
    Year52HighWithRangeContraction, // year_high_pct < 3 AND hod_dist + lod_dist < 1 AND change_pct.abs() < 0.5 — at 52w high + tight range + flat change (structural pause at the highs; coiling at top before next leg)
    Year52LowWithRangeContraction,  // year_low_pct < 3 AND hod_dist + lod_dist < 1 AND change_pct.abs() < 0.5 — at 52w low + tight range + flat change (structural pause at the lows; coiling at bottom before next leg)
    GapAndIntradayHarmonic,         // gap_pct.abs() between 0.5 and 2 AND day_pct.abs() between 0.5 and 2 AND (gap_pct.abs() - day_pct.abs()).abs() < 0.3 AND rel_volume >= 1 — gap and intraday similar magnitude + decent vol (harmonic day; balanced overnight and intraday contributions)
    MicroDayEarlyShakeout,          // change_pct.abs() < 0.5 AND hod_dist + lod_dist > 2 AND day_pct.abs() < 0.3 AND rel_volume >= 1.5 — small change + wide intraday + flat day + hot vol (early shakeout day; explored both extremes early but settled flat with hot vol)
    GreenDaySubOptimalClose,        // change_pct > 1 AND hod_dist_pct.abs() > 2 AND rel_volume >= 1 — green day but closed significantly off HOD + decent vol (high-conviction green with sub-optimal close; pullback from peak before close)
    RedDaySubOptimalClose,          // change_pct < -1 AND lod_dist_pct.abs() > 2 AND rel_volume >= 1 — red day but closed significantly off LOD + decent vol (high-conviction red with sub-optimal close; bounce from trough before close)
    WideRangeNoVolFlat,             // hod_dist + lod_dist > 3 AND rel_volume < 0.7 AND change_pct.abs() < 0.5 — wide intraday range + dry vol + flat change (fake-liquidity range exploration; few prints across wide spread; possible spoof / wash)
    NarrowRangeMeaningfulChange,    // hod_dist + lod_dist < 1.5 AND change_pct.abs() > 1 — narrow intraday range + meaningful change (one-print day; price moved without exploring; mostly gap-driven without intraday discovery)
    Year52HighGapDownReclaimed,     // year_high_pct < 5 AND gap_pct < -0.5 AND change_pct >= 0 AND rel_volume >= 1 — at 52w high + gap down + reclaimed positive + decent vol (resilience at the highs; gap-down absorbed by intraday strength)
    Year52LowGapUpFaded,            // year_low_pct < 5 AND gap_pct > 0.5 AND change_pct <= 0 AND rel_volume >= 1 — at 52w low + gap up + faded negative + decent vol (relief gap rejected at the lows; continuation lower)
    IntradayMatchesChange,          // gap_pct.abs() < 0.2 AND change_pct.abs() > 1 AND (change_pct - day_pct).abs() < 0.3 — no gap + meaningful change + intraday matches change (entire move came from regular session; no overnight component)
    IntradayOpposesChange,          // change_pct.abs() > 1 AND day_pct * change_pct < 0 AND day_pct.abs() > 0.5 — meaningful change + opposite-sign intraday + meaningful intraday (gap dominated; intraday reversed but couldn't overpower gap)
    SymmetricMidRangeBalance,       // (hod_dist - lod_dist).abs() < 0.2 AND hod_dist + lod_dist > 1 AND change_pct.abs() < 0.3 — close exactly mid-range + meaningful range visited + flat change (geometric symmetry; perfect balance day)
    AsymmetricExtremeBias,          // (hod_dist - lod_dist).abs() > 2 AND hod_dist + lod_dist > 3 — close strongly biased to one extreme + wide range visited (one-sided range; close clearly favored one side of the day's exploration)
    YearLowExplosiveSqueezeIgnition, // year_low_pct < 3 AND change_pct > 5 AND rel_volume >= 3 — at 52w low + huge gain + extreme vol (squeeze ignition from 52w low; potential reversal of a multi-month downtrend)
    YearHighSharpDistribution,       // year_high_pct < 3 AND change_pct < -5 AND rel_volume >= 3 — at 52w high + huge drop + extreme vol (sharp distribution from the highs; potential trend break)
    LargeChangeOnNormalVol,          // change_pct.abs() > 3 AND rel_volume between 0.7 and 1.3 AND hod_dist + lod_dist > 2 — big change + normal vol + wide range (quality move without extreme participation; orderly directional day)
    MassiveIntradayWithoutGap,       // gap_pct.abs() < 0.1 AND day_pct.abs() > 3 AND rel_volume >= 2 — basically no gap + massive intraday + hot vol (huge intraday move with zero overnight bias; pure intraday discovery)
    MidYearBothSidesTagged,          // year_high_pct > 10 AND year_low_pct > 10 AND hod_dist_pct.abs() > 1 AND lod_dist_pct.abs() > 1 AND rel_volume >= 1 — well within 52w + both extremes visited + decent vol (mid-range double-test day; structurally undecided)
    ExtremeSilentRange,              // (year_high_pct < 5 OR year_low_pct < 5) AND hod_dist + lod_dist < 0.8 AND rel_volume < 1 — at 52w extreme + very tight range + dry vol (silence at extreme; pre-reversal exhaustion signal)
    MultiAxisDryDay,                 // change_pct.abs() < 0.5 AND rel_volume < 0.7 AND hod_dist + lod_dist < 1.5 AND gap_pct.abs() < 0.3 — flat change + dry vol + narrow range + small gap (quiet day across multiple axes; near-silent maintenance)
    BigGapBigVolBigChange,           // gap_pct.abs() > 2 AND rel_volume >= 2 AND change_pct.abs() > 2 — big gap + big vol + big change (catalyst day; news-driven gap with sustained intraday activity)
    GapDownClosedNearHODHotVol,      // gap_pct < -1 AND change_pct > 0 AND hod_dist_pct.abs() < 0.5 AND rel_volume >= 1.5 — gapped down + finished green + closing tick near HOD + hot vol (V-shape reclaim closing on the highs; strong end-of-day demand)
    GapUpClosedNearLODHotVol,        // gap_pct > 1 AND change_pct < 0 AND lod_dist_pct.abs() < 0.5 AND rel_volume >= 1.5 — gapped up + closed red + closing tick near LOD + hot vol (full fade with finish weakness on the lows; sellers stay in control into close)
    Year52HighGapUpHotVolBigChange,  // year_high_pct < 1 AND gap_pct > 1 AND rel_volume >= 2 AND change_pct > 1 — within 1% of 52w high + gap up + hot vol + finished up (near-high breakout attempt with sustained demand at multi-year highs)
    Year52LowGapDownHotVolBigDrop,   // year_low_pct < 1 AND gap_pct < -1 AND rel_volume >= 2 AND change_pct < -1 — within 1% of 52w low + gap down + hot vol + finished down (breakdown capitulation day at multi-year lows)
    WideRangeFlatCloseHotVol,        // hod_dist_pct.abs() + lod_dist_pct.abs() > 5 AND change_pct.abs() < 0.5 AND rel_volume >= 2 — wide intraday range + flat close + hot vol (tug-of-war battle with heavy participation; bulls and bears trade aggressively, neither wins)
    BigGapNarrowIntradayHotVol,      // gap_pct.abs() > 3 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — huge gap + tight intraday range + hot vol (gap holds; no follow-through movement either direction; participants accept the new gap level on volume)
    Year52HighDryVolNarrowRange,     // year_high_pct < 2 AND rel_volume < 0.7 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND change_pct.abs() < 0.3 — near 52w high + dry vol + narrow range + flat (silent compression at multi-year highs; coiled spring before next leg or rejection)
    Year52LowDryVolNarrowRange,      // year_low_pct < 2 AND rel_volume < 0.7 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND change_pct.abs() < 0.3 — near 52w low + dry vol + narrow range + flat (silent compression at multi-year lows; capitulation exhaustion or coiled bounce setup)
    CompressedYearRangeFlatDay,      // year_high_pct < 5 AND year_low_pct < 5 AND change_pct.abs() < 0.5 AND rel_volume < 1 — within 5% of BOTH 52w extremes simultaneously (compressed 52w range) + flat + dry vol (structurally narrow asset having yet another silent day; maximum coil at the regime level)
    CompressedYearRangeRegimeBreak,  // year_high_pct < 5 AND year_low_pct < 5 AND change_pct.abs() > 2 AND rel_volume >= 1.5 — within 5% of BOTH 52w extremes (compressed regime) BUT big change + hot vol (sudden break of a long-compressed 52w sideways range; regime-level breakout/breakdown candidate)
    IntradayClimaxTopFade,           // hod_dist_pct.abs() > 4 AND lod_dist_pct.abs() < 0.5 AND change_pct < 0 AND rel_volume >= 2 — far from HOD + closed at LOD + finished red + hot vol (intraday climax-top fade: pumped early then sold all day to finish red at the lows on volume)
    IntradayClimaxBottomReclaim,     // lod_dist_pct.abs() > 4 AND hod_dist_pct.abs() < 0.5 AND change_pct > 0 AND rel_volume >= 2 — far from LOD + closed at HOD + finished green + hot vol (intraday climax-bottom reclaim: panicked early then bid up all day to finish green at the highs on volume)
    BigChangeDryVolWideRange,        // change_pct.abs() > 3 AND rel_volume < 0.7 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 3 — big change + dry vol + wide intraday range (low-conviction trend day with thin tape; volatile but uncrowded; potentially manipulated or fake breakout)
    BigChangeDryVolFromGap,          // change_pct.abs() > 3 AND rel_volume < 0.7 AND gap_pct.abs() > 2 — big change + dry vol + significant gap (overnight repricing held with minimal intraday participation; pure pre-market re-rating absorbed without daytime confirmation)
    ExtremeVolGapDownReversal,       // rel_volume >= 5 AND gap_pct < -3 AND change_pct > 0 — extreme vol (5×+) + gap down >3 + finished green (extreme institutional reversal of overnight gap-down; max-conviction reclaim)
    ExtremeVolGapUpReversal,         // rel_volume >= 5 AND gap_pct > 3 AND change_pct < 0 — extreme vol (5×+) + gap up >3 + finished red (extreme institutional reversal of overnight gap-up; max-conviction distribution)
    AtYearHighRangeExpansionDryVol,  // year_high_pct < 1 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 4 AND rel_volume < 0.7 — within 1% of 52w high + wide intraday range + dry vol (no-volume rally at all-time highs; distribution warning; wide range without participation = supply meeting thin demand)
    AtYearLowRangeExpansionDryVol,   // year_low_pct < 1 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 4 AND rel_volume < 0.7 — within 1% of 52w low + wide intraday range + dry vol (no-volume capitulation at multi-year lows; thin-tape bounces without conviction; unlikely to stick)
    IntradayBigDayGapAgainstHotVol,  // day_pct.abs() > 3 AND gap_pct * day_pct < 0 AND gap_pct.abs() > 1 AND rel_volume >= 2 — big intraday move + gap-against-day-direction + hot vol (institutional gap-against-trend reversal day; open faded, then reversed and ran hard against the gap on volume)
    IntradayBigDayGapWithHotVol,     // day_pct.abs() > 3 AND gap_pct * day_pct > 0 AND gap_pct.abs() > 1 AND rel_volume >= 2 — big intraday move + gap-with-day-direction + hot vol (textbook gap-and-go continuation: open kept running same direction as the gap on heavy participation)
    OvernightDriftDryVol,            // gap_pct.abs() > 2 AND day_pct.abs() < 0.3 AND rel_volume < 0.5 — significant gap + flat intraday + very dry vol (overnight news repriced and nobody traded during the day; max-silence post-news; news absorbed instantly)
    HotVolHugeGapTinyDay,            // rel_volume >= 3 AND gap_pct.abs() > 2 AND day_pct.abs() < 0.3 — hot vol (3×+) + big gap + nearly flat intraday (heavy volume but no intraday movement; institutional repositioning at the new gap level; massive churn at one price)
    Year52LowGapUpHeldHotVol,        // year_low_pct < 5 AND gap_pct > 1 AND change_pct > 2 AND rel_volume >= 2 — at 52w low + gap up + finished up >2 + hot vol (relief gap HELD and extended on volume; reversal candidate at the floor; opposite of Year52LowGapUpFaded)
    Year52HighGapDownHeldHotVol,     // year_high_pct < 5 AND gap_pct < -1 AND change_pct < -2 AND rel_volume >= 2 — at 52w high + gap down + finished down <-2 + hot vol (rejection gap HELD and extended; distribution at the highs; opposite of Year52HighGapDownReclaimed)
    HotVolSmallChangeSmallGapWideRange, // rel_volume >= 2 AND change_pct.abs() < 0.5 AND gap_pct.abs() < 0.3 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 1.5 — hot vol + tiny change + tiny gap + moderate range (heavy participation that visited both sides + flat finish + no overnight bias; pure intraday redistribution without directional resolution)
    HotVolFlatCloseBigGap,           // change_pct.abs() < 0.5 AND gap_pct.abs() > 2 AND rel_volume >= 2 — flat finish + big gap + hot vol (intraday fully absorbed the gap on heavy participation; full round-trip on volume; market rejected the overnight move with confirmation)
    OrganicMicroGainNormalVol,       // change_pct between 0.3 and 1 AND day_pct > 0.2 AND rel_volume between 0.9 and 1.2 AND gap_pct.abs() < 0.2 — modest gain (0.3-1%) + green intraday + normal vol + no gap (silent drift up; pure organic accumulation under the radar; ideal for long-term adds without alerting)
    OrganicMicroDropNormalVol,       // change_pct between -1 and -0.3 AND day_pct < -0.2 AND rel_volume between 0.9 and 1.2 AND gap_pct.abs() < 0.2 — modest drop (-1 to -0.3%) + red intraday + normal vol + no gap (silent drift down; pure organic distribution under the radar)
    IntradayRangeWiderThanGapHotVol, // hod_dist_pct.abs() + lod_dist_pct.abs() > gap_pct.abs() * 2 AND gap_pct.abs() > 1 AND rel_volume >= 2 — intraday range > 2× the gap + meaningful gap + hot vol (intraday discovery dominates the gap; market traded a much wider range than overnight expected on volume)
    GapWiderThanIntradayRangeHotVol, // gap_pct.abs() > hod_dist_pct.abs() + lod_dist_pct.abs() AND gap_pct.abs() > 1.5 AND rel_volume >= 2 — gap > entire intraday range + significant gap + hot vol (gap dominates the day's move; intraday only consolidated in a narrow band near the new level on volume)
    BigGreenLowVolWeakClose,         // change_pct > 3 AND rel_volume < 1 AND hod_dist_pct.abs() > 1 — meaningful gain + dry vol + close not near HOD (fake-breakout warning; up-move on weak participation closing off the highs; reversion candidate)
    BigRedLowVolWeakClose,           // change_pct < -3 AND rel_volume < 1 AND lod_dist_pct.abs() > 1 — meaningful drop + dry vol + close not near LOD (fake-breakdown warning; down-move on weak participation closing off the lows; reversion candidate)
    GappingNearYearLowExtremeVol,    // year_low_pct < 5 AND gap_pct.abs() > 2 AND rel_volume >= 4 — near 52w low + significant gap + extreme vol (4×+) (high-intensity event at the floor; earnings/news catalyst at multi-year lows; max-event-driven reversal or capitulation candidate)
    GappingNearYearHighExtremeVol,   // year_high_pct < 5 AND gap_pct.abs() > 2 AND rel_volume >= 4 — near 52w high + significant gap + extreme vol (4×+) (high-intensity event at the highs; catalyst at the top; max-event-driven blow-off or distribution candidate)
    BothSidesTaggedDryVolFlat,       // hod_dist_pct.abs() > 1 AND lod_dist_pct.abs() > 1 AND change_pct.abs() < 0.3 AND rel_volume < 0.7 — both extremes well-distant from close + flat change + dry vol (thin-tape both-side raid that closed flat; algorithmic stop-hunt at low participation; possible spoof/wash on illiquid name)
    BothSidesTaggedBigChangeHotVol,  // hod_dist_pct.abs() > 1 AND lod_dist_pct.abs() > 1 AND change_pct.abs() > 2 AND rel_volume >= 2 — both extremes well-distant + big change + hot vol (full-range exploration ending decisively on volume; trend day that swept both sides first before resolving)
    ModerateGreenGapDownReversal,    // gap_pct < -1 AND change_pct between 1 and 2 AND rel_volume between 1 and 1.5 — gap down + moderate green finish (1-2%) + slightly elevated vol (moderate-conviction gap reversal; institutional buying without panic; conservative reclaim)
    ModerateRedGapUpFade,            // gap_pct > 1 AND change_pct between -2 and -1 AND rel_volume between 1 and 1.5 — gap up + moderate red finish (-1 to -2%) + slightly elevated vol (moderate-conviction gap fade; institutional selling without panic; conservative rejection)
    GapAndIntradayBothBigSameDirHotVol, // gap_pct.abs() > 2 AND day_pct.abs() > 2 AND gap_pct * day_pct > 0 AND rel_volume >= 1.5 — significant gap + significant intraday + same-direction + hot vol (gap-and-intraday-extend; both halves of the day pushed the same way on volume; conviction continuation)
    GapAndIntradayBothBigOpposingHotVol, // gap_pct.abs() > 2 AND day_pct.abs() > 2 AND gap_pct * day_pct < 0 AND rel_volume >= 1.5 — significant gap + significant intraday + opposite-direction + hot vol (gap rejected and reversed by significant intraday; full counter-move on volume)
    CountertrendBounceInDowntrend,   // change_pct > 2 AND year_high_pct > 20 AND year_low_pct < 10 AND rel_volume >= 2 — big green day + ≥20% below 52w high + within 10% of 52w low + hot vol (countertrend bounce in long-running downtrend; strong-hands buying near the floor)
    CountertrendFadeInUptrend,       // change_pct < -2 AND year_low_pct > 20 AND year_high_pct < 10 AND rel_volume >= 2 — big red day + ≥20% above 52w low + within 10% of 52w high + hot vol (countertrend fade in long-running uptrend; profit-taking near the ceiling)
    BigGreenNarrowRangeHotVol,       // change_pct > 2 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 2 — big green + tight intraday range + hot vol (gap-and-hold up; no intraday giveback; max-strength continuation candidate; entire move from gap held all day)
    BigRedNarrowRangeHotVol,         // change_pct < -2 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 2 — big red + tight intraday range + hot vol (gap-and-hold down; no intraday recovery; max-weakness continuation candidate; entire move from gap held all day)
    OneSidedRangeCloseAtHODGreen,    // hod_dist_pct.abs() < 0.5 AND lod_dist_pct.abs() > 2 AND change_pct > 1 AND rel_volume >= 1.5 — close at HOD + LOD >2% away + green + decent vol (one-sided up-day exploration; sellers shown low side but couldn't hold; trend day finished on the highs)
    OneSidedRangeCloseAtLODRed,      // lod_dist_pct.abs() < 0.5 AND hod_dist_pct.abs() > 2 AND change_pct < -1 AND rel_volume >= 1.5 — close at LOD + HOD >2% away + red + decent vol (one-sided down-day exploration; buyers shown high side but couldn't hold; trend day finished on the lows)
    BullishOutsideDayHotVol,         // change_pct > 1 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 4 AND change_pct * day_pct > 0 AND rel_volume >= 2 — green close + wide range + same-direction intraday + hot vol (bullish outside day with conviction; large-range exploration ending up on volume; institutional accumulation through range expansion)
    BearishOutsideDayHotVol,         // change_pct < -1 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 4 AND change_pct * day_pct > 0 AND rel_volume >= 2 — red close + wide range + same-direction intraday + hot vol (bearish outside day with conviction; large-range exploration ending down on volume; institutional distribution through range expansion)
    BelowAvgVolBigChangeGreen,       // change_pct > 2 AND rel_volume >= 0.5 AND rel_volume < 0.9 AND gap_pct.abs() < 0.5 — big green + below-avg vol (50-90%) + no gap (efficient organic up move on quiet tape; small participants chasing without catalyst; low-effort momentum)
    BelowAvgVolBigChangeRed,         // change_pct < -2 AND rel_volume >= 0.5 AND rel_volume < 0.9 AND gap_pct.abs() < 0.5 — big red + below-avg vol (50-90%) + no gap (efficient organic down move on quiet tape; small participants distributing without catalyst; low-effort weakness)
    MidRangeFullExpansionHotVol,     // year_high_pct > 15 AND year_low_pct > 15 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 4 AND rel_volume >= 2 — structurally mid-52w + wide intraday range + hot vol (undecided asset having a high-vol expansion day; potential trend genesis from balance; range break candidate)
    MidRangeCompressionDryVol,       // year_high_pct > 15 AND year_low_pct > 15 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 0.8 AND rel_volume < 0.5 — structurally mid-52w + tight intraday + dry vol (undecided + intraday compression + nobody trading; classic dormancy day; pre-event coil)
    OpeningRangeHoldCloseAtHODGreen, // gap_pct.abs() < 0.3 AND hod_dist_pct.abs() < 0.2 AND day_pct > 1 AND rel_volume between 1 and 2 — flat open + close exactly at HOD + green intraday >1% + normal-elevated vol (clean intraday discovery to highs with no overnight bias; pure organic trend day up)
    OpeningRangeHoldCloseAtLODRed,   // gap_pct.abs() < 0.3 AND lod_dist_pct.abs() < 0.2 AND day_pct < -1 AND rel_volume between 1 and 2 — flat open + close exactly at LOD + red intraday >1% + normal-elevated vol (clean intraday discovery to lows with no overnight bias; pure organic trend day down)
    WindowDressingMarkUp,            // change_pct.abs() < 0.3 AND change_pct > 0 AND hod_dist_pct.abs() < 0.3 AND rel_volume >= 1.5 — tiny green + close near HOD + hot vol (mark-the-close behavior; possible window-dressing at quarter/month end; deliberate end-of-day mark-up on volume)
    WindowDressingMarkDown,          // change_pct.abs() < 0.3 AND change_pct < 0 AND lod_dist_pct.abs() < 0.3 AND rel_volume >= 1.5 — tiny red + close near LOD + hot vol (mark-down behavior; possible reverse window-dressing; deliberate end-of-day mark-down on volume)
    Year52HighSustainedStrengthHotVol, // year_high_pct < 5 AND day_pct > 1 AND change_pct > 1 AND rel_volume >= 2 — near 52w high + green intraday + green close + hot vol (sustained strength confirmation at the highs; intraday-and-daily both confirm; high-conviction breakout candidate)
    Year52LowSustainedWeaknessHotVol,  // year_low_pct < 5 AND day_pct < -1 AND change_pct < -1 AND rel_volume >= 2 — near 52w low + red intraday + red close + hot vol (sustained weakness confirmation at the lows; intraday-and-daily both confirm; high-conviction breakdown candidate)
    BigGreenWithModestGapDecentVol,    // change_pct > 3 AND gap_pct between 0.5 and 1.5 AND rel_volume >= 1.5 — meaningful gain >3 + modest gap (0.5-1.5%) + decent vol (gap-assisted rally; overnight bias kicked off the day but intraday extended substantially on volume)
    BigRedWithModestGapDownDecentVol,  // change_pct < -3 AND gap_pct between -1.5 and -0.5 AND rel_volume >= 1.5 — meaningful drop <-3 + modest gap down (-0.5 to -1.5%) + decent vol (gap-assisted decline; overnight bias kicked off the day but intraday extended substantially on volume)
    CompoundConfirmedBigGreen,         // change_pct > 3 AND day_pct > 1 AND gap_pct > 0.5 AND hod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — big green close + green intraday + positive gap + close near HOD + decent vol (every signal aligned bullish; full conviction up day; max-confirmation long candidate)
    CompoundConfirmedBigRed,           // change_pct < -3 AND day_pct < -1 AND gap_pct < -0.5 AND lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — big red close + red intraday + negative gap + close near LOD + decent vol (every signal aligned bearish; full conviction down day; max-confirmation short candidate)
    FollowThroughGreen,                // change_pct in [1, 3] AND day_pct in [0.5, 2] AND gap_pct in [-0.5, 0.5] AND hod_dist_pct.abs() < 0.5 AND rel_volume in [1.2, 2] — modest green + green intraday + small gap + close near HOD + above-avg vol (clean follow-through up day; no catalyst spike, just steady accumulation with intraday confirmation)
    FollowThroughRed,                  // change_pct in [-3, -1] AND day_pct in [-2, -0.5] AND gap_pct in [-0.5, 0.5] AND lod_dist_pct.abs() < 0.5 AND rel_volume in [1.2, 2] — modest red + red intraday + small gap + close near LOD + above-avg vol (clean follow-through down day; no catalyst spike, just steady distribution with intraday confirmation)
    Year52HighGapDownStrongCloseHotVol,  // year_high_pct < 3 AND gap_pct < -0.5 AND change_pct > 1 AND hod_dist_pct.abs() < 0.5 AND rel_volume >= 1.5 — near 52w high + gap down + green close + close near HOD + decent vol (resilience day at the highs; gap rejected, recovered and closed strong; bullish continuation candidate)
    Year52LowGapUpWeakCloseHotVol,       // year_low_pct < 3 AND gap_pct > 0.5 AND change_pct < -1 AND lod_dist_pct.abs() < 0.5 AND rel_volume >= 1.5 — near 52w low + gap up + red close + close near LOD + decent vol (rejection day at the lows; relief gap faded, sold all day to close weak; bearish continuation candidate)
    FlatOpenTrendUpModerate,             // gap_pct.abs() < 0.3 AND change_pct in [1, 3] AND day_pct > 1 AND change_pct * day_pct > 0 AND rel_volume in [1, 2] — flat open + moderate green (1-3%) + green intraday + same-sign + normal-elevated vol (no overnight bias + clean intraday trend up; mid-magnitude organic move; conviction without spike)
    FlatOpenTrendDownModerate,           // gap_pct.abs() < 0.3 AND change_pct in [-3, -1] AND day_pct < -1 AND change_pct * day_pct > 0 AND rel_volume in [1, 2] — flat open + moderate red + red intraday + same-sign + normal-elevated vol (no overnight bias + clean intraday trend down; mid-magnitude organic move; conviction without spike)
    MidRangeRecoveryRallyHotVol,         // year_high_pct > 10 AND year_low_pct > 10 AND change_pct > 3 AND rel_volume >= 2 — recovered well off 52w lows + still below highs + big green + hot vol (sustained recovery rally in mid-52w; not at either extreme; mid-range bullish move with conviction)
    MidRangeSelloffHotVol,               // year_high_pct > 10 AND year_low_pct > 10 AND change_pct < -3 AND rel_volume >= 2 — sold off well from 52w highs + still above lows + big red + hot vol (sustained selloff in mid-52w; not at either extreme; mid-range bearish move with conviction)
    IntermediateGreenStrongClose,        // change_pct in [3, 7] AND rel_volume in [1.5, 3] AND hod_dist_pct.abs() < 1 — meaningful green (3-7%) + decent vol (1.5-3×) + close near HOD (intermediate gain on intermediate vol with strong finish; sweet spot between organic and parabolic; momentum without exhaustion)
    IntermediateRedWeakClose,            // change_pct in [-7, -3] AND rel_volume in [1.5, 3] AND lod_dist_pct.abs() < 1 — meaningful red (-3 to -7%) + decent vol (1.5-3×) + close near LOD (intermediate drop with weak finish; sweet spot between organic and crash; weakness without panic)
    MaxVolatilityEventHotVol,            // gap_pct.abs() > 2 AND change_pct.abs() > 3 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 4 AND rel_volume >= 2 — big gap + big change + wide intraday range + hot vol (max-volatility event day; gap caught attention, intraday explored wide range, hot vol confirmed; catalyst-driven volatility expansion)
    MaxRangeFakeOutDryVol,               // gap_pct.abs() > 2 AND change_pct.abs() > 3 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 4 AND rel_volume < 1 — big gap + big change + wide intraday range + DRY vol (max-range fake-out; wide intraday with thin tape suggests stop-runs without true conviction; algorithmic noise on illiquid name)
    BigGreenIntradayOnlyHotVol,          // change_pct > 3 AND gap_pct.abs() < 0.5 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 4 AND rel_volume >= 2 — big green close + flat open + wide intraday + hot vol (intraday-only rally; no overnight bias; pure intraday discovery to new highs; all of the day's gain from intraday participation)
    BigRedIntradayOnlyHotVol,            // change_pct < -3 AND gap_pct.abs() < 0.5 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 4 AND rel_volume >= 2 — big red close + flat open + wide intraday + hot vol (intraday-only decline; no overnight bias; pure intraday discovery to new lows; all of the day's loss from intraday participation)
    BrokeAbove52wHighHotVol,             // year_high_pct > 0 AND change_pct > 1 AND rel_volume >= 2 — closed ABOVE prior 52w high + green + hot vol (true new-high breakout with volume confirmation; institutional initiation at a multi-year extreme)
    BrokeBelow52wLowHotVol,              // year_low_pct < 0 AND change_pct < -1 AND rel_volume >= 2 — closed BELOW prior 52w low + red + hot vol (true new-low breakdown with volume confirmation; institutional capitulation at a multi-year extreme)
    ChangeIntradayDisagreeBothTagged,    // change_pct * day_pct < 0 AND hod_dist_pct.abs() > 1 AND lod_dist_pct.abs() > 1 AND rel_volume >= 1.5 — change/day signs disagree + both extremes visited + decent vol (full schizophrenic day; gap dominates close direction but intraday went opposite and explored both sides; institutional repositioning vs retail)
    ChangeIntradayDisagreeFlatRange,     // change_pct * day_pct < 0 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — change/day signs disagree + tight intraday + decent vol (gap-vs-intraday sign disagreement but intraday compressed; overnight news held even as intraday tried to fade in narrow range)
    BigGapHugeVolHalfFade,               // gap_pct.abs() > 2 AND rel_volume >= 3 AND change_pct.abs() < gap_pct.abs() * 0.5 — big gap + extreme vol (3×+) + change < half the gap (gap absorbed substantially even on extreme volume; institutional offloading at gap level; >50% gap fade with conviction)
    BigGapHugeVolFullExtension,          // gap_pct.abs() > 2 AND rel_volume >= 3 AND change_pct.abs() > gap_pct.abs() * 1.5 — big gap + extreme vol (3×+) + change > 1.5× the gap (gap extended substantially on extreme volume; institutional commitment beyond the gap; momentum continuation on max participation)
    GapWithChangeWideRangeHotVol,        // change_pct * gap_pct > 0 AND gap_pct.abs() > 1 AND change_pct.abs() > 2 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 3 AND rel_volume >= 2 — gap and change same-sign + meaningful gap + bigger change + wide range + hot vol (gap extended through wide intraday exploration in same direction on volume; max-conviction trend day with both halves contributing)
    GapAgainstChangeWideRangeHotVol,     // change_pct * gap_pct < 0 AND gap_pct.abs() > 1 AND change_pct.abs() > 2 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 3 AND rel_volume >= 2 — gap and change opposite-sign + meaningful gap + bigger change + wide range + hot vol (intraday more than reversed the gap with wide range and hot vol; full institutional reversal with extreme volatility)
    HotVolNoChangeNoGapTightRange,       // rel_volume >= 2 AND change_pct.abs() < 0.5 AND gap_pct.abs() < 0.3 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1 — hot vol + flat change + flat gap + tight intraday (heavy participation but absolutely no movement; classic absorption pattern — institutional accumulation/distribution disguised as nothing)
    ColdVolBigChangeWideRange,           // rel_volume < 0.5 AND change_pct.abs() > 3 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 3 — very dry vol + big change + wide range (max-thin-tape exception; nobody traded but price moved a lot through wide range — algorithmic noise / illiquid stop-runs)
    MicroPinTinyRangeHotVol,             // hod_dist_pct.abs() < 0.1 AND lod_dist_pct.abs() < 0.1 AND rel_volume >= 2 — close pinned at exactly HOD AND LOD (effectively zero intraday range) + hot vol (extreme pin; heavy participation at a single price; possible OPEX pin or institutional volume crossed at one tick)
    MicroPinTinyRangeDryVol,             // hod_dist_pct.abs() < 0.1 AND lod_dist_pct.abs() < 0.1 AND rel_volume < 0.5 — close pinned at exactly HOD AND LOD + DRY vol (dead day; nothing happened at all; market truly absent; possibly halted or extremely illiquid)
    FullRange52wAtHighSide,              // year_high_pct < 2 AND year_low_pct > 50 AND rel_volume >= 1.5 — within 2% of 52w high + >50% above 52w low + decent vol (stock has doubled+ from 52w low and is at the highs; max-uptrend + at decision point for breakout/exhaustion)
    FullRange52wAtLowSide,               // year_low_pct < 2 AND year_high_pct > 50 AND rel_volume >= 1.5 — within 2% of 52w low + >50% below 52w high + decent vol (stock has dropped 50%+ from 52w high and is at the lows; max-downtrend + at decision point for capitulation/reversal)
    PullbackAndRallyAtYearHigh,          // change_pct in [0.5, 1.5] AND day_pct > 0.5 AND gap_pct in [-0.5, 0] AND year_high_pct < 5 AND rel_volume in [1, 1.8] — moderate green (0.5-1.5%) + green intraday + small red gap + within 5% of 52w high + decent vol (textbook pullback-and-rally setup near the highs; gap-down bought intraday)
    DeadCatBounceAtYearLow,              // change_pct in [-1.5, -0.5] AND day_pct < -0.5 AND gap_pct in [0, 0.5] AND year_low_pct < 5 AND rel_volume in [1, 1.8] — moderate red + red intraday + small green gap + within 5% of 52w low + decent vol (textbook dead-cat-bounce setup near the lows; gap-up sold intraday)
    GapDownIntradayReversalCloseAtHOD,   // gap_pct < -0.5 AND day_pct > 0.5 AND hod_dist_pct.abs() < 0.3 AND rel_volume >= 1.5 — gap down + intraday went up + close near HOD + decent vol (textbook reversal-up day; gap-down was bought, rallied through the day to close at the highs; strong bullish reversal candidate)
    GapUpIntradayReversalCloseAtLOD,     // gap_pct > 0.5 AND day_pct < -0.5 AND lod_dist_pct.abs() < 0.3 AND rel_volume >= 1.5 — gap up + intraday went down + close near LOD + decent vol (textbook reversal-down day; gap-up was sold, declined through the day to close at the lows; strong bearish reversal candidate)
    BigGreenMidYearSweetSpot,            // change_pct > 3 AND year_high_pct in [5, 20] AND year_low_pct > 15 AND rel_volume >= 1.5 — big green + 5-20% below 52w high + well above 52w low + decent vol (sweet-spot up move: high-momentum stock pushing higher from mid-range; room to run before resistance)
    BigRedMidYearSweetSpot,              // change_pct < -3 AND year_low_pct in [5, 20] AND year_high_pct > 15 AND rel_volume >= 1.5 — big red + 5-20% above 52w low + well below 52w high + decent vol (sweet-spot down move: high-momentum drop from mid-range; room to fall before support)
    TripleZeroHotVol,                    // gap_pct.abs() < 0.1 AND change_pct.abs() < 0.1 AND day_pct.abs() < 0.1 AND rel_volume >= 2 — gap + change + day ALL near zero + hot vol (massive participation produced literally zero movement on all three axes; max-absorption pattern; institutional cross at one price)
    TripleZeroDryVol,                    // gap_pct.abs() < 0.1 AND change_pct.abs() < 0.1 AND day_pct.abs() < 0.1 AND rel_volume < 0.5 — gap + change + day ALL near zero + dry vol (universal dormancy; market completely absent on all axes; near-dead-tape day; possibly halted or unattended)
    ExtremeGapModerateMoveHotVol,        // gap_pct.abs() > 5 AND change_pct.abs() in [2, 5] AND rel_volume >= 2 — huge gap (>5%) + moderate retained change (2-5%) + hot vol (extreme gap retained most but not all of itself on volume; partial-fill move with conviction at a new level)
    ExtremeGapBigContinuationHotVol,     // gap_pct.abs() > 5 AND change_pct.abs() > gap_pct.abs() AND rel_volume >= 2 — extreme gap + change EXCEEDS gap + hot vol (extreme gap that EXTENDED on volume; max-momentum continuation; >100% gap extension)
    BigGreenBigGapDryVol,                // change_pct > 5 AND gap_pct > 3 AND rel_volume < 0.8 — big green + big gap up + below-avg vol (huge gap-up that held without participation; suspect rally; possibly fake breakout or stealth squeeze in thin tape)
    BigRedBigGapDownDryVol,              // change_pct < -5 AND gap_pct < -3 AND rel_volume < 0.8 — big red + big gap down + below-avg vol (huge gap-down that held without participation; suspect breakdown; possibly forced-selling in thin tape or low-conviction capitulation)
    SmoothBigGreenNormalVol,             // change_pct > 3 AND rel_volume in [1, 1.5] AND hod_dist_pct.abs() < 0.5 AND gap_pct.abs() < 0.5 — big green + normal vol (1-1.5×) + close at HOD + no gap (orderly trend day; not parabolic, but conviction; sweet-spot entry signal)
    SmoothBigRedNormalVol,               // change_pct < -3 AND rel_volume in [1, 1.5] AND lod_dist_pct.abs() < 0.5 AND gap_pct.abs() < 0.5 — big red + normal vol + close at LOD + no gap (orderly down day; not panic, but conviction; sweet-spot entry for shorts)
    BigDayPctFlatChangeHotVol,           // day_pct.abs() > 2 AND change_pct.abs() < 0.5 AND rel_volume >= 1.5 — big intraday move + flat close + decent vol (gap absorbed all intraday move on volume; full round-trip with participation; gap-and-fade pattern)
    BigDayPctBigChangeAlignedHotVol,     // day_pct.abs() > 2 AND change_pct.abs() > 4 AND change_pct * day_pct > 0 AND rel_volume >= 1.5 — big intraday + big change + same sign + decent vol (max-aligned trend day; both gap and intraday push same way through big move on volume)
    Year52HighBigDayDryVol,              // year_high_pct < 2 AND day_pct > 2 AND rel_volume < 0.7 — near 52w high + big green intraday + dry vol (no-volume push to new highs intraday; distribution suspicion / fake breakout / thin-tape rally)
    Year52LowBigDayDryVol,               // year_low_pct < 2 AND day_pct < -2 AND rel_volume < 0.7 — near 52w low + big red intraday + dry vol (no-volume push to new lows intraday; capitulation without conviction / thin-tape breakdown)
    MidMagnitudeGreenMidWickHotVol,      // change_pct in [1, 3] AND hod_dist_pct.abs() in [0.5, 2] AND lod_dist_pct.abs() in [0.5, 2] AND rel_volume >= 1.5 — moderate green + mid-range close + decent vol (moderate-conviction up day with mid-wick finish; less extreme than BigUpMidRangeClose; consolidation candidate)
    MidMagnitudeRedMidWickHotVol,        // change_pct in [-3, -1] AND hod_dist_pct.abs() in [0.5, 2] AND lod_dist_pct.abs() in [0.5, 2] AND rel_volume >= 1.5 — moderate red + mid-range close + decent vol (moderate-conviction down day with mid-wick finish; less extreme than BigDownMidRangeClose; basing candidate)
    HotVolHugeRangeBigChange,            // rel_volume >= 3 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 6 AND change_pct.abs() > 4 — extreme vol (3×+) + extreme range (>6%) + big change (>4%) (catalyst day with massive participation, wide exploration, and big finish; max-volatility resolution)
    HotVolHugeRangeFlatClose,            // rel_volume >= 3 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 6 AND change_pct.abs() < 0.5 — extreme vol + extreme range + flat close (max-absorption pattern at scale: huge intraday exploration but no net result; institutional indecision day)
    Year52HighDistributionChurn,         // year_high_pct < 2 AND change_pct.abs() < 0.3 AND rel_volume >= 2 — near 52w high + flat close + hot vol (distribution at the highs; heavy churn without movement; institutional offloading at the top)
    Year52LowAccumulationChurn,          // year_low_pct < 2 AND change_pct.abs() < 0.3 AND rel_volume >= 2 — near 52w low + flat close + hot vol (accumulation at the lows; heavy churn without movement; institutional bottom-fishing at the floor)
    Year52HighBigGreenBreakoutHotVol,    // year_high_pct < 0 AND change_pct > 4 AND rel_volume >= 2 — breakout to new 52w high + big green + hot vol (decisive breakout from year resistance with institutional sponsorship; trend-following long signal)
    Year52LowBigRedBreakdownHotVol,      // year_low_pct < 0 AND change_pct < -4 AND rel_volume >= 2 — breakdown to new 52w low + big red + hot vol (decisive breakdown from year support with institutional sponsorship; trend-following short signal)
    GapUpFailBigRedHotVol,               // gap_pct > 3 AND change_pct < -2 AND rel_volume >= 2 — gap up but closed red + hot vol (failed gap up; trapped longs; reversal short signal)
    GapDownReclaimBigGreenHotVol,        // gap_pct < -3 AND change_pct > 2 AND rel_volume >= 2 — gap down but closed green + hot vol (reclaimed gap down; trapped shorts; reversal long signal)
    InsideRangeHotVolCoil,               // hod_dist_pct.abs() < 1 AND lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — tight intraday range + hot vol (inside-range coil with absorption; pre-breakout compression with elevated participation)
    OutsideRangeFlatCloseHotVol,         // hod_dist_pct.abs() + lod_dist_pct.abs() > 5 AND change_pct.abs() < 0.5 AND rel_volume >= 1.5 — wide intraday range + flat close + hot vol (outside-range whip; high participation but no commitment; institutional indecision with wide whipsaw)
    CloseAtHodTinyLodHotVol,             // hod_dist_pct.abs() < 0.3 AND lod_dist_pct > 4 AND rel_volume >= 1.5 — closed pinned to HOD + LOD far below + hot vol (full intraday range claim; momentum buy ramp into the close with elevated participation)
    CloseAtLodTinyHodHotVol,             // lod_dist_pct.abs() < 0.3 AND hod_dist_pct < -4 AND rel_volume >= 1.5 — closed pinned to LOD + HOD far above + hot vol (full intraday range collapse; momentum sell ramp into the close with elevated participation)
    BigGreenCloseAtHodHotVol,            // change_pct > 3 AND hod_dist_pct.abs() < 0.5 AND rel_volume >= 2 — big green + closed pinned to HOD + hot vol (strong trend day closing on the highs with institutional sponsorship; trend-following long signal)
    BigRedCloseAtLodHotVol,              // change_pct < -3 AND lod_dist_pct.abs() < 0.5 AND rel_volume >= 2 — big red + closed pinned to LOD + hot vol (strong trend day closing on the lows with institutional sponsorship; trend-following short signal)
    GapAndGoBigGreenCloseAtHod,          // gap_pct > 2 AND change_pct > gap_pct AND hod_dist_pct.abs() < 0.5 AND rel_volume >= 1.5 — gapped up + continued higher + closed at HOD + hot vol (gap-and-go continuation; momentum sustained through the close)
    GapAndDropBigRedCloseAtLod,          // gap_pct < -2 AND change_pct < gap_pct AND lod_dist_pct.abs() < 0.5 AND rel_volume >= 1.5 — gapped down + continued lower + closed at LOD + hot vol (gap-and-drop continuation; selling sustained through the close)
    GapUpFillReverseHotVol,              // gap_pct > 3 AND change_pct < 0 AND change_pct > -gap_pct AND rel_volume >= 1.5 — gap up + closed below open but above prior close + hot vol (gap fill in progress; partial reversion to mean with elevated participation)
    GapDownFillReverseHotVol,            // gap_pct < -3 AND change_pct > 0 AND change_pct < -gap_pct AND rel_volume >= 1.5 — gap down + closed above open but below prior close + hot vol (gap fill in progress; partial reversion to mean with elevated participation)
    Year52HighSqueezeShort,              // year_high_pct < 0 AND change_pct > 5 AND rel_volume >= 3 — new 52w high + big green + extreme vol (short squeeze at the highs; trapped shorts forced to cover into resistance breakout)
    Year52LowCapitulation,               // year_low_pct < 0 AND change_pct < -5 AND rel_volume >= 3 — new 52w low + big red + extreme vol (capitulation at the lows; forced selling at floor; trapped longs flushed)
    DragonflyDojiHotVol,                 // change_pct.abs() < 0.3 AND lod_dist_pct > 4 AND hod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — flat close + LOD far below + close near HOD + hot vol (dragonfly doji recovery; intraday plunge fully reclaimed by close with elevated participation)
    GravestoneDojiHotVol,                // change_pct.abs() < 0.3 AND hod_dist_pct < -4 AND lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — flat close + HOD far above + close near LOD + hot vol (gravestone doji rejection; intraday rip fully sold by close with elevated participation)
    HammerReversalHotVol,                // change_pct > 1 AND lod_dist_pct > 3 AND hod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — green close + LOD far below + close near HOD + hot vol (hammer reversal; intraday plunge reclaimed + green finish; reversal long signal with elevated participation)
    ShootingStarReversalHotVol,          // change_pct < -1 AND hod_dist_pct < -3 AND lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — red close + HOD far above + close near LOD + hot vol (shooting star reversal; intraday rip sold + red finish; reversal short signal with elevated participation)
    MarubozuGreenHotVol,                 // change_pct > 3 AND hod_dist_pct.abs() < 0.3 AND gap_pct.abs() < 1 AND rel_volume >= 2 — big green + closed at HOD + no overnight gap + hot vol (green marubozu; full intraday trend day with no gap aid; max-conviction long built entirely during regular hours)
    MarubozuRedHotVol,                   // change_pct < -3 AND lod_dist_pct.abs() < 0.3 AND gap_pct.abs() < 1 AND rel_volume >= 2 — big red + closed at LOD + no overnight gap + hot vol (red marubozu; full intraday trend day with no gap aid; max-conviction short built entirely during regular hours)
    Year52HighParabolicExtreme,          // year_high_pct < 0 AND change_pct > 10 AND rel_volume >= 5 — new 52w high + parabolic green + extreme vol (parabolic blow-off at new highs; exhaustion-vol squeeze; either continuation rocket or terminal top)
    Year52LowParabolicExtreme,           // year_low_pct < 0 AND change_pct < -10 AND rel_volume >= 5 — new 52w low + parabolic red + extreme vol (panic capitulation at new lows; exhaustion-vol flush; either continuation or terminal bottom)
    HotVolNoChangeTightRange,            // rel_volume >= 3 AND change_pct.abs() < 0.5 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 2 AND gap_pct.abs() < 0.5 — hot vol + tight intraday range + flat close + no gap (extreme absorption coil; institutional accumulation / distribution with no price expansion; pre-breakout compression at scale)
    DryVolBigMoveNoFollow,               // rel_volume < 0.5 AND change_pct.abs() > 4 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 3 — dry vol + big move + tight close range (low-participation thrust; illiquidity-driven move without follow-through; fade candidate)
    BigGapBigContinuationBigRange,       // gap_pct.abs() > 4 AND change_pct.abs() > 2 * gap_pct.abs() AND hod_dist_pct.abs() + lod_dist_pct.abs() > 6 AND rel_volume >= 2 — big gap + 2x-gap continuation + wide range + hot vol (gap-and-rip extension; momentum doubled the overnight thrust during regular hours; conviction trend day with full range expansion)
    BigGapFullReversalBigRange,          // gap_pct.abs() > 4 AND change_pct.abs() > 2 AND gap_pct * change_pct < 0 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 6 AND rel_volume >= 2 — big gap + sign-flipped intraday move + wide range + hot vol (full gap reversal; opposite-side dominance after the gap; trapped gap traders flushed both ways during the session)
    TinyGapBigMoveTightWicks,            // gap_pct.abs() < 0.5 AND change_pct.abs() > 4 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 3 AND rel_volume >= 2 — no gap + big intraday move + tight wicks + hot vol (clean trend day off the open with no gap aid and minimal noise; pure directional conviction built entirely intraday)
    BigGapTinyMoveTightWicks,            // gap_pct.abs() > 4 AND change_pct.abs() < 1 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 3 AND rel_volume >= 2 — big gap + flat intraday + tight wicks + hot vol (overnight gap held with intraday consolidation; market accepted the gap with no participation rotation; pre-extension coil)
    HotVolBigGreenWideRangeYearLow,      // change_pct > 5 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 6 AND year_low_pct < 2 AND rel_volume >= 2 — big green + wide range + near 52w low + hot vol (bottom-fishing thrust; capitulation reversal off the floor with elevated participation; potential bear-trap reversal)
    HotVolBigRedWideRangeYearHigh,       // change_pct < -5 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 6 AND year_high_pct < 2 AND rel_volume >= 2 — big red + wide range + near 52w high + hot vol (distribution flush; rejection reversal off the ceiling with elevated participation; potential bull-trap reversal)
    HotVolBigGreenWideRangeYearHigh,     // change_pct > 5 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 6 AND year_high_pct < 2 AND rel_volume >= 2 — big green + wide range + near 52w high + hot vol (breakout extension off the ceiling with elevated participation; trend acceleration into new highs with full range expansion)
    HotVolBigRedWideRangeYearLow,        // change_pct < -5 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 6 AND year_low_pct < 2 AND rel_volume >= 2 — big red + wide range + near 52w low + hot vol (breakdown extension off the floor with elevated participation; trend acceleration into new lows with full range expansion)
    RangeContractionHotVolBigGap,        // hod_dist_pct.abs() + lod_dist_pct.abs() < 1.5 AND change_pct.abs() < 0.5 AND rel_volume >= 2 AND gap_pct.abs() > 3 — tight intraday range + flat close + hot vol + big overnight gap (gap absorbed into intraday coil; market accepted the gap with elevated participation but no further expansion; trapped gap traders compressing into a spring)
    RangeExpansionHotVolBigIntraday,     // hod_dist_pct.abs() + lod_dist_pct.abs() > 5 AND day_pct.abs() > 3 AND rel_volume >= 2 AND gap_pct.abs() < 1 — wide intraday range + big intraday move from open + hot vol + no overnight gap (full intraday range expansion off the open with no gap aid; pure intraday breakout day driven by regular-hours conviction)
    GapPlusDriveBullHotVol,              // gap_pct > 1 AND day_pct > 3 AND change_pct > 4 AND rel_volume >= 1.5 — gap up + big intraday drive up + hot vol (two-leg bullish conviction: overnight gap held + extended further during regular hours; gap-and-extend trend day)
    GapPlusDriveBearHotVol,              // gap_pct < -1 AND day_pct < -3 AND change_pct < -4 AND rel_volume >= 1.5 — gap down + big intraday drive down + hot vol (two-leg bearish conviction: overnight gap held + extended further during regular hours; gap-and-extend trend day)
    GapFadeBullDayPctOpposite,           // gap_pct < -2 AND day_pct > 3 AND change_pct > 0 AND rel_volume >= 1.5 — gap down + closed up from open + net green + hot vol (full gap-down fade reversal: opened lower, recovered intraday and closed above prior close; trapped overnight shorts squeezed during regular hours)
    GapFadeBearDayPctOpposite,           // gap_pct > 2 AND day_pct < -3 AND change_pct < 0 AND rel_volume >= 1.5 — gap up + closed down from open + net red + hot vol (full gap-up fade rejection: opened higher, sold off intraday and closed below prior close; trapped overnight longs flushed during regular hours)
    DayPctBigGreenChangeFlat,            // day_pct > 4 AND change_pct.abs() < 0.5 AND gap_pct < -3 AND rel_volume >= 1.5 — big intraday drive up + flat net close + big overnight gap down + hot vol (full gap-down recovery: opened way below prior close, rallied hard intraday and finished flat for the session; intraday short-cover squeeze fully unwound the overnight drop)
    DayPctBigRedChangeFlat,              // day_pct < -4 AND change_pct.abs() < 0.5 AND gap_pct > 3 AND rel_volume >= 1.5 — big intraday drive down + flat net close + big overnight gap up + hot vol (full gap-up rejection: opened way above prior close, sold off intraday and finished flat for the session; intraday long-liquidation fully unwound the overnight pop)
    Year52HighBreakoutOpenDriveHotVol,   // year_high_pct < 0 AND day_pct > 3 AND change_pct > 4 AND gap_pct.abs() < 1 AND rel_volume >= 2 — new 52w high + big intraday drive from open + no overnight gap + hot vol (intraday breakout to new 52w high built entirely in regular hours with no overnight aid; pure conviction breakout day)
    Year52LowBreakdownOpenDriveHotVol,   // year_low_pct < 0 AND day_pct < -3 AND change_pct < -4 AND gap_pct.abs() < 1 AND rel_volume >= 2 — new 52w low + big intraday drive from open + no overnight gap + hot vol (intraday breakdown to new 52w low built entirely in regular hours with no overnight aid; pure conviction breakdown day)
    Year52HighGapAndGoExtremeVol,        // year_high_pct < 0 AND gap_pct > 3 AND day_pct > 2 AND change_pct > 5 AND rel_volume >= 3 — new 52w high + big gap up + intraday continuation + extreme vol (gap-and-go breakout at new highs with overnight gap held and extended during regular hours; max-conviction trend acceleration)
    Year52LowGapAndDropExtremeVol,       // year_low_pct < 0 AND gap_pct < -3 AND day_pct < -2 AND change_pct < -5 AND rel_volume >= 3 — new 52w low + big gap down + intraday continuation + extreme vol (gap-and-drop breakdown at new lows with overnight gap held and extended during regular hours; max-conviction trend acceleration)
    Year52HighFailedBreakoutFade,        // year_high_pct >= 0 AND year_high_pct < 3 AND gap_pct > 1 AND day_pct < -1 AND change_pct < 0 AND rel_volume >= 2 — close just below 52w high + gap up + intraday sold from open + red close + hot vol (failed breakout at the highs: ran into resistance, gap rejected and faded all session; trapped breakout buyers flushed during the session)
    Year52LowFailedBreakdownReclaim,     // year_low_pct >= 0 AND year_low_pct < 3 AND gap_pct < -1 AND day_pct > 1 AND change_pct > 0 AND rel_volume >= 2 — close just above 52w low + gap down + intraday recovered from open + green close + hot vol (failed breakdown at the lows: bounced off support, gap reclaimed and rallied all session; trapped breakdown shorts squeezed during the session)
    Year52HighRangeCompressionLowVol,    // year_high_pct >= 0 AND year_high_pct < 3 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1.5 AND change_pct.abs() < 0.5 AND rel_volume < 0.7 — close just below 52w high + tight intraday range + flat close + dry vol (low-vol compression just below resistance; no participation rotation; pre-breakout coil at the ceiling)
    Year52LowRangeCompressionLowVol,     // year_low_pct >= 0 AND year_low_pct < 3 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1.5 AND change_pct.abs() < 0.5 AND rel_volume < 0.7 — close just above 52w low + tight intraday range + flat close + dry vol (low-vol compression just above support; no participation rotation; pre-breakdown coil at the floor)
    DistantFromYearHighDryVolCoil,       // year_high_pct >= 30 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1.5 AND change_pct.abs() < 0.5 AND rel_volume < 0.6 — far below 52w high + tight intraday range + flat close + extremely dry vol (deep-discount basing; no participation; potential turnaround setup after extended pullback)
    DistantFromYearLowDryVolCoil,        // year_low_pct >= 30 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1.5 AND change_pct.abs() < 0.5 AND rel_volume < 0.6 — far above 52w low + tight intraday range + flat close + extremely dry vol (deep-premium basing; no participation; potential exhaustion setup after extended uptrend)
    DistantFromYearHighBigGreenHotVol,   // year_high_pct >= 20 AND change_pct > 5 AND rel_volume >= 2 — far below 52w high + big green + hot vol (snap-back rally from deep discount; mean-reversion thrust with elevated participation toward the prior peak)
    DistantFromYearLowBigRedHotVol,      // year_low_pct >= 20 AND change_pct < -5 AND rel_volume >= 2 — far above 52w low + big red + hot vol (snap-back decline from deep premium; mean-reversion drop with elevated participation toward the prior trough)
    MidRangeChurnHotVolBigDayPct,        // hod_dist_pct.abs().min(lod_dist_pct.abs()) >= 1.5 AND hod_dist_pct.abs().max(lod_dist_pct.abs()) <= 5 AND day_pct.abs() > 3 AND rel_volume >= 2 — close near mid of intraday range + big intraday move + hot vol (mid-range churn with intraday displacement; net move but no follow-through to either extreme; failed-trend day with continued participation)
    MidRangeChurnHotVolFlatDayPct,       // hod_dist_pct.abs().min(lod_dist_pct.abs()) >= 1.5 AND hod_dist_pct.abs().max(lod_dist_pct.abs()) <= 5 AND day_pct.abs() < 0.5 AND rel_volume >= 2 — close near mid of intraday range + flat intraday move + hot vol (max-indecision day at scale; full range with no net direction and elevated participation; institutional indecision with rotation)
    Year52HighRetestPullbackDryVol,      // year_high_pct >= 3 AND year_high_pct < 10 AND change_pct < -1 AND change_pct > -3 AND rel_volume < 0.8 — pulled back 3-10 % from 52w high + small red + dry vol (low-conviction pullback toward retest of recent highs; potential continuation setup with shallow consolidation)
    Year52LowRetestBounceDryVol,         // year_low_pct >= 3 AND year_low_pct < 10 AND change_pct > 1 AND change_pct < 3 AND rel_volume < 0.8 — bounced 3-10 % off 52w low + small green + dry vol (low-conviction bounce toward retest of recent lows; potential continuation setup with shallow rebound)
    Year52HighRetestPullbackHotVol,      // year_high_pct >= 3 AND year_high_pct < 10 AND change_pct < -2 AND change_pct > -5 AND rel_volume >= 2 — pulled back 3-10 % from 52w high + meaningful red + hot vol (high-conviction pullback toward retest of recent highs; institutional profit-taking with elevated participation; potential continuation setup)
    Year52LowRetestBounceHotVol,         // year_low_pct >= 3 AND year_low_pct < 10 AND change_pct > 2 AND change_pct < 5 AND rel_volume >= 2 — bounced 3-10 % off 52w low + meaningful green + hot vol (high-conviction bounce toward retest of recent lows; institutional bottom-fishing with elevated participation; potential continuation setup)
    HotVolBigChangeDayPctOpposite,       // change_pct.abs() > 3 AND day_pct * change_pct < 0 AND day_pct.abs() > 1 AND rel_volume >= 2 — big net move + intraday move in opposite direction + hot vol (intraday reversal fading the prior-close direction: gap dominated the net change, but regular hours pushed back the other way with elevated participation)
    HotVolBigChangeDayPctAligned,        // change_pct.abs() > 3 AND day_pct.abs() > 3 AND day_pct * change_pct > 0 AND rel_volume >= 2 — big net move + intraday move aligned with same direction + hot vol (full-conviction directional day: both overnight + regular hours pushed the same direction with elevated participation; two-leg trend confirmation)
    Year52HighBreakoutHotVolNoExtreme,   // year_high_pct < 0 AND change_pct > 1.5 AND change_pct < 4 AND rel_volume >= 1.5 AND rel_volume < 3 — new 52w high + modest green + moderate hot vol (controlled-conviction breakout to new highs; institutional accumulation without exhaustion; sustainable trend continuation candidate)
    Year52LowBreakdownHotVolNoExtreme,   // year_low_pct < 0 AND change_pct < -1.5 AND change_pct > -4 AND rel_volume >= 1.5 AND rel_volume < 3 — new 52w low + modest red + moderate hot vol (controlled-conviction breakdown to new lows; institutional distribution without panic; sustainable trend continuation candidate)
    BigGreenTopWickRejectHotVol,         // change_pct > 1 AND hod_dist_pct < -2 AND rel_volume >= 1.5 — green close + HOD far above (long upper wick) + hot vol (upper-wick rejection on a green day: rally faded into the close but still finished green; supply tested with elevated participation; potential follow-through hesitation)
    BigRedBottomWickRejectHotVol,        // change_pct < -1 AND lod_dist_pct > 2 AND rel_volume >= 1.5 — red close + LOD far below (long lower wick) + hot vol (lower-wick rejection on a red day: sell-off bounced into the close but still finished red; demand tested with elevated participation; potential follow-through hesitation)
    DryVolGreenCloseAtHodTinyRange,      // change_pct > 0.5 AND hod_dist_pct.abs() < 0.5 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 2 AND rel_volume < 0.7 — small green + closed at HOD + tight intraday range + dry vol (low-conviction grind-up day; small directional drift with no participation; weak-hands trend continuation candidate)
    DryVolRedCloseAtLodTinyRange,        // change_pct < -0.5 AND lod_dist_pct.abs() < 0.5 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 2 AND rel_volume < 0.7 — small red + closed at LOD + tight intraday range + dry vol (low-conviction grind-down day; small directional drift with no participation; weak-hands trend continuation candidate)
    Year52HighGapDownDryVolReclaim,      // year_high_pct < 3 AND year_high_pct >= -2 AND gap_pct < -1.5 AND change_pct > 0 AND rel_volume < 0.8 — near 52w high + opened with gap down + recovered to positive close + dry vol (gap reclaimed back into the breakout zone on light vol; weak-hands shaken out without participation flush; bullish continuation setup at the highs)
    Year52LowGapUpDryVolReject,          // year_low_pct < 3 AND year_low_pct >= -2 AND gap_pct > 1.5 AND change_pct < 0 AND rel_volume < 0.8 — near 52w low + opened with gap up + sold back into red close + dry vol (gap rejected back into the breakdown zone on light vol; weak-hands trapped without participation flush; bearish continuation setup at the lows)
    Year52HighInsideDayHotVol,           // year_high_pct < 3 AND year_high_pct >= -2 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1.5 AND change_pct.abs() < 0.5 AND rel_volume >= 1.5 — near 52w high + tight intraday range + flat close + hot vol (inside-day coil at the breakout zone with absorption; institutional accumulation just below resistance; high-probability breakout setup)
    Year52LowInsideDayHotVol,            // year_low_pct < 3 AND year_low_pct >= -2 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1.5 AND change_pct.abs() < 0.5 AND rel_volume >= 1.5 — near 52w low + tight intraday range + flat close + hot vol (inside-day coil at the breakdown zone with absorption; institutional distribution just above support; high-probability breakdown setup)
    Year52HighOutsideDayHotVol,          // year_high_pct < 3 AND year_high_pct >= -2 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 4 AND rel_volume >= 2 — near 52w high + wide intraday range + hot vol (outside-day rotation at the breakout zone; both supply and demand active just below resistance; volatility expansion preceding directional resolution)
    Year52LowOutsideDayHotVol,           // year_low_pct < 3 AND year_low_pct >= -2 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 4 AND rel_volume >= 2 — near 52w low + wide intraday range + hot vol (outside-day rotation at the breakdown zone; both supply and demand active just above support; volatility expansion preceding directional resolution)
    YearHighGapDownHotVolRecovery,       // year_high_pct < 0 AND gap_pct < -2 AND day_pct > 3 AND change_pct > 1 AND rel_volume >= 2 — new 52w high prior + gap down opening + huge intraday recovery + green close + hot vol (failed gap-down at the highs: shorts overpressed overnight, intraday short-squeeze fully reclaimed and pushed back into trend with elevated participation)
    YearLowGapUpHotVolRejection,         // year_low_pct < 0 AND gap_pct > 2 AND day_pct < -3 AND change_pct < -1 AND rel_volume >= 2 — new 52w low prior + gap up opening + huge intraday rejection + red close + hot vol (failed gap-up at the lows: longs overpressed overnight, intraday long-liquidation fully unwound and pushed back into trend with elevated participation)
    Year52HighReclaimAfterFlush,         // year_high_pct < 0 AND lod_dist_pct > 4 AND hod_dist_pct.abs() < 1 AND change_pct > 1 AND rel_volume >= 1.5 — new 52w high + LOD far below + close near HOD + green close + hot vol (intraday flush below the breakout level reclaimed back to highs by close; trapped breakdown shorts squeezed; conviction continuation candidate)
    Year52LowReclaimAfterPop,            // year_low_pct < 0 AND hod_dist_pct < -4 AND lod_dist_pct.abs() < 1 AND change_pct < -1 AND rel_volume >= 1.5 — new 52w low + HOD far above + close near LOD + red close + hot vol (intraday pop above the breakdown level rejected back to lows by close; trapped breakout longs flushed; conviction continuation candidate)
    BigDayPctSmallChangeHotVol,          // day_pct.abs() > 3 AND change_pct.abs() < 0.5 AND rel_volume >= 2 — big intraday move + flat net close + hot vol (full intraday reversal of overnight position: regular hours fully unwound any prior-close drift with elevated participation; rotation day with no net commitment)
    SmallDayPctBigChangeHotVol,          // day_pct.abs() < 0.5 AND change_pct.abs() > 3 AND rel_volume >= 2 — flat intraday move + big net close + hot vol (overnight gap held intact through regular hours: the entire daily move was the gap, intraday flat acceptance with elevated participation; gap-acceptance day)
    Year52HighRangeExpansionHotVol,      // year_high_pct < 0 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 5 AND change_pct > 2 AND rel_volume >= 2 — new 52w high + wide intraday range + green close + hot vol (volatility-expansion breakout at new highs: wide-range trend day after the breakout level; institutional follow-through with elevated participation)
    Year52LowRangeExpansionHotVol,       // year_low_pct < 0 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 5 AND change_pct < -2 AND rel_volume >= 2 — new 52w low + wide intraday range + red close + hot vol (volatility-expansion breakdown at new lows: wide-range trend day after the breakdown level; institutional follow-through with elevated participation)
    Year52HighRangeContractionHotVol,    // year_high_pct < 0 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1.5 AND change_pct > 0 AND change_pct < 1 AND rel_volume >= 2 — new 52w high + tight intraday range + small green close + hot vol (post-breakout absorption coil at new highs: tight digestion with elevated participation; pre-extension consolidation)
    Year52LowRangeContractionHotVol,     // year_low_pct < 0 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1.5 AND change_pct < 0 AND change_pct > -1 AND rel_volume >= 2 — new 52w low + tight intraday range + small red close + hot vol (post-breakdown absorption coil at new lows: tight digestion with elevated participation; pre-continuation consolidation)
    Year52HighBreakoutDryVolPullback,    // year_high_pct >= 0 AND year_high_pct < 2 AND change_pct < -0.5 AND change_pct > -2 AND rel_volume < 0.8 — just below 52w high + small red pullback + dry vol (low-vol pullback to retest the prior breakout level; weak-hands shaken out without participation; reclaim setup)
    Year52LowBreakdownDryVolBounce,      // year_low_pct >= 0 AND year_low_pct < 2 AND change_pct > 0.5 AND change_pct < 2 AND rel_volume < 0.8 — just above 52w low + small green bounce + dry vol (low-vol bounce to retest the prior breakdown level; weak-hands shaken out without participation; rejection setup)
    BigGapBigCounterMoveBigRangeHotVol,  // gap_pct.abs() > 3 AND day_pct * gap_pct < 0 AND day_pct.abs() > 3 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 5 AND rel_volume >= 2 — big gap + big opposite intraday move + wide range + hot vol (gap-and-fight reversal: significant overnight gap fully battled by the intraday session; two-sided rotation day with extended range and elevated participation; trapped gap traders flushed)
    BigGapBigContinuationBigDayPctHotVol, // gap_pct.abs() > 3 AND day_pct * gap_pct > 0 AND day_pct.abs() > 3 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 5 AND rel_volume >= 2 — big gap + big same-direction intraday + wide range + hot vol (gap-and-go acceleration: overnight gap held + extended further intraday with wide range; two-leg directional conviction with full range expansion)
    NoGapBigChangeBigDayPctHotVol,       // gap_pct.abs() < 0.5 AND change_pct.abs() > 4 AND day_pct.abs() > 4 AND rel_volume >= 2 — no overnight gap + big net move + big intraday move + hot vol (intraday-only conviction trend day: entire move built during regular hours with no overnight aid + matching intraday displacement; pure regular-hours directional thrust)
    MidYearHighBigGreenHotVol,           // year_high_pct >= 5 AND year_high_pct < 20 AND change_pct > 3 AND rel_volume >= 2 — middle-of-year-range + big green + hot vol (mid-range bullish thrust well above the floor but well below the ceiling; institutional momentum without breakout-fatigue or basing context)
    MidYearHighBigRedHotVol,             // year_high_pct >= 5 AND year_high_pct < 20 AND change_pct < -3 AND rel_volume >= 2 — middle-of-year-range + big red + hot vol (mid-range bearish thrust well below the ceiling but well above the floor; institutional distribution without breakdown-fatigue or topping context)
    MidYearLowBigRedHotVol,              // year_low_pct >= 5 AND year_low_pct < 20 AND change_pct < -3 AND rel_volume >= 2 — middle-of-year-range from low + big red + hot vol (rejection thrust well off the floor but still well below the ceiling; institutional distribution in the rebuild zone)
    MidYearLowBigGreenHotVol,            // year_low_pct >= 5 AND year_low_pct < 20 AND change_pct > 3 AND rel_volume >= 2 — middle-of-year-range from low + big green + hot vol (continuation thrust off the floor with institutional accumulation in the rebuild zone; recovery momentum without near-extreme volatility)
    Year52HighFullRangeDryVol,           // year_high_pct < 2 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 4 AND rel_volume < 0.8 — at 52w high + wide intraday range + dry vol (low-participation outside-day rotation at the highs; supply tested without conviction; failed exhaustion-vol setup)
    Year52LowFullRangeDryVol,            // year_low_pct < 2 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 4 AND rel_volume < 0.8 — at 52w low + wide intraday range + dry vol (low-participation outside-day rotation at the lows; demand tested without conviction; failed capitulation-vol setup)
    BigChangeBigRangeDryVol,             // change_pct.abs() > 4 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 5 AND rel_volume < 0.7 — big net move + wide intraday range + dry vol (no-participation thrust + wide range; illiquidity-driven volatility expansion without institutional commitment; fade candidate at scale)
    ExtremeVolFlatDay,                   // rel_volume >= 5 AND change_pct.abs() < 0.5 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 2 — extreme vol + flat net + tight range (stealth absorption at scale: extreme participation with no price expansion; institutional accumulation or distribution masked as a quiet day)
    ExtremeVolBigChangeClimax,           // rel_volume >= 5 AND change_pct.abs() > 5 — extreme vol + big net move (climax-style print: extreme participation + extreme directional commitment; potential trend continuation or terminal exhaustion depending on follow-through)
    ExtremeGapBigContinuationExtremeVol, // gap_pct.abs() > 5 AND change_pct.abs() > 8 AND gap_pct * change_pct > 0 AND rel_volume >= 5 — extreme gap + same-direction extreme continuation + extreme vol (gap-and-go acceleration at extreme scale: overnight thrust extended further during regular hours with climax-level participation; max-conviction trend day)
    ExtremeGapFullReversalExtremeVol,    // gap_pct.abs() > 5 AND gap_pct * change_pct < 0 AND change_pct.abs() > 3 AND rel_volume >= 5 — extreme gap + sign-flipped net close + extreme vol (extreme-gap fade: overnight thrust fully reversed by the intraday session with climax-level participation; trapped gap traders flushed at scale)
    ApathyAtYearHigh,                    // year_high_pct < 2 AND rel_volume < 0.3 AND change_pct.abs() < 0.3 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1.5 — at 52w high + extreme dry vol + flat close + tight range (total apathy at the breakout zone; neither buyers nor sellers committed; coiled-spring pre-breakout setup with no participation flush yet)
    ApathyAtYearLow,                     // year_low_pct < 2 AND rel_volume < 0.3 AND change_pct.abs() < 0.3 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1.5 — at 52w low + extreme dry vol + flat close + tight range (total apathy at the breakdown zone; neither buyers nor sellers committed; coiled-spring pre-breakdown setup with no participation flush yet)
    StealthAtYear52High,                 // year_high_pct < 2 AND change_pct.abs() < 0.5 AND rel_volume >= 5 — at 52w high + flat close + extreme vol (stealth distribution at the breakout zone: extreme participation with no net price expansion; institutional offloading masked as quiet acceptance)
    StealthAtYear52Low,                  // year_low_pct < 2 AND change_pct.abs() < 0.5 AND rel_volume >= 5 — at 52w low + flat close + extreme vol (stealth accumulation at the breakdown zone: extreme participation with no net price expansion; institutional bottom-fishing masked as quiet acceptance)
    ExtremeVolCloseAtHod,                // rel_volume >= 5 AND hod_dist_pct.abs() < 0.5 — extreme vol + close pinned to HOD (max-conviction bullish close at any price level: extreme participation finishing on the highs; institutional ramp into the close)
    ExtremeVolCloseAtLod,                // rel_volume >= 5 AND lod_dist_pct.abs() < 0.5 — extreme vol + close pinned to LOD (max-conviction bearish close at any price level: extreme participation finishing on the lows; institutional dump into the close)
    ExtremeRangeExtremeVol,              // hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 5 — extreme intraday range + extreme vol (extreme two-sided rotation: institutional fight day with wide whipsaw range and climax-level participation; max-volatility regime print)
    ExtremeRangeDryVol,                  // hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume < 0.5 — extreme intraday range + dry vol (thin-liquidity whipsaw: extreme range expansion with no institutional sponsorship; gappy market-maker void or low-volume rip; fade with caution)
    BigGreenUpperRangeHotVol,            // change_pct > 2 AND lod_dist_pct > 2 * hod_dist_pct.abs() AND rel_volume >= 2 — big green + close clearly in upper portion of intraday range + hot vol (bullish strength close in the top half of the intraday range without requiring close pinned to HOD; demand-side dominance with elevated participation)
    BigRedLowerRangeHotVol,              // change_pct < -2 AND hod_dist_pct.abs() > 2 * lod_dist_pct AND rel_volume >= 2 — big red + close clearly in lower portion of intraday range + hot vol (bearish weakness close in the bottom half of the intraday range without requiring close pinned to LOD; supply-side dominance with elevated participation)
    BigBreakoutAboveYearHigh,            // year_high_pct < -3 AND change_pct > 1 AND rel_volume >= 1.5 — close more than 3% above the prior 52w high + green + hot vol (deep breakout extension: not just a fresh peak but materially above it; price-discovery expansion with elevated participation)
    BigBreakdownBelowYearLow,            // year_low_pct < -3 AND change_pct < -1 AND rel_volume >= 1.5 — close more than 3% below the prior 52w low + red + hot vol (deep breakdown extension: not just a fresh trough but materially below it; price-discovery contraction with elevated participation)
    DeepPullbackBigGreenHotVol,          // year_high_pct >= 10 AND year_high_pct < 30 AND change_pct > 2 AND rel_volume >= 2 — 10-30 % below 52w high + big green + hot vol (recovery thrust from deep-pullback zone with institutional accumulation; pre-retest momentum candidate)
    DeepPullbackBigRedHotVol,            // year_high_pct >= 10 AND year_high_pct < 30 AND change_pct < -2 AND rel_volume >= 2 — 10-30 % below 52w high + big red + hot vol (continuation thrust deeper into pullback zone with institutional distribution; trend-break confirmation candidate)
    DeepBounceBigGreenHotVol,            // year_low_pct >= 10 AND year_low_pct < 30 AND change_pct > 2 AND rel_volume >= 2 — 10-30 % above 52w low + big green + hot vol (continuation thrust away from the floor with institutional accumulation; recovery momentum well off the trough)
    DeepBounceBigRedHotVol,              // year_low_pct >= 10 AND year_low_pct < 30 AND change_pct < -2 AND rel_volume >= 2 — 10-30 % above 52w low + big red + hot vol (retracement thrust back toward the floor with institutional distribution; bounce-failure confirmation candidate)
    BigGapDownReclaimedToHodHotVol,      // gap_pct < -2 AND hod_dist_pct.abs() < 0.5 AND rel_volume >= 2 — gapped down + close pinned to HOD + hot vol (full intraday recovery from the gap-down open to the session high; trapped overnight shorts squeezed all the way to the highs with elevated participation)
    BigGapUpRejectedToLodHotVol,         // gap_pct > 2 AND lod_dist_pct.abs() < 0.5 AND rel_volume >= 2 — gapped up + close pinned to LOD + hot vol (full intraday rejection from the gap-up open to the session low; trapped overnight longs flushed all the way to the lows with elevated participation)
    TenXVolMicroChange,                  // rel_volume >= 10 AND change_pct.abs() < 0.3 — 10x average vol + microchange close (rare absorption-at-scale print: extreme participation with virtually no net price movement; large institutional position-build or unwind masked as a quiet day)
    TenXVolNoGapBigIntradayMove,         // rel_volume >= 10 AND gap_pct.abs() < 0.3 AND change_pct.abs() > 3 — 10x average vol + no overnight gap + big intraday move (pure regular-hours extreme thrust: no overnight aid, climax-level participation, all directional commitment built during the session)
    MicroVolBigChange,                   // rel_volume < 0.1 AND change_pct.abs() > 3 — 10 % of average vol + big net move (dead-stock surprise: illiquidity-driven extreme move with virtually no participation; thin market-maker quote rip or holiday/holiday-eve thin-tape print)
    MicroVolFlatDay,                     // rel_volume < 0.1 AND change_pct.abs() < 0.3 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1.5 — 10 % of average vol + flat close + tight range (total dead stock: no participation and no price movement; delisting candidate or fully forgotten name)
    ConfirmedBreakoutAboveYearHigh,      // year_high_pct >= -3 AND year_high_pct < -1 AND change_pct > 1 AND rel_volume >= 1.5 — 1-3 % above prior 52w high + green + hot vol (solid confirmed breakout: clearly past resistance but not yet parabolic; trend-establishment zone for new highs)
    ConfirmedBreakdownBelowYearLow,      // year_low_pct >= -3 AND year_low_pct < -1 AND change_pct < -1 AND rel_volume >= 1.5 — 1-3 % below prior 52w low + red + hot vol (solid confirmed breakdown: clearly past support but not yet panicked; trend-establishment zone for new lows)
    UpperWickFlatCloseHotVol,            // hod_dist_pct < -3 AND change_pct.abs() < 1 AND gap_pct.abs() < 1 AND rel_volume >= 1.5 — long upper wick + flat net close + no overnight gap + hot vol (pure supply test: intraday rally rejected back to roughly the open with elevated participation; neither bull nor bear net but ceiling tested)
    LowerWickFlatCloseHotVol,            // lod_dist_pct > 3 AND change_pct.abs() < 1 AND gap_pct.abs() < 1 AND rel_volume >= 1.5 — long lower wick + flat net close + no overnight gap + hot vol (pure demand test: intraday sell-off bounced back to roughly the open with elevated participation; neither bull nor bear net but floor tested)
    PartialGapUpHoldHotVol,              // gap_pct > 2 AND day_pct < -0.5 AND change_pct > 0 AND rel_volume >= 1.5 — gap up + intraday sold from open + still closed green + hot vol (partial gap-up fade: overnight thrust partially eroded intraday but the gap held the prior close; tested but not flushed)
    PartialGapDownHoldHotVol,            // gap_pct < -2 AND day_pct > 0.5 AND change_pct < 0 AND rel_volume >= 1.5 — gap down + intraday recovered from open + still closed red + hot vol (partial gap-down fade: overnight thrust partially recovered intraday but the gap held below the prior close; tested but not reclaimed)
    BreakoutZoneRangeExpansionHotVol,    // year_high_pct >= -3 AND year_high_pct < 0 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 5 AND rel_volume >= 2 — fresh 52w breakout zone + wide intraday range + hot vol (volatility expansion right at the breakout level: institutional fight day occurring as new high is being established with elevated participation)
    BreakdownZoneRangeExpansionHotVol,   // year_low_pct >= -3 AND year_low_pct < 0 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 5 AND rel_volume >= 2 — fresh 52w breakdown zone + wide intraday range + hot vol (volatility expansion right at the breakdown level: institutional fight day occurring as new low is being established with elevated participation)
    Year52HighFreshConsolidationDryVol,  // year_high_pct < 0 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1.5 AND change_pct.abs() < 0.5 AND rel_volume < 0.8 — close above prior 52w high + tight intraday range + flat close + dry vol (quiet acceptance of the new high: post-breakout consolidation without participation flush; move stalled but did not reverse)
    Year52LowFreshConsolidationDryVol,   // year_low_pct < 0 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1.5 AND change_pct.abs() < 0.5 AND rel_volume < 0.8 — close below prior 52w low + tight intraday range + flat close + dry vol (quiet acceptance of the new low: post-breakdown consolidation without participation flush; move stalled but did not reverse)
    ModerateGapBullContinuationHotVol,   // gap_pct >= 1 AND gap_pct <= 2 AND change_pct > 2 AND rel_volume >= 2 — moderate gap up + bull continuation + hot vol (modest gap held + extended further during regular hours; the in-between gap regime not large enough for blow-off but big enough for directional commitment with elevated participation)
    ModerateGapBearContinuationHotVol,   // gap_pct >= -2 AND gap_pct <= -1 AND change_pct < -2 AND rel_volume >= 2 — moderate gap down + bear continuation + hot vol (modest gap held + extended further during regular hours; the in-between gap regime not large enough for panic but big enough for directional commitment with elevated participation)
    ModerateGapBullContinuationDryVol,   // gap_pct >= 1 AND gap_pct <= 2 AND change_pct > 2 AND rel_volume < 0.8 — moderate gap up + bull continuation + dry vol (modest gap extended on no participation; suspect rally without institutional sponsorship; fade-prone setup despite positive net move)
    ModerateGapBearContinuationDryVol,   // gap_pct >= -2 AND gap_pct <= -1 AND change_pct < -2 AND rel_volume < 0.8 — moderate gap down + bear continuation + dry vol (modest gap extended on no participation; suspect decline without institutional sponsorship; fade-prone setup despite negative net move)
    ModerateGapBullFadeHotVol,           // gap_pct >= 1 AND gap_pct <= 2 AND change_pct < -1 AND rel_volume >= 2 — moderate gap up + closed red + hot vol (moderate-gap fade reversal: gap up was sold throughout the session below prior close; trapped overnight longs flushed with elevated participation; reversal-short signal)
    ModerateGapBearReclaimHotVol,        // gap_pct >= -2 AND gap_pct <= -1 AND change_pct > 1 AND rel_volume >= 2 — moderate gap down + closed green + hot vol (moderate-gap reclaim reversal: gap down was bought throughout the session above prior close; trapped overnight shorts squeezed with elevated participation; reversal-long signal)
    ConfirmedBreakoutFadeHotVol,         // year_high_pct >= -3 AND year_high_pct < -1 AND change_pct < -0.5 AND rel_volume >= 1.5 — 1-3 % above prior 52w high + red + hot vol (confirmed-breakout pullback: was clearly past resistance, now dropping back into the breakout zone with elevated participation; failed-breakout risk signal)
    ConfirmedBreakdownReclaimHotVol,     // year_low_pct >= -3 AND year_low_pct < -1 AND change_pct > 0.5 AND rel_volume >= 1.5 — 1-3 % below prior 52w low + green + hot vol (confirmed-breakdown bounce: was clearly past support, now rallying back into the breakdown zone with elevated participation; failed-breakdown risk signal)
    IntradayBullDriveAtYear52High,       // year_high_pct < 2 AND day_pct > 3 AND rel_volume >= 2 — at 52w high + big intraday drive up + hot vol (intraday-led bullish thrust at the breakout zone regardless of overnight context; regular-hours momentum confirmation right at resistance)
    IntradayBearDriveAtYear52Low,        // year_low_pct < 2 AND day_pct < -3 AND rel_volume >= 2 — at 52w low + big intraday drive down + hot vol (intraday-led bearish thrust at the breakdown zone regardless of overnight context; regular-hours momentum confirmation right at support)
    IntradayBearDriveAtYear52High,       // year_high_pct < 2 AND day_pct < -3 AND rel_volume >= 2 — at 52w high + big intraday drive DOWN + hot vol (intraday-led bearish rejection at the breakout zone: selling pressure through regular hours pushed the price back from the highs with elevated participation; failed-breakout candidate)
    IntradayBullDriveAtYear52Low,        // year_low_pct < 2 AND day_pct > 3 AND rel_volume >= 2 — at 52w low + big intraday drive UP + hot vol (intraday-led bullish recovery at the breakdown zone: buying pressure through regular hours lifted the price off the lows with elevated participation; failed-breakdown candidate)
    IntradayBullDriveBelowYearHigh,      // year_high_pct >= 0 AND year_high_pct < 5 AND day_pct > 3 AND rel_volume >= 2 — within 5% below 52w high + big intraday drive up + hot vol (pre-breakout intraday surge: approaching resistance with regular-hours momentum and elevated participation; breakout-setup candidate)
    IntradayBearDriveAboveYearLow,       // year_low_pct >= 0 AND year_low_pct < 5 AND day_pct < -3 AND rel_volume >= 2 — within 5% above 52w low + big intraday drive down + hot vol (pre-breakdown intraday plunge: approaching support with regular-hours momentum and elevated participation; breakdown-setup candidate)
    HammerAtYear52Low,                   // year_low_pct < 2 AND lod_dist_pct > 3 AND hod_dist_pct.abs() < 1 AND change_pct > 0 AND rel_volume >= 1.5 — at 52w low + long lower wick + close near HOD + green close + hot vol (classic hammer reversal at the breakdown floor: intraday plunge reclaimed with green finish and elevated participation; high-probability bottom-fishing signal)
    ShootingStarAtYear52High,            // year_high_pct < 2 AND hod_dist_pct < -3 AND lod_dist_pct.abs() < 1 AND change_pct < 0 AND rel_volume >= 1.5 — at 52w high + long upper wick + close near LOD + red close + hot vol (classic shooting star reversal at the breakout ceiling: intraday rip sold with red finish and elevated participation; high-probability topping signal)
    MarubozuGreenAtYear52High,           // year_high_pct < 2 AND change_pct > 3 AND hod_dist_pct.abs() < 0.3 AND gap_pct.abs() < 1 AND rel_volume >= 2 — at 52w high + green marubozu + no overnight gap + hot vol (full intraday breakout trend day at the breakout zone: regular-hours conviction climbed from the open to the high with no gap aid; max-conviction breakout day)
    MarubozuRedAtYear52Low,              // year_low_pct < 2 AND change_pct < -3 AND lod_dist_pct.abs() < 0.3 AND gap_pct.abs() < 1 AND rel_volume >= 2 — at 52w low + red marubozu + no overnight gap + hot vol (full intraday breakdown trend day at the breakdown zone: regular-hours conviction fell from the open to the low with no gap aid; max-conviction breakdown day)
    DragonflyDojiAtYear52Low,            // year_low_pct < 2 AND change_pct.abs() < 0.3 AND lod_dist_pct > 4 AND hod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — at 52w low + flat close + LOD far below + close near HOD + hot vol (classic dragonfly doji at the breakdown floor: intraday plunge fully reclaimed by the close with elevated participation; high-probability bottom-fishing signal at the year low)
    GravestoneDojiAtYear52High,          // year_high_pct < 2 AND change_pct.abs() < 0.3 AND hod_dist_pct < -4 AND lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — at 52w high + flat close + HOD far above + close near LOD + hot vol (classic gravestone doji at the breakout ceiling: intraday rip fully sold by the close with elevated participation; high-probability topping signal at the year high)
    HammerAtMidYearLowRange,             // year_low_pct >= 5 AND year_low_pct < 20 AND lod_dist_pct > 3 AND hod_dist_pct.abs() < 1 AND change_pct > 0 AND rel_volume >= 1.5 — mid-range from low (5-20% above 52w low) + long lower wick + close near HOD + green close + hot vol (hammer reversal in the recovery zone: intraday plunge reclaimed with green finish above the floor; mid-cycle bottom-fishing signal)
    ShootingStarAtMidYearHighRange,      // year_high_pct >= 5 AND year_high_pct < 20 AND hod_dist_pct < -3 AND lod_dist_pct.abs() < 1 AND change_pct < 0 AND rel_volume >= 1.5 — mid-range from high (5-20% below 52w high) + long upper wick + close near LOD + red close + hot vol (shooting star reversal in the topping zone: intraday rip sold with red finish below the ceiling; mid-cycle topping signal)
    HammerAtDeepPullback,                // year_high_pct >= 10 AND year_high_pct < 30 AND lod_dist_pct > 3 AND hod_dist_pct.abs() < 1 AND change_pct > 0 AND rel_volume >= 1.5 — deep pullback zone (10-30% below 52w high) + long lower wick + close near HOD + green close + hot vol (hammer reversal deep into the pullback: intraday plunge reclaimed in the rebuild zone; counter-trend bounce candidate)
    ShootingStarAtDeepBounce,            // year_low_pct >= 10 AND year_low_pct < 30 AND hod_dist_pct < -3 AND lod_dist_pct.abs() < 1 AND change_pct < 0 AND rel_volume >= 1.5 — deep bounce zone (10-30% above 52w low) + long upper wick + close near LOD + red close + hot vol (shooting star reversal deep into the bounce: intraday rip sold in the rebuild zone; counter-trend rejection candidate)
    DragonflyDojiAtMidYearLow,           // year_low_pct >= 5 AND year_low_pct < 20 AND change_pct.abs() < 0.3 AND lod_dist_pct > 4 AND hod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — mid-range from low (5-20% above 52w low) + flat close + LOD far below + close near HOD + hot vol (dragonfly doji reversal in the recovery zone: intraday plunge fully reclaimed by close in the mid-range; demand-test signal away from the floor)
    GravestoneDojiAtMidYearHigh,         // year_high_pct >= 5 AND year_high_pct < 20 AND change_pct.abs() < 0.3 AND hod_dist_pct < -4 AND lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — mid-range from high (5-20% below 52w high) + flat close + HOD far above + close near LOD + hot vol (gravestone doji reversal in the topping zone: intraday rip fully sold by close in the mid-range; supply-test signal away from the ceiling)
    DragonflyDojiAtDeepPullback,         // year_high_pct >= 10 AND year_high_pct < 30 AND change_pct.abs() < 0.3 AND lod_dist_pct > 4 AND hod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — deep pullback zone (10-30% below 52w high) + flat close + LOD far below + close near HOD + hot vol (dragonfly doji reversal deep into the pullback: intraday plunge reclaimed in the rebuild zone with flat close; counter-trend demand-test signal)
    GravestoneDojiAtDeepBounce,          // year_low_pct >= 10 AND year_low_pct < 30 AND change_pct.abs() < 0.3 AND hod_dist_pct < -4 AND lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — deep bounce zone (10-30% above 52w low) + flat close + HOD far above + close near LOD + hot vol (gravestone doji reversal deep into the bounce: intraday rip sold in the rebuild zone with flat close; counter-trend supply-test signal)
    MarubozuGreenAtMidYearHigh,          // year_high_pct >= 5 AND year_high_pct < 20 AND change_pct > 3 AND hod_dist_pct.abs() < 0.3 AND gap_pct.abs() < 1 AND rel_volume >= 2 — mid-range from high (5-20% below 52w high) + green marubozu + no overnight gap + hot vol (full intraday recovery trend day in the pullback zone: regular-hours conviction lifted from the open to the high with no gap aid; max-conviction mid-cycle bounce day)
    MarubozuRedAtMidYearLow,             // year_low_pct >= 5 AND year_low_pct < 20 AND change_pct < -3 AND lod_dist_pct.abs() < 0.3 AND gap_pct.abs() < 1 AND rel_volume >= 2 — mid-range from low (5-20% above 52w low) + red marubozu + no overnight gap + hot vol (full intraday retracement trend day in the bounce zone: regular-hours conviction fell from the open to the low with no gap aid; max-conviction mid-cycle rejection day)
    MarubozuGreenAtDeepPullback,         // year_high_pct >= 10 AND year_high_pct < 30 AND change_pct > 3 AND hod_dist_pct.abs() < 0.3 AND gap_pct.abs() < 1 AND rel_volume >= 2 — deep pullback zone (10-30% below 52w high) + green marubozu + no overnight gap + hot vol (full intraday recovery trend day deep in the pullback: regular-hours conviction lifted from the open to the high with no gap aid; max-conviction counter-trend bounce thrust)
    MarubozuRedAtDeepBounce,             // year_low_pct >= 10 AND year_low_pct < 30 AND change_pct < -3 AND lod_dist_pct.abs() < 0.3 AND gap_pct.abs() < 1 AND rel_volume >= 2 — deep bounce zone (10-30% above 52w low) + red marubozu + no overnight gap + hot vol (full intraday retracement trend day deep in the bounce: regular-hours conviction fell from the open to the low with no gap aid; max-conviction counter-trend rejection thrust)
    HammerAtDeepDiscount,                // year_high_pct >= 30 AND lod_dist_pct > 3 AND hod_dist_pct.abs() < 1 AND change_pct > 0 AND rel_volume >= 1.5 — deep discount zone (>=30% below 52w high) + long lower wick + close near HOD + green close + hot vol (hammer reversal at the deep-discount floor: intraday plunge reclaimed in a beaten-down name with elevated participation; turnaround candidate after extended decline)
    ShootingStarAtDeepPremium,           // year_low_pct >= 30 AND hod_dist_pct < -3 AND lod_dist_pct.abs() < 1 AND change_pct < 0 AND rel_volume >= 1.5 — deep premium zone (>=30% above 52w low) + long upper wick + close near LOD + red close + hot vol (shooting star reversal at the deep-premium ceiling: intraday rip sold in a runaway name with elevated participation; exhaustion candidate after extended advance)
    DragonflyDojiAtDeepDiscount,         // year_high_pct >= 30 AND change_pct.abs() < 0.3 AND lod_dist_pct > 4 AND hod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — deep discount zone (>=30% below 52w high) + flat close + LOD far below + close near HOD + hot vol (dragonfly doji reversal in the deep-discount floor: intraday plunge fully reclaimed by close in a beaten-down name with elevated participation; turnaround demand-test signal)
    GravestoneDojiAtDeepPremium,         // year_low_pct >= 30 AND change_pct.abs() < 0.3 AND hod_dist_pct < -4 AND lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — deep premium zone (>=30% above 52w low) + flat close + HOD far above + close near LOD + hot vol (gravestone doji reversal in the deep-premium ceiling: intraday rip fully sold by close in a runaway name with elevated participation; exhaustion supply-test signal)
    MarubozuGreenAtDeepDiscount,         // year_high_pct >= 30 AND change_pct > 3 AND hod_dist_pct.abs() < 0.3 AND gap_pct.abs() < 1 AND rel_volume >= 2 — deep discount zone (>=30% below 52w high) + green marubozu + no overnight gap + hot vol (full intraday recovery trend day in a beaten-down name: regular-hours conviction lifted from open to high with no gap aid; max-conviction turnaround thrust after extended decline)
    MarubozuRedAtDeepPremium,            // year_low_pct >= 30 AND change_pct < -3 AND lod_dist_pct.abs() < 0.3 AND gap_pct.abs() < 1 AND rel_volume >= 2 — deep premium zone (>=30% above 52w low) + red marubozu + no overnight gap + hot vol (full intraday rejection trend day in a runaway name: regular-hours conviction fell from open to low with no gap aid; max-conviction exhaustion thrust after extended advance)
    HammerAtYear52High,                  // year_high_pct < 2 AND lod_dist_pct > 3 AND hod_dist_pct.abs() < 1 AND change_pct > 0 AND rel_volume >= 1.5 — at 52w high + long lower wick + close near HOD + green close + hot vol (buying pressure tested but reclaimed at the breakout ceiling: intraday plunge bought back to the highs; continuation-of-trend resilience signal at the top)
    ShootingStarAtYear52Low,             // year_low_pct < 2 AND hod_dist_pct < -3 AND lod_dist_pct.abs() < 1 AND change_pct < 0 AND rel_volume >= 1.5 — at 52w low + long upper wick + close near LOD + red close + hot vol (selling pressure persists at the breakdown floor: intraday rip sold back to the lows; continuation-of-trend weakness signal at the bottom)
    HammerAtMidYearHighRange,            // year_high_pct >= 5 AND year_high_pct < 20 AND lod_dist_pct > 3 AND hod_dist_pct.abs() < 1 AND change_pct > 0 AND rel_volume >= 1.5 — mid-range from high (5-20% below 52w high) + long lower wick + close near HOD + green close + hot vol (buying pressure tested but reclaimed in the pullback zone: intraday plunge bought back toward the prior peak; bull-continuation hammer signaling pullback exhaustion)
    ShootingStarAtMidYearLowRange,       // year_low_pct >= 5 AND year_low_pct < 20 AND hod_dist_pct < -3 AND lod_dist_pct.abs() < 1 AND change_pct < 0 AND rel_volume >= 1.5 — mid-range from low (5-20% above 52w low) + long upper wick + close near LOD + red close + hot vol (selling pressure tested in the bounce zone: intraday rip sold back toward the prior trough; bear-continuation shooting star signaling bounce exhaustion)
    HammerAtDeepBounceContinuation,      // year_low_pct >= 10 AND year_low_pct < 30 AND lod_dist_pct > 3 AND hod_dist_pct.abs() < 1 AND change_pct > 0 AND rel_volume >= 1.5 — deep bounce zone (10-30% above 52w low) + long lower wick + close near HOD + green close + hot vol (bull-continuation hammer in the rebuild zone: intraday plunge reclaimed without retesting the floor; recovery momentum confirmation deep into the bounce)
    ShootingStarAtDeepPullbackContinuation, // year_high_pct >= 10 AND year_high_pct < 30 AND hod_dist_pct < -3 AND lod_dist_pct.abs() < 1 AND change_pct < 0 AND rel_volume >= 1.5 — deep pullback zone (10-30% below 52w high) + long upper wick + close near LOD + red close + hot vol (bear-continuation shooting star in the pullback zone: intraday rip sold without retesting the ceiling; decline momentum confirmation deep into the pullback)
    HammerAtDeepPremiumContinuation,     // year_low_pct >= 30 AND lod_dist_pct > 3 AND hod_dist_pct.abs() < 1 AND change_pct > 0 AND rel_volume >= 1.5 — deep premium zone (>=30% above 52w low) + long lower wick + close near HOD + green close + hot vol (bull-continuation hammer in a runaway name: intraday plunge reclaimed deep in the extended advance; trend-resilience signal far from the floor)
    ShootingStarAtDeepDiscountContinuation, // year_high_pct >= 30 AND hod_dist_pct < -3 AND lod_dist_pct.abs() < 1 AND change_pct < 0 AND rel_volume >= 1.5 — deep discount zone (>=30% below 52w high) + long upper wick + close near LOD + red close + hot vol (bear-continuation shooting star in a beaten-down name: intraday rip sold deep in the extended decline; trend-weakness signal far from the ceiling)
    BothLongWicksHotVol,                 // hod_dist_pct.abs() > 2 AND lod_dist_pct > 2 AND rel_volume >= 2 — long upper wick + long lower wick + hot vol (two-sided exploration day with elevated participation: both supply and demand tested at extremes; high-rotation indecision day with full intraday whipsaw range)
    BothShortWicksTinyChangeHotVol,      // hod_dist_pct.abs() < 0.5 AND lod_dist_pct < 0.5 AND change_pct.abs() < 0.3 AND rel_volume >= 2 — short upper wick + short lower wick + flat close + hot vol (compressed-cylinder day: close pinned with no wick exploration on either side; pre-breakout coil with elevated absorption at a specific price)
    HammerAtConfirmedBreakdown,          // year_low_pct >= -3 AND year_low_pct < -1 AND lod_dist_pct > 3 AND hod_dist_pct.abs() < 1 AND change_pct > 0 AND rel_volume >= 1.5 — 1-3% below prior 52w low + long lower wick + close near HOD + green close + hot vol (failed-breakdown hammer: confirmed breakdown level retested intraday then reclaimed back above prior support; potential failed-breakdown reversal signal)
    ShootingStarAtConfirmedBreakout,     // year_high_pct >= -3 AND year_high_pct < -1 AND hod_dist_pct < -3 AND lod_dist_pct.abs() < 1 AND change_pct < 0 AND rel_volume >= 1.5 — 1-3% above prior 52w high + long upper wick + close near LOD + red close + hot vol (failed-breakout shooting star: confirmed breakout level retested intraday then rejected back below prior resistance; potential failed-breakout reversal signal)
    DragonflyDojiAtConfirmedBreakdown,   // year_low_pct >= -3 AND year_low_pct < -1 AND change_pct.abs() < 0.3 AND lod_dist_pct > 4 AND hod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — 1-3% below prior 52w low + flat close + LOD far below + close near HOD + hot vol (dragonfly doji at confirmed breakdown: intraday plunge fully reclaimed by close with flat finish above prior support; failed-breakdown demand-test signal)
    GravestoneDojiAtConfirmedBreakout,   // year_high_pct >= -3 AND year_high_pct < -1 AND change_pct.abs() < 0.3 AND hod_dist_pct < -4 AND lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — 1-3% above prior 52w high + flat close + HOD far above + close near LOD + hot vol (gravestone doji at confirmed breakout: intraday rip fully sold by close with flat finish below prior resistance; failed-breakout supply-test signal)
    MarubozuGreenAtConfirmedBreakout,    // year_high_pct >= -3 AND year_high_pct < -1 AND change_pct > 3 AND hod_dist_pct.abs() < 0.3 AND gap_pct.abs() < 1 AND rel_volume >= 2 — 1-3% above prior 52w high + green marubozu + no overnight gap + hot vol (full intraday extension trend day after confirmed breakout: regular-hours conviction lifted from open to high with no gap aid; max-conviction follow-through above resistance)
    MarubozuRedAtConfirmedBreakdown,     // year_low_pct >= -3 AND year_low_pct < -1 AND change_pct < -3 AND lod_dist_pct.abs() < 0.3 AND gap_pct.abs() < 1 AND rel_volume >= 2 — 1-3% below prior 52w low + red marubozu + no overnight gap + hot vol (full intraday extension trend day after confirmed breakdown: regular-hours conviction fell from open to low with no gap aid; max-conviction follow-through below support)
    MarubozuRedAtConfirmedBreakout,      // year_high_pct >= -3 AND year_high_pct < -1 AND change_pct < -3 AND lod_dist_pct.abs() < 0.3 AND gap_pct.abs() < 1 AND rel_volume >= 2 — 1-3% above prior 52w high + red marubozu + no overnight gap + hot vol (full intraday rejection trend day after confirmed breakout: regular-hours conviction fell from open to low with no gap aid; max-conviction failed-breakout fade returning below resistance)
    MarubozuGreenAtConfirmedBreakdown,   // year_low_pct >= -3 AND year_low_pct < -1 AND change_pct > 3 AND hod_dist_pct.abs() < 0.3 AND gap_pct.abs() < 1 AND rel_volume >= 2 — 1-3% below prior 52w low + green marubozu + no overnight gap + hot vol (full intraday recovery trend day after confirmed breakdown: regular-hours conviction lifted from open to high with no gap aid; max-conviction failed-breakdown reclaim above support)
    TripleAlignedBullBigConvictionDay,   // gap_pct > 1.5 AND change_pct > 3 AND day_pct > 1.5 AND rel_volume >= 2 — gap up + big net move + big intraday up + hot vol (triple-aligned bullish conviction: overnight, regular hours, and net all moved meaningfully in the same direction with elevated participation; full-stack directional commitment day)
    TripleAlignedBearBigConvictionDay,   // gap_pct < -1.5 AND change_pct < -3 AND day_pct < -1.5 AND rel_volume >= 2 — gap down + big net move + big intraday down + hot vol (triple-aligned bearish conviction: overnight, regular hours, and net all moved meaningfully in the same direction with elevated participation; full-stack directional commitment day)
    DistantFromYearHighRangeContractionHotVol, // year_high_pct >= 20 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1.5 AND change_pct.abs() < 0.5 AND rel_volume >= 2 — far below 52w high (>=20%) + tight intraday range + flat close + hot vol (stealth absorption in the pullback territory: tight digestion with elevated participation deep below the prior peak; pre-reversal coil with institutional positioning)
    DistantFromYearLowRangeContractionHotVol,  // year_low_pct >= 20 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1.5 AND change_pct.abs() < 0.5 AND rel_volume >= 2 — far above 52w low (>=20%) + tight intraday range + flat close + hot vol (stealth absorption in the advance territory: tight digestion with elevated participation deep above the prior trough; pre-reversal coil with institutional positioning)
    DistantFromYearHighRangeExpansionHotVol,   // year_high_pct >= 20 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 5 AND rel_volume >= 2 — far below 52w high (>=20%) + wide intraday range + hot vol (volatility expansion deep in the pullback territory: institutional fight day occurring well below the prior peak with elevated participation; regime-shift candidate after extended decline)
    DistantFromYearLowRangeExpansionHotVol,    // year_low_pct >= 20 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 5 AND rel_volume >= 2 — far above 52w low (>=20%) + wide intraday range + hot vol (volatility expansion deep in the advance territory: institutional fight day occurring well above the prior trough with elevated participation; regime-shift candidate after extended advance)
    CloseAtHodMidYearLowHotVol,                // year_low_pct >= 5 AND year_low_pct < 20 AND hod_dist_pct.abs() < 0.5 AND change_pct > 1 AND rel_volume >= 1.5 — mid-range from low (5-20%) + close pinned to HOD + green close + hot vol (closing-strength signal in the recovery zone: bull conviction ramped into the close without requiring a long lower-wick reclaim; demand-led mid-cycle continuation)
    CloseAtLodMidYearHighHotVol,               // year_high_pct >= 5 AND year_high_pct < 20 AND lod_dist_pct.abs() < 0.5 AND change_pct < -1 AND rel_volume >= 1.5 — mid-range from high (5-20%) + close pinned to LOD + red close + hot vol (closing-weakness signal in the pullback zone: bear conviction dumped into the close without requiring a long upper-wick rejection; supply-led mid-cycle continuation)
    CloseAtHodDeepBelowYearHighHotVol,         // year_high_pct >= 20 AND hod_dist_pct.abs() < 0.5 AND change_pct > 1 AND rel_volume >= 1.5 — far below 52w high (>=20%) + close pinned to HOD + green close + hot vol (closing-strength signal deep in the pullback zone: bull conviction ramped into the close well below the prior peak; early-reversal candidate after extended decline without requiring wick-rejection context)
    CloseAtLodDeepAboveYearLowHotVol,          // year_low_pct >= 20 AND lod_dist_pct.abs() < 0.5 AND change_pct < -1 AND rel_volume >= 1.5 — far above 52w low (>=20%) + close pinned to LOD + red close + hot vol (closing-weakness signal deep in the advance zone: bear conviction dumped into the close well above the prior trough; early-reversal candidate after extended advance without requiring wick-rejection context)
    CloseAtHodNearYearHighHotVol,              // year_high_pct < 2 AND hod_dist_pct.abs() < 0.5 AND change_pct > 1 AND rel_volume >= 1.5 — at/near 52w high (<2%) + close pinned to HOD + green close + hot vol (freshest possible breakout signal: closing at the high of day at the high of the year on elevated participation; momentum-continuation with no overhead supply)
    CloseAtLodNearYearLowHotVol,               // year_low_pct < 2 AND lod_dist_pct.abs() < 0.5 AND change_pct < -1 AND rel_volume >= 1.5 — at/near 52w low (<2%) + close pinned to LOD + red close + hot vol (freshest possible breakdown signal: closing at the low of day at the low of the year on elevated participation; momentum-continuation with no underlying support)
    CloseAtHodJustOffYearHighHotVol,           // year_high_pct >= 2 AND year_high_pct < 5 AND hod_dist_pct.abs() < 0.5 AND change_pct > 1 AND rel_volume >= 1.5 — just off 52w high (2-5%) + close pinned to HOD + green close + hot vol (re-assertion of breakout momentum after a shallow pullback: closing strength in the immediate post-extreme zone signals continuation candidate with minimal overhead resistance)
    CloseAtLodJustOffYearLowHotVol,            // year_low_pct >= 2 AND year_low_pct < 5 AND lod_dist_pct.abs() < 0.5 AND change_pct < -1 AND rel_volume >= 1.5 — just off 52w low (2-5%) + close pinned to LOD + red close + hot vol (re-assertion of breakdown momentum after a shallow bounce: closing weakness in the immediate post-extreme zone signals continuation candidate with minimal underlying support)
    CloseAtHodConfirmedAboveYearHighHotVol,    // year_high_pct >= -3 AND year_high_pct <= -1 AND hod_dist_pct.abs() < 0.5 AND change_pct > 1 AND rel_volume >= 1.5 — already 1-3% past 52w high + close pinned to HOD + green close + hot vol (confirmed-breakout closing strength: price has cleared the prior peak and continues to close at the day's high; momentum-continuation with breakout already validated)
    CloseAtLodConfirmedBelowYearLowHotVol,     // year_low_pct >= -3 AND year_low_pct <= -1 AND lod_dist_pct.abs() < 0.5 AND change_pct < -1 AND rel_volume >= 1.5 — already 1-3% past 52w low + close pinned to LOD + red close + hot vol (confirmed-breakdown closing weakness: price has cleared the prior trough and continues to close at the day's low; momentum-continuation with breakdown already validated)
    MidpointCloseNearYearHighHotVol,           // year_high_pct < 2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND rel_volume >= 1.5 — at/near 52w high (<2%) + close in mid-range between HOD and LOD + hot vol (stall at the 52w extreme: neither bulls nor bears closed in control at the high of the year on elevated participation; potential indecision-reversal candidate after extended advance)
    MidpointCloseNearYearLowHotVol,            // year_low_pct < 2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND rel_volume >= 1.5 — at/near 52w low (<2%) + close in mid-range between HOD and LOD + hot vol (stall at the 52w extreme: neither bulls nor bears closed in control at the low of the year on elevated participation; potential indecision-reversal candidate after extended decline)
    MidpointCloseConfirmedAboveYearHighHotVol, // year_high_pct >= -3 AND year_high_pct <= -1 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND rel_volume >= 1.5 — already 1-3% past 52w high + midpoint close between HOD and LOD + hot vol (stall in the confirmed-breakout zone: price has cleared the prior peak but failed to push higher into the close on elevated participation; potential failed-breakout warning)
    MidpointCloseConfirmedBelowYearLowHotVol,  // year_low_pct >= -3 AND year_low_pct <= -1 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND rel_volume >= 1.5 — already 1-3% past 52w low + midpoint close between HOD and LOD + hot vol (stall in the confirmed-breakdown zone: price has cleared the prior trough but failed to push lower into the close on elevated participation; potential failed-breakdown warning)
    MidpointCloseDeepBelowYearHighHotVol,      // year_high_pct >= 20 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND rel_volume >= 1.5 — far below 52w high (>=20%) + midpoint close between HOD and LOD + hot vol (stall deep in the pullback zone: neither bulls nor bears closed in control well below the prior peak after extended decline on elevated participation; potential trend-exhaustion candidate)
    MidpointCloseDeepAboveYearLowHotVol,       // year_low_pct >= 20 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND rel_volume >= 1.5 — far above 52w low (>=20%) + midpoint close between HOD and LOD + hot vol (stall deep in the advance zone: neither bulls nor bears closed in control well above the prior trough after extended advance on elevated participation; potential trend-exhaustion candidate)
    MidpointCloseMidYearHighHotVol,            // year_high_pct >= 5 AND year_high_pct < 20 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND rel_volume >= 1.5 — mid-range from high (5-20%) + midpoint close between HOD and LOD + hot vol (context-free intraday indecision in the mid-cycle pullback zone: neither bulls nor bears closed in control in the proper consolidation range on elevated participation; standoff in the middle of the year-range)
    MidpointCloseMidYearLowHotVol,             // year_low_pct >= 5 AND year_low_pct < 20 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND rel_volume >= 1.5 — mid-range from low (5-20%) + midpoint close between HOD and LOD + hot vol (context-free intraday indecision in the mid-cycle recovery zone: neither bulls nor bears closed in control in the proper consolidation range on elevated participation; standoff in the middle of the year-range)
    MidpointCloseJustOffYearHighHotVol,        // year_high_pct >= 2 AND year_high_pct < 5 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND rel_volume >= 1.5 — just off 52w high (2-5%) + midpoint close between HOD and LOD + hot vol (post-tag intraday indecision: neither bulls nor bears closed in control immediately after fresh pullback from the 52w high on elevated participation; standoff in the post-extreme zone)
    MidpointCloseJustOffYearLowHotVol,         // year_low_pct >= 2 AND year_low_pct < 5 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND rel_volume >= 1.5 — just off 52w low (2-5%) + midpoint close between HOD and LOD + hot vol (post-tag intraday indecision: neither bulls nor bears closed in control immediately after fresh bounce from the 52w low on elevated participation; standoff in the post-extreme zone)
    GapUpCloseAtHodHotVol,                     // gap_pct > 2 AND hod_dist_pct.abs() < 0.5 AND change_pct > 1 AND rel_volume >= 1.5 — gap up (>2%) + close pinned to HOD + green close + hot vol (strongest possible bullish gap: gap up held without fade and price closed at the day's high on elevated participation; sustained buying through the bell with no profit-taking)
    GapDownCloseAtLodHotVol,                   // gap_pct < -2 AND lod_dist_pct.abs() < 0.5 AND change_pct < -1 AND rel_volume >= 1.5 — gap down (<-2%) + close pinned to LOD + red close + hot vol (strongest possible bearish gap: gap down held without bounce and price closed at the day's low on elevated participation; sustained selling through the bell with no dip-buying)
    GapUpCloseAtLodHotVol,                     // gap_pct > 2 AND lod_dist_pct.abs() < 0.5 AND change_pct < 0 AND rel_volume >= 1.5 — gap up (>2%) faded completely to LOD + red close + hot vol (bull-trap reversal: sellers absorbed the entire gap and pushed below the open on elevated participation; classic gap-and-reverse failed-breakout pattern)
    GapDownCloseAtHodHotVol,                   // gap_pct < -2 AND hod_dist_pct.abs() < 0.5 AND change_pct > 0 AND rel_volume >= 1.5 — gap down (<-2%) absorbed completely to HOD + green close + hot vol (bear-trap reversal: buyers absorbed the entire gap and pushed above the open on elevated participation; classic gap-and-reverse failed-breakdown pattern)
    GapUpMidpointCloseHotVol,                  // gap_pct > 2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND rel_volume >= 1.5 — gap up (>2%) + midpoint close between HOD and LOD + hot vol (inconclusive gap-up follow-through: gap held but neither extended to a HOD close nor failed to a LOD close on elevated participation; standoff inside the gap day with no directional resolution)
    GapDownMidpointCloseHotVol,                // gap_pct < -2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND rel_volume >= 1.5 — gap down (<-2%) + midpoint close between HOD and LOD + hot vol (inconclusive gap-down follow-through: gap held but neither extended to a LOD close nor absorbed to a HOD close on elevated participation; standoff inside the gap day with no directional resolution)
    BigGapUpCloseAtHodHotVol,                  // gap_pct > 5 AND hod_dist_pct.abs() < 0.5 AND change_pct > 3 AND rel_volume >= 2 — large gap up (>5%) + close pinned to HOD + big green close + hot vol (institutional-conviction gap-up with no profit-taking: large gap held all session and closed at the day's high on doubled participation; earnings-reaction / news-driven sustained buying through the bell)
    BigGapDownCloseAtLodHotVol,                // gap_pct < -5 AND lod_dist_pct.abs() < 0.5 AND change_pct < -3 AND rel_volume >= 2 — large gap down (<-5%) + close pinned to LOD + big red close + hot vol (institutional-conviction gap-down with no dip-buying: large gap held all session and closed at the day's low on doubled participation; earnings-disappointment / news-driven sustained selling through the bell)
    BigGapUpCloseAtLodHotVol,                  // gap_pct > 5 AND lod_dist_pct.abs() < 0.5 AND change_pct < 0 AND rel_volume >= 2 — large gap up (>5%) completely faded to LOD + red close + hot vol (institutional bull-trap reversal: dramatic earnings-reaction gap fully absorbed and pushed below the open on doubled participation; high-conviction failed-breakout signal rare enough to mark a regime-shift candidate)
    BigGapDownCloseAtHodHotVol,                // gap_pct < -5 AND hod_dist_pct.abs() < 0.5 AND change_pct > 0 AND rel_volume >= 2 — large gap down (<-5%) completely absorbed to HOD + green close + hot vol (institutional bear-trap reversal: dramatic capitulation gap fully absorbed and pushed above the open on doubled participation; high-conviction failed-breakdown signal rare enough to mark a regime-shift candidate)
    BigGapUpMidpointCloseHotVol,               // gap_pct > 5 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND rel_volume >= 2 — large gap up (>5%) + midpoint close between HOD and LOD + hot vol (inconclusive institutional reaction: large gap held but neither extended to a HOD close nor failed to a LOD close on doubled participation; high-stakes standoff after an earnings/news gap with no directional resolution)
    BigGapDownMidpointCloseHotVol,             // gap_pct < -5 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND rel_volume >= 2 — large gap down (<-5%) + midpoint close between HOD and LOD + hot vol (inconclusive institutional capitulation: large gap held but neither extended to a LOD close nor absorbed to a HOD close on doubled participation; high-stakes standoff after an earnings/news gap with no directional resolution)
    GapUpCloseAtHodNearYearHighHotVol,         // gap_pct > 2 AND hod_dist_pct.abs() < 0.5 AND year_high_pct < 2 AND change_pct > 1 AND rel_volume >= 1.5 — gap up (>2%) + close pinned to HOD + at/near 52w high (<2%) + green close + hot vol (maximum-conviction breakout day: gap up at the 52w high held all session and closed at the day's high on elevated participation; rarest multi-axis aligned bullish signal with no overhead supply)
    GapDownCloseAtLodNearYearLowHotVol,        // gap_pct < -2 AND lod_dist_pct.abs() < 0.5 AND year_low_pct < 2 AND change_pct < -1 AND rel_volume >= 1.5 — gap down (<-2%) + close pinned to LOD + at/near 52w low (<2%) + red close + hot vol (maximum-conviction breakdown day: gap down at the 52w low held all session and closed at the day's low on elevated participation; rarest multi-axis aligned bearish signal with no underlying support)
    GapUpCloseAtHodDeepBelowYearHighHotVol,    // gap_pct > 2 AND hod_dist_pct.abs() < 0.5 AND year_high_pct >= 20 AND change_pct > 1 AND rel_volume >= 1.5 — gap up (>2%) + close pinned to HOD + far below 52w high (>=20%) + green close + hot vol (high-conviction recovery from extended pullback: gap up well below the prior peak held all session and closed at the day's high on elevated participation; trend-change reversal candidate after extended decline)
    GapDownCloseAtLodDeepAboveYearLowHotVol,   // gap_pct < -2 AND lod_dist_pct.abs() < 0.5 AND year_low_pct >= 20 AND change_pct < -1 AND rel_volume >= 1.5 — gap down (<-2%) + close pinned to LOD + far above 52w low (>=20%) + red close + hot vol (high-conviction rejection of extended advance: gap down well above the prior trough held all session and closed at the day's low on elevated participation; trend-change reversal candidate after extended advance)
    GapUpCloseAtHodConfirmedAboveYearHighHotVol, // gap_pct > 2 AND hod_dist_pct.abs() < 0.5 AND year_high_pct >= -3 AND year_high_pct <= -1 AND change_pct > 1 AND rel_volume >= 1.5 — gap up (>2%) + close pinned to HOD + already 1-3% past 52w high + green close + hot vol (confirmed-breakout day-trade signal: gap up continues breakout that was already validated and closed at the day's high on elevated participation; aligned-axis momentum extension above the prior peak)
    GapDownCloseAtLodConfirmedBelowYearLowHotVol, // gap_pct < -2 AND lod_dist_pct.abs() < 0.5 AND year_low_pct >= -3 AND year_low_pct <= -1 AND change_pct < -1 AND rel_volume >= 1.5 — gap down (<-2%) + close pinned to LOD + already 1-3% past 52w low + red close + hot vol (confirmed-breakdown day-trade signal: gap down continues breakdown that was already validated and closed at the day's low on elevated participation; aligned-axis momentum extension below the prior trough)
    GapUpCloseAtHodJustOffYearHighHotVol,      // gap_pct > 2 AND hod_dist_pct.abs() < 0.5 AND year_high_pct >= 2 AND year_high_pct < 5 AND change_pct > 1 AND rel_volume >= 1.5 — gap up (>2%) + close pinned to HOD + just off 52w high (2-5%) + green close + hot vol (post-pullback re-assertion of breakout momentum: gap up after shallow pullback held all session and closed at the day's high on elevated participation; aligned-axis continuation candidate immediately back toward the prior peak)
    GapDownCloseAtLodJustOffYearLowHotVol,     // gap_pct < -2 AND lod_dist_pct.abs() < 0.5 AND year_low_pct >= 2 AND year_low_pct < 5 AND change_pct < -1 AND rel_volume >= 1.5 — gap down (<-2%) + close pinned to LOD + just off 52w low (2-5%) + red close + hot vol (post-bounce re-assertion of breakdown momentum: gap down after shallow bounce held all session and closed at the day's low on elevated participation; aligned-axis continuation candidate immediately back toward the prior trough)
    GapUpCloseAtHodMidYearHighHotVol,          // gap_pct > 2 AND hod_dist_pct.abs() < 0.5 AND year_high_pct >= 5 AND year_high_pct < 20 AND change_pct > 1 AND rel_volume >= 1.5 — gap up (>2%) + close pinned to HOD + mid-range from high (5-20%) + green close + hot vol (mid-cycle recovery momentum: gap up well into the consolidation zone held all session and closed at the day's high on elevated participation; aligned-axis push back toward the prior peak from a proper pullback)
    GapDownCloseAtLodMidYearLowHotVol,         // gap_pct < -2 AND lod_dist_pct.abs() < 0.5 AND year_low_pct >= 5 AND year_low_pct < 20 AND change_pct < -1 AND rel_volume >= 1.5 — gap down (<-2%) + close pinned to LOD + mid-range from low (5-20%) + red close + hot vol (mid-cycle reversal momentum: gap down well into the consolidation zone held all session and closed at the day's low on elevated participation; aligned-axis push back toward the prior trough from a proper recovery)
    GapUpCloseAtLodNearYearHighHotVol,         // gap_pct > 2 AND lod_dist_pct.abs() < 0.5 AND year_high_pct < 2 AND change_pct < 0 AND rel_volume >= 1.5 — gap up (>2%) faded completely to LOD + at/near 52w high (<2%) + red close + hot vol (distribution-top signal at the 52w high: gap up attempt at the year peak completely absorbed and pushed below the open on elevated participation; high-conviction failed-breakout at the worst possible location for bulls)
    GapDownCloseAtHodNearYearLowHotVol,        // gap_pct < -2 AND hod_dist_pct.abs() < 0.5 AND year_low_pct < 2 AND change_pct > 0 AND rel_volume >= 1.5 — gap down (<-2%) absorbed completely to HOD + at/near 52w low (<2%) + green close + hot vol (accumulation-bottom signal at the 52w low: gap down attempt at the year trough completely absorbed and pushed above the open on elevated participation; high-conviction failed-breakdown at the worst possible location for bears)
    GapUpCloseAtLodConfirmedAboveYearHighHotVol,  // gap_pct > 2 AND lod_dist_pct.abs() < 0.5 AND year_high_pct >= -3 AND year_high_pct <= -1 AND change_pct < 0 AND rel_volume >= 1.5 — gap up (>2%) faded completely to LOD + confirmed-breakout zone (1-3% past 52w high) + red close + hot vol (post-breakout distribution signal: gap up in the already-confirmed breakout zone completely reversed and closed at the day's low on elevated participation; failed-extension warning that breakout buyers are getting trapped above the prior peak)
    GapDownCloseAtHodConfirmedBelowYearLowHotVol, // gap_pct < -2 AND hod_dist_pct.abs() < 0.5 AND year_low_pct >= -3 AND year_low_pct <= -1 AND change_pct > 0 AND rel_volume >= 1.5 — gap down (<-2%) absorbed completely to HOD + confirmed-breakdown zone (1-3% past 52w low) + green close + hot vol (post-breakdown accumulation signal: gap down in the already-confirmed breakdown zone completely reversed and closed at the day's high on elevated participation; failed-extension warning that breakdown sellers are getting trapped below the prior trough)
    GapUpCloseAtLodDeepBelowYearHighHotVol,    // gap_pct > 2 AND lod_dist_pct.abs() < 0.5 AND year_high_pct >= 20 AND change_pct < 0 AND rel_volume >= 1.5 — gap up (>2%) faded completely to LOD + far below 52w high (>=20%) + red close + hot vol (failed dead-cat bounce signal: gap up deep in the pullback territory completely reversed and closed at the day's low on elevated participation; bear-market continuation candidate with sellers in control well below the prior peak)
    GapDownCloseAtHodDeepAboveYearLowHotVol,   // gap_pct < -2 AND hod_dist_pct.abs() < 0.5 AND year_low_pct >= 20 AND change_pct > 0 AND rel_volume >= 1.5 — gap down (<-2%) absorbed completely to HOD + far above 52w low (>=20%) + green close + hot vol (failed shake-out signal: gap down deep in the advance territory completely reversed and closed at the day's high on elevated participation; bull-market continuation candidate with buyers in control well above the prior trough)
    GapUpCloseAtLodJustOffYearHighHotVol,      // gap_pct > 2 AND lod_dist_pct.abs() < 0.5 AND year_high_pct >= 2 AND year_high_pct < 5 AND change_pct < 0 AND rel_volume >= 1.5 — gap up (>2%) faded completely to LOD + just off 52w high (2-5%) + red close + hot vol (failed recovery attempt: gap up immediately after shallow pullback from the year peak completely reversed and closed at the day's low on elevated participation; second leg lower starting candidate)
    GapDownCloseAtHodJustOffYearLowHotVol,     // gap_pct < -2 AND hod_dist_pct.abs() < 0.5 AND year_low_pct >= 2 AND year_low_pct < 5 AND change_pct > 0 AND rel_volume >= 1.5 — gap down (<-2%) absorbed completely to HOD + just off 52w low (2-5%) + green close + hot vol (failed rejection attempt: gap down immediately after shallow bounce from the year trough completely reversed and closed at the day's high on elevated participation; second leg higher starting candidate)
    GapUpCloseAtLodMidYearHighHotVol,          // gap_pct > 2 AND lod_dist_pct.abs() < 0.5 AND year_high_pct >= 5 AND year_high_pct < 20 AND change_pct < 0 AND rel_volume >= 1.5 — gap up (>2%) faded completely to LOD + mid-range from high (5-20%) + red close + hot vol (failed mid-cycle bounce: gap up in the proper consolidation zone completely reversed and closed at the day's low on elevated participation; continuation of mid-cycle pullback with sellers in control)
    GapDownCloseAtHodMidYearLowHotVol,         // gap_pct < -2 AND hod_dist_pct.abs() < 0.5 AND year_low_pct >= 5 AND year_low_pct < 20 AND change_pct > 0 AND rel_volume >= 1.5 — gap down (<-2%) absorbed completely to HOD + mid-range from low (5-20%) + green close + hot vol (failed mid-cycle pullback: gap down in the proper consolidation zone completely reversed and closed at the day's high on elevated participation; continuation of mid-cycle recovery with buyers in control)
    GapUpMidpointCloseNearYearHighHotVol,      // gap_pct > 2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND year_high_pct < 2 AND rel_volume >= 1.5 — gap up (>2%) + midpoint close between HOD and LOD + at/near 52w high (<2%) + hot vol (inconclusive breakout-day reaction: gap up at the year peak held but neither extended nor failed into the close on elevated participation; standoff at the 52w high with the next breakout still undecided)
    GapDownMidpointCloseNearYearLowHotVol,     // gap_pct < -2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND year_low_pct < 2 AND rel_volume >= 1.5 — gap down (<-2%) + midpoint close between HOD and LOD + at/near 52w low (<2%) + hot vol (inconclusive breakdown-day reaction: gap down at the year trough held but neither extended nor absorbed into the close on elevated participation; standoff at the 52w low with the next breakdown still undecided)
    GapUpMidpointCloseConfirmedAboveYearHighHotVol,  // gap_pct > 2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND year_high_pct >= -3 AND year_high_pct <= -1 AND rel_volume >= 1.5 — gap up (>2%) + midpoint close between HOD and LOD + confirmed-breakout zone (1-3% past 52w high) + hot vol (uncertain follow-through after validated breakout: gap up in the already-cleared zone held but neither extended nor failed into the close on elevated participation; post-breakout stall warning that the extension is losing conviction)
    GapDownMidpointCloseConfirmedBelowYearLowHotVol, // gap_pct < -2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND year_low_pct >= -3 AND year_low_pct <= -1 AND rel_volume >= 1.5 — gap down (<-2%) + midpoint close between HOD and LOD + confirmed-breakdown zone (1-3% past 52w low) + hot vol (uncertain follow-through after validated breakdown: gap down in the already-cleared zone held but neither extended nor absorbed into the close on elevated participation; post-breakdown stall warning that the extension is losing conviction)
    GapUpMidpointCloseDeepBelowYearHighHotVol, // gap_pct > 2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND year_high_pct >= 20 AND rel_volume >= 1.5 — gap up (>2%) + midpoint close between HOD and LOD + far below 52w high (>=20%) + hot vol (inconclusive bounce attempt deep in pullback territory: gap up well below the prior peak held but neither extended to a HOD close nor failed to a LOD close on elevated participation; standoff after extended decline with no directional commitment from the bounce)
    GapDownMidpointCloseDeepAboveYearLowHotVol,// gap_pct < -2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND year_low_pct >= 20 AND rel_volume >= 1.5 — gap down (<-2%) + midpoint close between HOD and LOD + far above 52w low (>=20%) + hot vol (inconclusive pullback attempt deep in advance territory: gap down well above the prior trough held but neither extended to a LOD close nor absorbed to a HOD close on elevated participation; standoff after extended advance with no directional commitment from the pullback)
    GapUpMidpointCloseMidYearHighHotVol,       // gap_pct > 2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND year_high_pct >= 5 AND year_high_pct < 20 AND rel_volume >= 1.5 — gap up (>2%) + midpoint close between HOD and LOD + mid-range from high (5-20%) + hot vol (inconclusive bounce in mid-cycle pullback: gap up in the proper consolidation zone held but neither extended to a HOD close nor failed to a LOD close on elevated participation; standoff at mid-range with no directional commitment toward the prior peak)
    GapDownMidpointCloseMidYearLowHotVol,      // gap_pct < -2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND year_low_pct >= 5 AND year_low_pct < 20 AND rel_volume >= 1.5 — gap down (<-2%) + midpoint close between HOD and LOD + mid-range from low (5-20%) + hot vol (inconclusive pullback in mid-cycle recovery: gap down in the proper consolidation zone held but neither extended to a LOD close nor absorbed to a HOD close on elevated participation; standoff at mid-range with no directional commitment back toward the prior trough)
    GapUpMidpointCloseJustOffYearHighHotVol,   // gap_pct > 2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND year_high_pct >= 2 AND year_high_pct < 5 AND rel_volume >= 1.5 — gap up (>2%) + midpoint close between HOD and LOD + just off 52w high (2-5%) + hot vol (inconclusive bounce just off the year peak: gap up in the post-extreme zone held but neither extended to a HOD close nor failed to a LOD close on elevated participation; standoff in the immediate post-tag zone with the recovery still undecided)
    GapDownMidpointCloseJustOffYearLowHotVol,  // gap_pct < -2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND year_low_pct >= 2 AND year_low_pct < 5 AND rel_volume >= 1.5 — gap down (<-2%) + midpoint close between HOD and LOD + just off 52w low (2-5%) + hot vol (inconclusive pullback just off the year trough: gap down in the post-extreme zone held but neither extended to a LOD close nor absorbed to a HOD close on elevated participation; standoff in the immediate post-tag zone with the rejection still undecided)
    HotVolFlatCloseNearYearHighHotVol,         // change_pct.abs() < 0.5 AND rel_volume >= 2 AND year_high_pct < 2 — flat close (|change|<0.5) + hot vol (>=2) + at/near 52w high (<2%) (institutional churn at the 52w high: doubled participation with no net price impact at the year peak; potential distribution-into-strength signal where smart money exchanges hands without moving the tape)
    HotVolFlatCloseNearYearLowHotVol,          // change_pct.abs() < 0.5 AND rel_volume >= 2 AND year_low_pct < 2 — flat close (|change|<0.5) + hot vol (>=2) + at/near 52w low (<2%) (institutional churn at the 52w low: doubled participation with no net price impact at the year trough; potential accumulation-into-weakness signal where smart money exchanges hands without moving the tape)
    HotVolFlatCloseConfirmedAboveYearHighHotVol, // change_pct.abs() < 0.5 AND rel_volume >= 2 AND year_high_pct >= -3 AND year_high_pct <= -1 — flat close + hot vol + confirmed-breakout zone (1-3% past 52w high) (stealth distribution in the confirmed-breakout zone: doubled participation with no net price impact above the prior peak; institutions handling supply at the validated breakout level without giving back the move)
    HotVolFlatCloseConfirmedBelowYearLowHotVol,  // change_pct.abs() < 0.5 AND rel_volume >= 2 AND year_low_pct >= -3 AND year_low_pct <= -1 — flat close + hot vol + confirmed-breakdown zone (1-3% past 52w low) (stealth accumulation in the confirmed-breakdown zone: doubled participation with no net price impact below the prior trough; institutions handling demand at the validated breakdown level without giving back the move)
    HotVolFlatCloseDeepBelowYearHighHotVol,    // change_pct.abs() < 0.5 AND rel_volume >= 2 AND year_high_pct >= 20 — flat close + hot vol + far below 52w high (>=20%) (stealth accumulation deep in pullback territory: doubled participation with no net price impact well below the prior peak; potential base-building signal where smart money builds position during depressed-tape conditions)
    HotVolFlatCloseDeepAboveYearLowHotVol,     // change_pct.abs() < 0.5 AND rel_volume >= 2 AND year_low_pct >= 20 — flat close + hot vol + far above 52w low (>=20%) (stealth distribution deep in advance territory: doubled participation with no net price impact well above the prior trough; potential topping signal where smart money exits position during euphoric-tape conditions)
    HotVolFlatCloseMidYearHighHotVol,          // change_pct.abs() < 0.5 AND rel_volume >= 2 AND year_high_pct >= 5 AND year_high_pct < 20 — flat close + hot vol + mid-range from high (5-20%) (Wyckoff-style stealth accumulation zone in mid-cycle pullback: doubled participation with no net price impact in the proper consolidation range; textbook accumulation phase where institutions absorb supply at fair value below the prior peak)
    HotVolFlatCloseMidYearLowHotVol,           // change_pct.abs() < 0.5 AND rel_volume >= 2 AND year_low_pct >= 5 AND year_low_pct < 20 — flat close + hot vol + mid-range from low (5-20%) (Wyckoff-style stealth distribution zone in mid-cycle recovery: doubled participation with no net price impact in the proper consolidation range; textbook distribution phase where institutions release supply at fair value above the prior trough)
    HotVolFlatCloseJustOffYearHighHotVol,      // change_pct.abs() < 0.5 AND rel_volume >= 2 AND year_high_pct >= 2 AND year_high_pct < 5 — flat close + hot vol + just off 52w high (2-5%) (post-tag stealth absorption: doubled participation with no net price impact immediately after fresh pullback from the 52w high; institutions exchange hands in the post-extreme zone while price digests the recent peak)
    HotVolFlatCloseJustOffYearLowHotVol,       // change_pct.abs() < 0.5 AND rel_volume >= 2 AND year_low_pct >= 2 AND year_low_pct < 5 — flat close + hot vol + just off 52w low (2-5%) (post-tag stealth release: doubled participation with no net price impact immediately after fresh bounce from the 52w low; institutions exchange hands in the post-extreme zone while price digests the recent trough)
    DryVolBigUpNearYearHighHotVol,             // change_pct > 3 AND rel_volume < 0.5 AND year_high_pct < 2 — big up move (>3%) + dry vol (<0.5) + at/near 52w high (<2%) (thin-tape breakout: large gains push price to new highs but participation is below average; air-pocket move with little resistance, fragile if vol returns)
    DryVolBigDownNearYearLowHotVol,            // change_pct < -3 AND rel_volume < 0.5 AND year_low_pct < 2 — big down move (<-3%) + dry vol (<0.5) + at/near 52w low (<2%) (thin-tape breakdown: large losses push price to new lows but participation is below average; air-pocket move with little support, fragile if vol returns)
    DryVolBigUpConfirmedAboveYearHighHotVol,   // change_pct > 3 AND rel_volume < 0.5 AND year_high_pct >= -3 AND year_high_pct <= -1 — big up move (>3%) + dry vol (<0.5) + confirmed-breakout zone (1-3% past 52w high) (thin-tape extension after validated breakout: momentum continues on below-average participation past the prior peak; volume-unconfirmed extension prone to mean-reversion)
    DryVolBigDownConfirmedBelowYearLowHotVol,  // change_pct < -3 AND rel_volume < 0.5 AND year_low_pct >= -3 AND year_low_pct <= -1 — big down move (<-3%) + dry vol (<0.5) + confirmed-breakdown zone (1-3% past 52w low) (thin-tape extension after validated breakdown: momentum continues on below-average participation past the prior trough; volume-unconfirmed extension prone to mean-reversion)
    DryVolBigUpDeepBelowYearHighHotVol,        // change_pct > 3 AND rel_volume < 0.5 AND year_high_pct >= 20 — big up move (>3%) + dry vol (<0.5) + far below 52w high (>=20%) (unconvincing recovery rally: large gains deep in the pullback territory on below-average participation; sympathy/short-cover bounce lacking institutional buy-in, fragile if vol returns)
    DryVolBigDownDeepAboveYearLowHotVol,       // change_pct < -3 AND rel_volume < 0.5 AND year_low_pct >= 20 — big down move (<-3%) + dry vol (<0.5) + far above 52w low (>=20%) (unconvincing pullback: large losses deep in the advance territory on below-average participation; sympathy/long-unwind dip lacking institutional sell-in, fragile if vol returns)
    DryVolBigUpMidYearHighHotVol,              // change_pct > 3 AND rel_volume < 0.5 AND year_high_pct >= 5 AND year_high_pct < 20 — big up move (>3%) + dry vol (<0.5) + mid-range from high (5-20%) (low-quality bounce in mid-cycle pullback: large gains in the proper consolidation zone on below-average participation; sympathy move lacking institutional follow-through, prone to fade)
    DryVolBigDownMidYearLowHotVol,             // change_pct < -3 AND rel_volume < 0.5 AND year_low_pct >= 5 AND year_low_pct < 20 — big down move (<-3%) + dry vol (<0.5) + mid-range from low (5-20%) (low-quality pullback in mid-cycle recovery: large losses in the proper consolidation zone on below-average participation; sympathy move lacking institutional follow-through, prone to bounce)
    DryVolBigUpJustOffYearHighHotVol,          // change_pct > 3 AND rel_volume < 0.5 AND year_high_pct >= 2 AND year_high_pct < 5 — big up move (>3%) + dry vol (<0.5) + just off 52w high (2-5%) (thin-tape recovery from shallow pullback: large gains in the immediate post-extreme zone on below-average participation; quick post-tag bounce without institutional buy-in, fragile re-test candidate)
    DryVolBigDownJustOffYearLowHotVol,         // change_pct < -3 AND rel_volume < 0.5 AND year_low_pct >= 2 AND year_low_pct < 5 — big down move (<-3%) + dry vol (<0.5) + just off 52w low (2-5%) (thin-tape rejection from shallow bounce: large losses in the immediate post-extreme zone on below-average participation; quick post-tag dip without institutional sell-in, fragile re-test candidate)
    UltraDeepBelowYearHighHotVol,              // year_high_pct >= 50 AND rel_volume >= 1.5 — ultra-deep distance from 52w high (>=50%) + hot vol (deep-value / distressed-equity territory: price has lost half or more of its 52w peak with elevated participation; either turnaround candidate or bankruptcy-watch depending on fundamentals)
    UltraDeepAboveYearLowHotVol,               // year_low_pct >= 50 AND rel_volume >= 1.5 — ultra-deep distance from 52w low (>=50%) + hot vol (multibagger territory: price has doubled or more off its 52w trough with elevated participation; momentum-leader candidate riding a multi-month trend with sustained institutional interest)
    UltraDeepBelowYearHighCloseAtHodHotVol,    // year_high_pct >= 50 AND hod_dist_pct.abs() < 0.5 AND change_pct > 1 AND rel_volume >= 1.5 — distressed stock (>=50% below 52w high) + close pinned to HOD + green close + hot vol (turnaround momentum signal: distressed equity closes at the day's high on elevated participation; rare bullish-conviction tape in beaten-down territory worth a reversal-trade screen)
    UltraDeepAboveYearLowCloseAtLodHotVol,     // year_low_pct >= 50 AND lod_dist_pct.abs() < 0.5 AND change_pct < -1 AND rel_volume >= 1.5 — multibagger (>=50% above 52w low) + close pinned to LOD + red close + hot vol (topping signal: extended-trend leader closes at the day's low on elevated participation; rare bearish-conviction tape in stretched territory worth a top-fade trade screen)
    UltraDeepBelowYearHighGapUpHotVol,         // year_high_pct >= 50 AND gap_pct > 2 AND rel_volume >= 1.5 — distressed stock (>=50% below 52w high) + gap up (>2%) + hot vol (catalyst-driven turn candidate: beaten-down equity gaps up on news/earnings with elevated participation; potential institutional re-rating event worth a turnaround-trade screen)
    UltraDeepAboveYearLowGapDownHotVol,        // year_low_pct >= 50 AND gap_pct < -2 AND rel_volume >= 1.5 — multibagger (>=50% above 52w low) + gap down (<-2%) + hot vol (catalyst-driven top candidate: extended-trend leader gaps down on news/earnings with elevated participation; potential institutional de-rating event worth a top-fade trade screen)
    UltraDeepBelowYearHighGapUpFadedHotVol,    // year_high_pct >= 50 AND gap_pct > 2 AND lod_dist_pct.abs() < 0.5 AND change_pct < 0 AND rel_volume >= 1.5 — distressed stock (>=50% below 52w high) + gap up (>2%) faded completely to LOD + red close + hot vol (failed turnaround catalyst: beaten-down equity attempts a catalyst-driven gap but sellers absorb the entire move on elevated participation; turnaround-thesis rejection that confirms downtrend control)
    UltraDeepAboveYearLowGapDownAbsorbedHotVol, // year_low_pct >= 50 AND gap_pct < -2 AND hod_dist_pct.abs() < 0.5 AND change_pct > 0 AND rel_volume >= 1.5 — multibagger (>=50% above 52w low) + gap down (<-2%) absorbed completely to HOD + green close + hot vol (failed topping catalyst: extended-trend leader attempts a catalyst-driven gap down but buyers absorb the entire move on elevated participation; top-fade thesis rejection that confirms uptrend control)
    UltraDeepBelowYearHighGapUpHeldHotVol,     // year_high_pct >= 50 AND gap_pct > 2 AND hod_dist_pct.abs() < 0.5 AND change_pct > 1 AND rel_volume >= 1.5 — distressed stock (>=50% below 52w high) + gap up (>2%) held all session + close pinned to HOD + green close + hot vol (validated turnaround day: beaten-down equity gaps up on catalyst, holds the entire gap, and closes at the day's high on elevated participation; institutional re-rating event with no profit-taking = highest-conviction turnaround signal in distressed territory)
    UltraDeepAboveYearLowGapDownHeldHotVol,    // year_low_pct >= 50 AND gap_pct < -2 AND lod_dist_pct.abs() < 0.5 AND change_pct < -1 AND rel_volume >= 1.5 — multibagger (>=50% above 52w low) + gap down (<-2%) held all session + close pinned to LOD + red close + hot vol (validated topping day: extended-trend leader gaps down on catalyst, holds the entire gap, and closes at the day's low on elevated participation; institutional de-rating event with no dip-buying = highest-conviction topping signal in stretched territory)
    UltraDeepBelowYearHighGapUpMidpointHotVol, // year_high_pct >= 50 AND gap_pct > 2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND rel_volume >= 1.5 — distressed stock (>=50% below 52w high) + gap up (>2%) + midpoint close between HOD and LOD + hot vol (uncertain turnaround follow-through: beaten-down equity gaps up on catalyst but neither holds the gap to a HOD close nor fully fades to a LOD close on elevated participation; ambiguous re-rating event requiring confirmation)
    UltraDeepAboveYearLowGapDownMidpointHotVol,// year_low_pct >= 50 AND gap_pct < -2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 AND rel_volume >= 1.5 — multibagger (>=50% above 52w low) + gap down (<-2%) + midpoint close between HOD and LOD + hot vol (uncertain topping follow-through: extended-trend leader gaps down on catalyst but neither holds the gap to a LOD close nor fully absorbs to a HOD close on elevated participation; ambiguous de-rating event requiring confirmation)
    UltraDeepBelowYearHighHammerHotVol,        // year_high_pct >= 50 AND lod_dist_pct.abs() > 3 AND hod_dist_pct.abs() < 0.5 AND change_pct > 0 AND rel_volume >= 1.5 — distressed stock (>=50% below 52w high) + long lower wick (>3%) + close pinned to HOD + green close + hot vol (capitulation-reversal hammer in distressed territory: beaten-down equity probed lower then reclaimed the entire move to close at the day's high on elevated participation; classic capitulation-day pattern at deep distress worth a bounce-trade screen)
    UltraDeepAboveYearLowShootingStarHotVol,   // year_low_pct >= 50 AND hod_dist_pct.abs() > 3 AND lod_dist_pct.abs() < 0.5 AND change_pct < 0 AND rel_volume >= 1.5 — multibagger (>=50% above 52w low) + long upper wick (>3%) + close pinned to LOD + red close + hot vol (exhaustion-reversal shooting star in multibagger territory: extended-trend leader probed higher then gave back the entire move to close at the day's low on elevated participation; classic exhaustion-day pattern at extended advance worth a top-fade trade screen)
    UltraDeepBelowYearHighShootingStarHotVol,  // year_high_pct >= 50 AND hod_dist_pct.abs() > 3 AND lod_dist_pct.abs() < 0.5 AND change_pct < 0 AND rel_volume >= 1.5 — distressed stock (>=50% below 52w high) + long upper wick (>3%) + close pinned to LOD + red close + hot vol (failed-bounce shooting star in distressed territory: beaten-down equity probed higher intraday then sellers reclaimed entire move to close at the day's low on elevated participation; bear-trend continuation candidate confirming downtrend control)
    UltraDeepAboveYearLowHammerHotVol,         // year_low_pct >= 50 AND lod_dist_pct.abs() > 3 AND hod_dist_pct.abs() < 0.5 AND change_pct > 0 AND rel_volume >= 1.5 — multibagger (>=50% above 52w low) + long lower wick (>3%) + close pinned to HOD + green close + hot vol (trend-continuation hammer in multibagger territory: extended-trend leader probed lower intraday then buyers reclaimed entire move to close at the day's high on elevated participation; bull-trend continuation candidate confirming uptrend control)
    WideYearRangeHotVol,                       // year_high_pct >= 20 AND year_low_pct >= 20 AND rel_volume >= 1.5 — wide annual range (both year_*_pct >=20%) + hot vol (high-beta volatile-equity territory: stock has traveled significant distance from both prior peak and prior trough indicating large 52w range; volatility-screen candidate for momentum and mean-reversion strategies)
    NarrowYearRangeHotVol,                     // year_high_pct < 5 AND year_low_pct < 5 AND rel_volume >= 1.5 — narrow annual range (both year_*_pct <5%) + hot vol (compressed range-bound territory: stock has stayed within ~10% band over 52w with elevated current participation; tightest possible coil at annual scale, breakout-candidate worth watching for direction commitment)
    NarrowYearRangeCloseAtHodHotVol,           // year_high_pct < 5 AND year_low_pct < 5 AND hod_dist_pct.abs() < 0.5 AND change_pct > 1 AND rel_volume >= 1.5 — narrow annual range + close pinned to HOD + green close + hot vol (annual-coil breakout candidate: compressed range-bound stock attempts directional commitment higher, closes at the day's high with hot vol; highest-conviction breakout-from-coil signal at the annual scale)
    NarrowYearRangeCloseAtLodHotVol,           // year_high_pct < 5 AND year_low_pct < 5 AND lod_dist_pct.abs() < 0.5 AND change_pct < -1 AND rel_volume >= 1.5 — narrow annual range + close pinned to LOD + red close + hot vol (annual-coil breakdown candidate: compressed range-bound stock attempts directional commitment lower, closes at the day's low with hot vol; highest-conviction breakdown-from-coil signal at the annual scale)
    NarrowYearRangeGapUpHotVol,                // year_high_pct < 5 AND year_low_pct < 5 AND gap_pct > 2 AND rel_volume >= 1.5 — narrow annual range + gap up (>2%) + hot vol (catalyst-driven attempt to break the annual coil higher: compressed range-bound stock gaps up on news/earnings with elevated participation; first sign of directional commitment after 52w of tight range)
    NarrowYearRangeGapDownHotVol,              // year_high_pct < 5 AND year_low_pct < 5 AND gap_pct < -2 AND rel_volume >= 1.5 — narrow annual range + gap down (<-2%) + hot vol (catalyst-driven attempt to break the annual coil lower: compressed range-bound stock gaps down on news/earnings with elevated participation; first sign of directional commitment after 52w of tight range)
    NarrowYearRangeBigUpHotVol,                // year_high_pct < 5 AND year_low_pct < 5 AND change_pct > 3 AND rel_volume >= 2 — narrow annual range + big up move (>3%) + hot vol (>=2) (intraday-led coil release higher: compressed range-bound stock prints first volatility expansion in 52w with doubled participation; regime-shift candidate from tight range to trending higher)
    NarrowYearRangeBigDownHotVol,              // year_high_pct < 5 AND year_low_pct < 5 AND change_pct < -3 AND rel_volume >= 2 — narrow annual range + big down move (<-3%) + hot vol (>=2) (intraday-led coil release lower: compressed range-bound stock prints first volatility expansion in 52w with doubled participation; regime-shift candidate from tight range to trending lower)
    WideYearRangeCloseAtHodHotVol,             // year_high_pct >= 20 AND year_low_pct >= 20 AND hod_dist_pct.abs() < 0.5 AND change_pct > 1 AND rel_volume >= 1.5 — wide annual range + close pinned to HOD + green close + hot vol (strong-conviction tape in high-beta name: volatile stock with large 52w range closes at the day's high on elevated participation; momentum-continuation signal where intraday strength aligns with the wider price-action regime)
    WideYearRangeCloseAtLodHotVol,             // year_high_pct >= 20 AND year_low_pct >= 20 AND lod_dist_pct.abs() < 0.5 AND change_pct < -1 AND rel_volume >= 1.5 — wide annual range + close pinned to LOD + red close + hot vol (strong-conviction tape in high-beta name: volatile stock with large 52w range closes at the day's low on elevated participation; momentum-continuation signal where intraday weakness aligns with the wider price-action regime)
    WideYearRangeGapUpHotVol,                  // year_high_pct >= 20 AND year_low_pct >= 20 AND gap_pct > 2 AND rel_volume >= 1.5 — wide annual range + gap up (>2%) + hot vol (catalyst-event in high-beta name: volatile stock with large 52w range gaps up on news/earnings with elevated participation; mean-reversion-from-gap candidate or trend-continuation depending on intraday follow-through)
    WideYearRangeGapDownHotVol,                // year_high_pct >= 20 AND year_low_pct >= 20 AND gap_pct < -2 AND rel_volume >= 1.5 — wide annual range + gap down (<-2%) + hot vol (catalyst-event in high-beta name: volatile stock with large 52w range gaps down on news/earnings with elevated participation; mean-reversion-from-gap candidate or trend-continuation depending on intraday follow-through)
    AsymmetricRangeNearLowFarHighHotVol,       // year_high_pct >= 20 AND year_low_pct < 5 AND rel_volume >= 1.5 — near 52w low (<5%) + far below 52w high (>=20%) + hot vol (persistent-downtrend stock testing the low again with elevated participation; either capitulation-bottom candidate or breakdown to fresh lows depending on price-action resolution at the support level)
    AsymmetricRangeNearHighFarLowHotVol,       // year_high_pct < 5 AND year_low_pct >= 20 AND rel_volume >= 1.5 — near 52w high (<5%) + far above 52w low (>=20%) + hot vol (persistent-uptrend stock testing the high again with elevated participation; either breakout-to-fresh-highs candidate or distribution-top depending on price-action resolution at the resistance level)
    AsymmetricRangeNearLowFarHighCloseAtLodHotVol, // year_high_pct >= 20 AND year_low_pct < 5 AND lod_dist_pct.abs() < 0.5 AND change_pct < -1 AND rel_volume >= 1.5 — persistent-downtrend stock (near low + far below high) + close pinned to LOD + red close + hot vol (breakdown-confirmation day: downtrend stock fails the support test at the 52w low and closes at the day's low on elevated participation; fresh-low extension confirmed by intraday weakness)
    AsymmetricRangeNearHighFarLowCloseAtHodHotVol, // year_high_pct < 5 AND year_low_pct >= 20 AND hod_dist_pct.abs() < 0.5 AND change_pct > 1 AND rel_volume >= 1.5 — persistent-uptrend stock (near high + far above low) + close pinned to HOD + green close + hot vol (breakout-confirmation day: uptrend stock clears the resistance test at the 52w high and closes at the day's high on elevated participation; fresh-high extension confirmed by intraday strength)
    AsymmetricRangeNearLowFarHighCloseAtHodHotVol, // year_high_pct >= 20 AND year_low_pct < 5 AND hod_dist_pct.abs() < 0.5 AND change_pct > 1 AND rel_volume >= 1.5 — persistent-downtrend stock (near low + far below high) + close pinned to HOD + green close + hot vol (capitulation-reversal candidate: downtrend stock defends the support test at the 52w low and closes at the day's high on elevated participation; potential bottom-formation signal worth a bounce-trade screen)
    AsymmetricRangeNearHighFarLowCloseAtLodHotVol, // year_high_pct < 5 AND year_low_pct >= 20 AND lod_dist_pct.abs() < 0.5 AND change_pct < -1 AND rel_volume >= 1.5 — persistent-uptrend stock (near high + far above low) + close pinned to LOD + red close + hot vol (distribution-top candidate: uptrend stock rejects the resistance test at the 52w high and closes at the day's low on elevated participation; potential top-formation signal worth a top-fade trade screen)
    AsymmetricRangeNearLowFarHighGapUpHotVol,  // year_high_pct >= 20 AND year_low_pct < 5 AND gap_pct > 2 AND rel_volume >= 1.5 — persistent-downtrend stock (near low + far below high) + gap up (>2%) + hot vol (relief-rally catalyst at the 52w low: downtrend stock attempts catalyst-driven turn off the year trough with elevated participation; potential turnaround-thesis signal worth a bounce-trade screen)
    AsymmetricRangeNearHighFarLowGapDownHotVol,// year_high_pct < 5 AND year_low_pct >= 20 AND gap_pct < -2 AND rel_volume >= 1.5 — persistent-uptrend stock (near high + far above low) + gap down (<-2%) + hot vol (pullback-catalyst at the 52w high: uptrend stock attempts catalyst-driven reversal off the year peak with elevated participation; potential top-warning signal worth a top-fade trade screen)
    AsymmetricRangeNearLowFarHighGapDownHotVol,// year_high_pct >= 20 AND year_low_pct < 5 AND gap_pct < -2 AND rel_volume >= 1.5 — persistent-downtrend stock (near low + far below high) + gap down (<-2%) + hot vol (catalyst-driven breakdown to fresh lows: downtrend stock attempts catalyst-confirmed continuation through the 52w support level with elevated participation; trend-extension signal worth a breakdown-trade screen)
    AsymmetricRangeNearHighFarLowGapUpHotVol,  // year_high_pct < 5 AND year_low_pct >= 20 AND gap_pct > 2 AND rel_volume >= 1.5 — persistent-uptrend stock (near high + far above low) + gap up (>2%) + hot vol (catalyst-driven breakout to fresh highs: uptrend stock attempts catalyst-confirmed continuation through the 52w resistance level with elevated participation; trend-extension signal worth a breakout-trade screen)
    GapUpFlatDayHotVol,                        // gap_pct > 2 AND day_pct.abs() < 0.5 AND rel_volume >= 1.5 — gap up (>2%) + flat-day bar (|day_pct|<0.5%) + hot vol (gap-and-hold institutional signal: overnight gap up held through the regular session without intraday giveback or further follow-through; supports the gap without re-test, suggesting buyer accumulation at the gap level)
    GapDownFlatDayHotVol,                      // gap_pct < -2 AND day_pct.abs() < 0.5 AND rel_volume >= 1.5 — gap down (<-2%) + flat-day bar (|day_pct|<0.5%) + hot vol (gap-and-hold institutional signal: overnight gap down held through the regular session without intraday recovery or further deterioration; supports the gap without re-test, suggesting seller distribution at the gap level)
    GapUpBigDayHotVol,                         // gap_pct > 2 AND day_pct > 2 AND rel_volume >= 1.5 — gap up (>2%) + big intraday up (>2%) + hot vol (double-momentum day: overnight gap up followed by intraday continuation higher with elevated participation; gap+intraday both aligned green = aggregate change_pct >4% from prior close with sustained buy pressure across both regular and after-hours sessions)
    GapDownBigDayHotVol,                       // gap_pct < -2 AND day_pct < -2 AND rel_volume >= 1.5 — gap down (<-2%) + big intraday down (<-2%) + hot vol (double-momentum day: overnight gap down followed by intraday continuation lower with elevated participation; gap+intraday both aligned red = aggregate change_pct <-4% from prior close with sustained sell pressure across both regular and after-hours sessions)
    GapUpBigDayDownHotVol,                     // gap_pct > 2 AND day_pct < -2 AND rel_volume >= 1.5 — gap up (>2%) + big intraday down (<-2%) + hot vol (gap-fade pressure: overnight gap up met with intraday selling that erodes the gap by 2%+; counter-trend intraday pressure on a positive gap that may or may not flip the close to red depending on gap size; suggests active sellers stepping in at the gap level)
    GapDownBigDayUpHotVol,                     // gap_pct < -2 AND day_pct > 2 AND rel_volume >= 1.5 — gap down (<-2%) + big intraday up (>2%) + hot vol (gap-absorb pressure: overnight gap down met with intraday buying that reclaims the gap by 2%+; counter-trend intraday pressure on a negative gap that may or may not flip the close to green depending on gap size; suggests active buyers stepping in at the gap level)
    SmallGapBigDayUpHotVol,                    // gap_pct.abs() < 0.5 AND day_pct > 3 AND rel_volume >= 1.5 — small gap (|gap|<0.5%) + big intraday up (>3%) + hot vol (clean intraday-led rally: open is essentially flat to prior close, then regular session prints a sustained buy-driven move higher; intraday momentum signal isolated from overnight repricing or after-hours catalyst noise)
    SmallGapBigDayDownHotVol,                  // gap_pct.abs() < 0.5 AND day_pct < -3 AND rel_volume >= 1.5 — small gap (|gap|<0.5%) + big intraday down (<-3%) + hot vol (clean intraday-led decline: open is essentially flat to prior close, then regular session prints a sustained sell-driven move lower; intraday momentum signal isolated from overnight repricing or after-hours catalyst noise)
    SmallGapBigDayUpNearYearHighHotVol,        // gap_pct.abs() < 0.5 AND day_pct > 3 AND year_high_pct < 2 AND rel_volume >= 1.5 — small gap (|gap|<0.5%) + big intraday up (>3%) + at/near 52w high (<2%) + hot vol (pure intraday-driven breakout to 52w high: open is essentially flat to prior close then regular session prints a sustained buy-driven move to the year peak; cleanest possible breakout signal with no overnight gap contribution muddying the cause)
    SmallGapBigDayDownNearYearLowHotVol,       // gap_pct.abs() < 0.5 AND day_pct < -3 AND year_low_pct < 2 AND rel_volume >= 1.5 — small gap (|gap|<0.5%) + big intraday down (<-3%) + at/near 52w low (<2%) + hot vol (pure intraday-driven breakdown to 52w low: open is essentially flat to prior close then regular session prints a sustained sell-driven move to the year trough; cleanest possible breakdown signal with no overnight gap contribution muddying the cause)
    SmallGapBigDayUpConfirmedAboveYearHighHotVol,  // gap_pct.abs() < 0.5 AND day_pct > 3 AND year_high_pct >= -3 AND year_high_pct <= -1 AND rel_volume >= 1.5 — small gap (|gap|<0.5%) + big intraday up (>3%) + confirmed-breakout zone (1-3% past 52w high) + hot vol (pure intraday extension past validated breakout: regular session prints a sustained buy-driven move further past the already-cleared peak with no overnight repricing component; high-conviction intraday extension confirming breakout follow-through)
    SmallGapBigDayDownConfirmedBelowYearLowHotVol, // gap_pct.abs() < 0.5 AND day_pct < -3 AND year_low_pct >= -3 AND year_low_pct <= -1 AND rel_volume >= 1.5 — small gap (|gap|<0.5%) + big intraday down (<-3%) + confirmed-breakdown zone (1-3% past 52w low) + hot vol (pure intraday extension past validated breakdown: regular session prints a sustained sell-driven move further past the already-cleared trough with no overnight repricing component; high-conviction intraday extension confirming breakdown follow-through)
    SmallGapBigDayUpDeepBelowYearHighHotVol,   // gap_pct.abs() < 0.5 AND day_pct > 3 AND year_high_pct >= 20 AND rel_volume >= 1.5 — small gap (|gap|<0.5%) + big intraday up (>3%) + far below 52w high (>=20%) + hot vol (pure intraday recovery rally from depressed level: open is essentially flat to prior close then regular session prints a sustained buy-driven move deep in pullback territory; institutional-bid activation signal that contrasts with overnight catalyst noise)
    SmallGapBigDayDownDeepAboveYearLowHotVol,  // gap_pct.abs() < 0.5 AND day_pct < -3 AND year_low_pct >= 20 AND rel_volume >= 1.5 — small gap (|gap|<0.5%) + big intraday down (<-3%) + far above 52w low (>=20%) + hot vol (pure intraday rejection from elevated level: open is essentially flat to prior close then regular session prints a sustained sell-driven move deep in advance territory; institutional-offer activation signal that contrasts with overnight catalyst noise)
    SmallGapBigDayUpMidYearHighHotVol,         // gap_pct.abs() < 0.5 AND day_pct > 3 AND year_high_pct >= 5 AND year_high_pct < 20 AND rel_volume >= 1.5 — small gap (|gap|<0.5%) + big intraday up (>3%) + mid-range from high (5-20%) + hot vol (pure intraday rally in mid-cycle pullback zone: open is essentially flat to prior close then regular session prints a sustained buy-driven move in the proper consolidation range; pure-intraday push back toward the prior peak without overnight catalyst contribution)
    SmallGapBigDayDownMidYearLowHotVol,        // gap_pct.abs() < 0.5 AND day_pct < -3 AND year_low_pct >= 5 AND year_low_pct < 20 AND rel_volume >= 1.5 — small gap (|gap|<0.5%) + big intraday down (<-3%) + mid-range from low (5-20%) + hot vol (pure intraday rejection in mid-cycle recovery zone: open is essentially flat to prior close then regular session prints a sustained sell-driven move in the proper consolidation range; pure-intraday push back toward the prior trough without overnight catalyst contribution)
    SmallGapBigDayUpJustOffYearHighHotVol,     // gap_pct.abs() < 0.5 AND day_pct > 3 AND year_high_pct >= 2 AND year_high_pct < 5 AND rel_volume >= 1.5 — small gap (|gap|<0.5%) + big intraday up (>3%) + just off 52w high (2-5%) + hot vol (pure intraday recovery from shallow pullback: open is essentially flat to prior close then regular session prints a sustained buy-driven move back toward the recent peak; pure-intraday post-tag re-test attempt without overnight catalyst contribution)
    SmallGapBigDayDownJustOffYearLowHotVol,    // gap_pct.abs() < 0.5 AND day_pct < -3 AND year_low_pct >= 2 AND year_low_pct < 5 AND rel_volume >= 1.5 — small gap (|gap|<0.5%) + big intraday down (<-3%) + just off 52w low (2-5%) + hot vol (pure intraday rejection from shallow bounce: open is essentially flat to prior close then regular session prints a sustained sell-driven move back toward the recent trough; pure-intraday post-tag re-test attempt without overnight catalyst contribution)
    BigUpDayCloseAtHodHotVol,                  // day_pct > 3 AND hod_dist_pct.abs() < 0.5 AND rel_volume >= 1.5 — big intraday up (>3%) + close pinned to HOD + hot vol (strongest possible intraday rally pattern: regular session prints a sustained buy-driven move from open to close with no end-of-day fade; isolates intraday-only conviction without conflating gap contribution; ideal for measuring real-session bull pressure)
    BigDownDayCloseAtLodHotVol,                // day_pct < -3 AND lod_dist_pct.abs() < 0.5 AND rel_volume >= 1.5 — big intraday down (<-3%) + close pinned to LOD + hot vol (strongest possible intraday selloff pattern: regular session prints a sustained sell-driven move from open to close with no end-of-day bounce; isolates intraday-only conviction without conflating gap contribution; ideal for measuring real-session bear pressure)
    BigUpDayDoubledVolHotVol,                  // day_pct > 3 AND rel_volume >= 2 — big intraday up (>3%) + doubled vol (>=2) (institutional intraday accumulation signal: regular session prints a sustained buy-driven move on doubled participation regardless of gap and close-position context; pure-intraday volume-conviction filter that ignores overnight repricing noise)
    BigDownDayDoubledVolHotVol,                // day_pct < -3 AND rel_volume >= 2 — big intraday down (<-3%) + doubled vol (>=2) (institutional intraday distribution signal: regular session prints a sustained sell-driven move on doubled participation regardless of gap and close-position context; pure-intraday volume-conviction filter that ignores overnight repricing noise)
    BigUpDayDoubledVolNearYearHighHotVol,      // day_pct > 3 AND rel_volume >= 2 AND year_high_pct < 2 — big intraday up (>3%) + doubled vol (>=2) + at/near 52w high (<2%) (institutional intraday accumulation at the year peak: regular session prints a sustained buy-driven move on doubled participation while price prints fresh highs; gap-agnostic conviction filter for breakout participation)
    BigDownDayDoubledVolNearYearLowHotVol,     // day_pct < -3 AND rel_volume >= 2 AND year_low_pct < 2 — big intraday down (<-3%) + doubled vol (>=2) + at/near 52w low (<2%) (institutional intraday distribution at the year trough: regular session prints a sustained sell-driven move on doubled participation while price prints fresh lows; gap-agnostic conviction filter for breakdown participation)
    BigUpDayDoubledVolConfirmedAboveYearHighHotVol,  // day_pct > 3 AND rel_volume >= 2 AND year_high_pct >= -3 AND year_high_pct <= -1 — big intraday up (>3%) + doubled vol (>=2) + confirmed-breakout zone (1-3% past 52w high) (institutional intraday accumulation in the validated-breakout zone: regular session prints a sustained buy-driven move on doubled participation while price extends past the prior peak; gap-agnostic conviction filter for breakout follow-through)
    BigDownDayDoubledVolConfirmedBelowYearLowHotVol, // day_pct < -3 AND rel_volume >= 2 AND year_low_pct >= -3 AND year_low_pct <= -1 — big intraday down (<-3%) + doubled vol (>=2) + confirmed-breakdown zone (1-3% past 52w low) (institutional intraday distribution in the validated-breakdown zone: regular session prints a sustained sell-driven move on doubled participation while price extends past the prior trough; gap-agnostic conviction filter for breakdown follow-through)
    BigUpDayDoubledVolDeepBelowYearHighHotVol, // day_pct > 3 AND rel_volume >= 2 AND year_high_pct >= 20 — big intraday up (>3%) + doubled vol (>=2) + far below 52w high (>=20%) (institutional intraday accumulation deep in pullback territory: regular session prints a sustained buy-driven move on doubled participation while price remains well below the prior peak; conviction-recovery-rally signal worth a turnaround-screen)
    BigDownDayDoubledVolDeepAboveYearLowHotVol,// day_pct < -3 AND rel_volume >= 2 AND year_low_pct >= 20 — big intraday down (<-3%) + doubled vol (>=2) + far above 52w low (>=20%) (institutional intraday distribution deep in advance territory: regular session prints a sustained sell-driven move on doubled participation while price remains well above the prior trough; conviction-rejection signal worth a top-fade-screen)
    BigUpDayDoubledVolMidYearHighHotVol,       // day_pct > 3 AND rel_volume >= 2 AND year_high_pct >= 5 AND year_high_pct < 20 — big intraday up (>3%) + doubled vol (>=2) + mid-range from high (5-20%) (institutional intraday accumulation in mid-cycle pullback zone: regular session prints a sustained buy-driven move on doubled participation in the proper consolidation range below the prior peak; conviction-mid-cycle-rally signal worth a swing-screen)
    BigDownDayDoubledVolMidYearLowHotVol,      // day_pct < -3 AND rel_volume >= 2 AND year_low_pct >= 5 AND year_low_pct < 20 — big intraday down (<-3%) + doubled vol (>=2) + mid-range from low (5-20%) (institutional intraday distribution in mid-cycle recovery zone: regular session prints a sustained sell-driven move on doubled participation in the proper consolidation range above the prior trough; conviction-mid-cycle-rejection signal worth a swing-screen)
    BigUpDayDoubledVolJustOffYearHighHotVol,   // day_pct > 3 AND rel_volume >= 2 AND year_high_pct >= 2 AND year_high_pct < 5 — big intraday up (>3%) + doubled vol (>=2) + just off 52w high (2-5%) (institutional intraday accumulation just off the year peak: regular session prints a sustained buy-driven move on doubled participation immediately after a shallow pullback from the 52w high; conviction-post-tag-recovery signal worth a re-test-screen)
    BigDownDayDoubledVolJustOffYearLowHotVol,  // day_pct < -3 AND rel_volume >= 2 AND year_low_pct >= 2 AND year_low_pct < 5 — big intraday down (<-3%) + doubled vol (>=2) + just off 52w low (2-5%) (institutional intraday distribution just off the year trough: regular session prints a sustained sell-driven move on doubled participation immediately after a shallow bounce from the 52w low; conviction-post-tag-rejection signal worth a re-test-screen)
    QuintupledVolUpHotVol,                     // rel_volume >= 5 AND change_pct > 3 — quintupled vol (>=5) + big up move (>3%) (extreme participation event with bull-direction: vol is 5x its average and price prints a significant up move; rare news/earnings/catalyst day at the highest possible conviction tier, typically a once-per-quarter occurrence per name)
    QuintupledVolDownHotVol,                   // rel_volume >= 5 AND change_pct < -3 — quintupled vol (>=5) + big down move (<-3%) (extreme participation event with bear-direction: vol is 5x its average and price prints a significant down move; rare news/earnings/catalyst day at the highest possible conviction tier, typically a once-per-quarter occurrence per name)
    QuintupledVolUpNearYearHighHotVol,         // rel_volume >= 5 AND change_pct > 3 AND year_high_pct < 2 — quintupled vol (>=5) + big up move (>3%) + at/near 52w high (<2%) (once-per-quarter breakout event at the year peak: vol is 5x its average, price prints a significant up move and reaches the 52w high simultaneously; highest-conviction breakout-at-extreme signal worth a tier-1 alert)
    QuintupledVolDownNearYearLowHotVol,        // rel_volume >= 5 AND change_pct < -3 AND year_low_pct < 2 — quintupled vol (>=5) + big down move (<-3%) + at/near 52w low (<2%) (once-per-quarter breakdown event at the year trough: vol is 5x its average, price prints a significant down move and reaches the 52w low simultaneously; highest-conviction breakdown-at-extreme signal worth a tier-1 alert)
    QuintupledVolUpDeepBelowYearHighHotVol,    // rel_volume >= 5 AND change_pct > 3 AND year_high_pct >= 20 — quintupled vol (>=5) + big up move (>3%) + far below 52w high (>=20%) (extreme catalyst recovery from deep pullback: vol is 5x average and price prints a significant up move while still well below the prior peak; turnaround-catalyst at the highest possible tier worth a regime-change alert)
    QuintupledVolDownDeepAboveYearLowHotVol,   // rel_volume >= 5 AND change_pct < -3 AND year_low_pct >= 20 — quintupled vol (>=5) + big down move (<-3%) + far above 52w low (>=20%) (extreme catalyst rejection from deep advance: vol is 5x average and price prints a significant down move while still well above the prior trough; top-fade catalyst at the highest possible tier worth a regime-change alert)
    QuintupledVolUpConfirmedAboveYearHighHotVol,  // rel_volume >= 5 AND change_pct > 3 AND year_high_pct >= -3 AND year_high_pct <= -1 — quintupled vol (>=5) + big up move (>3%) + confirmed-breakout zone (1-3% past 52w high) (extreme catalyst extending validated breakout: vol is 5x average and price prints a significant up move while extending further past the prior peak; trend-extension at the highest possible tier worth a follow-through alert)
    QuintupledVolDownConfirmedBelowYearLowHotVol, // rel_volume >= 5 AND change_pct < -3 AND year_low_pct >= -3 AND year_low_pct <= -1 — quintupled vol (>=5) + big down move (<-3%) + confirmed-breakdown zone (1-3% past 52w low) (extreme catalyst extending validated breakdown: vol is 5x average and price prints a significant down move while extending further past the prior trough; trend-extension at the highest possible tier worth a follow-through alert)
    QuintupledVolUpMidYearHighHotVol,          // rel_volume >= 5 AND change_pct > 3 AND year_high_pct >= 5 AND year_high_pct < 20 — quintupled vol (>=5) + big up move (>3%) + mid-range from high (5-20%) (extreme catalyst rally in mid-cycle pullback zone: vol is 5x average and price prints a significant up move in the proper consolidation range below the prior peak; tier-1 conviction-recovery signal worth a swing-screen)
    QuintupledVolDownMidYearLowHotVol,         // rel_volume >= 5 AND change_pct < -3 AND year_low_pct >= 5 AND year_low_pct < 20 — quintupled vol (>=5) + big down move (<-3%) + mid-range from low (5-20%) (extreme catalyst rejection in mid-cycle recovery zone: vol is 5x average and price prints a significant down move in the proper consolidation range above the prior trough; tier-1 conviction-rejection signal worth a swing-screen)
    QuintupledVolUpJustOffYearHighHotVol,      // rel_volume >= 5 AND change_pct > 3 AND year_high_pct >= 2 AND year_high_pct < 5 — quintupled vol (>=5) + big up move (>3%) + just off 52w high (2-5%) (extreme catalyst rally just off the year peak: vol is 5x average and price prints a significant up move immediately after a shallow pullback from the 52w high; tier-1 post-tag-recovery catalyst worth a re-test-screen)
    QuintupledVolDownJustOffYearLowHotVol,     // rel_volume >= 5 AND change_pct < -3 AND year_low_pct >= 2 AND year_low_pct < 5 — quintupled vol (>=5) + big down move (<-3%) + just off 52w low (2-5%) (extreme catalyst rejection just off the year trough: vol is 5x average and price prints a significant down move immediately after a shallow bounce from the 52w low; tier-1 post-tag-rejection catalyst worth a re-test-screen)
    QuintupledVolCloseAtHodHotVol,             // rel_volume >= 5 AND hod_dist_pct.abs() < 0.5 AND change_pct > 1 — quintupled vol (>=5) + close pinned to HOD + green close + hot vol (tier-1 institutional rally with no end-of-day fade: vol is 5x average and close pins to the day's high with positive change; rarest possible bull-conviction close at the highest participation tier)
    QuintupledVolCloseAtLodHotVol,             // rel_volume >= 5 AND lod_dist_pct.abs() < 0.5 AND change_pct < -1 — quintupled vol (>=5) + close pinned to LOD + red close + hot vol (tier-1 institutional selloff with no end-of-day bounce: vol is 5x average and close pins to the day's low with negative change; rarest possible bear-conviction close at the highest participation tier)
    QuintupledVolGapUpHotVol,                  // rel_volume >= 5 AND gap_pct > 2 — quintupled vol (>=5) + gap up (>2%) (tier-1 catalyst gap-up: vol is 5x average and overnight repricing pushes the open more than 2% above prior close; rare news/earnings/sector-rotation event with full session participation confirming the bull-direction catalyst)
    QuintupledVolGapDownHotVol,                // rel_volume >= 5 AND gap_pct < -2 — quintupled vol (>=5) + gap down (<-2%) (tier-1 catalyst gap-down: vol is 5x average and overnight repricing pushes the open more than 2% below prior close; rare news/earnings/sector-rotation event with full session participation confirming the bear-direction catalyst)
    QuintupledVolGapUpCloseAtHodHotVol,        // rel_volume >= 5 AND gap_pct > 2 AND hod_dist_pct.abs() < 0.5 AND change_pct > 1 — quintupled vol (>=5) + gap up (>2%) + close pinned to HOD + green close (highest-conviction catalyst gap-up that holds: vol is 5x average, overnight gap up holds without fade and price closes at the day's high; rarest possible validated bull-catalyst event across all sessions)
    QuintupledVolGapDownCloseAtLodHotVol,      // rel_volume >= 5 AND gap_pct < -2 AND lod_dist_pct.abs() < 0.5 AND change_pct < -1 — quintupled vol (>=5) + gap down (<-2%) + close pinned to LOD + red close (highest-conviction catalyst gap-down that holds: vol is 5x average, overnight gap down holds without bounce and price closes at the day's low; rarest possible validated bear-catalyst event across all sessions)
    QuintupledVolGapUpCloseAtLodHotVol,        // rel_volume >= 5 AND gap_pct > 2 AND lod_dist_pct.abs() < 0.5 AND change_pct < 0 — quintupled vol (>=5) + gap up (>2%) faded completely to LOD + red close (highest-conviction failed catalyst at tier-1: vol is 5x average, overnight gap up fails completely and price closes at the day's low; rarest possible failed-bull-catalyst event, capital-S-shift regime-rejection signal)
    QuintupledVolGapDownCloseAtHodHotVol,      // rel_volume >= 5 AND gap_pct < -2 AND hod_dist_pct.abs() < 0.5 AND change_pct > 0 — quintupled vol (>=5) + gap down (<-2%) absorbed completely to HOD + green close (highest-conviction failed catalyst at tier-1: vol is 5x average, overnight gap down absorbed completely and price closes at the day's high; rarest possible failed-bear-catalyst event, capital-S-shift regime-acceptance signal)
    QuintupledVolGapUpMidpointHotVol,          // rel_volume >= 5 AND gap_pct > 2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 — quintupled vol (>=5) + gap up (>2%) + midpoint close (tier-1 catalyst gap-up with inconclusive intraday follow-through: vol is 5x average, overnight gap up holds but regular session neither extends nor fails decisively; high-stakes standoff after catalyst event with unresolved direction)
    QuintupledVolGapDownMidpointHotVol,        // rel_volume >= 5 AND gap_pct < -2 AND hod_dist_pct.abs() > 0.5 AND lod_dist_pct.abs() > 0.5 AND (hod_dist_pct.abs() - lod_dist_pct.abs()).abs() < 0.5 — quintupled vol (>=5) + gap down (<-2%) + midpoint close (tier-1 catalyst gap-down with inconclusive intraday follow-through: vol is 5x average, overnight gap down holds but regular session neither extends nor absorbs decisively; high-stakes standoff after catalyst event with unresolved direction)
    BigIntradayRangeHotVol,                    // hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 — wide intraday range (>8% high-low spread) + hot vol (volatility expansion day: regular session prints a much wider than normal trading range with elevated participation; high-volatility regime worth a directional-bias-screen at the close and an overnight-gap-screen the next morning)
    TightIntradayRangeHotVol,                  // hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — tight intraday range (<1% high-low spread) + hot vol (intraday compression with elevated participation: regular session prints a much narrower than normal trading range despite hot vol; institutional positioning event where heavy hands trade without moving the tape; breakout-candidate worth a watch-list-add)
    BigIntradayRangeNearYearHighHotVol,        // hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 AND year_high_pct < 2 — wide intraday range (>8%) + hot vol + at/near 52w high (<2%) (volatility-expansion battle at the year peak: regular session prints a wide trading range right at the 52w high with elevated participation; bulls and bears fighting hard at the key resistance with no decisive winner from range alone)
    BigIntradayRangeNearYearLowHotVol,         // hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 AND year_low_pct < 2 — wide intraday range (>8%) + hot vol + at/near 52w low (<2%) (volatility-expansion battle at the year trough: regular session prints a wide trading range right at the 52w low with elevated participation; bulls and bears fighting hard at the key support with no decisive winner from range alone)
    BigIntradayRangeConfirmedAboveYearHighHotVol,  // hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 AND year_high_pct >= -3 AND year_high_pct <= -1 — wide intraday range (>8%) + hot vol + confirmed-breakout zone (1-3% past 52w high) (volatility-expansion battle in the validated-breakout zone: regular session prints a wide trading range right after price cleared the prior peak with elevated participation; post-breakout consolidation fight where bulls defend the breakout and bears test it)
    BigIntradayRangeConfirmedBelowYearLowHotVol,   // hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 AND year_low_pct >= -3 AND year_low_pct <= -1 — wide intraday range (>8%) + hot vol + confirmed-breakdown zone (1-3% past 52w low) (volatility-expansion battle in the validated-breakdown zone: regular session prints a wide trading range right after price cleared the prior trough with elevated participation; post-breakdown consolidation fight where bears defend the breakdown and bulls test it)
    BigIntradayRangeDeepBelowYearHighHotVol,   // hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 AND year_high_pct >= 20 — wide intraday range (>8%) + hot vol + far below 52w high (>=20%) (volatility-expansion battle deep in pullback territory: regular session prints a wide trading range well below the prior peak with elevated participation; capitulation-or-recovery decision fight where extended decline meets institutional pushback)
    BigIntradayRangeDeepAboveYearLowHotVol,    // hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 AND year_low_pct >= 20 — wide intraday range (>8%) + hot vol + far above 52w low (>=20%) (volatility-expansion battle deep in advance territory: regular session prints a wide trading range well above the prior trough with elevated participation; top-or-continuation decision fight where extended advance meets institutional pushback)
    BigIntradayRangeMidYearHighHotVol,         // hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 AND year_high_pct >= 5 AND year_high_pct < 20 — wide intraday range (>8%) + hot vol + mid-range from high (5-20%) (volatility-expansion battle in mid-cycle pullback zone: regular session prints a wide trading range in the proper consolidation range below the prior peak with elevated participation; mid-cycle indecision-resolution fight requiring close-position confirmation)
    BigIntradayRangeMidYearLowHotVol,          // hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 AND year_low_pct >= 5 AND year_low_pct < 20 — wide intraday range (>8%) + hot vol + mid-range from low (5-20%) (volatility-expansion battle in mid-cycle recovery zone: regular session prints a wide trading range in the proper consolidation range above the prior trough with elevated participation; mid-cycle indecision-resolution fight requiring close-position confirmation)
    BigIntradayRangeJustOffYearHighHotVol,     // hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 AND year_high_pct >= 2 AND year_high_pct < 5 — wide intraday range (>8%) + hot vol + just off 52w high (2-5%) (volatility-expansion battle just off the year peak: regular session prints a wide trading range immediately after a shallow pullback from the 52w high with elevated participation; post-tag re-test fight where bulls attempt the high again and bears defend it)
    BigIntradayRangeJustOffYearLowHotVol,      // hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 AND year_low_pct >= 2 AND year_low_pct < 5 — wide intraday range (>8%) + hot vol + just off 52w low (2-5%) (volatility-expansion battle just off the year trough: regular session prints a wide trading range immediately after a shallow bounce from the 52w low with elevated participation; post-tag re-test fight where bears attempt the low again and bulls defend it)
    TightIntradayRangeNearYearHighHotVol,      // hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 AND year_high_pct < 2 — tight intraday range (<1%) + hot vol + at/near 52w high (<2%) (institutional compression at the year peak: regular session prints a tight trading range right at the 52w high with elevated participation; coiled-spring breakout setup where heavy hands position without moving the tape and the next directional break carries weight)
    TightIntradayRangeNearYearLowHotVol,       // hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 AND year_low_pct < 2 — tight intraday range (<1%) + hot vol + at/near 52w low (<2%) (institutional compression at the year trough: regular session prints a tight trading range right at the 52w low with elevated participation; coiled-spring breakdown-or-bottom setup where heavy hands position without moving the tape and the next directional break carries weight)
    TightIntradayRangeConfirmedAboveYearHighHotVol,  // hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 AND year_high_pct >= -3 AND year_high_pct <= -1 — tight intraday range (<1%) + hot vol + confirmed-breakout zone (1-3% past 52w high) (institutional digestion of validated breakout: regular session prints a tight trading range immediately after price cleared the prior peak with elevated participation; post-breakout consolidation where bulls absorb and digest the breakout before the next leg)
    TightIntradayRangeConfirmedBelowYearLowHotVol,   // hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 AND year_low_pct >= -3 AND year_low_pct <= -1 — tight intraday range (<1%) + hot vol + confirmed-breakdown zone (1-3% past 52w low) (institutional digestion of validated breakdown: regular session prints a tight trading range immediately after price cleared the prior trough with elevated participation; post-breakdown consolidation where bears absorb and digest the breakdown before the next leg)
    TightIntradayRangeDeepBelowYearHighHotVol, // hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 AND year_high_pct >= 20 — tight intraday range (<1%) + hot vol + far below 52w high (>=20%) (institutional accumulation deep in pullback territory: regular session prints a tight trading range well below the prior peak with elevated participation; basing-pattern signal where smart money builds position quietly in depressed-tape conditions)
    TightIntradayRangeDeepAboveYearLowHotVol,  // hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 AND year_low_pct >= 20 — tight intraday range (<1%) + hot vol + far above 52w low (>=20%) (institutional distribution deep in advance territory: regular session prints a tight trading range well above the prior trough with elevated participation; topping-pattern signal where smart money exits position quietly in euphoric-tape conditions)
    TightIntradayRangeMidYearHighHotVol,       // hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 AND year_high_pct >= 5 AND year_high_pct < 20 — tight intraday range (<1%) + hot vol + mid-range from high (5-20%) (institutional pause in mid-cycle pullback zone: regular session prints a tight trading range in the proper consolidation range below the prior peak with elevated participation; mid-cycle accumulation/pause where smart money positions before the next directional move)
    TightIntradayRangeMidYearLowHotVol,        // hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 AND year_low_pct >= 5 AND year_low_pct < 20 — tight intraday range (<1%) + hot vol + mid-range from low (5-20%) (institutional pause in mid-cycle recovery zone: regular session prints a tight trading range in the proper consolidation range above the prior trough with elevated participation; mid-cycle distribution/pause where smart money positions before the next directional move)
    TightIntradayRangeJustOffYearHighHotVol,   // hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 AND year_high_pct >= 2 AND year_high_pct < 5 — tight intraday range (<1%) + hot vol + just off 52w high (2-5%) (institutional positioning just off the year peak: regular session prints a tight trading range immediately after a shallow pullback from the 52w high with elevated participation; post-tag pause where heavy hands re-position quietly before attempting the high again)
    TightIntradayRangeJustOffYearLowHotVol,    // hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 AND year_low_pct >= 2 AND year_low_pct < 5 — tight intraday range (<1%) + hot vol + just off 52w low (2-5%) (institutional positioning just off the year trough: regular session prints a tight trading range immediately after a shallow bounce from the 52w low with elevated participation; post-tag pause where heavy hands re-position quietly before testing the low again)
    GapUpTightRangeHotVol,                     // gap_pct > 2 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — gap up (>2%) + tight intraday range (<1%) + hot vol (gap-and-park institutional signal: overnight gap up but regular session prints a very tight intraday range with no follow-through or fade; price pins at the gap level with heavy participation, suggesting strong absorption at the new price)
    GapDownTightRangeHotVol,                   // gap_pct < -2 AND hod_dist_pct.abs() + lod_dist_pct.abs() < 1 AND rel_volume >= 1.5 — gap down (<-2%) + tight intraday range (<1%) + hot vol (gap-and-park institutional signal: overnight gap down but regular session prints a very tight intraday range with no follow-through or recovery; price pins at the gap level with heavy participation, suggesting strong absorption at the new price)
    GapUpWideRangeHotVol,                      // gap_pct > 2 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 — gap up (>2%) + wide intraday range (>8%) + hot vol (gap-then-volatility-expansion catalyst signal: overnight gap up followed by a wide trading range with elevated participation; two-way fight intraday after the catalyst with bulls and bears trading aggressively in the gap zone; close-position resolves direction)
    GapDownWideRangeHotVol,                    // gap_pct < -2 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 — gap down (<-2%) + wide intraday range (>8%) + hot vol (gap-then-volatility-expansion catalyst signal: overnight gap down followed by a wide trading range with elevated participation; two-way fight intraday after the catalyst with bears and bulls trading aggressively in the gap zone; close-position resolves direction)
    GapUpWideRangeNearYearHighHotVol,          // gap_pct > 2 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 AND year_high_pct < 2 — gap up (>2%) + wide intraday range (>8%) + hot vol + at/near 52w high (<2%) (gap-and-fight at the year peak: overnight gap up followed by a wide trading range at the 52w high with elevated participation; high-stakes breakout-day battle where catalyst meets prior peak resistance; close-position resolves whether the breakout sticks)
    GapDownWideRangeNearYearLowHotVol,         // gap_pct < -2 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 AND year_low_pct < 2 — gap down (<-2%) + wide intraday range (>8%) + hot vol + at/near 52w low (<2%) (gap-and-fight at the year trough: overnight gap down followed by a wide trading range at the 52w low with elevated participation; high-stakes breakdown-day battle where catalyst meets prior trough support; close-position resolves whether the breakdown sticks)
    GapUpWideRangeConfirmedAboveYearHighHotVol,// gap_pct > 2 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 AND year_high_pct >= -3 AND year_high_pct <= -1 — gap up (>2%) + wide intraday range (>8%) + hot vol + confirmed-breakout zone (1-3% past 52w high) (gap-and-fight in the validated-breakout zone: overnight gap up followed by a wide trading range right after price cleared the prior peak with elevated participation; post-breakout extension battle where bulls defend the breakout and bears test it)
    GapDownWideRangeConfirmedBelowYearLowHotVol,// gap_pct < -2 AND hod_dist_pct.abs() + lod_dist_pct.abs() > 8 AND rel_volume >= 1.5 AND year_low_pct >= -3 AND year_low_pct <= -1 — gap down (<-2%) + wide intraday range (>8%) + hot vol + confirmed-breakdown zone (1-3% past 52w low) (gap-and-fight in the validated-breakdown zone: overnight gap down followed by a wide trading range right after price cleared the prior trough with elevated participation; post-breakdown extension battle where bears defend the breakdown and bulls test it)
}

pub fn matches(hit: &ScanHit, preset: Preset) -> bool {
    match preset {
        Preset::PremarketGappers => hit.gap_pct.abs() >= 5.0,
        Preset::MomentumMovers => hit.change_pct >= 5.0 && hit.rel_volume >= 2.0,
        Preset::HighOfDay => hit.hod_dist_pct.abs() <= 0.5,
        Preset::LowFloatRunners => hit.change_pct >= 10.0 && hit.rel_volume >= 5.0,
        Preset::Pct52wHigh => hit.year_high_pct >= -1.0,
        Preset::Pct52wLow => hit.year_low_pct <= 1.0,
        Preset::VolumeSurge => hit.rel_volume >= 3.0,
        Preset::Breakdown => hit.change_pct <= -5.0,
        Preset::Breakout => hit.day_pct > 0.0 && hit.hod_dist_pct.abs() <= 0.5,
        Preset::OversoldBounce => hit.change_pct > 0.0, // simplified — needs prior bar context
        Preset::GapAndGo => {
            hit.gap_pct >= 3.0
                && hit.day_pct > 0.0
                && hit.hod_dist_pct.abs() <= 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapAndFade => {
            hit.gap_pct >= 3.0
                && hit.day_pct < 0.0
                && hit.lod_dist_pct.abs() <= 1.0
        }
        Preset::InsideDayLow => hit.lod_dist_pct.abs() <= 0.5,
        Preset::InsideDayHigh => {
            hit.hod_dist_pct.abs() <= 0.5 && hit.change_pct.abs() <= 1.0
        }
        Preset::RangeContractionDay => {
            hit.day_pct.abs() <= 0.5 && hit.gap_pct.abs() <= 0.5 && hit.rel_volume <= 0.7
        }
        Preset::DistributionDay => hit.change_pct <= -2.0 && hit.rel_volume >= 1.5,
        Preset::AccumulationDay => hit.change_pct >= 2.0 && hit.rel_volume >= 1.5,
        Preset::NearYearHighLowVol => hit.year_high_pct >= -1.0 && hit.rel_volume < 1.0,
        Preset::InsideDaySqueeze => {
            hit.day_pct.abs() <= 1.0
                && hit.rel_volume <= 0.8
                && hit.hod_dist_pct.abs() <= 1.5
                && hit.lod_dist_pct.abs() <= 1.5
        }
        Preset::LowVolSqueeze => hit.rel_volume < 0.5 && hit.day_pct.abs() < 1.0,
        Preset::CoilingSqueeze => {
            hit.change_pct.abs() < 1.0 && hit.rel_volume < 0.7 && hit.gap_pct.abs() < 0.5
        }
        Preset::MidRangeSqueeze => {
            hit.year_high_pct <= -10.0
                && hit.year_low_pct >= 10.0
                && hit.rel_volume <= 0.8
                && hit.day_pct.abs() <= 1.0
        }
        Preset::BracketSqueeze => {
            hit.day_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() < 0.5
                && hit.lod_dist_pct.abs() < 0.5
        }
        Preset::DojiSqueeze => {
            hit.change_pct.abs() < 0.2
                && hit.day_pct.abs() < 0.5
                && hit.gap_pct.abs() < 0.3
        }
        Preset::GapFillSqueeze => {
            hit.gap_pct.abs() >= 1.0
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume <= 0.8
        }
        Preset::EndOfRangeSqueeze => {
            hit.day_pct.abs() < 1.0
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.hod_dist_pct.abs() < 1.5
                && hit.lod_dist_pct.abs() < 1.5
        }
        Preset::PreBreakoutSqueeze => {
            hit.year_high_pct >= -3.0
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.8
        }
        Preset::PreBreakdownSqueeze => {
            hit.year_low_pct <= 3.0
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.8
        }
        Preset::SymmetricSqueeze => {
            (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.2
                && hit.change_pct.abs() < 0.3
                && hit.gap_pct.abs() < 0.3
                && hit.rel_volume < 0.8
        }
        Preset::OpenCloseSqueeze => {
            hit.day_pct.abs() < 0.3 && hit.rel_volume < 0.8
        }
        Preset::TightHodSqueeze => {
            hit.hod_dist_pct.abs() < 0.3
                && hit.change_pct.abs() < 1.0
                && hit.rel_volume < 0.8
        }
        Preset::TightLodSqueeze => {
            hit.lod_dist_pct.abs() < 0.3
                && hit.change_pct.abs() < 1.0
                && hit.rel_volume < 0.8
        }
        Preset::NoGapNoChangeSqueeze => {
            hit.gap_pct.abs() < 0.2
                && hit.change_pct.abs() < 0.2
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.8
        }
        Preset::QuietTickSqueeze => {
            hit.rel_volume < 0.3 && hit.day_pct.abs() < 0.5
        }
        Preset::NarrowGapPostMomentum => {
            hit.gap_pct.abs() < 0.3
                && hit.change_pct.abs() >= 3.0
                && hit.day_pct.abs() < 1.0
                && hit.rel_volume < 1.0
        }
        Preset::DistantExtremesSqueeze => {
            hit.year_high_pct <= -20.0
                && hit.year_low_pct >= 20.0
                && hit.rel_volume < 0.9
                && hit.day_pct.abs() < 1.5
        }
        Preset::BalancedDriftSqueeze => {
            hit.gap_pct.abs() < 0.3
                && hit.change_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() < 1.5
                && hit.lod_dist_pct.abs() < 1.5
                && hit.rel_volume < 0.9
        }
        Preset::PennyMoveSqueeze => {
            hit.change_pct.abs() < 0.05
        }
        Preset::DryUpSqueeze => {
            hit.rel_volume < 0.4
                && hit.change_pct.abs() < 1.5
                && hit.gap_pct.abs() < 0.5
        }
        Preset::UpperRangeSqueeze => {
            hit.lod_dist_pct.abs() > 2.0 * hit.hod_dist_pct.abs()
                && hit.day_pct.abs() < 1.5
                && hit.rel_volume < 0.9
                && hit.hod_dist_pct.abs() < 1.0
        }
        Preset::LowerRangeSqueeze => {
            hit.hod_dist_pct.abs() > 2.0 * hit.lod_dist_pct.abs()
                && hit.day_pct.abs() < 1.5
                && hit.rel_volume < 0.9
                && hit.lod_dist_pct.abs() < 1.0
        }
        Preset::GapReversalSqueeze => {
            hit.gap_pct.signum() != hit.change_pct.signum()
                && hit.gap_pct.abs() >= 0.5
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 1.0
        }
        Preset::Pct52wMidSqueeze => {
            hit.year_high_pct <= -5.0
                && hit.year_high_pct >= -15.0
                && hit.year_low_pct >= 5.0
                && hit.year_low_pct <= 15.0
                && hit.day_pct.abs() < 1.0
                && hit.rel_volume < 0.9
        }
        Preset::DeepDiscountSqueeze => {
            hit.year_high_pct <= -30.0
                && hit.day_pct.abs() < 1.0
                && hit.rel_volume < 0.9
        }
        Preset::FlatRangeQuietSqueeze => {
            hit.day_pct.abs() < 0.2
                && hit.gap_pct.abs() < 0.2
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.5
        }
        Preset::NearAthQuietSqueeze => {
            hit.year_high_pct >= -1.0
                && hit.rel_volume < 0.6
                && hit.day_pct.abs() < 1.0
        }
        Preset::NearAtlQuietSqueeze => {
            hit.year_low_pct <= 1.0
                && hit.rel_volume < 0.6
                && hit.day_pct.abs() < 1.0
        }
        Preset::SilentBreakoutSetup => {
            hit.hod_dist_pct.abs() < 0.5
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.7
                && hit.year_high_pct >= -5.0
        }
        Preset::SilentBreakdownSetup => {
            hit.lod_dist_pct.abs() < 0.5
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.7
                && hit.year_low_pct <= 5.0
        }
        Preset::GapDownNoFollowSqueeze => {
            hit.gap_pct <= -1.0
                && hit.change_pct >= -0.5
                && hit.day_pct.abs() < 0.5
        }
        Preset::GapUpNoFollowSqueeze => {
            hit.gap_pct >= 1.0
                && hit.change_pct <= 0.5
                && hit.day_pct.abs() < 0.5
        }
        Preset::UnchVolDryUpSqueeze => {
            hit.change_pct.abs() < 0.1 && hit.rel_volume < 0.5
        }
        Preset::NarrowAfterTrendSqueeze => {
            hit.day_pct.abs() < 0.5 && hit.change_pct.abs() >= 5.0
        }
        Preset::DeadCenterSqueeze => {
            (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.4
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume < 0.7
        }
        Preset::AnchorDriftSqueeze => {
            hit.day_pct.abs() < 1.0
                && hit.change_pct.abs() < 1.0
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume < 0.8
        }
        Preset::PostGapFillSqueeze => {
            hit.gap_pct.signum() != hit.change_pct.signum()
                && hit.change_pct.abs() < hit.gap_pct.abs() / 2.0
                && hit.gap_pct.abs() >= 1.0
                && hit.day_pct.abs() < 0.5
        }
        Preset::PostSpikeQuietSqueeze => {
            hit.change_pct.abs() > 2.0
                && hit.day_pct.abs() < 0.3
                && hit.rel_volume < 0.6
        }
        Preset::HighSqueezeBracket => {
            hit.hod_dist_pct.abs() < 1.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.year_high_pct >= -3.0
        }
        Preset::LowSqueezeBracket => {
            hit.hod_dist_pct.abs() < 1.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.year_low_pct <= 3.0
        }
        Preset::HighRelVolStallSqueeze => {
            hit.rel_volume >= 1.5
                && hit.change_pct.abs() < 0.3
                && hit.day_pct.abs() < 0.5
        }
        Preset::SlightLeanLongSqueeze => {
            hit.change_pct >= 0.2
                && hit.change_pct <= 1.0
                && hit.rel_volume < 0.8
                && hit.day_pct.abs() < 0.6
        }
        Preset::SlightLeanShortSqueeze => {
            hit.change_pct <= -0.2
                && hit.change_pct >= -1.0
                && hit.rel_volume < 0.8
                && hit.day_pct.abs() < 0.6
        }
        Preset::GapWithChangeMatchSqueeze => {
            hit.gap_pct.signum() == hit.change_pct.signum()
                && hit.gap_pct.abs() < 0.5
                && hit.change_pct.abs() < 0.5
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.8
        }
        Preset::SlackBetweenExtremesSqueeze => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::PivotPinSqueeze => {
            hit.day_pct.abs() < 0.3
                && hit.hod_dist_pct.abs() < 1.0
                && hit.lod_dist_pct.abs() < 1.0
        }
        Preset::EvenSidesSqueeze => {
            hit.gap_pct.signum() != hit.change_pct.signum()
                && hit.gap_pct.abs() < 1.0
                && hit.change_pct.abs() < 1.0
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.8
        }
        Preset::InsideQuarterDaySqueeze => {
            hit.day_pct.abs() < 0.25
                && hit.change_pct.abs() < 1.0
                && hit.rel_volume < 0.8
        }
        Preset::EvenVolumeQuietSqueeze => {
            hit.rel_volume >= 0.9
                && hit.rel_volume <= 1.1
                && hit.day_pct.abs() < 0.3
                && hit.change_pct.abs() < 0.5
        }
        Preset::TightCoilHighSqueeze => {
            hit.year_high_pct >= -2.0
                && hit.day_pct.abs() < 0.5
                && hit.change_pct.abs() < 0.8
                && hit.hod_dist_pct.abs() < 1.0
        }
        Preset::TightCoilLowSqueeze => {
            hit.year_low_pct <= 2.0
                && hit.day_pct.abs() < 0.5
                && hit.change_pct.abs() < 0.8
                && hit.lod_dist_pct.abs() < 1.0
        }
        Preset::EvenWidthSqueeze => {
            hit.hod_dist_pct.abs() >= 1.0
                && hit.hod_dist_pct.abs() <= 2.0
                && hit.lod_dist_pct.abs() >= 1.0
                && hit.lod_dist_pct.abs() <= 2.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.9
        }
        Preset::SmallGapNoFollowSqueeze => {
            hit.gap_pct.abs() >= 0.3
                && hit.gap_pct.abs() <= 0.8
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume < 0.8
        }
        Preset::HoldingHighsSqueeze => {
            hit.change_pct >= 0.0
                && hit.change_pct < 1.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.rel_volume < 1.2
                && hit.year_high_pct >= -5.0
        }
        Preset::HoldingLowsSqueeze => {
            hit.change_pct <= 0.0
                && hit.change_pct > -1.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.rel_volume < 1.2
                && hit.year_low_pct <= 5.0
        }
        Preset::StableMidSqueeze => {
            let denom = hit.year_high_pct.abs() + hit.year_low_pct.abs();
            let mid_frac = if denom > 1e-9 {
                1.0 - hit.year_high_pct.abs() / denom
            } else { 0.5 };
            mid_frac > 0.30
                && mid_frac < 0.70
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.9
        }
        Preset::LeanGapMatchSqueeze => {
            hit.gap_pct.signum() == hit.change_pct.signum()
                && hit.gap_pct.abs() >= 0.5
                && hit.gap_pct.abs() <= 1.5
                && hit.change_pct.abs() >= 0.5
                && hit.change_pct.abs() <= 1.5
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.9
        }
        Preset::LongShadowQuietSqueeze => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 6.0
                && hit.day_pct.abs() < 1.0
                && hit.rel_volume < 0.9
        }
        Preset::ChangeNoDayPctSqueeze => {
            hit.change_pct.abs() >= 1.0
                && hit.day_pct.abs() < 0.2
                && hit.rel_volume < 1.0
        }
        Preset::DayPctNoChangeSqueeze => {
            hit.day_pct.abs() >= 1.0
                && hit.change_pct.abs() < 0.2
                && hit.rel_volume < 1.0
        }
        Preset::HotDryUpSqueeze => {
            hit.year_high_pct >= -1.0
                && hit.rel_volume < 0.5
                && hit.day_pct.abs() < 0.5
        }
        Preset::ColdDryUpSqueeze => {
            hit.year_low_pct <= 1.0
                && hit.rel_volume < 0.5
                && hit.day_pct.abs() < 0.5
        }
        Preset::HighVolGapFadeSqueeze => {
            hit.gap_pct.abs() >= 1.0
                && hit.gap_pct.signum() != hit.change_pct.signum()
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume >= 1.2
        }
        Preset::NearZeroChangeQuietSqueeze => {
            hit.change_pct.abs() < 0.5
                && hit.gap_pct.abs() < 0.5
                && hit.day_pct.abs() < 1.0
                && hit.rel_volume < 0.7
        }
        Preset::SilentInsideSqueeze => {
            hit.day_pct.abs() < 0.4
                && hit.change_pct.abs() < 0.4
                && hit.gap_pct.abs() < 0.4
                && hit.rel_volume < 0.7
        }
        Preset::HighVolNoMoveSqueeze => {
            hit.rel_volume >= 2.0
                && hit.change_pct.abs() < 0.5
                && hit.day_pct.abs() < 0.7
        }
        Preset::ChangeButLodNearbySqueeze => {
            hit.change_pct >= 1.0 && hit.lod_dist_pct.abs() < 1.0
        }
        Preset::ChangeButHodNearbySqueeze => {
            hit.change_pct <= -1.0 && hit.hod_dist_pct.abs() < 1.0
        }
        Preset::GapAndCloseAtHodSqueeze => {
            hit.gap_pct >= 0.5
                && hit.hod_dist_pct.abs() < 0.5
                && hit.day_pct >= 0.0
        }
        Preset::GapAndCloseAtLodSqueeze => {
            hit.gap_pct <= -0.5
                && hit.lod_dist_pct.abs() < 0.5
                && hit.day_pct <= 0.0
        }
        Preset::LongInsideQuietSqueeze => {
            let span = hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs();
            span >= 2.0
                && span <= 4.0
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.8
        }
        Preset::TripleZeroSqueeze => {
            hit.gap_pct.abs() < 0.1
                && hit.change_pct.abs() < 0.1
                && hit.day_pct.abs() < 0.1
        }
        Preset::Pct52wQuarterFromHighSqueeze => {
            hit.year_high_pct >= -25.0
                && hit.year_high_pct <= -15.0
                && hit.day_pct.abs() < 0.7
                && hit.rel_volume < 0.9
        }
        Preset::Pct52wQuarterFromLowSqueeze => {
            hit.year_low_pct >= 15.0
                && hit.year_low_pct <= 25.0
                && hit.day_pct.abs() < 0.7
                && hit.rel_volume < 0.9
        }
        Preset::NoExtremeAndQuietSqueeze => {
            hit.year_high_pct <= -5.0
                && hit.year_low_pct >= 5.0
                && hit.rel_volume < 0.7
                && hit.day_pct.abs() < 0.8
        }
        Preset::SmallChangeNarrowGapSqueeze => {
            hit.change_pct >= 0.5
                && hit.change_pct <= 1.0
                && hit.gap_pct.abs() < 0.3
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.9
        }
        Preset::BigRangeNoCommitSqueeze => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) > 6.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 1.5
        }
        Preset::EvenSwingSqueeze => {
            hit.hod_dist_pct.abs() >= 1.0
                && hit.hod_dist_pct.abs() <= 3.0
                && hit.lod_dist_pct.abs() >= 1.0
                && hit.lod_dist_pct.abs() <= 3.0
                && hit.day_pct.abs() < 1.0
        }
        Preset::NoMoveAtMidSqueeze => {
            hit.change_pct.abs() < 0.2
                && hit.hod_dist_pct.abs() > 1.0
                && hit.lod_dist_pct.abs() > 1.0
        }
        Preset::BarelyMovingHighSqueeze => {
            hit.year_high_pct >= -8.0
                && hit.day_pct.abs() < 0.3
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.9
        }
        Preset::BarelyMovingLowSqueeze => {
            hit.year_low_pct <= 8.0
                && hit.day_pct.abs() < 0.3
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.9
        }
        Preset::MicroRangeSqueeze => {
            hit.hod_dist_pct.abs() < 0.2 && hit.lod_dist_pct.abs() < 0.2
        }
        Preset::LowVolGapHoldSqueeze => {
            hit.gap_pct.abs() >= 0.5
                && hit.change_pct.abs() < hit.gap_pct.abs() / 4.0
                && hit.rel_volume < 0.8
        }
        Preset::HighVolGapHoldSqueeze => {
            hit.gap_pct.abs() >= 0.5
                && hit.change_pct.abs() < hit.gap_pct.abs() / 4.0
                && hit.rel_volume >= 1.5
        }
        Preset::UpsideAttemptedSqueeze => {
            hit.hod_dist_pct.abs() >= 1.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < 0.0
        }
        Preset::DownsideAttemptedSqueeze => {
            hit.lod_dist_pct.abs() >= 1.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 0.0
        }
        Preset::TightGapSmallChangeSqueeze => {
            hit.gap_pct.abs() < 0.2
                && hit.change_pct >= -2.0
                && hit.change_pct <= 2.0
                && hit.day_pct.abs() < 0.5
        }
        Preset::Pct52wMidWideRangeSqueeze => {
            hit.year_high_pct >= -10.0
                && hit.year_high_pct <= -5.0
                && hit.year_low_pct >= 5.0
                && hit.year_low_pct <= 10.0
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) > 3.0
        }
        Preset::InsideAndCoiledSqueeze => {
            hit.day_pct.abs() < 0.6
                && hit.hod_dist_pct.abs() < 0.6
                && hit.lod_dist_pct.abs() < 0.6
                && hit.rel_volume < 0.8
        }
        Preset::Pct52wHighBreathSqueeze => {
            hit.year_high_pct >= -1.0
                && hit.day_pct.abs() < 0.4
                && hit.change_pct.abs() < 0.4
        }
        Preset::Pct52wLowBreathSqueeze => {
            hit.year_low_pct <= 1.0
                && hit.day_pct.abs() < 0.4
                && hit.change_pct.abs() < 0.4
        }
        Preset::GapAroundCloseSqueeze => {
            hit.gap_pct.abs() < 0.4
                && hit.change_pct.abs() < 0.4
                && hit.day_pct.abs() < 1.5
                && hit.rel_volume < 1.0
        }
        Preset::TightCloseSplitSqueeze => {
            hit.hod_dist_pct.abs() >= 0.5
                && hit.hod_dist_pct.abs() <= 1.5
                && hit.lod_dist_pct.abs() >= 0.5
                && hit.lod_dist_pct.abs() <= 1.5
                && hit.day_pct.abs() < 0.5
        }
        Preset::HiVolNoExtremeSqueeze => {
            hit.rel_volume >= 2.0
                && hit.hod_dist_pct.abs() > 1.0
                && hit.lod_dist_pct.abs() > 1.0
                && hit.change_pct.abs() < 1.0
        }
        Preset::TinyMoveWithGapSqueeze => {
            hit.gap_pct.abs() >= 0.5
                && hit.gap_pct.abs() <= 1.5
                && hit.change_pct.abs() < 0.5
                && hit.day_pct.abs() < 0.5
        }
        Preset::LowVolatilityGreenSqueeze => {
            hit.change_pct > 0.0
                && hit.change_pct < 1.0
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.5
        }
        Preset::LowVolatilityRedSqueeze => {
            hit.change_pct < 0.0
                && hit.change_pct > -1.0
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.5
        }
        Preset::GapAlignsChangeSqueeze => {
            hit.gap_pct.signum() == hit.change_pct.signum()
                && (hit.gap_pct.abs() + hit.change_pct.abs()) < 1.5
                && hit.day_pct.abs() < 0.5
        }
        Preset::UnaffectedGapSqueeze => {
            hit.gap_pct.abs() >= 0.3
                && hit.gap_pct.abs() <= 1.0
                && hit.day_pct.abs() < 0.3
                && (hit.change_pct - hit.gap_pct).abs() < 0.3
        }
        Preset::StackedClosesSqueeze => {
            hit.hod_dist_pct.abs() < 0.5
                && hit.lod_dist_pct.abs() < 0.5
                && hit.day_pct.abs() < 0.5
                && hit.change_pct.abs() < 1.0
        }
        Preset::PullbackToMidSqueeze => {
            hit.change_pct <= -0.5
                && hit.change_pct >= -2.0
                && hit.hod_dist_pct.abs() > 1.5
                && hit.year_high_pct >= -10.0
        }
        Preset::BounceFromMidSqueeze => {
            hit.change_pct >= 0.5
                && hit.change_pct <= 2.0
                && hit.lod_dist_pct.abs() > 1.5
                && hit.year_low_pct <= 10.0
        }
        Preset::NarrowGapHotCloseSqueeze => {
            hit.gap_pct.abs() < 0.2
                && hit.year_high_pct >= -2.0
                && hit.day_pct.abs() < 0.5
        }
        Preset::NarrowGapColdCloseSqueeze => {
            hit.gap_pct.abs() < 0.2
                && hit.year_low_pct <= 2.0
                && hit.day_pct.abs() < 0.5
        }
        Preset::AbsorptionUpSqueeze => {
            hit.change_pct >= 0.5
                && hit.change_pct <= 2.0
                && hit.rel_volume >= 2.0
                && hit.day_pct.abs() < 1.0
        }
        Preset::AbsorptionDownSqueeze => {
            hit.change_pct <= -0.5
                && hit.change_pct >= -2.0
                && hit.rel_volume >= 2.0
                && hit.day_pct.abs() < 1.0
        }
        Preset::StallAtMidSqueeze => {
            hit.year_high_pct <= -40.0
                && hit.year_high_pct >= -60.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.8
        }
        Preset::NoCloseDecisionSqueeze => {
            hit.hod_dist_pct.abs() >= 0.4
                && hit.hod_dist_pct.abs() <= 0.6
                && hit.lod_dist_pct.abs() >= 0.4
                && hit.lod_dist_pct.abs() <= 0.6
                && hit.day_pct.abs() < 0.3
        }
        Preset::GapInsideRangeSqueeze => {
            hit.gap_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() < 1.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume < 0.8
        }
        Preset::SubpointMoveSqueeze => {
            hit.change_pct.abs() < 0.05 && hit.day_pct.abs() < 0.05
        }
        Preset::NoVolNoMoveSqueeze => {
            hit.rel_volume < 0.3
                && hit.change_pct.abs() < 0.3
                && hit.day_pct.abs() < 0.3
        }
        Preset::VolWithoutChangeSqueeze => {
            hit.rel_volume >= 1.5
                && hit.change_pct.abs() < 0.2
                && hit.day_pct.abs() < 0.5
        }
        Preset::TickInsideOpenSqueeze => {
            hit.day_pct.abs() < 0.15
                && hit.change_pct.abs() < 0.5
                && hit.gap_pct.abs() < 0.3
        }
        Preset::Pct52wExactHalfSqueeze => {
            hit.year_high_pct <= -45.0
                && hit.year_high_pct >= -55.0
                && hit.year_low_pct >= 45.0
                && hit.year_low_pct <= 55.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::UnchangedOnVolumeSqueeze => {
            hit.change_pct.abs() < 0.1 && hit.rel_volume >= 1.0
        }
        Preset::WideHodNarrowLodSqueeze => {
            hit.hod_dist_pct.abs() >= 2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < 0.0
        }
        Preset::NarrowHodWideLodSqueeze => {
            hit.hod_dist_pct.abs() < 0.5
                && hit.lod_dist_pct.abs() >= 2.0
                && hit.change_pct > 0.0
        }
        Preset::PerfectBalanceSqueeze => {
            (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.1
                && hit.change_pct.abs() < 0.3
        }
        Preset::LowVolHotZoneSqueeze => {
            hit.year_high_pct >= -5.0 && hit.rel_volume < 0.4
        }
        Preset::LowVolColdZoneSqueeze => {
            hit.year_low_pct <= 5.0 && hit.rel_volume < 0.4
        }
        Preset::DriftHigherSqueeze => {
            hit.change_pct > 0.0
                && hit.change_pct < 2.0
                && hit.day_pct > 0.0
                && hit.day_pct < 1.0
                && hit.rel_volume < 0.9
        }
        Preset::DriftLowerSqueeze => {
            hit.change_pct < 0.0
                && hit.change_pct > -2.0
                && hit.day_pct < 0.0
                && hit.day_pct > -1.0
                && hit.rel_volume < 0.9
        }
        Preset::ExtremeQuietSqueeze => {
            hit.rel_volume < 0.2
                && hit.change_pct.abs() < 0.5
                && hit.day_pct.abs() < 0.5
                && hit.gap_pct.abs() < 0.2
        }
        Preset::PinnedToOpenSqueeze => {
            hit.day_pct.abs() < 0.05
                && hit.hod_dist_pct.abs() < 1.0
                && hit.lod_dist_pct.abs() < 1.0
        }
        Preset::BigGapSmallDaySqueeze => {
            hit.gap_pct.abs() >= 2.0
                && hit.day_pct.abs() < 0.5
                && hit.change_pct.abs() < 0.5
        }
        Preset::PostCrashSqueeze => {
            hit.change_pct <= -3.0
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 1.0
        }
        Preset::PostSpikeStabilizeSqueeze => {
            hit.change_pct >= 3.0
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 1.0
        }
        Preset::TightWithSmallGapSqueeze => {
            hit.gap_pct.abs() < 0.5
                && hit.change_pct.abs() >= 0.3
                && hit.change_pct.abs() <= 0.8
                && hit.day_pct.abs() < 0.4
        }
        Preset::BigVolWithTinyChangeSqueeze => {
            hit.rel_volume >= 3.0 && hit.change_pct.abs() < 0.1
        }
        Preset::QuietExpansionSqueeze => {
            let span = hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs();
            span >= 2.0
                && span <= 4.0
                && hit.change_pct.abs() < 0.2
                && hit.rel_volume < 0.7
        }
        Preset::InsideBarHighSqueeze => {
            hit.hod_dist_pct.abs() < 1.5
                && hit.lod_dist_pct.abs() < 1.5
                && hit.year_high_pct >= -2.0
        }
        Preset::InsideBarLowSqueeze => {
            hit.hod_dist_pct.abs() < 1.5
                && hit.lod_dist_pct.abs() < 1.5
                && hit.year_low_pct <= 2.0
        }
        Preset::FlatGapInsideRangeSqueeze => {
            hit.gap_pct.abs() < 0.1
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 2.0
        }
        Preset::Pct52wEdgeDryUp => {
            (hit.year_high_pct >= -2.0 || hit.year_low_pct <= 2.0)
                && hit.rel_volume < 0.3
        }
        Preset::NarrowCenterSqueeze => {
            hit.hod_dist_pct.abs() >= 0.5
                && hit.hod_dist_pct.abs() <= 1.0
                && hit.lod_dist_pct.abs() >= 0.5
                && hit.lod_dist_pct.abs() <= 1.0
                && hit.day_pct.abs() < 0.5
        }
        Preset::LopsidedQuietSqueeze => {
            (hit.hod_dist_pct.abs() < 0.5 || hit.lod_dist_pct.abs() < 0.5)
                && hit.rel_volume < 0.5
                && hit.day_pct.abs() < 0.5
        }
        Preset::SilentLeaderSqueeze => {
            hit.year_high_pct >= -3.0
                && hit.year_low_pct >= 50.0
                && hit.day_pct.abs() < 0.5
        }
        Preset::SilentLaggardSqueeze => {
            hit.year_low_pct <= 3.0
                && hit.year_high_pct <= -50.0
                && hit.day_pct.abs() < 0.5
        }
        Preset::NearVwapQuietSqueeze => {
            hit.day_pct.abs() < 0.3
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume < 0.8
        }
        Preset::BarelyMovingMidSqueeze => {
            hit.year_high_pct >= -50.0
                && hit.year_high_pct <= -30.0
                && hit.day_pct.abs() < 0.3
                && hit.rel_volume < 0.8
        }
        Preset::Pct52wThirdFromHighSqueeze => {
            hit.year_high_pct >= -33.0
                && hit.year_high_pct <= -20.0
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.9
        }
        Preset::Pct52wThirdFromLowSqueeze => {
            hit.year_low_pct >= 20.0
                && hit.year_low_pct <= 33.0
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.9
        }
        Preset::HighRangeNoChangeSqueeze => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) > 5.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 1.0
        }
        Preset::LowRangeNoChangeSqueeze => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 1.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::LowVolumeUpDaySqueeze => {
            hit.change_pct > 1.0
                && hit.change_pct < 3.0
                && hit.rel_volume < 0.5
        }
        Preset::LowVolumeDownDaySqueeze => {
            hit.change_pct < -1.0
                && hit.change_pct > -3.0
                && hit.rel_volume < 0.5
        }
        Preset::HighVolumeUpDayNoExtreme => {
            hit.change_pct >= 1.0
                && hit.change_pct <= 2.0
                && hit.rel_volume >= 2.0
                && hit.hod_dist_pct.abs() > 0.5
        }
        Preset::HighVolumeDownDayNoExtreme => {
            hit.change_pct <= -1.0
                && hit.change_pct >= -2.0
                && hit.rel_volume >= 2.0
                && hit.lod_dist_pct.abs() > 0.5
        }
        Preset::GapUpFadeToFlat => {
            hit.gap_pct > 2.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::GapDownReclaimToFlat => {
            hit.gap_pct < -2.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::GapUpHeldGreen => {
            hit.gap_pct > 2.0
                && hit.change_pct > hit.gap_pct
                && hit.rel_volume >= 1.0
        }
        Preset::GapDownHeldRed => {
            hit.gap_pct < -2.0
                && hit.change_pct < hit.gap_pct
                && hit.rel_volume >= 1.0
        }
        Preset::GapUpHalfFade => {
            hit.gap_pct > 2.0
                && hit.change_pct > 0.0
                && hit.change_pct < hit.gap_pct * 0.5
        }
        Preset::GapDownHalfReclaim => {
            hit.gap_pct < -2.0
                && hit.change_pct < 0.0
                && hit.change_pct > hit.gap_pct * 0.5
        }
        Preset::GapAndGoXl => {
            hit.gap_pct > 3.0
                && hit.change_pct > 5.0
                && hit.rel_volume >= 2.0
        }
        Preset::GapAndCrashXl => {
            hit.gap_pct < -3.0
                && hit.change_pct < -5.0
                && hit.rel_volume >= 2.0
        }
        Preset::GapUpButDayRed => {
            hit.gap_pct > 1.0 && hit.change_pct < -1.0
        }
        Preset::GapDownButDayGreen => {
            hit.gap_pct < -1.0 && hit.change_pct > 1.0
        }
        Preset::GapUpFlushOnVolume => {
            hit.gap_pct > 2.0
                && hit.change_pct < -2.0
                && hit.rel_volume >= 2.0
        }
        Preset::GapDownReversalOnVolume => {
            hit.gap_pct < -2.0
                && hit.change_pct > 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::Pct52wTopDecileHotVol => {
            hit.year_high_pct > -10.0 && hit.rel_volume >= 2.0
        }
        Preset::Pct52wBottomDecileHotVol => {
            hit.year_low_pct < 10.0 && hit.rel_volume >= 2.0
        }
        Preset::Pct52wTopDecileDryVol => {
            hit.year_high_pct > -10.0 && hit.rel_volume < 0.5
        }
        Preset::Pct52wBottomDecileDryVol => {
            hit.year_low_pct < 10.0 && hit.rel_volume < 0.5
        }
        Preset::NewHighGreenDay => {
            hit.year_high_pct >= 0.0 && hit.change_pct >= 1.0
        }
        Preset::NewLowRedDay => {
            hit.year_low_pct <= 0.0 && hit.change_pct <= -1.0
        }
        Preset::NewHighRedDay => {
            hit.year_high_pct >= 0.0 && hit.change_pct <= -1.0
        }
        Preset::NewLowGreenDay => {
            hit.year_low_pct <= 0.0 && hit.change_pct >= 1.0
        }
        Preset::NewHighOnHotVol => {
            hit.year_high_pct >= 0.0 && hit.rel_volume >= 3.0
        }
        Preset::NewLowOnHotVol => {
            hit.year_low_pct <= 0.0 && hit.rel_volume >= 3.0
        }
        Preset::QuietNearTheTop => {
            hit.year_high_pct > -3.0
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 1.5
                && hit.rel_volume < 1.0
        }
        Preset::QuietNearTheBottom => {
            hit.year_low_pct < 3.0
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 1.5
                && hit.rel_volume < 1.0
        }
        Preset::NoisyNearTheTop => {
            hit.year_high_pct > -3.0
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) > 4.0
                && hit.rel_volume >= 2.0
        }
        Preset::NoisyNearTheBottom => {
            hit.year_low_pct < 3.0
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) > 4.0
                && hit.rel_volume >= 2.0
        }
        Preset::MidRangeChopHotVol => {
            let h = hit.hod_dist_pct.abs();
            let l = hit.lod_dist_pct.abs();
            h >= 1.0 && h <= 3.0
                && l >= 1.0 && l <= 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::MidRangeChopDryVol => {
            let h = hit.hod_dist_pct.abs();
            let l = hit.lod_dist_pct.abs();
            h >= 1.0 && h <= 3.0
                && l >= 1.0 && l <= 3.0
                && hit.rel_volume < 0.5
        }
        Preset::CloseNearHodNoBreakout => {
            hit.hod_dist_pct.abs() < 0.5 && hit.change_pct < 1.0
        }
        Preset::CloseNearLodNoBreakdown => {
            hit.lod_dist_pct.abs() < 0.5 && hit.change_pct > -1.0
        }
        Preset::CloseNearHodStrongDay => {
            hit.hod_dist_pct.abs() < 0.5 && hit.change_pct > 3.0
        }
        Preset::CloseNearLodWeakDay => {
            hit.lod_dist_pct.abs() < 0.5 && hit.change_pct < -3.0
        }
        Preset::InsideRangeNoVolume => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 2.0
                && hit.rel_volume < 0.5
        }
        Preset::OutsideRangeOnVolume => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) > 6.0
                && hit.rel_volume >= 2.0
        }
        Preset::UpDayLowerHigh => {
            hit.change_pct > 1.0
                && hit.hod_dist_pct.abs() > 1.0
                && hit.lod_dist_pct.abs() < 1.0
        }
        Preset::DownDayHigherLow => {
            hit.change_pct < -1.0
                && hit.lod_dist_pct.abs() > 1.0
                && hit.hod_dist_pct.abs() < 1.0
        }
        Preset::StrongDayBalancedRange => {
            hit.change_pct > 3.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.lod_dist_pct.abs() < 1.0
        }
        Preset::WeakDayBalancedRange => {
            hit.change_pct < -3.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.lod_dist_pct.abs() < 1.0
        }
        Preset::ChannelRideUp => {
            hit.change_pct > 1.0
                && hit.day_pct > 0.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.lod_dist_pct.abs() > 3.0
        }
        Preset::ChannelRideDown => {
            hit.change_pct < -1.0
                && hit.day_pct < 0.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() > 3.0
        }
        Preset::PullbackInUptrend => {
            hit.year_high_pct > -15.0
                && hit.change_pct < 0.0
                && hit.change_pct > -3.0
                && hit.rel_volume < 1.0
        }
        Preset::BounceInDowntrend => {
            hit.year_low_pct < 15.0
                && hit.change_pct > 0.0
                && hit.change_pct < 3.0
                && hit.rel_volume < 1.0
        }
        Preset::DeepPullbackInUptrend => {
            hit.year_high_pct > -25.0
                && hit.change_pct < -3.0
                && hit.change_pct > -10.0
                && hit.rel_volume >= 2.0
        }
        Preset::DeepBounceInDowntrend => {
            hit.year_low_pct < 25.0
                && hit.change_pct > 3.0
                && hit.change_pct < 10.0
                && hit.rel_volume >= 2.0
        }
        Preset::TightAboveMidStrong => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 1.5
                && hit.day_pct > 0.0
                && hit.change_pct > 0.5
        }
        Preset::TightBelowMidWeak => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 1.5
                && hit.day_pct < 0.0
                && hit.change_pct < -0.5
        }
        Preset::HotVolNoMoveAtHigh => {
            hit.year_high_pct > -5.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::HotVolNoMoveAtLow => {
            hit.year_low_pct < 5.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::BigUpGapInsideDay => {
            hit.gap_pct > 3.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct >= 1.0
                && hit.change_pct <= 3.0
        }
        Preset::BigDownGapInsideDay => {
            hit.gap_pct < -3.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct <= -1.0
                && hit.change_pct >= -3.0
        }
        Preset::SteadyUpDryVol => {
            hit.change_pct >= 0.5
                && hit.change_pct <= 2.0
                && hit.day_pct > 0.0
                && hit.rel_volume < 0.7
        }
        Preset::SteadyDownDryVol => {
            hit.change_pct <= -0.5
                && hit.change_pct >= -2.0
                && hit.day_pct < 0.0
                && hit.rel_volume < 0.7
        }
        Preset::ImpulsiveUpHotVol => {
            hit.change_pct >= 2.0
                && hit.change_pct <= 5.0
                && hit.day_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::ImpulsiveDownHotVol => {
            hit.change_pct <= -2.0
                && hit.change_pct >= -5.0
                && hit.day_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::ParabolicUp => {
            hit.change_pct > 10.0
                && hit.rel_volume >= 3.0
                && hit.hod_dist_pct.abs() < 0.5
        }
        Preset::ParabolicDown => {
            hit.change_pct < -10.0
                && hit.rel_volume >= 3.0
                && hit.lod_dist_pct.abs() < 0.5
        }
        Preset::BlowOffTop => {
            hit.change_pct > 5.0
                && hit.rel_volume >= 5.0
                && hit.year_high_pct > -2.0
        }
        Preset::SellingClimaxBottom => {
            hit.change_pct < -5.0
                && hit.rel_volume >= 5.0
                && hit.year_low_pct < 2.0
        }
        Preset::UpDayGapOnlyMove => {
            hit.change_pct >= 1.0
                && hit.change_pct <= 3.0
                && (hit.change_pct - hit.gap_pct).abs() < 0.3
                && hit.rel_volume < 1.0
        }
        Preset::DownDayGapOnlyMove => {
            hit.change_pct <= -1.0
                && hit.change_pct >= -3.0
                && (hit.change_pct - hit.gap_pct).abs() < 0.3
                && hit.rel_volume < 1.0
        }
        Preset::IntradayOnlyGreenDay => {
            hit.change_pct > 1.0
                && hit.gap_pct.abs() < 0.3
                && hit.rel_volume >= 1.0
        }
        Preset::IntradayOnlyRedDay => {
            hit.change_pct < -1.0
                && hit.gap_pct.abs() < 0.3
                && hit.rel_volume >= 1.0
        }
        Preset::ReversalUpFromOpen => {
            hit.gap_pct < -1.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
                && hit.hod_dist_pct.abs() < 0.5
        }
        Preset::ReversalDownFromOpen => {
            hit.gap_pct > 1.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
                && hit.lod_dist_pct.abs() < 0.5
        }
        Preset::TrendDayUp => {
            hit.change_pct > 2.0
                && hit.day_pct > 1.0
                && hit.rel_volume >= 1.2
                && hit.hod_dist_pct.abs() < 0.5
                && hit.lod_dist_pct.abs() > 2.0
        }
        Preset::TrendDayDown => {
            hit.change_pct < -2.0
                && hit.day_pct < -1.0
                && hit.rel_volume >= 1.2
                && hit.lod_dist_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() > 2.0
        }
        Preset::DoubleBottomCandidate => {
            hit.year_low_pct < 5.0
                && hit.hod_dist_pct.abs() > 1.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct > 0.0
        }
        Preset::DoubleTopCandidate => {
            hit.year_high_pct > -5.0
                && hit.lod_dist_pct.abs() > 1.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct < 0.0
        }
        Preset::Pct52wMidZone => {
            hit.year_high_pct < -40.0
                && hit.year_high_pct > -60.0
                && hit.year_low_pct > 40.0
                && hit.year_low_pct < 60.0
        }
        Preset::Pct52wRangeBreakoutTriggered => {
            hit.year_high_pct >= 0.0
                && hit.change_pct > 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::Pct52wRangeBreakdownTriggered => {
            hit.year_low_pct <= 0.0
                && hit.change_pct < -2.0
                && hit.rel_volume >= 2.0
        }
        Preset::Pct52wTightCoil => {
            hit.year_high_pct >= -10.0
                && hit.year_high_pct <= -5.0
                && hit.year_low_pct >= 5.0
                && hit.year_low_pct <= 10.0
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 2.0
        }
        Preset::SymmetricTriangle => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 3.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 0.7
                && hit.rel_volume <= 1.3
        }
        Preset::NarrowingRangeOnFlat => {
            let h = hit.hod_dist_pct.abs();
            let l = hit.lod_dist_pct.abs();
            h >= 0.5 && h <= 2.0
                && l >= 0.5 && l <= 2.0
                && hit.change_pct.abs() < 0.3
        }
        Preset::GapTooFarBigPullback => {
            hit.gap_pct > 4.0
                && hit.change_pct < hit.gap_pct - 3.0
        }
        Preset::GapTooFarBigBounce => {
            hit.gap_pct < -4.0
                && hit.change_pct > hit.gap_pct + 3.0
        }
        Preset::ChainBreakoutLevel => {
            hit.hod_dist_pct.abs() < 0.3
                && hit.lod_dist_pct.abs() > 2.0
                && hit.change_pct > 1.0
        }
        Preset::ChainBreakdownLevel => {
            hit.lod_dist_pct.abs() < 0.3
                && hit.hod_dist_pct.abs() > 2.0
                && hit.change_pct < -1.0
        }
        Preset::Pct52wRangePosTop => {
            hit.year_high_pct > -20.0 && hit.year_low_pct > 30.0
        }
        Preset::Pct52wRangePosBottom => {
            hit.year_high_pct < -50.0 && hit.year_low_pct < 30.0
        }
        Preset::HighRangeHighVolStrong => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) > 4.0
                && hit.change_pct > 3.0
                && hit.rel_volume >= 1.5
        }
        Preset::HighRangeHighVolWeak => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) > 4.0
                && hit.change_pct < -3.0
                && hit.rel_volume >= 1.5
        }
        Preset::LowRangeLowVolNeutral => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 1.5
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.7
        }
        Preset::AvgRangeAvgVolNeutral => {
            let r = hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs();
            r >= 2.0 && r <= 4.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 0.8 && hit.rel_volume <= 1.2
        }
        Preset::FailedBreakoutHighReclaim => {
            hit.year_high_pct > -1.0
                && hit.hod_dist_pct.abs() > 1.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::FailedBreakdownLowReclaim => {
            hit.year_low_pct < 1.0
                && hit.lod_dist_pct.abs() > 1.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::HotVolHotGap => {
            hit.gap_pct.abs() > 2.0 && hit.rel_volume >= 2.0
        }
        Preset::DryVolDryGap => {
            hit.gap_pct.abs() < 0.5 && hit.rel_volume < 0.5
        }
        Preset::OuterEdgePushUp => {
            hit.year_high_pct > -10.0
                && hit.change_pct > 5.0
                && hit.rel_volume >= 2.0
        }
        Preset::OuterEdgePushDown => {
            hit.year_low_pct < 10.0
                && hit.change_pct < -5.0
                && hit.rel_volume >= 2.0
        }
        Preset::MiddleZoneUpDrift => {
            hit.year_high_pct >= -50.0
                && hit.year_high_pct <= -20.0
                && hit.year_low_pct >= 20.0
                && hit.year_low_pct <= 50.0
                && hit.change_pct > 0.5
                && hit.rel_volume < 1.0
        }
        Preset::MiddleZoneDownDrift => {
            hit.year_high_pct >= -50.0
                && hit.year_high_pct <= -20.0
                && hit.year_low_pct >= 20.0
                && hit.year_low_pct <= 50.0
                && hit.change_pct < -0.5
                && hit.rel_volume < 1.0
        }
        Preset::MiddleZoneHotVolBreakoutHigh => {
            hit.year_high_pct >= -50.0
                && hit.year_high_pct <= -20.0
                && hit.change_pct > 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::MiddleZoneHotVolBreakoutLow => {
            hit.year_low_pct >= 20.0
                && hit.year_low_pct <= 50.0
                && hit.change_pct < -2.0
                && hit.rel_volume >= 2.0
        }
        Preset::GapUpSmallButHotVol => {
            hit.gap_pct >= 0.5 && hit.gap_pct <= 1.5
                && hit.rel_volume >= 2.0
        }
        Preset::GapDownSmallButHotVol => {
            hit.gap_pct <= -0.5 && hit.gap_pct >= -1.5
                && hit.rel_volume >= 2.0
        }
        Preset::GapUpMediumNeutral => {
            hit.gap_pct >= 1.5 && hit.gap_pct <= 3.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 1.0
        }
        Preset::GapDownMediumNeutral => {
            hit.gap_pct <= -1.5 && hit.gap_pct >= -3.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 1.0
        }
        Preset::HodReclaimAfterFlush => {
            hit.change_pct > 0.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.lod_dist_pct.abs() > 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::LodFailAfterPush => {
            hit.change_pct < 0.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() > 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::HodReclaimFromFlatGap => {
            hit.gap_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 1.0
        }
        Preset::LodFailFromFlatGap => {
            hit.gap_pct.abs() < 0.5
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -1.0
        }
        Preset::Pct52wTopBoundaryReject => {
            hit.year_high_pct >= -1.0
                && hit.year_high_pct <= 0.0
                && hit.change_pct < -0.5
        }
        Preset::Pct52wBottomBoundaryReject => {
            hit.year_low_pct >= 0.0
                && hit.year_low_pct <= 1.0
                && hit.change_pct > 0.5
        }
        Preset::Pct52wTopBoundaryAccept => {
            hit.year_high_pct >= -1.0
                && hit.year_high_pct <= 0.0
                && hit.change_pct > 0.5
        }
        Preset::Pct52wBottomBoundaryAccept => {
            hit.year_low_pct >= 0.0
                && hit.year_low_pct <= 1.0
                && hit.change_pct < -0.5
        }
        Preset::UpFromBottomSpring => {
            hit.year_low_pct < 10.0
                && hit.change_pct > 5.0
                && hit.rel_volume >= 2.0
        }
        Preset::DownFromTopUpthrust => {
            hit.year_high_pct > -10.0
                && hit.change_pct < -5.0
                && hit.rel_volume >= 2.0
        }
        Preset::UpThrustBarReject => {
            hit.hod_dist_pct.abs() > 3.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -1.0
                && hit.rel_volume >= 2.0
        }
        Preset::DownThrustBarReject => {
            hit.lod_dist_pct.abs() > 3.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::ExhaustionTopWideRange => {
            hit.year_high_pct > -5.0
                && hit.hod_dist_pct.abs() > 5.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct > 5.0
                && hit.rel_volume >= 3.0
        }
        Preset::ExhaustionBottomWideRange => {
            hit.year_low_pct < 5.0
                && hit.lod_dist_pct.abs() > 5.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct < -5.0
                && hit.rel_volume >= 3.0
        }
        Preset::UpTrendDayWideRange => {
            hit.hod_dist_pct.abs() < 0.3
                && hit.lod_dist_pct.abs() > 5.0
                && hit.change_pct > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::DownTrendDayWideRange => {
            hit.lod_dist_pct.abs() < 0.3
                && hit.hod_dist_pct.abs() > 5.0
                && hit.change_pct < -3.0
                && hit.rel_volume >= 2.0
        }
        Preset::SilentSpringNear52wLow => {
            hit.year_low_pct < 5.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.5
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 1.5
        }
        Preset::SilentUpThrustNear52wHigh => {
            hit.year_high_pct > -5.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.5
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 1.5
        }
        Preset::GapStrongDayOpenPivot => {
            hit.gap_pct >= 1.0 && hit.gap_pct <= 3.0
                && hit.change_pct > 4.0
                && hit.rel_volume >= 2.0
        }
        Preset::GapWeakDayOpenPivot => {
            hit.gap_pct <= -1.0 && hit.gap_pct >= -3.0
                && hit.change_pct < -4.0
                && hit.rel_volume >= 2.0
        }
        Preset::ConvictionBreakoutCombo => {
            hit.year_high_pct >= 0.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 3.0
                && hit.rel_volume >= 2.5
        }
        Preset::ConvictionBreakdownCombo => {
            hit.year_low_pct <= 0.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -3.0
                && hit.rel_volume >= 2.5
        }
        Preset::PullbackInsideTrendUp => {
            hit.year_high_pct >= -20.0
                && hit.year_high_pct <= -5.0
                && hit.change_pct >= -1.0
                && hit.change_pct <= -0.2
                && hit.rel_volume >= 0.7
                && hit.rel_volume <= 1.3
        }
        Preset::PullbackInsideTrendDown => {
            hit.year_low_pct >= 5.0
                && hit.year_low_pct <= 20.0
                && hit.change_pct >= 0.2
                && hit.change_pct <= 1.0
                && hit.rel_volume >= 0.7
                && hit.rel_volume <= 1.3
        }
        Preset::RangeContractionSqueezeHigh => {
            hit.year_high_pct > -5.0
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 1.0
                && hit.rel_volume < 0.5
                && hit.change_pct.abs() < 0.3
        }
        Preset::RangeContractionSqueezeLow => {
            hit.year_low_pct < 5.0
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 1.0
                && hit.rel_volume < 0.5
                && hit.change_pct.abs() < 0.3
        }
        Preset::RangeExpansionAtTopOnVol => {
            hit.year_high_pct > -5.0
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) > 6.0
                && hit.rel_volume >= 2.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::RangeExpansionAtBottomOnVol => {
            hit.year_low_pct < 5.0
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) > 6.0
                && hit.rel_volume >= 2.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::GapInsideRangeBalanced => {
            hit.gap_pct.abs() < 1.0
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 2.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::GapInsideRangeImpulse => {
            hit.gap_pct.abs() < 1.0
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) > 4.0
                && hit.change_pct.abs() > 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::OneWickCloseAtMid => {
            hit.hod_dist_pct.abs() > 2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct.abs() < 0.5
        }
        Preset::OneWickCloseAtMidDown => {
            hit.lod_dist_pct.abs() > 2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct.abs() < 0.5
        }
        Preset::UpperWickGreenDayConfirm => {
            hit.hod_dist_pct.abs() > 2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct > 1.0
        }
        Preset::LowerWickRedDayConfirm => {
            hit.lod_dist_pct.abs() > 2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct < -1.0
        }
        Preset::InsideBarTightAtMid => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 1.0
                && hit.change_pct.abs() < 0.2
                && hit.rel_volume < 0.8
        }
        Preset::OutsideBarVolumeBoth => {
            hit.hod_dist_pct.abs() > 3.0
                && hit.lod_dist_pct.abs() > 3.0
                && hit.rel_volume >= 2.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::LeadingUpDayLightVol => {
            hit.change_pct > 2.0
                && hit.rel_volume < 0.7
                && hit.hod_dist_pct.abs() < 0.5
        }
        Preset::LeadingDownDayLightVol => {
            hit.change_pct < -2.0
                && hit.rel_volume < 0.7
                && hit.lod_dist_pct.abs() < 0.5
        }
        Preset::SmallChangeOnVolNearHigh => {
            hit.year_high_pct > -3.0
                && hit.change_pct.abs() >= 0.5
                && hit.change_pct.abs() <= 1.5
                && hit.rel_volume >= 1.5
        }
        Preset::SmallChangeOnVolNearLow => {
            hit.year_low_pct < 3.0
                && hit.change_pct.abs() >= 0.5
                && hit.change_pct.abs() <= 1.5
                && hit.rel_volume >= 1.5
        }
        Preset::BigGapBigVolBigDay => {
            hit.gap_pct.abs() > 3.0
                && hit.change_pct.abs() > 5.0
                && hit.rel_volume >= 3.0
        }
        Preset::BigGapNoFollowThrough => {
            hit.gap_pct.abs() > 3.0
                && hit.change_pct.abs() < 1.0
                && hit.rel_volume < 1.0
        }
        Preset::ConfluenceLongSetup => {
            hit.gap_pct.abs() < 0.5
                && hit.year_low_pct >= 5.0
                && hit.year_low_pct <= 15.0
                && hit.change_pct >= 0.5
                && hit.change_pct <= 1.5
                && hit.rel_volume >= 1.2
        }
        Preset::ConfluenceShortSetup => {
            hit.gap_pct.abs() < 0.5
                && hit.year_high_pct >= -15.0
                && hit.year_high_pct <= -5.0
                && hit.change_pct <= -0.5
                && hit.change_pct >= -1.5
                && hit.rel_volume >= 1.2
        }
        Preset::NoExtremeDay => {
            hit.year_high_pct < -10.0
                && hit.year_high_pct > -40.0
                && hit.year_low_pct > 10.0
                && hit.year_low_pct < 40.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::AcceleratingUpTrend => {
            hit.change_pct > 1.0
                && hit.day_pct > 0.0
                && hit.year_high_pct > -5.0
                && hit.rel_volume > 1.0
        }
        Preset::AcceleratingDownTrend => {
            hit.change_pct < -1.0
                && hit.day_pct < 0.0
                && hit.year_low_pct < 5.0
                && hit.rel_volume > 1.0
        }
        Preset::DivergencePushFromTop => {
            hit.year_high_pct > -3.0
                && hit.change_pct < -1.0
                && hit.rel_volume < 0.8
        }
        Preset::DivergencePushFromBottom => {
            hit.year_low_pct < 3.0
                && hit.change_pct > 1.0
                && hit.rel_volume < 0.8
        }
        Preset::PriceFlatVolHotAboveMid => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 2.0
                && hit.day_pct > 0.0
                && hit.rel_volume >= 1.5
                && hit.change_pct.abs() < 0.3
        }
        Preset::PriceFlatVolHotBelowMid => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 2.0
                && hit.day_pct < 0.0
                && hit.rel_volume >= 1.5
                && hit.change_pct.abs() < 0.3
        }
        Preset::SmallChangeOnVolMid => {
            hit.year_high_pct >= -50.0
                && hit.year_high_pct <= -20.0
                && hit.year_low_pct >= 20.0
                && hit.year_low_pct <= 50.0
                && hit.change_pct.abs() >= 0.5
                && hit.change_pct.abs() <= 1.5
                && hit.rel_volume >= 1.5
        }
        Preset::HotRollingVolGap => {
            hit.gap_pct.abs() > 1.5
                && hit.change_pct.abs() > 1.5
                && hit.rel_volume >= 2.0
        }
        Preset::SilentDriftGap => {
            hit.gap_pct.abs() > 1.0
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume < 0.7
        }
        Preset::UpDayOnDryVolNear52wHigh => {
            hit.year_high_pct > -10.0
                && hit.change_pct > 1.0
                && hit.rel_volume < 0.7
        }
        Preset::DownDayOnDryVolNear52wLow => {
            hit.year_low_pct < 10.0
                && hit.change_pct < -1.0
                && hit.rel_volume < 0.7
        }
        Preset::UpDayOnHotVolNear52wHigh => {
            hit.year_high_pct > -10.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::DownDayOnHotVolNear52wLow => {
            hit.year_low_pct < 10.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 2.0
        }
        Preset::NarrowDayDryVolMid => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 1.5
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume < 0.5
                && hit.year_high_pct >= -30.0
                && hit.year_high_pct <= -15.0
                && hit.year_low_pct >= 15.0
                && hit.year_low_pct <= 30.0
        }
        Preset::WideDayHotVolMid => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) > 5.0
                && hit.rel_volume >= 2.0
                && hit.year_high_pct >= -30.0
                && hit.year_high_pct <= -15.0
                && hit.year_low_pct >= 15.0
                && hit.year_low_pct <= 30.0
        }
        Preset::HotVolAtMidNoMove => {
            hit.year_high_pct >= -50.0
                && hit.year_high_pct <= -20.0
                && hit.year_low_pct >= 20.0
                && hit.year_low_pct <= 50.0
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume >= 1.5
        }
        Preset::DryVolAtMidNoMove => {
            hit.year_high_pct >= -50.0
                && hit.year_high_pct <= -20.0
                && hit.year_low_pct >= 20.0
                && hit.year_low_pct <= 50.0
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume < 0.5
        }
        Preset::BigChangeTinyRangeUp => {
            hit.change_pct > 2.0
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::BigChangeTinyRangeDown => {
            hit.change_pct < -2.0
                && (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::TinyChangeWideRangeOnVol => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) > 5.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::TinyChangeWideRangeOnDryVol => {
            (hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()) > 5.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.7
        }
        Preset::LargeGapModerateMoveHotVol => {
            hit.gap_pct.abs() > 3.0
                && hit.change_pct.abs() >= 1.5
                && hit.change_pct.abs() <= 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::SmallGapBigMoveHotVol => {
            hit.gap_pct.abs() < 0.5
                && hit.change_pct.abs() > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::NoVolTrendUp => {
            hit.change_pct > 1.0
                && hit.day_pct > 0.0
                && hit.rel_volume < 0.4
                && hit.hod_dist_pct.abs() < 0.5
        }
        Preset::NoVolTrendDown => {
            hit.change_pct < -1.0
                && hit.day_pct < 0.0
                && hit.rel_volume < 0.4
                && hit.lod_dist_pct.abs() < 0.5
        }
        Preset::ChurnAtTopDryVol => {
            hit.year_high_pct > -3.0
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume < 0.6
        }
        Preset::ChurnAtBottomDryVol => {
            hit.year_low_pct < 3.0
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume < 0.6
        }
        Preset::HugeGapFlatChange => {
            hit.gap_pct.abs() > 5.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 1.0
        }
        Preset::NoGapHugeChange => {
            hit.gap_pct.abs() < 0.3
                && hit.change_pct.abs() > 5.0
                && hit.rel_volume >= 1.5
        }
        Preset::ExtremeVolFlatGapFlatDay => {
            hit.rel_volume >= 3.0
                && hit.gap_pct.abs() < 0.3
                && hit.change_pct.abs() < 1.0
        }
        Preset::IlliquidBigGapFlatDay => {
            hit.rel_volume < 0.4
                && hit.gap_pct.abs() > 3.0
                && hit.change_pct.abs() < 1.0
        }
        Preset::OrganicUpDayCloseAtHod => {
            hit.gap_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() < 0.3
                && hit.day_pct > 2.0
                && hit.rel_volume >= 0.7
                && hit.rel_volume <= 1.3
        }
        Preset::OrganicDownDayCloseAtLod => {
            hit.gap_pct.abs() < 0.5
                && hit.lod_dist_pct.abs() < 0.3
                && hit.day_pct < -2.0
                && hit.rel_volume >= 0.7
                && hit.rel_volume <= 1.3
        }
        Preset::StrongDayDryVolUp => {
            hit.change_pct > 3.0
                && hit.day_pct > 2.0
                && hit.rel_volume < 0.5
        }
        Preset::StrongDayDryVolDown => {
            hit.change_pct < -3.0
                && hit.day_pct < -2.0
                && hit.rel_volume < 0.5
        }
        Preset::TightCoilAtMidRange => {
            hit.hod_dist_pct.abs() >= 0.5
                && hit.hod_dist_pct.abs() <= 1.5
                && hit.lod_dist_pct.abs() >= 0.5
                && hit.lod_dist_pct.abs() <= 1.5
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.7
        }
        Preset::WideOutsideRangeDryVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 6.0
                && hit.rel_volume < 0.6
        }
        Preset::GapHeldAndExtendedUp => {
            hit.gap_pct > 1.0
                && hit.day_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapHeldAndExtendedDown => {
            hit.gap_pct < -1.0
                && hit.day_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::Pct52wHighBreakoutCloseAtHod => {
            hit.year_high_pct > 0.0
                && hit.day_pct > 1.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::Pct52wLowBreakdownCloseAtLod => {
            hit.year_low_pct < 0.0
                && hit.day_pct < -1.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::Pct52wMidHotVolFlat => {
            hit.year_high_pct >= -55.0
                && hit.year_high_pct <= -35.0
                && hit.year_low_pct >= 35.0
                && hit.year_low_pct <= 55.0
                && hit.change_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::Pct52wMidDryVolFlat => {
            hit.year_high_pct >= -55.0
                && hit.year_high_pct <= -35.0
                && hit.year_low_pct >= 35.0
                && hit.year_low_pct <= 55.0
                && hit.change_pct.abs() < 1.0
                && hit.rel_volume < 0.5
        }
        Preset::VolSpikeNoTrend => {
            hit.rel_volume >= 5.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::VolSpikeOnTrend => {
            hit.rel_volume >= 5.0
                && hit.change_pct.abs() > 3.0
        }
        Preset::TightCoilAtHighDryVol => {
            hit.hod_dist_pct.abs() < 0.3
                && hit.lod_dist_pct.abs() < 1.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.7
                && hit.year_high_pct > -5.0
        }
        Preset::TightCoilAtLowDryVol => {
            hit.lod_dist_pct.abs() < 0.3
                && hit.hod_dist_pct.abs() < 1.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.7
                && hit.year_low_pct < 5.0
        }
        Preset::OrderlyTrendAtHighs => {
            hit.change_pct > 0.0
                && hit.day_pct > 0.0
                && hit.year_high_pct > -1.0
                && hit.rel_volume >= 0.7
                && hit.rel_volume <= 1.5
        }
        Preset::OrderlyTrendAtLows => {
            hit.change_pct < 0.0
                && hit.day_pct < 0.0
                && hit.year_low_pct < 1.0
                && hit.rel_volume >= 0.7
                && hit.rel_volume <= 1.5
        }
        Preset::HotVolMidRangeChurn => {
            hit.rel_volume >= 3.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
        }
        Preset::DryVolAtExtremeClose => {
            hit.rel_volume < 0.4
                && (hit.hod_dist_pct.abs() < 0.3 || hit.lod_dist_pct.abs() < 0.3)
        }
        Preset::DayChangeMismatch => {
            hit.change_pct * hit.day_pct < 0.0
                && hit.change_pct.abs() > 1.0
                && hit.day_pct.abs() > 1.0
        }
        Preset::DayChangeAlignedBig => {
            hit.change_pct * hit.day_pct > 0.0
                && hit.change_pct.abs() > 3.0
                && hit.day_pct.abs() > 3.0
                && hit.rel_volume >= 1.5
        }
        Preset::HugeRangeHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 3.0
        }
        Preset::HugeRangeDryVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume < 0.5
        }
        Preset::Pct52wLowHotVolUp => {
            hit.year_low_pct < 10.0
                && hit.change_pct > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::Pct52wHighHotVolDown => {
            hit.year_high_pct > -10.0
                && hit.change_pct < -3.0
                && hit.rel_volume >= 2.0
        }
        Preset::GapHeldNoExtension => {
            hit.gap_pct.abs() > 1.0
                && hit.day_pct.abs() < 0.3
                && hit.rel_volume >= 0.7
                && hit.rel_volume <= 1.5
        }
        Preset::GapPartialFade => {
            hit.gap_pct.abs() > 2.0
                && hit.change_pct * hit.gap_pct > 0.0
                && hit.change_pct.abs() < hit.gap_pct.abs() / 2.0
                && hit.rel_volume >= 1.2
        }
        Preset::YearHighIntradayWeak => {
            hit.year_high_pct > -1.0
                && hit.day_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::YearLowIntradayStrong => {
            hit.year_low_pct < 1.0
                && hit.day_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::WeakHandsAtHighs => {
            hit.year_high_pct > -2.0
                && hit.change_pct < -0.5
                && hit.day_pct < -0.3
                && hit.rel_volume >= 1.0
                && hit.rel_volume <= 2.0
        }
        Preset::StrongHandsAtLows => {
            hit.year_low_pct < 2.0
                && hit.change_pct > 0.5
                && hit.day_pct > 0.3
                && hit.rel_volume >= 1.0
                && hit.rel_volume <= 2.0
        }
        Preset::NarrowRangeHotVolSqueeze => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 3.0
        }
        Preset::WideRangeDryVolDrift => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.rel_volume < 0.5
                && hit.change_pct.abs() < 1.0
        }
        Preset::LeadershipTrendDay => {
            hit.year_high_pct > -5.0
                && hit.change_pct > 2.0
                && hit.day_pct > 1.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::WorstActorFlushDay => {
            hit.year_low_pct < 5.0
                && hit.change_pct < -2.0
                && hit.day_pct < -1.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpAtYearLow => {
            hit.gap_pct > 2.0
                && hit.year_low_pct < 5.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownAtYearHigh => {
            hit.gap_pct < -2.0
                && hit.year_high_pct > -5.0
                && hit.rel_volume >= 1.5
        }
        Preset::BigUpMidRangeClose => {
            hit.change_pct > 3.0
                && hit.hod_dist_pct.abs() > 1.5
                && hit.lod_dist_pct.abs() > 1.5
                && hit.rel_volume >= 1.5
        }
        Preset::BigDownMidRangeClose => {
            hit.change_pct < -3.0
                && hit.hod_dist_pct.abs() > 1.5
                && hit.lod_dist_pct.abs() > 1.5
                && hit.rel_volume >= 1.5
        }
        Preset::HodCloseHotVolFlat => {
            hit.hod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::LodCloseHotVolFlat => {
            hit.lod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::RisingWedgeCoil => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 2.0
                && hit.change_pct > 0.0
                && hit.day_pct > 0.0
                && hit.rel_volume < 0.8
        }
        Preset::FallingWedgeCoil => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 2.0
                && hit.change_pct < 0.0
                && hit.day_pct < 0.0
                && hit.rel_volume < 0.8
        }
        Preset::BigGapAndExtend => {
            hit.gap_pct.abs() > 3.0
                && hit.change_pct * hit.gap_pct > 0.0
                && hit.change_pct.abs() > hit.gap_pct.abs()
                && hit.rel_volume >= 1.5
        }
        Preset::BigGapAndReverse => {
            hit.gap_pct.abs() > 3.0
                && hit.change_pct * hit.gap_pct < 0.0
                && hit.change_pct.abs() > hit.gap_pct.abs()
                && hit.rel_volume >= 1.5
        }
        Preset::EfficientMoverHotVol => {
            hit.rel_volume >= 1.5
                && hit.change_pct.abs() >= hit.rel_volume * 1.5
        }
        Preset::InefficientChurnHotVol => {
            hit.rel_volume >= 2.0
                && hit.change_pct.abs() < hit.rel_volume * 0.3
        }
        Preset::GapUpAtMidRange => {
            hit.gap_pct > 1.0
                && hit.year_high_pct >= -50.0
                && hit.year_high_pct <= -20.0
                && hit.year_low_pct >= 20.0
                && hit.year_low_pct <= 50.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownAtMidRange => {
            hit.gap_pct < -1.0
                && hit.year_high_pct >= -50.0
                && hit.year_high_pct <= -20.0
                && hit.year_low_pct >= 20.0
                && hit.year_low_pct <= 50.0
                && hit.rel_volume >= 1.5
        }
        Preset::BattleBarHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 3.0
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume >= 2.5
        }
        Preset::IlliquidSwingDryVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 3.0
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume < 0.5
        }
        Preset::GapDownIntradayReclaimUp => {
            hit.gap_pct < -1.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 0.5
        }
        Preset::GapUpIntradayRejectDown => {
            hit.gap_pct > 1.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -0.5
        }
        Preset::HotVolModerateChangeFlatDay => {
            hit.rel_volume >= 2.0
                && hit.change_pct.abs() >= 1.0
                && hit.change_pct.abs() <= 2.0
                && hit.day_pct.abs() < 0.5
        }
        Preset::DryVolModerateChangeFlatDay => {
            hit.rel_volume < 0.6
                && hit.change_pct.abs() >= 1.0
                && hit.change_pct.abs() <= 2.0
                && hit.day_pct.abs() < 0.5
        }
        Preset::WideRangeAtYearHigh => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 5.0
                && hit.year_high_pct > -5.0
                && hit.rel_volume >= 1.5
        }
        Preset::WideRangeAtYearLow => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 5.0
                && hit.year_low_pct < 5.0
                && hit.rel_volume >= 1.5
        }
        Preset::HotVolGapHeldAndExtended => {
            hit.rel_volume >= 2.0
                && hit.gap_pct.abs() > 2.0
                && hit.change_pct * hit.gap_pct > 0.0
                && hit.change_pct.abs() >= hit.gap_pct.abs() * 0.8
        }
        Preset::HotVolGapFadedDeep => {
            hit.rel_volume >= 2.0
                && hit.gap_pct.abs() > 2.0
                && hit.change_pct * hit.gap_pct < 0.0
                && hit.change_pct.abs() >= hit.gap_pct.abs() * 0.5
        }
        Preset::TightRangeAtYearHigh => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.year_high_pct > -3.0
                && hit.rel_volume >= 0.7
                && hit.rel_volume <= 1.3
        }
        Preset::TightRangeAtYearLow => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.year_low_pct < 3.0
                && hit.rel_volume >= 0.7
                && hit.rel_volume <= 1.3
        }
        Preset::BalancedMidWickHotVol => {
            hit.hod_dist_pct.abs() >= 0.3
                && hit.hod_dist_pct.abs() <= 1.5
                && hit.lod_dist_pct.abs() >= 0.3
                && hit.lod_dist_pct.abs() <= 1.5
                && hit.rel_volume >= 1.5
        }
        Preset::BalancedMidWickDryVol => {
            hit.hod_dist_pct.abs() >= 0.3
                && hit.hod_dist_pct.abs() <= 1.5
                && hit.lod_dist_pct.abs() >= 0.3
                && hit.lod_dist_pct.abs() <= 1.5
                && hit.rel_volume < 0.6
        }
        Preset::GapUpHodCloseControlled => {
            hit.gap_pct > 0.5
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 0.0
                && hit.rel_volume >= 0.7
                && hit.rel_volume <= 1.5
        }
        Preset::GapDownLodCloseControlled => {
            hit.gap_pct < -0.5
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < 0.0
                && hit.rel_volume >= 0.7
                && hit.rel_volume <= 1.5
        }
        Preset::AllGreenTightDay => {
            hit.change_pct > 0.0
                && hit.day_pct > 0.0
                && hit.gap_pct > 0.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 2.0
        }
        Preset::AllRedTightDay => {
            hit.change_pct < 0.0
                && hit.day_pct < 0.0
                && hit.gap_pct < 0.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 2.0
        }
        Preset::MicroRangeAtYearHigh => {
            hit.hod_dist_pct.abs() < 0.3
                && hit.lod_dist_pct.abs() < 0.3
                && hit.year_high_pct > -3.0
        }
        Preset::MicroRangeAtYearLow => {
            hit.hod_dist_pct.abs() < 0.3
                && hit.lod_dist_pct.abs() < 0.3
                && hit.year_low_pct < 3.0
        }
        Preset::ConsolidationBreakUp => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 3.0
                && hit.year_high_pct >= -20.0
                && hit.year_high_pct <= -5.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::ConsolidationBreakDown => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 3.0
                && hit.year_low_pct >= 5.0
                && hit.year_low_pct <= 20.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 2.0
        }
        Preset::HotVolGapHeldFlatChange => {
            hit.rel_volume >= 2.0
                && hit.gap_pct.abs() > 1.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::DryVolGapHeldFlatChange => {
            hit.rel_volume < 0.5
                && hit.gap_pct.abs() > 1.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::AllDirectionsAlignedHotVolUp => {
            hit.rel_volume >= 3.0
                && hit.change_pct > 0.0
                && hit.day_pct > 0.0
                && hit.gap_pct > 0.0
        }
        Preset::AllDirectionsAlignedHotVolDown => {
            hit.rel_volume >= 3.0
                && hit.change_pct < 0.0
                && hit.day_pct < 0.0
                && hit.gap_pct < 0.0
        }
        Preset::IntradayRecoveryFromGapDown => {
            hit.year_high_pct >= -30.0
                && hit.year_high_pct <= -10.0
                && hit.day_pct > 1.0
                && hit.gap_pct < -0.5
                && hit.rel_volume >= 1.5
        }
        Preset::IntradayRejectionFromGapUp => {
            hit.year_low_pct >= 10.0
                && hit.year_low_pct <= 30.0
                && hit.day_pct < -1.0
                && hit.gap_pct > 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::Pct52wMidUpperHotVolDown => {
            hit.year_high_pct >= -30.0
                && hit.year_high_pct <= -10.0
                && hit.change_pct < -3.0
                && hit.rel_volume >= 2.0
        }
        Preset::Pct52wMidLowerHotVolUp => {
            hit.year_low_pct >= 10.0
                && hit.year_low_pct <= 30.0
                && hit.change_pct > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::OrderlyMidRangeRally => {
            hit.year_high_pct >= -25.0
                && hit.year_high_pct <= -15.0
                && hit.year_low_pct >= 15.0
                && hit.year_low_pct <= 25.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 0.8
                && hit.rel_volume <= 1.5
        }
        Preset::OrderlyMidRangePullback => {
            hit.year_high_pct >= -25.0
                && hit.year_high_pct <= -15.0
                && hit.year_low_pct >= 15.0
                && hit.year_low_pct <= 25.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 0.8
                && hit.rel_volume <= 1.5
        }
        Preset::StrongBreakoutDay => {
            hit.year_high_pct > -5.0
                && hit.gap_pct > 2.0
                && hit.change_pct > 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::StrongBreakdownDay => {
            hit.year_low_pct < 5.0
                && hit.gap_pct < -2.0
                && hit.change_pct < -2.0
                && hit.rel_volume >= 2.0
        }
        Preset::VolSpikeBigGapBigChange => {
            hit.rel_volume >= 3.0
                && hit.gap_pct.abs() > 3.0
                && hit.change_pct.abs() > 3.0
        }
        Preset::VolSpikeTinyGapBigChange => {
            hit.rel_volume >= 3.0
                && hit.gap_pct.abs() < 0.3
                && hit.change_pct.abs() > 3.0
        }
        Preset::StrongCloseAtHodHotVol => {
            hit.hod_dist_pct.abs() < 0.3
                && hit.day_pct > 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::WeakCloseAtLodHotVol => {
            hit.lod_dist_pct.abs() < 0.3
                && hit.day_pct < -1.0
                && hit.rel_volume >= 2.0
        }
        Preset::Pct52wHighDryVolFlat => {
            hit.year_high_pct > -3.0
                && hit.rel_volume < 0.5
                && hit.change_pct.abs() < 0.5
        }
        Preset::Pct52wLowDryVolFlat => {
            hit.year_low_pct < 3.0
                && hit.rel_volume < 0.5
                && hit.change_pct.abs() < 0.5
        }
        Preset::OvernightReversalRepositioning => {
            hit.change_pct * hit.gap_pct < 0.0
                && hit.change_pct.abs() > 0.5
                && hit.gap_pct.abs() > 0.5
                && hit.day_pct.abs() < 0.5
        }
        Preset::OrganicIntradayTrendDay => {
            hit.gap_pct.abs() < 0.3
                && hit.change_pct.abs() > 2.0
                && hit.day_pct.abs() > 1.0
                && hit.rel_volume >= 0.7
                && hit.rel_volume <= 1.3
        }
        Preset::TightRangeFlatDayHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.day_pct.abs() < 0.2
                && hit.rel_volume >= 2.0
        }
        Preset::TightRangeFlatDayDryVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.day_pct.abs() < 0.2
                && hit.rel_volume < 0.5
        }
        Preset::HodHotVolMicroRange => {
            hit.hod_dist_pct.abs() < 0.2
                && hit.lod_dist_pct.abs() >= 0.5
                && hit.lod_dist_pct.abs() <= 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::LodHotVolMicroRange => {
            hit.lod_dist_pct.abs() < 0.2
                && hit.hod_dist_pct.abs() >= 0.5
                && hit.hod_dist_pct.abs() <= 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::GapAndGoStrongClose => {
            hit.gap_pct > 1.0
                && hit.change_pct > hit.gap_pct
                && hit.hod_dist_pct.abs() < 0.5
        }
        Preset::GapAndFadeWeakClose => {
            hit.gap_pct > 1.0
                && hit.change_pct < 0.0
                && hit.lod_dist_pct.abs() < 0.5
        }
        Preset::InsideDayDryVolCoiled => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.7
                && hit.gap_pct.abs() < 0.3
        }
        Preset::OutsideDayHotVolExpansion => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.change_pct.abs() > 2.0
                && hit.rel_volume >= 1.8
        }
        Preset::MidRangeDryVolNoConviction => {
            hit.hod_dist_pct.abs() >= 1.0
                && hit.hod_dist_pct.abs() <= 3.0
                && hit.lod_dist_pct.abs() >= 1.0
                && hit.lod_dist_pct.abs() <= 3.0
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume < 0.7
        }
        Preset::LowOfYearHotVolPanic => {
            hit.year_low_pct < 1.0
                && hit.rel_volume >= 2.0
                && hit.change_pct < -2.0
        }
        Preset::HighOfYearHotVolEuphoria => {
            hit.year_high_pct < 1.0
                && hit.rel_volume >= 2.0
                && hit.change_pct > 2.0
        }
        Preset::WideRangeFlatCloseHeavyChurn => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume >= 1.8
        }
        Preset::RangeExpansionDryVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 3.0
                && hit.rel_volume < 0.7
        }
        Preset::YearHighGapDownHotVol => {
            hit.year_high_pct < 5.0
                && hit.gap_pct < -2.0
                && hit.rel_volume >= 1.5
        }
        Preset::YearLowGapUpHotVol => {
            hit.year_low_pct < 5.0
                && hit.gap_pct > 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::IntradayFakeoutTopReject => {
            hit.hod_dist_pct.abs() > 2.0
                && hit.day_pct < -0.5
                && hit.rel_volume >= 1.5
        }
        Preset::IntradayFakeoutBottomReject => {
            hit.lod_dist_pct.abs() > 2.0
                && hit.day_pct > 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::RangeContractionAfterMove => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 0.8
                && hit.day_pct.abs() < 0.3
                && hit.change_pct.abs() > 1.0
                && hit.rel_volume >= 0.8
                && hit.rel_volume <= 1.3
        }
        Preset::RelativeStrengthBuild => {
            hit.change_pct > 1.5
                && hit.rel_volume >= 1.0
                && hit.rel_volume <= 1.5
                && hit.gap_pct.abs() < 0.5
        }
        Preset::RelativeWeaknessBuild => {
            hit.change_pct < -1.5
                && hit.rel_volume >= 1.0
                && hit.rel_volume <= 1.5
                && hit.gap_pct.abs() < 0.5
        }
        Preset::HighVolAbsorbingChange => {
            hit.rel_volume >= 2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.change_pct.abs() > 1.0
        }
        Preset::LowVolWideRangeAccumulator => {
            hit.rel_volume < 0.5
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 2.0
                && hit.change_pct.abs() > 1.0
        }
        Preset::BullishEngulfingHotVol => {
            hit.gap_pct < -0.5
                && hit.change_pct > 1.5
                && hit.day_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::BearishEngulfingHotVol => {
            hit.gap_pct > 0.5
                && hit.change_pct < -1.5
                && hit.day_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::DoubleBottomRetest => {
            hit.year_low_pct < 3.0
                && hit.rel_volume >= 1.2
                && hit.change_pct >= 0.0
                && hit.day_pct >= 0.0
        }
        Preset::DoubleTopRetest => {
            hit.year_high_pct < 3.0
                && hit.rel_volume >= 1.2
                && hit.change_pct <= 0.0
                && hit.day_pct <= 0.0
        }
        Preset::LiquiditySweepBothSides => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 3.0
                && hit.hod_dist_pct.abs() > 1.0
                && hit.lod_dist_pct.abs() > 1.0
                && hit.day_pct.abs() < 0.3
                && hit.rel_volume >= 1.5
        }
        Preset::SteadyGrinderNoVolPickup => {
            hit.change_pct >= 0.5
                && hit.change_pct <= 2.0
                && hit.rel_volume >= 0.8
                && hit.rel_volume <= 1.0
                && hit.day_pct >= 0.3
                && hit.day_pct <= 1.5
                && hit.gap_pct.abs() < 0.2
        }
        Preset::SteadyDeclinerNoVolPickup => {
            hit.change_pct <= -0.5
                && hit.change_pct >= -2.0
                && hit.rel_volume >= 0.8
                && hit.rel_volume <= 1.0
                && hit.day_pct <= -0.3
                && hit.day_pct >= -1.5
                && hit.gap_pct.abs() < 0.2
        }
        Preset::HighVolStallNearHighOfYear => {
            hit.year_high_pct < 2.0
                && hit.rel_volume >= 2.0
                && hit.day_pct.abs() < 0.5
        }
        Preset::HighVolStallNearLowOfYear => {
            hit.year_low_pct < 2.0
                && hit.rel_volume >= 2.0
                && hit.day_pct.abs() < 0.5
        }
        Preset::OutlierSessionBigMoveBigVol => {
            hit.change_pct.abs() > 3.0
                && hit.rel_volume >= 3.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 2.5
        }
        Preset::EodParabolicAccelerationUp => {
            hit.change_pct > 2.0
                && hit.day_pct > hit.change_pct * 0.7
                && hit.hod_dist_pct.abs() < 0.3
                && hit.rel_volume >= 1.5
        }
        Preset::EodParabolicAccelerationDown => {
            hit.change_pct < -2.0
                && hit.day_pct < hit.change_pct * 0.7
                && hit.lod_dist_pct.abs() < 0.3
                && hit.rel_volume >= 1.5
        }
        Preset::FullSpectrumDayUp => {
            hit.change_pct > 1.0
                && hit.day_pct > 0.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.lod_dist_pct.abs() > 1.0
                && hit.rel_volume >= 1.2
        }
        Preset::FullSpectrumDayDown => {
            hit.change_pct < -1.0
                && hit.day_pct < 0.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() > 1.0
                && hit.rel_volume >= 1.2
        }
        Preset::GreenStreakAccumulator => {
            hit.change_pct > 0.5
                && hit.day_pct > 0.0
                && hit.rel_volume >= 1.2
                && hit.gap_pct >= 0.0
        }
        Preset::RedStreakDistributor => {
            hit.change_pct < -0.5
                && hit.day_pct < 0.0
                && hit.rel_volume >= 1.2
                && hit.gap_pct <= 0.0
        }
        Preset::GapDownReclaim => {
            hit.gap_pct < -1.0
                && hit.change_pct >= 0.0
                && hit.day_pct > -hit.gap_pct * 0.8
        }
        Preset::GapUpFailReclaimed => {
            hit.gap_pct > 1.0
                && hit.change_pct <= 0.0
                && hit.day_pct < -hit.gap_pct * 0.8
        }
        Preset::MidYearRangeConsolidation => {
            hit.year_low_pct > 20.0
                && hit.year_high_pct > 20.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 1.0
        }
        Preset::AtYearExtremeVolatilityExpansion => {
            (hit.year_high_pct < 3.0 || hit.year_low_pct < 3.0)
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 3.0
                && hit.rel_volume >= 1.5
        }
        Preset::BreakoutFromMidLevels => {
            hit.year_high_pct < 10.0
                && hit.year_low_pct >= 20.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::BreakdownFromMidLevels => {
            hit.year_low_pct < 10.0
                && hit.year_high_pct >= 20.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::IntradayStrongerThanGap => {
            hit.gap_pct.abs() < 1.0
                && hit.day_pct.abs() > 1.5
                && hit.change_pct.abs() > 1.0
                && hit.rel_volume >= 1.2
        }
        Preset::OvernightStrongerThanIntraday => {
            hit.gap_pct.abs() > 1.5
                && hit.day_pct.abs() < 0.5
                && hit.change_pct.abs() > 1.0
        }
        Preset::EfficientMoveLowEffort => {
            hit.change_pct.abs() > 1.0
                && hit.rel_volume < 0.7
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
        }
        Preset::SignalVsNoiseChurn => {
            hit.change_pct.abs() < 0.2
                && hit.rel_volume >= 2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 2.0
        }
        Preset::GreenCloseRedIntraday => {
            hit.change_pct > 0.0
                && hit.day_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::RedCloseGreenIntraday => {
            hit.change_pct < 0.0
                && hit.day_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::FullConvictionUpDay => {
            hit.gap_pct > 0.0
                && hit.change_pct > 1.0
                && hit.day_pct > 0.5
                && hit.hod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::FullConvictionDownDay => {
            hit.gap_pct < 0.0
                && hit.change_pct < -1.0
                && hit.day_pct < -0.5
                && hit.lod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::YearLowProximityRallyAttempt => {
            hit.year_low_pct < 5.0
                && hit.gap_pct >= 0.0
                && hit.day_pct > 1.0
                && hit.change_pct > 0.5
                && hit.rel_volume >= 1.0
        }
        Preset::YearHighProximityFailAttempt => {
            hit.year_high_pct < 5.0
                && hit.gap_pct <= 0.0
                && hit.day_pct < -1.0
                && hit.change_pct < -0.5
                && hit.rel_volume >= 1.0
        }
        Preset::OpenGapFilledNetFlat => {
            hit.gap_pct.abs() > 1.5
                && hit.change_pct.abs() < 0.5
        }
        Preset::CompressedRangeVolatileSession => {
            hit.year_high_pct < 15.0
                && hit.year_low_pct < 25.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 3.0
                && hit.rel_volume >= 1.5
        }
        Preset::OrderlyNewHighContinuation => {
            hit.year_high_pct < 1.0
                && hit.change_pct >= 0.5
                && hit.change_pct <= 1.5
                && hit.rel_volume >= 1.0
        }
        Preset::OrderlyNewLowContinuation => {
            hit.year_low_pct < 1.0
                && hit.change_pct <= -0.5
                && hit.change_pct >= -1.5
                && hit.rel_volume >= 1.0
        }
        Preset::DryVolGapUpFade => {
            hit.gap_pct > 1.0
                && hit.change_pct < 0.0
                && hit.rel_volume < 0.7
        }
        Preset::DryVolGapDownReclaim => {
            hit.gap_pct < -1.0
                && hit.change_pct > 0.0
                && hit.rel_volume < 0.7
        }
        Preset::InstitutionalChurnDay => {
            hit.rel_volume >= 3.0
                && hit.change_pct.abs() < 0.3
        }
        Preset::ExtremeTailEvent => {
            hit.rel_volume >= 3.0
                && hit.change_pct.abs() > 5.0
        }
        Preset::Year52HighRetestStrongClose => {
            hit.year_high_pct < 5.0
                && hit.change_pct > 0.0
                && hit.hod_dist_pct.abs() < 0.3
                && hit.rel_volume >= 1.5
        }
        Preset::Year52LowRetestWeakClose => {
            hit.year_low_pct < 5.0
                && hit.change_pct < 0.0
                && hit.lod_dist_pct.abs() < 0.3
                && hit.rel_volume >= 1.5
        }
        Preset::DivergentGapVsIntraday => {
            hit.gap_pct * hit.day_pct < 0.0
                && hit.gap_pct.abs() > 0.5
                && hit.day_pct.abs() > 0.5
        }
        Preset::CongruentGapAndIntradaySameDir => {
            hit.gap_pct * hit.day_pct > 0.0
                && hit.gap_pct.abs() > 0.5
                && hit.day_pct.abs() > 0.5
        }
        Preset::DeepMidRangeQuietSiesta => {
            hit.year_high_pct > 30.0
                && hit.year_low_pct > 30.0
                && hit.rel_volume < 0.5
                && hit.change_pct.abs() < 0.5
        }
        Preset::DeepMidRangeActiveOutlier => {
            hit.year_high_pct > 30.0
                && hit.year_low_pct > 30.0
                && hit.rel_volume >= 2.0
                && hit.change_pct.abs() > 1.0
        }
        Preset::IntradayDirectionExceedsChange => {
            hit.day_pct * hit.change_pct > 0.0
                && hit.day_pct.abs() > hit.change_pct.abs() * 1.5
                && hit.change_pct.abs() > 0.5
        }
        Preset::ChangeExceedsIntradayMagnitude => {
            hit.change_pct.abs() > hit.day_pct.abs() * 2.0
                && hit.change_pct.abs() > 1.0
        }
        Preset::JustOffYearLowBouncingUp => {
            hit.year_low_pct >= 5.0
                && hit.year_low_pct <= 15.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::JustOffYearHighFadingDown => {
            hit.year_high_pct >= 5.0
                && hit.year_high_pct <= 15.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::OverextendedHighPullbackHealthy => {
            hit.year_high_pct < 3.0
                && hit.hod_dist_pct.abs() > 1.5
                && hit.day_pct < -0.5
                && hit.change_pct >= 0.0
        }
        Preset::OverextendedLowBounceHealthy => {
            hit.year_low_pct < 3.0
                && hit.lod_dist_pct.abs() > 1.5
                && hit.day_pct > 0.5
                && hit.change_pct <= 0.0
        }
        Preset::CleanTrendDayUp => {
            hit.change_pct > 0.0
                && hit.day_pct > 0.0
                && hit.gap_pct > 0.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.0
        }
        Preset::CleanTrendDayDown => {
            hit.change_pct < 0.0
                && hit.day_pct < 0.0
                && hit.gap_pct < 0.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.0
        }
        Preset::ClimaxRedBouncedFromLod => {
            hit.change_pct < -3.0
                && hit.day_pct > 1.0
                && hit.rel_volume >= 2.0
                && hit.lod_dist_pct.abs() > 1.5
        }
        Preset::ClimaxGreenFadedFromHod => {
            hit.change_pct > 3.0
                && hit.day_pct < -1.0
                && hit.rel_volume >= 2.0
                && hit.hod_dist_pct.abs() > 1.5
        }
        Preset::WideRangeChopMixedVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 3.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 0.7
                && hit.rel_volume <= 1.5
        }
        Preset::NarrowRangeBigChangeNoIntraday => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.change_pct.abs() > 1.5
                && hit.day_pct.abs() < 0.3
        }
        Preset::EveryAxisExtreme => {
            hit.gap_pct.abs() > 1.0
                && hit.day_pct.abs() > 1.0
                && hit.change_pct.abs() > 2.0
                && hit.rel_volume >= 2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 2.0
        }
        Preset::EveryAxisFlat => {
            hit.gap_pct.abs() < 0.2
                && hit.day_pct.abs() < 0.2
                && hit.change_pct.abs() < 0.2
                && hit.rel_volume < 0.7
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
        }
        Preset::Year52HighRejectedToLod => {
            hit.year_high_pct < 5.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 1.5
                && hit.change_pct < 0.0
        }
        Preset::Year52LowReclaimedToHod => {
            hit.year_low_pct < 5.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 1.5
                && hit.change_pct > 0.0
        }
        Preset::HighVolNoGapModerateChange => {
            hit.gap_pct.abs() < 0.3
                && hit.rel_volume >= 2.0
                && hit.change_pct.abs() < 1.0
        }
        Preset::LowVolWithLargeGap => {
            hit.gap_pct.abs() > 2.0
                && hit.rel_volume < 0.7
                && hit.change_pct.abs() > 1.0
        }
        Preset::GapErasedByIntradayFlat => {
            hit.gap_pct.abs() > 1.0
                && hit.change_pct.abs() < 0.3
                && hit.gap_pct * hit.day_pct < 0.0
        }
        Preset::BothSidesTaggedFlatBalance => {
            hit.hod_dist_pct.abs() > 1.0
                && hit.lod_dist_pct.abs() > 1.0
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume >= 1.2
        }
        Preset::OutsideDayWideBalanceHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 5.0
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume >= 1.5
        }
        Preset::InsideDayBigChangeBigVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.change_pct.abs() > 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::LongCandleUpTrendDay => {
            hit.change_pct > 2.0
                && hit.day_pct > 1.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 2.0
                && hit.rel_volume >= 1.5
                && hit.hod_dist_pct.abs() < 0.5
        }
        Preset::LongCandleDownTrendDay => {
            hit.change_pct < -2.0
                && hit.day_pct < -1.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 2.0
                && hit.rel_volume >= 1.5
                && hit.lod_dist_pct.abs() < 0.5
        }
        Preset::Year52HighWithRangeContraction => {
            hit.year_high_pct < 3.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::Year52LowWithRangeContraction => {
            hit.year_low_pct < 3.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::GapAndIntradayHarmonic => {
            hit.gap_pct.abs() >= 0.5
                && hit.gap_pct.abs() <= 2.0
                && hit.day_pct.abs() >= 0.5
                && hit.day_pct.abs() <= 2.0
                && (hit.gap_pct.abs() - hit.day_pct.abs()).abs() < 0.3
                && hit.rel_volume >= 1.0
        }
        Preset::MicroDayEarlyShakeout => {
            hit.change_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 2.0
                && hit.day_pct.abs() < 0.3
                && hit.rel_volume >= 1.5
        }
        Preset::GreenDaySubOptimalClose => {
            hit.change_pct > 1.0
                && hit.hod_dist_pct.abs() > 2.0
                && hit.rel_volume >= 1.0
        }
        Preset::RedDaySubOptimalClose => {
            hit.change_pct < -1.0
                && hit.lod_dist_pct.abs() > 2.0
                && hit.rel_volume >= 1.0
        }
        Preset::WideRangeNoVolFlat => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 3.0
                && hit.rel_volume < 0.7
                && hit.change_pct.abs() < 0.5
        }
        Preset::NarrowRangeMeaningfulChange => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.change_pct.abs() > 1.0
        }
        Preset::Year52HighGapDownReclaimed => {
            hit.year_high_pct < 5.0
                && hit.gap_pct < -0.5
                && hit.change_pct >= 0.0
                && hit.rel_volume >= 1.0
        }
        Preset::Year52LowGapUpFaded => {
            hit.year_low_pct < 5.0
                && hit.gap_pct > 0.5
                && hit.change_pct <= 0.0
                && hit.rel_volume >= 1.0
        }
        Preset::IntradayMatchesChange => {
            hit.gap_pct.abs() < 0.2
                && hit.change_pct.abs() > 1.0
                && (hit.change_pct - hit.day_pct).abs() < 0.3
        }
        Preset::IntradayOpposesChange => {
            hit.change_pct.abs() > 1.0
                && hit.day_pct * hit.change_pct < 0.0
                && hit.day_pct.abs() > 0.5
        }
        Preset::SymmetricMidRangeBalance => {
            (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.2
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 1.0
                && hit.change_pct.abs() < 0.3
        }
        Preset::AsymmetricExtremeBias => {
            (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() > 2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 3.0
        }
        Preset::YearLowExplosiveSqueezeIgnition => {
            hit.year_low_pct < 3.0
                && hit.change_pct > 5.0
                && hit.rel_volume >= 3.0
        }
        Preset::YearHighSharpDistribution => {
            hit.year_high_pct < 3.0
                && hit.change_pct < -5.0
                && hit.rel_volume >= 3.0
        }
        Preset::LargeChangeOnNormalVol => {
            hit.change_pct.abs() > 3.0
                && hit.rel_volume >= 0.7
                && hit.rel_volume <= 1.3
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 2.0
        }
        Preset::MassiveIntradayWithoutGap => {
            hit.gap_pct.abs() < 0.1
                && hit.day_pct.abs() > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::MidYearBothSidesTagged => {
            hit.year_high_pct > 10.0
                && hit.year_low_pct > 10.0
                && hit.hod_dist_pct.abs() > 1.0
                && hit.lod_dist_pct.abs() > 1.0
                && hit.rel_volume >= 1.0
        }
        Preset::ExtremeSilentRange => {
            (hit.year_high_pct < 5.0 || hit.year_low_pct < 5.0)
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 0.8
                && hit.rel_volume < 1.0
        }
        Preset::MultiAxisDryDay => {
            hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.7
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.gap_pct.abs() < 0.3
        }
        Preset::BigGapBigVolBigChange => {
            hit.gap_pct.abs() > 2.0
                && hit.rel_volume >= 2.0
                && hit.change_pct.abs() > 2.0
        }
        Preset::GapDownClosedNearHODHotVol => {
            hit.gap_pct < -1.0
                && hit.change_pct > 0.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpClosedNearLODHotVol => {
            hit.gap_pct > 1.0
                && hit.change_pct < 0.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::Year52HighGapUpHotVolBigChange => {
            hit.year_high_pct < 1.0
                && hit.gap_pct > 1.0
                && hit.rel_volume >= 2.0
                && hit.change_pct > 1.0
        }
        Preset::Year52LowGapDownHotVolBigDrop => {
            hit.year_low_pct < 1.0
                && hit.gap_pct < -1.0
                && hit.rel_volume >= 2.0
                && hit.change_pct < -1.0
        }
        Preset::WideRangeFlatCloseHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 5.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::BigGapNarrowIntradayHotVol => {
            hit.gap_pct.abs() > 3.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::Year52HighDryVolNarrowRange => {
            hit.year_high_pct < 2.0
                && hit.rel_volume < 0.7
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.change_pct.abs() < 0.3
        }
        Preset::Year52LowDryVolNarrowRange => {
            hit.year_low_pct < 2.0
                && hit.rel_volume < 0.7
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.change_pct.abs() < 0.3
        }
        Preset::CompressedYearRangeFlatDay => {
            hit.year_high_pct < 5.0
                && hit.year_low_pct < 5.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 1.0
        }
        Preset::CompressedYearRangeRegimeBreak => {
            hit.year_high_pct < 5.0
                && hit.year_low_pct < 5.0
                && hit.change_pct.abs() > 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::IntradayClimaxTopFade => {
            hit.hod_dist_pct.abs() > 4.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < 0.0
                && hit.rel_volume >= 2.0
        }
        Preset::IntradayClimaxBottomReclaim => {
            hit.lod_dist_pct.abs() > 4.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 0.0
                && hit.rel_volume >= 2.0
        }
        Preset::BigChangeDryVolWideRange => {
            hit.change_pct.abs() > 3.0
                && hit.rel_volume < 0.7
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 3.0
        }
        Preset::BigChangeDryVolFromGap => {
            hit.change_pct.abs() > 3.0
                && hit.rel_volume < 0.7
                && hit.gap_pct.abs() > 2.0
        }
        Preset::ExtremeVolGapDownReversal => {
            hit.rel_volume >= 5.0
                && hit.gap_pct < -3.0
                && hit.change_pct > 0.0
        }
        Preset::ExtremeVolGapUpReversal => {
            hit.rel_volume >= 5.0
                && hit.gap_pct > 3.0
                && hit.change_pct < 0.0
        }
        Preset::AtYearHighRangeExpansionDryVol => {
            hit.year_high_pct < 1.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.rel_volume < 0.7
        }
        Preset::AtYearLowRangeExpansionDryVol => {
            hit.year_low_pct < 1.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.rel_volume < 0.7
        }
        Preset::IntradayBigDayGapAgainstHotVol => {
            hit.day_pct.abs() > 3.0
                && hit.gap_pct * hit.day_pct < 0.0
                && hit.gap_pct.abs() > 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::IntradayBigDayGapWithHotVol => {
            hit.day_pct.abs() > 3.0
                && hit.gap_pct * hit.day_pct > 0.0
                && hit.gap_pct.abs() > 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::OvernightDriftDryVol => {
            hit.gap_pct.abs() > 2.0
                && hit.day_pct.abs() < 0.3
                && hit.rel_volume < 0.5
        }
        Preset::HotVolHugeGapTinyDay => {
            hit.rel_volume >= 3.0
                && hit.gap_pct.abs() > 2.0
                && hit.day_pct.abs() < 0.3
        }
        Preset::Year52LowGapUpHeldHotVol => {
            hit.year_low_pct < 5.0
                && hit.gap_pct > 1.0
                && hit.change_pct > 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52HighGapDownHeldHotVol => {
            hit.year_high_pct < 5.0
                && hit.gap_pct < -1.0
                && hit.change_pct < -2.0
                && hit.rel_volume >= 2.0
        }
        Preset::HotVolSmallChangeSmallGapWideRange => {
            hit.rel_volume >= 2.0
                && hit.change_pct.abs() < 0.5
                && hit.gap_pct.abs() < 0.3
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 1.5
        }
        Preset::HotVolFlatCloseBigGap => {
            hit.change_pct.abs() < 0.5
                && hit.gap_pct.abs() > 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::OrganicMicroGainNormalVol => {
            hit.change_pct >= 0.3
                && hit.change_pct <= 1.0
                && hit.day_pct > 0.2
                && hit.rel_volume >= 0.9
                && hit.rel_volume <= 1.2
                && hit.gap_pct.abs() < 0.2
        }
        Preset::OrganicMicroDropNormalVol => {
            hit.change_pct <= -0.3
                && hit.change_pct >= -1.0
                && hit.day_pct < -0.2
                && hit.rel_volume >= 0.9
                && hit.rel_volume <= 1.2
                && hit.gap_pct.abs() < 0.2
        }
        Preset::IntradayRangeWiderThanGapHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > hit.gap_pct.abs() * 2.0
                && hit.gap_pct.abs() > 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::GapWiderThanIntradayRangeHotVol => {
            hit.gap_pct.abs() > hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs()
                && hit.gap_pct.abs() > 1.5
                && hit.rel_volume >= 2.0
        }
        Preset::BigGreenLowVolWeakClose => {
            hit.change_pct > 3.0
                && hit.rel_volume < 1.0
                && hit.hod_dist_pct.abs() > 1.0
        }
        Preset::BigRedLowVolWeakClose => {
            hit.change_pct < -3.0
                && hit.rel_volume < 1.0
                && hit.lod_dist_pct.abs() > 1.0
        }
        Preset::GappingNearYearLowExtremeVol => {
            hit.year_low_pct < 5.0
                && hit.gap_pct.abs() > 2.0
                && hit.rel_volume >= 4.0
        }
        Preset::GappingNearYearHighExtremeVol => {
            hit.year_high_pct < 5.0
                && hit.gap_pct.abs() > 2.0
                && hit.rel_volume >= 4.0
        }
        Preset::BothSidesTaggedDryVolFlat => {
            hit.hod_dist_pct.abs() > 1.0
                && hit.lod_dist_pct.abs() > 1.0
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume < 0.7
        }
        Preset::BothSidesTaggedBigChangeHotVol => {
            hit.hod_dist_pct.abs() > 1.0
                && hit.lod_dist_pct.abs() > 1.0
                && hit.change_pct.abs() > 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::ModerateGreenGapDownReversal => {
            hit.gap_pct < -1.0
                && hit.change_pct >= 1.0
                && hit.change_pct <= 2.0
                && hit.rel_volume >= 1.0
                && hit.rel_volume <= 1.5
        }
        Preset::ModerateRedGapUpFade => {
            hit.gap_pct > 1.0
                && hit.change_pct <= -1.0
                && hit.change_pct >= -2.0
                && hit.rel_volume >= 1.0
                && hit.rel_volume <= 1.5
        }
        Preset::GapAndIntradayBothBigSameDirHotVol => {
            hit.gap_pct.abs() > 2.0
                && hit.day_pct.abs() > 2.0
                && hit.gap_pct * hit.day_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapAndIntradayBothBigOpposingHotVol => {
            hit.gap_pct.abs() > 2.0
                && hit.day_pct.abs() > 2.0
                && hit.gap_pct * hit.day_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::CountertrendBounceInDowntrend => {
            hit.change_pct > 2.0
                && hit.year_high_pct > 20.0
                && hit.year_low_pct < 10.0
                && hit.rel_volume >= 2.0
        }
        Preset::CountertrendFadeInUptrend => {
            hit.change_pct < -2.0
                && hit.year_low_pct > 20.0
                && hit.year_high_pct < 10.0
                && hit.rel_volume >= 2.0
        }
        Preset::BigGreenNarrowRangeHotVol => {
            hit.change_pct > 2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::BigRedNarrowRangeHotVol => {
            hit.change_pct < -2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::OneSidedRangeCloseAtHODGreen => {
            hit.hod_dist_pct.abs() < 0.5
                && hit.lod_dist_pct.abs() > 2.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::OneSidedRangeCloseAtLODRed => {
            hit.lod_dist_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() > 2.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::BullishOutsideDayHotVol => {
            hit.change_pct > 1.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.change_pct * hit.day_pct > 0.0
                && hit.rel_volume >= 2.0
        }
        Preset::BearishOutsideDayHotVol => {
            hit.change_pct < -1.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.change_pct * hit.day_pct > 0.0
                && hit.rel_volume >= 2.0
        }
        Preset::BelowAvgVolBigChangeGreen => {
            hit.change_pct > 2.0
                && hit.rel_volume >= 0.5
                && hit.rel_volume < 0.9
                && hit.gap_pct.abs() < 0.5
        }
        Preset::BelowAvgVolBigChangeRed => {
            hit.change_pct < -2.0
                && hit.rel_volume >= 0.5
                && hit.rel_volume < 0.9
                && hit.gap_pct.abs() < 0.5
        }
        Preset::MidRangeFullExpansionHotVol => {
            hit.year_high_pct > 15.0
                && hit.year_low_pct > 15.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.rel_volume >= 2.0
        }
        Preset::MidRangeCompressionDryVol => {
            hit.year_high_pct > 15.0
                && hit.year_low_pct > 15.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 0.8
                && hit.rel_volume < 0.5
        }
        Preset::OpeningRangeHoldCloseAtHODGreen => {
            hit.gap_pct.abs() < 0.3
                && hit.hod_dist_pct.abs() < 0.2
                && hit.day_pct > 1.0
                && hit.rel_volume >= 1.0
                && hit.rel_volume <= 2.0
        }
        Preset::OpeningRangeHoldCloseAtLODRed => {
            hit.gap_pct.abs() < 0.3
                && hit.lod_dist_pct.abs() < 0.2
                && hit.day_pct < -1.0
                && hit.rel_volume >= 1.0
                && hit.rel_volume <= 2.0
        }
        Preset::WindowDressingMarkUp => {
            hit.change_pct.abs() < 0.3
                && hit.change_pct > 0.0
                && hit.hod_dist_pct.abs() < 0.3
                && hit.rel_volume >= 1.5
        }
        Preset::WindowDressingMarkDown => {
            hit.change_pct.abs() < 0.3
                && hit.change_pct < 0.0
                && hit.lod_dist_pct.abs() < 0.3
                && hit.rel_volume >= 1.5
        }
        Preset::Year52HighSustainedStrengthHotVol => {
            hit.year_high_pct < 5.0
                && hit.day_pct > 1.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52LowSustainedWeaknessHotVol => {
            hit.year_low_pct < 5.0
                && hit.day_pct < -1.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 2.0
        }
        Preset::BigGreenWithModestGapDecentVol => {
            hit.change_pct > 3.0
                && hit.gap_pct >= 0.5
                && hit.gap_pct <= 1.5
                && hit.rel_volume >= 1.5
        }
        Preset::BigRedWithModestGapDownDecentVol => {
            hit.change_pct < -3.0
                && hit.gap_pct >= -1.5
                && hit.gap_pct <= -0.5
                && hit.rel_volume >= 1.5
        }
        Preset::CompoundConfirmedBigGreen => {
            hit.change_pct > 3.0
                && hit.day_pct > 1.0
                && hit.gap_pct > 0.5
                && hit.hod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::CompoundConfirmedBigRed => {
            hit.change_pct < -3.0
                && hit.day_pct < -1.0
                && hit.gap_pct < -0.5
                && hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::FollowThroughGreen => {
            hit.change_pct >= 1.0
                && hit.change_pct <= 3.0
                && hit.day_pct >= 0.5
                && hit.day_pct <= 2.0
                && hit.gap_pct >= -0.5
                && hit.gap_pct <= 0.5
                && hit.hod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 1.2
                && hit.rel_volume <= 2.0
        }
        Preset::FollowThroughRed => {
            hit.change_pct >= -3.0
                && hit.change_pct <= -1.0
                && hit.day_pct >= -2.0
                && hit.day_pct <= -0.5
                && hit.gap_pct >= -0.5
                && hit.gap_pct <= 0.5
                && hit.lod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 1.2
                && hit.rel_volume <= 2.0
        }
        Preset::Year52HighGapDownStrongCloseHotVol => {
            hit.year_high_pct < 3.0
                && hit.gap_pct < -0.5
                && hit.change_pct > 1.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::Year52LowGapUpWeakCloseHotVol => {
            hit.year_low_pct < 3.0
                && hit.gap_pct > 0.5
                && hit.change_pct < -1.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::FlatOpenTrendUpModerate => {
            hit.gap_pct.abs() < 0.3
                && hit.change_pct >= 1.0
                && hit.change_pct <= 3.0
                && hit.day_pct > 1.0
                && hit.change_pct * hit.day_pct > 0.0
                && hit.rel_volume >= 1.0
                && hit.rel_volume <= 2.0
        }
        Preset::FlatOpenTrendDownModerate => {
            hit.gap_pct.abs() < 0.3
                && hit.change_pct >= -3.0
                && hit.change_pct <= -1.0
                && hit.day_pct < -1.0
                && hit.change_pct * hit.day_pct > 0.0
                && hit.rel_volume >= 1.0
                && hit.rel_volume <= 2.0
        }
        Preset::MidRangeRecoveryRallyHotVol => {
            hit.year_high_pct > 10.0
                && hit.year_low_pct > 10.0
                && hit.change_pct > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::MidRangeSelloffHotVol => {
            hit.year_high_pct > 10.0
                && hit.year_low_pct > 10.0
                && hit.change_pct < -3.0
                && hit.rel_volume >= 2.0
        }
        Preset::IntermediateGreenStrongClose => {
            hit.change_pct >= 3.0
                && hit.change_pct <= 7.0
                && hit.rel_volume >= 1.5
                && hit.rel_volume <= 3.0
                && hit.hod_dist_pct.abs() < 1.0
        }
        Preset::IntermediateRedWeakClose => {
            hit.change_pct >= -7.0
                && hit.change_pct <= -3.0
                && hit.rel_volume >= 1.5
                && hit.rel_volume <= 3.0
                && hit.lod_dist_pct.abs() < 1.0
        }
        Preset::MaxVolatilityEventHotVol => {
            hit.gap_pct.abs() > 2.0
                && hit.change_pct.abs() > 3.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.rel_volume >= 2.0
        }
        Preset::MaxRangeFakeOutDryVol => {
            hit.gap_pct.abs() > 2.0
                && hit.change_pct.abs() > 3.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.rel_volume < 1.0
        }
        Preset::BigGreenIntradayOnlyHotVol => {
            hit.change_pct > 3.0
                && hit.gap_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.rel_volume >= 2.0
        }
        Preset::BigRedIntradayOnlyHotVol => {
            hit.change_pct < -3.0
                && hit.gap_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.rel_volume >= 2.0
        }
        Preset::BrokeAbove52wHighHotVol => {
            hit.year_high_pct > 0.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::BrokeBelow52wLowHotVol => {
            hit.year_low_pct < 0.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 2.0
        }
        Preset::ChangeIntradayDisagreeBothTagged => {
            hit.change_pct * hit.day_pct < 0.0
                && hit.hod_dist_pct.abs() > 1.0
                && hit.lod_dist_pct.abs() > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::ChangeIntradayDisagreeFlatRange => {
            hit.change_pct * hit.day_pct < 0.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::BigGapHugeVolHalfFade => {
            hit.gap_pct.abs() > 2.0
                && hit.rel_volume >= 3.0
                && hit.change_pct.abs() < hit.gap_pct.abs() * 0.5
        }
        Preset::BigGapHugeVolFullExtension => {
            hit.gap_pct.abs() > 2.0
                && hit.rel_volume >= 3.0
                && hit.change_pct.abs() > hit.gap_pct.abs() * 1.5
        }
        Preset::GapWithChangeWideRangeHotVol => {
            hit.change_pct * hit.gap_pct > 0.0
                && hit.gap_pct.abs() > 1.0
                && hit.change_pct.abs() > 2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::GapAgainstChangeWideRangeHotVol => {
            hit.change_pct * hit.gap_pct < 0.0
                && hit.gap_pct.abs() > 1.0
                && hit.change_pct.abs() > 2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::HotVolNoChangeNoGapTightRange => {
            hit.rel_volume >= 2.0
                && hit.change_pct.abs() < 0.5
                && hit.gap_pct.abs() < 0.3
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
        }
        Preset::ColdVolBigChangeWideRange => {
            hit.rel_volume < 0.5
                && hit.change_pct.abs() > 3.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 3.0
        }
        Preset::MicroPinTinyRangeHotVol => {
            hit.hod_dist_pct.abs() < 0.1
                && hit.lod_dist_pct.abs() < 0.1
                && hit.rel_volume >= 2.0
        }
        Preset::MicroPinTinyRangeDryVol => {
            hit.hod_dist_pct.abs() < 0.1
                && hit.lod_dist_pct.abs() < 0.1
                && hit.rel_volume < 0.5
        }
        Preset::FullRange52wAtHighSide => {
            hit.year_high_pct < 2.0
                && hit.year_low_pct > 50.0
                && hit.rel_volume >= 1.5
        }
        Preset::FullRange52wAtLowSide => {
            hit.year_low_pct < 2.0
                && hit.year_high_pct > 50.0
                && hit.rel_volume >= 1.5
        }
        Preset::PullbackAndRallyAtYearHigh => {
            hit.change_pct >= 0.5
                && hit.change_pct <= 1.5
                && hit.day_pct > 0.5
                && hit.gap_pct >= -0.5
                && hit.gap_pct <= 0.0
                && hit.year_high_pct < 5.0
                && hit.rel_volume >= 1.0
                && hit.rel_volume <= 1.8
        }
        Preset::DeadCatBounceAtYearLow => {
            hit.change_pct >= -1.5
                && hit.change_pct <= -0.5
                && hit.day_pct < -0.5
                && hit.gap_pct >= 0.0
                && hit.gap_pct <= 0.5
                && hit.year_low_pct < 5.0
                && hit.rel_volume >= 1.0
                && hit.rel_volume <= 1.8
        }
        Preset::GapDownIntradayReversalCloseAtHOD => {
            hit.gap_pct < -0.5
                && hit.day_pct > 0.5
                && hit.hod_dist_pct.abs() < 0.3
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpIntradayReversalCloseAtLOD => {
            hit.gap_pct > 0.5
                && hit.day_pct < -0.5
                && hit.lod_dist_pct.abs() < 0.3
                && hit.rel_volume >= 1.5
        }
        Preset::BigGreenMidYearSweetSpot => {
            hit.change_pct > 3.0
                && hit.year_high_pct >= 5.0
                && hit.year_high_pct <= 20.0
                && hit.year_low_pct > 15.0
                && hit.rel_volume >= 1.5
        }
        Preset::BigRedMidYearSweetSpot => {
            hit.change_pct < -3.0
                && hit.year_low_pct >= 5.0
                && hit.year_low_pct <= 20.0
                && hit.year_high_pct > 15.0
                && hit.rel_volume >= 1.5
        }
        Preset::TripleZeroHotVol => {
            hit.gap_pct.abs() < 0.1
                && hit.change_pct.abs() < 0.1
                && hit.day_pct.abs() < 0.1
                && hit.rel_volume >= 2.0
        }
        Preset::TripleZeroDryVol => {
            hit.gap_pct.abs() < 0.1
                && hit.change_pct.abs() < 0.1
                && hit.day_pct.abs() < 0.1
                && hit.rel_volume < 0.5
        }
        Preset::ExtremeGapModerateMoveHotVol => {
            hit.gap_pct.abs() > 5.0
                && hit.change_pct.abs() >= 2.0
                && hit.change_pct.abs() <= 5.0
                && hit.rel_volume >= 2.0
        }
        Preset::ExtremeGapBigContinuationHotVol => {
            hit.gap_pct.abs() > 5.0
                && hit.change_pct.abs() > hit.gap_pct.abs()
                && hit.rel_volume >= 2.0
        }
        Preset::BigGreenBigGapDryVol => {
            hit.change_pct > 5.0
                && hit.gap_pct > 3.0
                && hit.rel_volume < 0.8
        }
        Preset::BigRedBigGapDownDryVol => {
            hit.change_pct < -5.0
                && hit.gap_pct < -3.0
                && hit.rel_volume < 0.8
        }
        Preset::SmoothBigGreenNormalVol => {
            hit.change_pct > 3.0
                && hit.rel_volume >= 1.0
                && hit.rel_volume <= 1.5
                && hit.hod_dist_pct.abs() < 0.5
                && hit.gap_pct.abs() < 0.5
        }
        Preset::SmoothBigRedNormalVol => {
            hit.change_pct < -3.0
                && hit.rel_volume >= 1.0
                && hit.rel_volume <= 1.5
                && hit.lod_dist_pct.abs() < 0.5
                && hit.gap_pct.abs() < 0.5
        }
        Preset::BigDayPctFlatChangeHotVol => {
            hit.day_pct.abs() > 2.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::BigDayPctBigChangeAlignedHotVol => {
            hit.day_pct.abs() > 2.0
                && hit.change_pct.abs() > 4.0
                && hit.change_pct * hit.day_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::Year52HighBigDayDryVol => {
            hit.year_high_pct < 2.0
                && hit.day_pct > 2.0
                && hit.rel_volume < 0.7
        }
        Preset::Year52LowBigDayDryVol => {
            hit.year_low_pct < 2.0
                && hit.day_pct < -2.0
                && hit.rel_volume < 0.7
        }
        Preset::MidMagnitudeGreenMidWickHotVol => {
            hit.change_pct >= 1.0
                && hit.change_pct <= 3.0
                && hit.hod_dist_pct.abs() >= 0.5
                && hit.hod_dist_pct.abs() <= 2.0
                && hit.lod_dist_pct.abs() >= 0.5
                && hit.lod_dist_pct.abs() <= 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::MidMagnitudeRedMidWickHotVol => {
            hit.change_pct <= -1.0
                && hit.change_pct >= -3.0
                && hit.hod_dist_pct.abs() >= 0.5
                && hit.hod_dist_pct.abs() <= 2.0
                && hit.lod_dist_pct.abs() >= 0.5
                && hit.lod_dist_pct.abs() <= 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::HotVolHugeRangeBigChange => {
            hit.rel_volume >= 3.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 6.0
                && hit.change_pct.abs() > 4.0
        }
        Preset::HotVolHugeRangeFlatClose => {
            hit.rel_volume >= 3.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 6.0
                && hit.change_pct.abs() < 0.5
        }
        Preset::Year52HighDistributionChurn => {
            hit.year_high_pct < 2.0
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume >= 2.0
        }
        Preset::Year52LowAccumulationChurn => {
            hit.year_low_pct < 2.0
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume >= 2.0
        }
        Preset::Year52HighBigGreenBreakoutHotVol => {
            hit.year_high_pct < 0.0
                && hit.change_pct > 4.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52LowBigRedBreakdownHotVol => {
            hit.year_low_pct < 0.0
                && hit.change_pct < -4.0
                && hit.rel_volume >= 2.0
        }
        Preset::GapUpFailBigRedHotVol => {
            hit.gap_pct > 3.0
                && hit.change_pct < -2.0
                && hit.rel_volume >= 2.0
        }
        Preset::GapDownReclaimBigGreenHotVol => {
            hit.gap_pct < -3.0
                && hit.change_pct > 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::InsideRangeHotVolCoil => {
            hit.hod_dist_pct.abs() < 1.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::OutsideRangeFlatCloseHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 5.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::CloseAtHodTinyLodHotVol => {
            hit.hod_dist_pct.abs() < 0.3
                && hit.lod_dist_pct > 4.0
                && hit.rel_volume >= 1.5
        }
        Preset::CloseAtLodTinyHodHotVol => {
            hit.lod_dist_pct.abs() < 0.3
                && hit.hod_dist_pct < -4.0
                && hit.rel_volume >= 1.5
        }
        Preset::BigGreenCloseAtHodHotVol => {
            hit.change_pct > 3.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::BigRedCloseAtLodHotVol => {
            hit.change_pct < -3.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::GapAndGoBigGreenCloseAtHod => {
            hit.gap_pct > 2.0
                && hit.change_pct > hit.gap_pct
                && hit.hod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::GapAndDropBigRedCloseAtLod => {
            hit.gap_pct < -2.0
                && hit.change_pct < hit.gap_pct
                && hit.lod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpFillReverseHotVol => {
            hit.gap_pct > 3.0
                && hit.change_pct < 0.0
                && hit.change_pct > -hit.gap_pct
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownFillReverseHotVol => {
            hit.gap_pct < -3.0
                && hit.change_pct > 0.0
                && hit.change_pct < -hit.gap_pct
                && hit.rel_volume >= 1.5
        }
        Preset::Year52HighSqueezeShort => {
            hit.year_high_pct < 0.0
                && hit.change_pct > 5.0
                && hit.rel_volume >= 3.0
        }
        Preset::Year52LowCapitulation => {
            hit.year_low_pct < 0.0
                && hit.change_pct < -5.0
                && hit.rel_volume >= 3.0
        }
        Preset::DragonflyDojiHotVol => {
            hit.change_pct.abs() < 0.3
                && hit.lod_dist_pct > 4.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GravestoneDojiHotVol => {
            hit.change_pct.abs() < 0.3
                && hit.hod_dist_pct < -4.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::HammerReversalHotVol => {
            hit.change_pct > 1.0
                && hit.lod_dist_pct > 3.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::ShootingStarReversalHotVol => {
            hit.change_pct < -1.0
                && hit.hod_dist_pct < -3.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::MarubozuGreenHotVol => {
            hit.change_pct > 3.0
                && hit.hod_dist_pct.abs() < 0.3
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::MarubozuRedHotVol => {
            hit.change_pct < -3.0
                && hit.lod_dist_pct.abs() < 0.3
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52HighParabolicExtreme => {
            hit.year_high_pct < 0.0
                && hit.change_pct > 10.0
                && hit.rel_volume >= 5.0
        }
        Preset::Year52LowParabolicExtreme => {
            hit.year_low_pct < 0.0
                && hit.change_pct < -10.0
                && hit.rel_volume >= 5.0
        }
        Preset::HotVolNoChangeTightRange => {
            hit.rel_volume >= 3.0
                && hit.change_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 2.0
                && hit.gap_pct.abs() < 0.5
        }
        Preset::DryVolBigMoveNoFollow => {
            hit.rel_volume < 0.5
                && hit.change_pct.abs() > 4.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 3.0
        }
        Preset::BigGapBigContinuationBigRange => {
            hit.gap_pct.abs() > 4.0
                && hit.change_pct.abs() > 2.0 * hit.gap_pct.abs()
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 6.0
                && hit.rel_volume >= 2.0
        }
        Preset::BigGapFullReversalBigRange => {
            hit.gap_pct.abs() > 4.0
                && hit.change_pct.abs() > 2.0
                && hit.gap_pct * hit.change_pct < 0.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 6.0
                && hit.rel_volume >= 2.0
        }
        Preset::TinyGapBigMoveTightWicks => {
            hit.gap_pct.abs() < 0.5
                && hit.change_pct.abs() > 4.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::BigGapTinyMoveTightWicks => {
            hit.gap_pct.abs() > 4.0
                && hit.change_pct.abs() < 1.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::HotVolBigGreenWideRangeYearLow => {
            hit.change_pct > 5.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 6.0
                && hit.year_low_pct < 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::HotVolBigRedWideRangeYearHigh => {
            hit.change_pct < -5.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 6.0
                && hit.year_high_pct < 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::HotVolBigGreenWideRangeYearHigh => {
            hit.change_pct > 5.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 6.0
                && hit.year_high_pct < 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::HotVolBigRedWideRangeYearLow => {
            hit.change_pct < -5.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 6.0
                && hit.year_low_pct < 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::RangeContractionHotVolBigGap => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
                && hit.gap_pct.abs() > 3.0
        }
        Preset::RangeExpansionHotVolBigIntraday => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 5.0
                && hit.day_pct.abs() > 3.0
                && hit.rel_volume >= 2.0
                && hit.gap_pct.abs() < 1.0
        }
        Preset::GapPlusDriveBullHotVol => {
            hit.gap_pct > 1.0
                && hit.day_pct > 3.0
                && hit.change_pct > 4.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapPlusDriveBearHotVol => {
            hit.gap_pct < -1.0
                && hit.day_pct < -3.0
                && hit.change_pct < -4.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapFadeBullDayPctOpposite => {
            hit.gap_pct < -2.0
                && hit.day_pct > 3.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapFadeBearDayPctOpposite => {
            hit.gap_pct > 2.0
                && hit.day_pct < -3.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::DayPctBigGreenChangeFlat => {
            hit.day_pct > 4.0
                && hit.change_pct.abs() < 0.5
                && hit.gap_pct < -3.0
                && hit.rel_volume >= 1.5
        }
        Preset::DayPctBigRedChangeFlat => {
            hit.day_pct < -4.0
                && hit.change_pct.abs() < 0.5
                && hit.gap_pct > 3.0
                && hit.rel_volume >= 1.5
        }
        Preset::Year52HighBreakoutOpenDriveHotVol => {
            hit.year_high_pct < 0.0
                && hit.day_pct > 3.0
                && hit.change_pct > 4.0
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52LowBreakdownOpenDriveHotVol => {
            hit.year_low_pct < 0.0
                && hit.day_pct < -3.0
                && hit.change_pct < -4.0
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52HighGapAndGoExtremeVol => {
            hit.year_high_pct < 0.0
                && hit.gap_pct > 3.0
                && hit.day_pct > 2.0
                && hit.change_pct > 5.0
                && hit.rel_volume >= 3.0
        }
        Preset::Year52LowGapAndDropExtremeVol => {
            hit.year_low_pct < 0.0
                && hit.gap_pct < -3.0
                && hit.day_pct < -2.0
                && hit.change_pct < -5.0
                && hit.rel_volume >= 3.0
        }
        Preset::Year52HighFailedBreakoutFade => {
            hit.year_high_pct >= 0.0
                && hit.year_high_pct < 3.0
                && hit.gap_pct > 1.0
                && hit.day_pct < -1.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52LowFailedBreakdownReclaim => {
            hit.year_low_pct >= 0.0
                && hit.year_low_pct < 3.0
                && hit.gap_pct < -1.0
                && hit.day_pct > 1.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52HighRangeCompressionLowVol => {
            hit.year_high_pct >= 0.0
                && hit.year_high_pct < 3.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.7
        }
        Preset::Year52LowRangeCompressionLowVol => {
            hit.year_low_pct >= 0.0
                && hit.year_low_pct < 3.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.7
        }
        Preset::DistantFromYearHighDryVolCoil => {
            hit.year_high_pct >= 30.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.6
        }
        Preset::DistantFromYearLowDryVolCoil => {
            hit.year_low_pct >= 30.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.6
        }
        Preset::DistantFromYearHighBigGreenHotVol => {
            hit.year_high_pct >= 20.0
                && hit.change_pct > 5.0
                && hit.rel_volume >= 2.0
        }
        Preset::DistantFromYearLowBigRedHotVol => {
            hit.year_low_pct >= 20.0
                && hit.change_pct < -5.0
                && hit.rel_volume >= 2.0
        }
        Preset::MidRangeChurnHotVolBigDayPct => {
            hit.hod_dist_pct.abs().min(hit.lod_dist_pct.abs()) >= 1.5
                && hit.hod_dist_pct.abs().max(hit.lod_dist_pct.abs()) <= 5.0
                && hit.day_pct.abs() > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::MidRangeChurnHotVolFlatDayPct => {
            hit.hod_dist_pct.abs().min(hit.lod_dist_pct.abs()) >= 1.5
                && hit.hod_dist_pct.abs().max(hit.lod_dist_pct.abs()) <= 5.0
                && hit.day_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::Year52HighRetestPullbackDryVol => {
            hit.year_high_pct >= 3.0
                && hit.year_high_pct < 10.0
                && hit.change_pct < -1.0
                && hit.change_pct > -3.0
                && hit.rel_volume < 0.8
        }
        Preset::Year52LowRetestBounceDryVol => {
            hit.year_low_pct >= 3.0
                && hit.year_low_pct < 10.0
                && hit.change_pct > 1.0
                && hit.change_pct < 3.0
                && hit.rel_volume < 0.8
        }
        Preset::Year52HighRetestPullbackHotVol => {
            hit.year_high_pct >= 3.0
                && hit.year_high_pct < 10.0
                && hit.change_pct < -2.0
                && hit.change_pct > -5.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52LowRetestBounceHotVol => {
            hit.year_low_pct >= 3.0
                && hit.year_low_pct < 10.0
                && hit.change_pct > 2.0
                && hit.change_pct < 5.0
                && hit.rel_volume >= 2.0
        }
        Preset::HotVolBigChangeDayPctOpposite => {
            hit.change_pct.abs() > 3.0
                && hit.day_pct * hit.change_pct < 0.0
                && hit.day_pct.abs() > 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::HotVolBigChangeDayPctAligned => {
            hit.change_pct.abs() > 3.0
                && hit.day_pct.abs() > 3.0
                && hit.day_pct * hit.change_pct > 0.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52HighBreakoutHotVolNoExtreme => {
            hit.year_high_pct < 0.0
                && hit.change_pct > 1.5
                && hit.change_pct < 4.0
                && hit.rel_volume >= 1.5
                && hit.rel_volume < 3.0
        }
        Preset::Year52LowBreakdownHotVolNoExtreme => {
            hit.year_low_pct < 0.0
                && hit.change_pct < -1.5
                && hit.change_pct > -4.0
                && hit.rel_volume >= 1.5
                && hit.rel_volume < 3.0
        }
        Preset::BigGreenTopWickRejectHotVol => {
            hit.change_pct > 1.0
                && hit.hod_dist_pct < -2.0
                && hit.rel_volume >= 1.5
        }
        Preset::BigRedBottomWickRejectHotVol => {
            hit.change_pct < -1.0
                && hit.lod_dist_pct > 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::DryVolGreenCloseAtHodTinyRange => {
            hit.change_pct > 0.5
                && hit.hod_dist_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 2.0
                && hit.rel_volume < 0.7
        }
        Preset::DryVolRedCloseAtLodTinyRange => {
            hit.change_pct < -0.5
                && hit.lod_dist_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 2.0
                && hit.rel_volume < 0.7
        }
        Preset::Year52HighGapDownDryVolReclaim => {
            hit.year_high_pct < 3.0
                && hit.year_high_pct >= -2.0
                && hit.gap_pct < -1.5
                && hit.change_pct > 0.0
                && hit.rel_volume < 0.8
        }
        Preset::Year52LowGapUpDryVolReject => {
            hit.year_low_pct < 3.0
                && hit.year_low_pct >= -2.0
                && hit.gap_pct > 1.5
                && hit.change_pct < 0.0
                && hit.rel_volume < 0.8
        }
        Preset::Year52HighInsideDayHotVol => {
            hit.year_high_pct < 3.0
                && hit.year_high_pct >= -2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::Year52LowInsideDayHotVol => {
            hit.year_low_pct < 3.0
                && hit.year_low_pct >= -2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::Year52HighOutsideDayHotVol => {
            hit.year_high_pct < 3.0
                && hit.year_high_pct >= -2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52LowOutsideDayHotVol => {
            hit.year_low_pct < 3.0
                && hit.year_low_pct >= -2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.rel_volume >= 2.0
        }
        Preset::YearHighGapDownHotVolRecovery => {
            hit.year_high_pct < 0.0
                && hit.gap_pct < -2.0
                && hit.day_pct > 3.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::YearLowGapUpHotVolRejection => {
            hit.year_low_pct < 0.0
                && hit.gap_pct > 2.0
                && hit.day_pct < -3.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52HighReclaimAfterFlush => {
            hit.year_high_pct < 0.0
                && hit.lod_dist_pct > 4.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::Year52LowReclaimAfterPop => {
            hit.year_low_pct < 0.0
                && hit.hod_dist_pct < -4.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::BigDayPctSmallChangeHotVol => {
            hit.day_pct.abs() > 3.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::SmallDayPctBigChangeHotVol => {
            hit.day_pct.abs() < 0.5
                && hit.change_pct.abs() > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52HighRangeExpansionHotVol => {
            hit.year_high_pct < 0.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 5.0
                && hit.change_pct > 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52LowRangeExpansionHotVol => {
            hit.year_low_pct < 0.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 5.0
                && hit.change_pct < -2.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52HighRangeContractionHotVol => {
            hit.year_high_pct < 0.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.change_pct > 0.0
                && hit.change_pct < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52LowRangeContractionHotVol => {
            hit.year_low_pct < 0.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.change_pct < 0.0
                && hit.change_pct > -1.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52HighBreakoutDryVolPullback => {
            hit.year_high_pct >= 0.0
                && hit.year_high_pct < 2.0
                && hit.change_pct < -0.5
                && hit.change_pct > -2.0
                && hit.rel_volume < 0.8
        }
        Preset::Year52LowBreakdownDryVolBounce => {
            hit.year_low_pct >= 0.0
                && hit.year_low_pct < 2.0
                && hit.change_pct > 0.5
                && hit.change_pct < 2.0
                && hit.rel_volume < 0.8
        }
        Preset::BigGapBigCounterMoveBigRangeHotVol => {
            hit.gap_pct.abs() > 3.0
                && hit.day_pct * hit.gap_pct < 0.0
                && hit.day_pct.abs() > 3.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 5.0
                && hit.rel_volume >= 2.0
        }
        Preset::BigGapBigContinuationBigDayPctHotVol => {
            hit.gap_pct.abs() > 3.0
                && hit.day_pct * hit.gap_pct > 0.0
                && hit.day_pct.abs() > 3.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 5.0
                && hit.rel_volume >= 2.0
        }
        Preset::NoGapBigChangeBigDayPctHotVol => {
            hit.gap_pct.abs() < 0.5
                && hit.change_pct.abs() > 4.0
                && hit.day_pct.abs() > 4.0
                && hit.rel_volume >= 2.0
        }
        Preset::MidYearHighBigGreenHotVol => {
            hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
                && hit.change_pct > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::MidYearHighBigRedHotVol => {
            hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
                && hit.change_pct < -3.0
                && hit.rel_volume >= 2.0
        }
        Preset::MidYearLowBigRedHotVol => {
            hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
                && hit.change_pct < -3.0
                && hit.rel_volume >= 2.0
        }
        Preset::MidYearLowBigGreenHotVol => {
            hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
                && hit.change_pct > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52HighFullRangeDryVol => {
            hit.year_high_pct < 2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.rel_volume < 0.8
        }
        Preset::Year52LowFullRangeDryVol => {
            hit.year_low_pct < 2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 4.0
                && hit.rel_volume < 0.8
        }
        Preset::BigChangeBigRangeDryVol => {
            hit.change_pct.abs() > 4.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 5.0
                && hit.rel_volume < 0.7
        }
        Preset::ExtremeVolFlatDay => {
            hit.rel_volume >= 5.0
                && hit.change_pct.abs() < 0.5
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 2.0
        }
        Preset::ExtremeVolBigChangeClimax => {
            hit.rel_volume >= 5.0
                && hit.change_pct.abs() > 5.0
        }
        Preset::ExtremeGapBigContinuationExtremeVol => {
            hit.gap_pct.abs() > 5.0
                && hit.change_pct.abs() > 8.0
                && hit.gap_pct * hit.change_pct > 0.0
                && hit.rel_volume >= 5.0
        }
        Preset::ExtremeGapFullReversalExtremeVol => {
            hit.gap_pct.abs() > 5.0
                && hit.gap_pct * hit.change_pct < 0.0
                && hit.change_pct.abs() > 3.0
                && hit.rel_volume >= 5.0
        }
        Preset::ApathyAtYearHigh => {
            hit.year_high_pct < 2.0
                && hit.rel_volume < 0.3
                && hit.change_pct.abs() < 0.3
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
        }
        Preset::ApathyAtYearLow => {
            hit.year_low_pct < 2.0
                && hit.rel_volume < 0.3
                && hit.change_pct.abs() < 0.3
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
        }
        Preset::StealthAtYear52High => {
            hit.year_high_pct < 2.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 5.0
        }
        Preset::StealthAtYear52Low => {
            hit.year_low_pct < 2.0
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 5.0
        }
        Preset::ExtremeVolCloseAtHod => {
            hit.rel_volume >= 5.0
                && hit.hod_dist_pct.abs() < 0.5
        }
        Preset::ExtremeVolCloseAtLod => {
            hit.rel_volume >= 5.0
                && hit.lod_dist_pct.abs() < 0.5
        }
        Preset::ExtremeRangeExtremeVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 5.0
        }
        Preset::ExtremeRangeDryVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume < 0.5
        }
        Preset::BigGreenUpperRangeHotVol => {
            hit.change_pct > 2.0
                && hit.lod_dist_pct > 2.0 * hit.hod_dist_pct.abs()
                && hit.rel_volume >= 2.0
        }
        Preset::BigRedLowerRangeHotVol => {
            hit.change_pct < -2.0
                && hit.hod_dist_pct.abs() > 2.0 * hit.lod_dist_pct
                && hit.rel_volume >= 2.0
        }
        Preset::BigBreakoutAboveYearHigh => {
            hit.year_high_pct < -3.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::BigBreakdownBelowYearLow => {
            hit.year_low_pct < -3.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::DeepPullbackBigGreenHotVol => {
            hit.year_high_pct >= 10.0
                && hit.year_high_pct < 30.0
                && hit.change_pct > 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::DeepPullbackBigRedHotVol => {
            hit.year_high_pct >= 10.0
                && hit.year_high_pct < 30.0
                && hit.change_pct < -2.0
                && hit.rel_volume >= 2.0
        }
        Preset::DeepBounceBigGreenHotVol => {
            hit.year_low_pct >= 10.0
                && hit.year_low_pct < 30.0
                && hit.change_pct > 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::DeepBounceBigRedHotVol => {
            hit.year_low_pct >= 10.0
                && hit.year_low_pct < 30.0
                && hit.change_pct < -2.0
                && hit.rel_volume >= 2.0
        }
        Preset::BigGapDownReclaimedToHodHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::BigGapUpRejectedToLodHotVol => {
            hit.gap_pct > 2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::TenXVolMicroChange => {
            hit.rel_volume >= 10.0
                && hit.change_pct.abs() < 0.3
        }
        Preset::TenXVolNoGapBigIntradayMove => {
            hit.rel_volume >= 10.0
                && hit.gap_pct.abs() < 0.3
                && hit.change_pct.abs() > 3.0
        }
        Preset::MicroVolBigChange => {
            hit.rel_volume < 0.1
                && hit.change_pct.abs() > 3.0
        }
        Preset::MicroVolFlatDay => {
            hit.rel_volume < 0.1
                && hit.change_pct.abs() < 0.3
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
        }
        Preset::ConfirmedBreakoutAboveYearHigh => {
            hit.year_high_pct >= -3.0
                && hit.year_high_pct < -1.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::ConfirmedBreakdownBelowYearLow => {
            hit.year_low_pct >= -3.0
                && hit.year_low_pct < -1.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::UpperWickFlatCloseHotVol => {
            hit.hod_dist_pct < -3.0
                && hit.change_pct.abs() < 1.0
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::LowerWickFlatCloseHotVol => {
            hit.lod_dist_pct > 3.0
                && hit.change_pct.abs() < 1.0
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::PartialGapUpHoldHotVol => {
            hit.gap_pct > 2.0
                && hit.day_pct < -0.5
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::PartialGapDownHoldHotVol => {
            hit.gap_pct < -2.0
                && hit.day_pct > 0.5
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::BreakoutZoneRangeExpansionHotVol => {
            hit.year_high_pct >= -3.0
                && hit.year_high_pct < 0.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 5.0
                && hit.rel_volume >= 2.0
        }
        Preset::BreakdownZoneRangeExpansionHotVol => {
            hit.year_low_pct >= -3.0
                && hit.year_low_pct < 0.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 5.0
                && hit.rel_volume >= 2.0
        }
        Preset::Year52HighFreshConsolidationDryVol => {
            hit.year_high_pct < 0.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.8
        }
        Preset::Year52LowFreshConsolidationDryVol => {
            hit.year_low_pct < 0.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume < 0.8
        }
        Preset::ModerateGapBullContinuationHotVol => {
            hit.gap_pct >= 1.0
                && hit.gap_pct <= 2.0
                && hit.change_pct > 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::ModerateGapBearContinuationHotVol => {
            hit.gap_pct >= -2.0
                && hit.gap_pct <= -1.0
                && hit.change_pct < -2.0
                && hit.rel_volume >= 2.0
        }
        Preset::ModerateGapBullContinuationDryVol => {
            hit.gap_pct >= 1.0
                && hit.gap_pct <= 2.0
                && hit.change_pct > 2.0
                && hit.rel_volume < 0.8
        }
        Preset::ModerateGapBearContinuationDryVol => {
            hit.gap_pct >= -2.0
                && hit.gap_pct <= -1.0
                && hit.change_pct < -2.0
                && hit.rel_volume < 0.8
        }
        Preset::ModerateGapBullFadeHotVol => {
            hit.gap_pct >= 1.0
                && hit.gap_pct <= 2.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 2.0
        }
        Preset::ModerateGapBearReclaimHotVol => {
            hit.gap_pct >= -2.0
                && hit.gap_pct <= -1.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::ConfirmedBreakoutFadeHotVol => {
            hit.year_high_pct >= -3.0
                && hit.year_high_pct < -1.0
                && hit.change_pct < -0.5
                && hit.rel_volume >= 1.5
        }
        Preset::ConfirmedBreakdownReclaimHotVol => {
            hit.year_low_pct >= -3.0
                && hit.year_low_pct < -1.0
                && hit.change_pct > 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::IntradayBullDriveAtYear52High => {
            hit.year_high_pct < 2.0
                && hit.day_pct > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::IntradayBearDriveAtYear52Low => {
            hit.year_low_pct < 2.0
                && hit.day_pct < -3.0
                && hit.rel_volume >= 2.0
        }
        Preset::IntradayBearDriveAtYear52High => {
            hit.year_high_pct < 2.0
                && hit.day_pct < -3.0
                && hit.rel_volume >= 2.0
        }
        Preset::IntradayBullDriveAtYear52Low => {
            hit.year_low_pct < 2.0
                && hit.day_pct > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::IntradayBullDriveBelowYearHigh => {
            hit.year_high_pct >= 0.0
                && hit.year_high_pct < 5.0
                && hit.day_pct > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::IntradayBearDriveAboveYearLow => {
            hit.year_low_pct >= 0.0
                && hit.year_low_pct < 5.0
                && hit.day_pct < -3.0
                && hit.rel_volume >= 2.0
        }
        Preset::HammerAtYear52Low => {
            hit.year_low_pct < 2.0
                && hit.lod_dist_pct > 3.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::ShootingStarAtYear52High => {
            hit.year_high_pct < 2.0
                && hit.hod_dist_pct < -3.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::MarubozuGreenAtYear52High => {
            hit.year_high_pct < 2.0
                && hit.change_pct > 3.0
                && hit.hod_dist_pct.abs() < 0.3
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::MarubozuRedAtYear52Low => {
            hit.year_low_pct < 2.0
                && hit.change_pct < -3.0
                && hit.lod_dist_pct.abs() < 0.3
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::DragonflyDojiAtYear52Low => {
            hit.year_low_pct < 2.0
                && hit.change_pct.abs() < 0.3
                && hit.lod_dist_pct > 4.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GravestoneDojiAtYear52High => {
            hit.year_high_pct < 2.0
                && hit.change_pct.abs() < 0.3
                && hit.hod_dist_pct < -4.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::HammerAtMidYearLowRange => {
            hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
                && hit.lod_dist_pct > 3.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::ShootingStarAtMidYearHighRange => {
            hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
                && hit.hod_dist_pct < -3.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::HammerAtDeepPullback => {
            hit.year_high_pct >= 10.0
                && hit.year_high_pct < 30.0
                && hit.lod_dist_pct > 3.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::ShootingStarAtDeepBounce => {
            hit.year_low_pct >= 10.0
                && hit.year_low_pct < 30.0
                && hit.hod_dist_pct < -3.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::DragonflyDojiAtMidYearLow => {
            hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
                && hit.change_pct.abs() < 0.3
                && hit.lod_dist_pct > 4.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GravestoneDojiAtMidYearHigh => {
            hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
                && hit.change_pct.abs() < 0.3
                && hit.hod_dist_pct < -4.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::DragonflyDojiAtDeepPullback => {
            hit.year_high_pct >= 10.0
                && hit.year_high_pct < 30.0
                && hit.change_pct.abs() < 0.3
                && hit.lod_dist_pct > 4.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GravestoneDojiAtDeepBounce => {
            hit.year_low_pct >= 10.0
                && hit.year_low_pct < 30.0
                && hit.change_pct.abs() < 0.3
                && hit.hod_dist_pct < -4.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::MarubozuGreenAtMidYearHigh => {
            hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
                && hit.change_pct > 3.0
                && hit.hod_dist_pct.abs() < 0.3
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::MarubozuRedAtMidYearLow => {
            hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
                && hit.change_pct < -3.0
                && hit.lod_dist_pct.abs() < 0.3
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::MarubozuGreenAtDeepPullback => {
            hit.year_high_pct >= 10.0
                && hit.year_high_pct < 30.0
                && hit.change_pct > 3.0
                && hit.hod_dist_pct.abs() < 0.3
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::MarubozuRedAtDeepBounce => {
            hit.year_low_pct >= 10.0
                && hit.year_low_pct < 30.0
                && hit.change_pct < -3.0
                && hit.lod_dist_pct.abs() < 0.3
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::HammerAtDeepDiscount => {
            hit.year_high_pct >= 30.0
                && hit.lod_dist_pct > 3.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::ShootingStarAtDeepPremium => {
            hit.year_low_pct >= 30.0
                && hit.hod_dist_pct < -3.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::DragonflyDojiAtDeepDiscount => {
            hit.year_high_pct >= 30.0
                && hit.change_pct.abs() < 0.3
                && hit.lod_dist_pct > 4.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GravestoneDojiAtDeepPremium => {
            hit.year_low_pct >= 30.0
                && hit.change_pct.abs() < 0.3
                && hit.hod_dist_pct < -4.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::MarubozuGreenAtDeepDiscount => {
            hit.year_high_pct >= 30.0
                && hit.change_pct > 3.0
                && hit.hod_dist_pct.abs() < 0.3
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::MarubozuRedAtDeepPremium => {
            hit.year_low_pct >= 30.0
                && hit.change_pct < -3.0
                && hit.lod_dist_pct.abs() < 0.3
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::HammerAtYear52High => {
            hit.year_high_pct < 2.0
                && hit.lod_dist_pct > 3.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::ShootingStarAtYear52Low => {
            hit.year_low_pct < 2.0
                && hit.hod_dist_pct < -3.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::HammerAtMidYearHighRange => {
            hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
                && hit.lod_dist_pct > 3.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::ShootingStarAtMidYearLowRange => {
            hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
                && hit.hod_dist_pct < -3.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::HammerAtDeepBounceContinuation => {
            hit.year_low_pct >= 10.0
                && hit.year_low_pct < 30.0
                && hit.lod_dist_pct > 3.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::ShootingStarAtDeepPullbackContinuation => {
            hit.year_high_pct >= 10.0
                && hit.year_high_pct < 30.0
                && hit.hod_dist_pct < -3.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::HammerAtDeepPremiumContinuation => {
            hit.year_low_pct >= 30.0
                && hit.lod_dist_pct > 3.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::ShootingStarAtDeepDiscountContinuation => {
            hit.year_high_pct >= 30.0
                && hit.hod_dist_pct < -3.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::BothLongWicksHotVol => {
            hit.hod_dist_pct.abs() > 2.0
                && hit.lod_dist_pct > 2.0
                && hit.rel_volume >= 2.0
        }
        Preset::BothShortWicksTinyChangeHotVol => {
            hit.hod_dist_pct.abs() < 0.5
                && hit.lod_dist_pct < 0.5
                && hit.change_pct.abs() < 0.3
                && hit.rel_volume >= 2.0
        }
        Preset::HammerAtConfirmedBreakdown => {
            hit.year_low_pct >= -3.0
                && hit.year_low_pct < -1.0
                && hit.lod_dist_pct > 3.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::ShootingStarAtConfirmedBreakout => {
            hit.year_high_pct >= -3.0
                && hit.year_high_pct < -1.0
                && hit.hod_dist_pct < -3.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::DragonflyDojiAtConfirmedBreakdown => {
            hit.year_low_pct >= -3.0
                && hit.year_low_pct < -1.0
                && hit.change_pct.abs() < 0.3
                && hit.lod_dist_pct > 4.0
                && hit.hod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GravestoneDojiAtConfirmedBreakout => {
            hit.year_high_pct >= -3.0
                && hit.year_high_pct < -1.0
                && hit.change_pct.abs() < 0.3
                && hit.hod_dist_pct < -4.0
                && hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::MarubozuGreenAtConfirmedBreakout => {
            hit.year_high_pct >= -3.0
                && hit.year_high_pct < -1.0
                && hit.change_pct > 3.0
                && hit.hod_dist_pct.abs() < 0.3
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::MarubozuRedAtConfirmedBreakdown => {
            hit.year_low_pct >= -3.0
                && hit.year_low_pct < -1.0
                && hit.change_pct < -3.0
                && hit.lod_dist_pct.abs() < 0.3
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::MarubozuRedAtConfirmedBreakout => {
            hit.year_high_pct >= -3.0
                && hit.year_high_pct < -1.0
                && hit.change_pct < -3.0
                && hit.lod_dist_pct.abs() < 0.3
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::MarubozuGreenAtConfirmedBreakdown => {
            hit.year_low_pct >= -3.0
                && hit.year_low_pct < -1.0
                && hit.change_pct > 3.0
                && hit.hod_dist_pct.abs() < 0.3
                && hit.gap_pct.abs() < 1.0
                && hit.rel_volume >= 2.0
        }
        Preset::TripleAlignedBullBigConvictionDay => {
            hit.gap_pct > 1.5
                && hit.change_pct > 3.0
                && hit.day_pct > 1.5
                && hit.rel_volume >= 2.0
        }
        Preset::TripleAlignedBearBigConvictionDay => {
            hit.gap_pct < -1.5
                && hit.change_pct < -3.0
                && hit.day_pct < -1.5
                && hit.rel_volume >= 2.0
        }
        Preset::DistantFromYearHighRangeContractionHotVol => {
            hit.year_high_pct >= 20.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::DistantFromYearLowRangeContractionHotVol => {
            hit.year_low_pct >= 20.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.5
                && hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::DistantFromYearHighRangeExpansionHotVol => {
            hit.year_high_pct >= 20.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 5.0
                && hit.rel_volume >= 2.0
        }
        Preset::DistantFromYearLowRangeExpansionHotVol => {
            hit.year_low_pct >= 20.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 5.0
                && hit.rel_volume >= 2.0
        }
        Preset::CloseAtHodMidYearLowHotVol => {
            hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::CloseAtLodMidYearHighHotVol => {
            hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::CloseAtHodDeepBelowYearHighHotVol => {
            hit.year_high_pct >= 20.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::CloseAtLodDeepAboveYearLowHotVol => {
            hit.year_low_pct >= 20.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::CloseAtHodNearYearHighHotVol => {
            hit.year_high_pct < 2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::CloseAtLodNearYearLowHotVol => {
            hit.year_low_pct < 2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::CloseAtHodJustOffYearHighHotVol => {
            hit.year_high_pct >= 2.0
                && hit.year_high_pct < 5.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::CloseAtLodJustOffYearLowHotVol => {
            hit.year_low_pct >= 2.0
                && hit.year_low_pct < 5.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::CloseAtHodConfirmedAboveYearHighHotVol => {
            hit.year_high_pct >= -3.0
                && hit.year_high_pct <= -1.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::CloseAtLodConfirmedBelowYearLowHotVol => {
            hit.year_low_pct >= -3.0
                && hit.year_low_pct <= -1.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::MidpointCloseNearYearHighHotVol => {
            hit.year_high_pct < 2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::MidpointCloseNearYearLowHotVol => {
            hit.year_low_pct < 2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::MidpointCloseConfirmedAboveYearHighHotVol => {
            hit.year_high_pct >= -3.0
                && hit.year_high_pct <= -1.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::MidpointCloseConfirmedBelowYearLowHotVol => {
            hit.year_low_pct >= -3.0
                && hit.year_low_pct <= -1.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::MidpointCloseDeepBelowYearHighHotVol => {
            hit.year_high_pct >= 20.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::MidpointCloseDeepAboveYearLowHotVol => {
            hit.year_low_pct >= 20.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::MidpointCloseMidYearHighHotVol => {
            hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::MidpointCloseMidYearLowHotVol => {
            hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::MidpointCloseJustOffYearHighHotVol => {
            hit.year_high_pct >= 2.0
                && hit.year_high_pct < 5.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::MidpointCloseJustOffYearLowHotVol => {
            hit.year_low_pct >= 2.0
                && hit.year_low_pct < 5.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpCloseAtHodHotVol => {
            hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownCloseAtLodHotVol => {
            hit.gap_pct < -2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpCloseAtLodHotVol => {
            hit.gap_pct > 2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownCloseAtHodHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpMidpointCloseHotVol => {
            hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownMidpointCloseHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::BigGapUpCloseAtHodHotVol => {
            hit.gap_pct > 5.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::BigGapDownCloseAtLodHotVol => {
            hit.gap_pct < -5.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -3.0
                && hit.rel_volume >= 2.0
        }
        Preset::BigGapUpCloseAtLodHotVol => {
            hit.gap_pct > 5.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < 0.0
                && hit.rel_volume >= 2.0
        }
        Preset::BigGapDownCloseAtHodHotVol => {
            hit.gap_pct < -5.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 0.0
                && hit.rel_volume >= 2.0
        }
        Preset::BigGapUpMidpointCloseHotVol => {
            hit.gap_pct > 5.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::BigGapDownMidpointCloseHotVol => {
            hit.gap_pct < -5.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.rel_volume >= 2.0
        }
        Preset::GapUpCloseAtHodNearYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.year_high_pct < 2.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownCloseAtLodNearYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.year_low_pct < 2.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpCloseAtHodDeepBelowYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.year_high_pct >= 20.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownCloseAtLodDeepAboveYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.year_low_pct >= 20.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpCloseAtHodConfirmedAboveYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.year_high_pct >= -3.0
                && hit.year_high_pct <= -1.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownCloseAtLodConfirmedBelowYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.year_low_pct >= -3.0
                && hit.year_low_pct <= -1.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpCloseAtHodJustOffYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.year_high_pct >= 2.0
                && hit.year_high_pct < 5.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownCloseAtLodJustOffYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.year_low_pct >= 2.0
                && hit.year_low_pct < 5.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpCloseAtHodMidYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownCloseAtLodMidYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpCloseAtLodNearYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.year_high_pct < 2.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownCloseAtHodNearYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.year_low_pct < 2.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpCloseAtLodConfirmedAboveYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.year_high_pct >= -3.0
                && hit.year_high_pct <= -1.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownCloseAtHodConfirmedBelowYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.year_low_pct >= -3.0
                && hit.year_low_pct <= -1.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpCloseAtLodDeepBelowYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.year_high_pct >= 20.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownCloseAtHodDeepAboveYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.year_low_pct >= 20.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpCloseAtLodJustOffYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.year_high_pct >= 2.0
                && hit.year_high_pct < 5.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownCloseAtHodJustOffYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.year_low_pct >= 2.0
                && hit.year_low_pct < 5.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpCloseAtLodMidYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownCloseAtHodMidYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpMidpointCloseNearYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.year_high_pct < 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownMidpointCloseNearYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.year_low_pct < 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpMidpointCloseConfirmedAboveYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.year_high_pct >= -3.0
                && hit.year_high_pct <= -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownMidpointCloseConfirmedBelowYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.year_low_pct >= -3.0
                && hit.year_low_pct <= -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpMidpointCloseDeepBelowYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.year_high_pct >= 20.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownMidpointCloseDeepAboveYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.year_low_pct >= 20.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpMidpointCloseMidYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownMidpointCloseMidYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpMidpointCloseJustOffYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.year_high_pct >= 2.0
                && hit.year_high_pct < 5.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownMidpointCloseJustOffYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.year_low_pct >= 2.0
                && hit.year_low_pct < 5.0
                && hit.rel_volume >= 1.5
        }
        Preset::HotVolFlatCloseNearYearHighHotVol => {
            hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
                && hit.year_high_pct < 2.0
        }
        Preset::HotVolFlatCloseNearYearLowHotVol => {
            hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
                && hit.year_low_pct < 2.0
        }
        Preset::HotVolFlatCloseConfirmedAboveYearHighHotVol => {
            hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
                && hit.year_high_pct >= -3.0
                && hit.year_high_pct <= -1.0
        }
        Preset::HotVolFlatCloseConfirmedBelowYearLowHotVol => {
            hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
                && hit.year_low_pct >= -3.0
                && hit.year_low_pct <= -1.0
        }
        Preset::HotVolFlatCloseDeepBelowYearHighHotVol => {
            hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
                && hit.year_high_pct >= 20.0
        }
        Preset::HotVolFlatCloseDeepAboveYearLowHotVol => {
            hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
                && hit.year_low_pct >= 20.0
        }
        Preset::HotVolFlatCloseMidYearHighHotVol => {
            hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
                && hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
        }
        Preset::HotVolFlatCloseMidYearLowHotVol => {
            hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
                && hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
        }
        Preset::HotVolFlatCloseJustOffYearHighHotVol => {
            hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
                && hit.year_high_pct >= 2.0
                && hit.year_high_pct < 5.0
        }
        Preset::HotVolFlatCloseJustOffYearLowHotVol => {
            hit.change_pct.abs() < 0.5
                && hit.rel_volume >= 2.0
                && hit.year_low_pct >= 2.0
                && hit.year_low_pct < 5.0
        }
        Preset::DryVolBigUpNearYearHighHotVol => {
            hit.change_pct > 3.0
                && hit.rel_volume < 0.5
                && hit.year_high_pct < 2.0
        }
        Preset::DryVolBigDownNearYearLowHotVol => {
            hit.change_pct < -3.0
                && hit.rel_volume < 0.5
                && hit.year_low_pct < 2.0
        }
        Preset::DryVolBigUpConfirmedAboveYearHighHotVol => {
            hit.change_pct > 3.0
                && hit.rel_volume < 0.5
                && hit.year_high_pct >= -3.0
                && hit.year_high_pct <= -1.0
        }
        Preset::DryVolBigDownConfirmedBelowYearLowHotVol => {
            hit.change_pct < -3.0
                && hit.rel_volume < 0.5
                && hit.year_low_pct >= -3.0
                && hit.year_low_pct <= -1.0
        }
        Preset::DryVolBigUpDeepBelowYearHighHotVol => {
            hit.change_pct > 3.0
                && hit.rel_volume < 0.5
                && hit.year_high_pct >= 20.0
        }
        Preset::DryVolBigDownDeepAboveYearLowHotVol => {
            hit.change_pct < -3.0
                && hit.rel_volume < 0.5
                && hit.year_low_pct >= 20.0
        }
        Preset::DryVolBigUpMidYearHighHotVol => {
            hit.change_pct > 3.0
                && hit.rel_volume < 0.5
                && hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
        }
        Preset::DryVolBigDownMidYearLowHotVol => {
            hit.change_pct < -3.0
                && hit.rel_volume < 0.5
                && hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
        }
        Preset::DryVolBigUpJustOffYearHighHotVol => {
            hit.change_pct > 3.0
                && hit.rel_volume < 0.5
                && hit.year_high_pct >= 2.0
                && hit.year_high_pct < 5.0
        }
        Preset::DryVolBigDownJustOffYearLowHotVol => {
            hit.change_pct < -3.0
                && hit.rel_volume < 0.5
                && hit.year_low_pct >= 2.0
                && hit.year_low_pct < 5.0
        }
        Preset::UltraDeepBelowYearHighHotVol => {
            hit.year_high_pct >= 50.0 && hit.rel_volume >= 1.5
        }
        Preset::UltraDeepAboveYearLowHotVol => {
            hit.year_low_pct >= 50.0 && hit.rel_volume >= 1.5
        }
        Preset::UltraDeepBelowYearHighCloseAtHodHotVol => {
            hit.year_high_pct >= 50.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::UltraDeepAboveYearLowCloseAtLodHotVol => {
            hit.year_low_pct >= 50.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::UltraDeepBelowYearHighGapUpHotVol => {
            hit.year_high_pct >= 50.0
                && hit.gap_pct > 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::UltraDeepAboveYearLowGapDownHotVol => {
            hit.year_low_pct >= 50.0
                && hit.gap_pct < -2.0
                && hit.rel_volume >= 1.5
        }
        Preset::UltraDeepBelowYearHighGapUpFadedHotVol => {
            hit.year_high_pct >= 50.0
                && hit.gap_pct > 2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::UltraDeepAboveYearLowGapDownAbsorbedHotVol => {
            hit.year_low_pct >= 50.0
                && hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::UltraDeepBelowYearHighGapUpHeldHotVol => {
            hit.year_high_pct >= 50.0
                && hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::UltraDeepAboveYearLowGapDownHeldHotVol => {
            hit.year_low_pct >= 50.0
                && hit.gap_pct < -2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::UltraDeepBelowYearHighGapUpMidpointHotVol => {
            hit.year_high_pct >= 50.0
                && hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::UltraDeepAboveYearLowGapDownMidpointHotVol => {
            hit.year_low_pct >= 50.0
                && hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
                && hit.rel_volume >= 1.5
        }
        Preset::UltraDeepBelowYearHighHammerHotVol => {
            hit.year_high_pct >= 50.0
                && hit.lod_dist_pct.abs() > 3.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::UltraDeepAboveYearLowShootingStarHotVol => {
            hit.year_low_pct >= 50.0
                && hit.hod_dist_pct.abs() > 3.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::UltraDeepBelowYearHighShootingStarHotVol => {
            hit.year_high_pct >= 50.0
                && hit.hod_dist_pct.abs() > 3.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::UltraDeepAboveYearLowHammerHotVol => {
            hit.year_low_pct >= 50.0
                && hit.lod_dist_pct.abs() > 3.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 0.0
                && hit.rel_volume >= 1.5
        }
        Preset::WideYearRangeHotVol => {
            hit.year_high_pct >= 20.0 && hit.year_low_pct >= 20.0 && hit.rel_volume >= 1.5
        }
        Preset::NarrowYearRangeHotVol => {
            hit.year_high_pct < 5.0 && hit.year_low_pct < 5.0 && hit.rel_volume >= 1.5
        }
        Preset::NarrowYearRangeCloseAtHodHotVol => {
            hit.year_high_pct < 5.0
                && hit.year_low_pct < 5.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::NarrowYearRangeCloseAtLodHotVol => {
            hit.year_high_pct < 5.0
                && hit.year_low_pct < 5.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::NarrowYearRangeGapUpHotVol => {
            hit.year_high_pct < 5.0
                && hit.year_low_pct < 5.0
                && hit.gap_pct > 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::NarrowYearRangeGapDownHotVol => {
            hit.year_high_pct < 5.0
                && hit.year_low_pct < 5.0
                && hit.gap_pct < -2.0
                && hit.rel_volume >= 1.5
        }
        Preset::NarrowYearRangeBigUpHotVol => {
            hit.year_high_pct < 5.0
                && hit.year_low_pct < 5.0
                && hit.change_pct > 3.0
                && hit.rel_volume >= 2.0
        }
        Preset::NarrowYearRangeBigDownHotVol => {
            hit.year_high_pct < 5.0
                && hit.year_low_pct < 5.0
                && hit.change_pct < -3.0
                && hit.rel_volume >= 2.0
        }
        Preset::WideYearRangeCloseAtHodHotVol => {
            hit.year_high_pct >= 20.0
                && hit.year_low_pct >= 20.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::WideYearRangeCloseAtLodHotVol => {
            hit.year_high_pct >= 20.0
                && hit.year_low_pct >= 20.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::WideYearRangeGapUpHotVol => {
            hit.year_high_pct >= 20.0
                && hit.year_low_pct >= 20.0
                && hit.gap_pct > 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::WideYearRangeGapDownHotVol => {
            hit.year_high_pct >= 20.0
                && hit.year_low_pct >= 20.0
                && hit.gap_pct < -2.0
                && hit.rel_volume >= 1.5
        }
        Preset::AsymmetricRangeNearLowFarHighHotVol => {
            hit.year_high_pct >= 20.0 && hit.year_low_pct < 5.0 && hit.rel_volume >= 1.5
        }
        Preset::AsymmetricRangeNearHighFarLowHotVol => {
            hit.year_high_pct < 5.0 && hit.year_low_pct >= 20.0 && hit.rel_volume >= 1.5
        }
        Preset::AsymmetricRangeNearLowFarHighCloseAtLodHotVol => {
            hit.year_high_pct >= 20.0
                && hit.year_low_pct < 5.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::AsymmetricRangeNearHighFarLowCloseAtHodHotVol => {
            hit.year_high_pct < 5.0
                && hit.year_low_pct >= 20.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::AsymmetricRangeNearLowFarHighCloseAtHodHotVol => {
            hit.year_high_pct >= 20.0
                && hit.year_low_pct < 5.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::AsymmetricRangeNearHighFarLowCloseAtLodHotVol => {
            hit.year_high_pct < 5.0
                && hit.year_low_pct >= 20.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::AsymmetricRangeNearLowFarHighGapUpHotVol => {
            hit.year_high_pct >= 20.0
                && hit.year_low_pct < 5.0
                && hit.gap_pct > 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::AsymmetricRangeNearHighFarLowGapDownHotVol => {
            hit.year_high_pct < 5.0
                && hit.year_low_pct >= 20.0
                && hit.gap_pct < -2.0
                && hit.rel_volume >= 1.5
        }
        Preset::AsymmetricRangeNearLowFarHighGapDownHotVol => {
            hit.year_high_pct >= 20.0
                && hit.year_low_pct < 5.0
                && hit.gap_pct < -2.0
                && hit.rel_volume >= 1.5
        }
        Preset::AsymmetricRangeNearHighFarLowGapUpHotVol => {
            hit.year_high_pct < 5.0
                && hit.year_low_pct >= 20.0
                && hit.gap_pct > 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpFlatDayHotVol => {
            hit.gap_pct > 2.0 && hit.day_pct.abs() < 0.5 && hit.rel_volume >= 1.5
        }
        Preset::GapDownFlatDayHotVol => {
            hit.gap_pct < -2.0 && hit.day_pct.abs() < 0.5 && hit.rel_volume >= 1.5
        }
        Preset::GapUpBigDayHotVol => {
            hit.gap_pct > 2.0 && hit.day_pct > 2.0 && hit.rel_volume >= 1.5
        }
        Preset::GapDownBigDayHotVol => {
            hit.gap_pct < -2.0 && hit.day_pct < -2.0 && hit.rel_volume >= 1.5
        }
        Preset::GapUpBigDayDownHotVol => {
            hit.gap_pct > 2.0 && hit.day_pct < -2.0 && hit.rel_volume >= 1.5
        }
        Preset::GapDownBigDayUpHotVol => {
            hit.gap_pct < -2.0 && hit.day_pct > 2.0 && hit.rel_volume >= 1.5
        }
        Preset::SmallGapBigDayUpHotVol => {
            hit.gap_pct.abs() < 0.5 && hit.day_pct > 3.0 && hit.rel_volume >= 1.5
        }
        Preset::SmallGapBigDayDownHotVol => {
            hit.gap_pct.abs() < 0.5 && hit.day_pct < -3.0 && hit.rel_volume >= 1.5
        }
        Preset::SmallGapBigDayUpNearYearHighHotVol => {
            hit.gap_pct.abs() < 0.5
                && hit.day_pct > 3.0
                && hit.year_high_pct < 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::SmallGapBigDayDownNearYearLowHotVol => {
            hit.gap_pct.abs() < 0.5
                && hit.day_pct < -3.0
                && hit.year_low_pct < 2.0
                && hit.rel_volume >= 1.5
        }
        Preset::SmallGapBigDayUpConfirmedAboveYearHighHotVol => {
            hit.gap_pct.abs() < 0.5
                && hit.day_pct > 3.0
                && hit.year_high_pct >= -3.0
                && hit.year_high_pct <= -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::SmallGapBigDayDownConfirmedBelowYearLowHotVol => {
            hit.gap_pct.abs() < 0.5
                && hit.day_pct < -3.0
                && hit.year_low_pct >= -3.0
                && hit.year_low_pct <= -1.0
                && hit.rel_volume >= 1.5
        }
        Preset::SmallGapBigDayUpDeepBelowYearHighHotVol => {
            hit.gap_pct.abs() < 0.5
                && hit.day_pct > 3.0
                && hit.year_high_pct >= 20.0
                && hit.rel_volume >= 1.5
        }
        Preset::SmallGapBigDayDownDeepAboveYearLowHotVol => {
            hit.gap_pct.abs() < 0.5
                && hit.day_pct < -3.0
                && hit.year_low_pct >= 20.0
                && hit.rel_volume >= 1.5
        }
        Preset::SmallGapBigDayUpMidYearHighHotVol => {
            hit.gap_pct.abs() < 0.5
                && hit.day_pct > 3.0
                && hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
                && hit.rel_volume >= 1.5
        }
        Preset::SmallGapBigDayDownMidYearLowHotVol => {
            hit.gap_pct.abs() < 0.5
                && hit.day_pct < -3.0
                && hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
                && hit.rel_volume >= 1.5
        }
        Preset::SmallGapBigDayUpJustOffYearHighHotVol => {
            hit.gap_pct.abs() < 0.5
                && hit.day_pct > 3.0
                && hit.year_high_pct >= 2.0
                && hit.year_high_pct < 5.0
                && hit.rel_volume >= 1.5
        }
        Preset::SmallGapBigDayDownJustOffYearLowHotVol => {
            hit.gap_pct.abs() < 0.5
                && hit.day_pct < -3.0
                && hit.year_low_pct >= 2.0
                && hit.year_low_pct < 5.0
                && hit.rel_volume >= 1.5
        }
        Preset::BigUpDayCloseAtHodHotVol => {
            hit.day_pct > 3.0 && hit.hod_dist_pct.abs() < 0.5 && hit.rel_volume >= 1.5
        }
        Preset::BigDownDayCloseAtLodHotVol => {
            hit.day_pct < -3.0 && hit.lod_dist_pct.abs() < 0.5 && hit.rel_volume >= 1.5
        }
        Preset::BigUpDayDoubledVolHotVol => {
            hit.day_pct > 3.0 && hit.rel_volume >= 2.0
        }
        Preset::BigDownDayDoubledVolHotVol => {
            hit.day_pct < -3.0 && hit.rel_volume >= 2.0
        }
        Preset::BigUpDayDoubledVolNearYearHighHotVol => {
            hit.day_pct > 3.0 && hit.rel_volume >= 2.0 && hit.year_high_pct < 2.0
        }
        Preset::BigDownDayDoubledVolNearYearLowHotVol => {
            hit.day_pct < -3.0 && hit.rel_volume >= 2.0 && hit.year_low_pct < 2.0
        }
        Preset::BigUpDayDoubledVolConfirmedAboveYearHighHotVol => {
            hit.day_pct > 3.0
                && hit.rel_volume >= 2.0
                && hit.year_high_pct >= -3.0
                && hit.year_high_pct <= -1.0
        }
        Preset::BigDownDayDoubledVolConfirmedBelowYearLowHotVol => {
            hit.day_pct < -3.0
                && hit.rel_volume >= 2.0
                && hit.year_low_pct >= -3.0
                && hit.year_low_pct <= -1.0
        }
        Preset::BigUpDayDoubledVolDeepBelowYearHighHotVol => {
            hit.day_pct > 3.0 && hit.rel_volume >= 2.0 && hit.year_high_pct >= 20.0
        }
        Preset::BigDownDayDoubledVolDeepAboveYearLowHotVol => {
            hit.day_pct < -3.0 && hit.rel_volume >= 2.0 && hit.year_low_pct >= 20.0
        }
        Preset::BigUpDayDoubledVolMidYearHighHotVol => {
            hit.day_pct > 3.0
                && hit.rel_volume >= 2.0
                && hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
        }
        Preset::BigDownDayDoubledVolMidYearLowHotVol => {
            hit.day_pct < -3.0
                && hit.rel_volume >= 2.0
                && hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
        }
        Preset::BigUpDayDoubledVolJustOffYearHighHotVol => {
            hit.day_pct > 3.0
                && hit.rel_volume >= 2.0
                && hit.year_high_pct >= 2.0
                && hit.year_high_pct < 5.0
        }
        Preset::BigDownDayDoubledVolJustOffYearLowHotVol => {
            hit.day_pct < -3.0
                && hit.rel_volume >= 2.0
                && hit.year_low_pct >= 2.0
                && hit.year_low_pct < 5.0
        }
        Preset::QuintupledVolUpHotVol => {
            hit.rel_volume >= 5.0 && hit.change_pct > 3.0
        }
        Preset::QuintupledVolDownHotVol => {
            hit.rel_volume >= 5.0 && hit.change_pct < -3.0
        }
        Preset::QuintupledVolUpNearYearHighHotVol => {
            hit.rel_volume >= 5.0 && hit.change_pct > 3.0 && hit.year_high_pct < 2.0
        }
        Preset::QuintupledVolDownNearYearLowHotVol => {
            hit.rel_volume >= 5.0 && hit.change_pct < -3.0 && hit.year_low_pct < 2.0
        }
        Preset::QuintupledVolUpDeepBelowYearHighHotVol => {
            hit.rel_volume >= 5.0 && hit.change_pct > 3.0 && hit.year_high_pct >= 20.0
        }
        Preset::QuintupledVolDownDeepAboveYearLowHotVol => {
            hit.rel_volume >= 5.0 && hit.change_pct < -3.0 && hit.year_low_pct >= 20.0
        }
        Preset::QuintupledVolUpConfirmedAboveYearHighHotVol => {
            hit.rel_volume >= 5.0
                && hit.change_pct > 3.0
                && hit.year_high_pct >= -3.0
                && hit.year_high_pct <= -1.0
        }
        Preset::QuintupledVolDownConfirmedBelowYearLowHotVol => {
            hit.rel_volume >= 5.0
                && hit.change_pct < -3.0
                && hit.year_low_pct >= -3.0
                && hit.year_low_pct <= -1.0
        }
        Preset::QuintupledVolUpMidYearHighHotVol => {
            hit.rel_volume >= 5.0
                && hit.change_pct > 3.0
                && hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
        }
        Preset::QuintupledVolDownMidYearLowHotVol => {
            hit.rel_volume >= 5.0
                && hit.change_pct < -3.0
                && hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
        }
        Preset::QuintupledVolUpJustOffYearHighHotVol => {
            hit.rel_volume >= 5.0
                && hit.change_pct > 3.0
                && hit.year_high_pct >= 2.0
                && hit.year_high_pct < 5.0
        }
        Preset::QuintupledVolDownJustOffYearLowHotVol => {
            hit.rel_volume >= 5.0
                && hit.change_pct < -3.0
                && hit.year_low_pct >= 2.0
                && hit.year_low_pct < 5.0
        }
        Preset::QuintupledVolCloseAtHodHotVol => {
            hit.rel_volume >= 5.0 && hit.hod_dist_pct.abs() < 0.5 && hit.change_pct > 1.0
        }
        Preset::QuintupledVolCloseAtLodHotVol => {
            hit.rel_volume >= 5.0 && hit.lod_dist_pct.abs() < 0.5 && hit.change_pct < -1.0
        }
        Preset::QuintupledVolGapUpHotVol => {
            hit.rel_volume >= 5.0 && hit.gap_pct > 2.0
        }
        Preset::QuintupledVolGapDownHotVol => {
            hit.rel_volume >= 5.0 && hit.gap_pct < -2.0
        }
        Preset::QuintupledVolGapUpCloseAtHodHotVol => {
            hit.rel_volume >= 5.0
                && hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 1.0
        }
        Preset::QuintupledVolGapDownCloseAtLodHotVol => {
            hit.rel_volume >= 5.0
                && hit.gap_pct < -2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < -1.0
        }
        Preset::QuintupledVolGapUpCloseAtLodHotVol => {
            hit.rel_volume >= 5.0
                && hit.gap_pct > 2.0
                && hit.lod_dist_pct.abs() < 0.5
                && hit.change_pct < 0.0
        }
        Preset::QuintupledVolGapDownCloseAtHodHotVol => {
            hit.rel_volume >= 5.0
                && hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() < 0.5
                && hit.change_pct > 0.0
        }
        Preset::QuintupledVolGapUpMidpointHotVol => {
            hit.rel_volume >= 5.0
                && hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
        }
        Preset::QuintupledVolGapDownMidpointHotVol => {
            hit.rel_volume >= 5.0
                && hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() > 0.5
                && hit.lod_dist_pct.abs() > 0.5
                && (hit.hod_dist_pct.abs() - hit.lod_dist_pct.abs()).abs() < 0.5
        }
        Preset::BigIntradayRangeHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0 && hit.rel_volume >= 1.5
        }
        Preset::TightIntradayRangeHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0 && hit.rel_volume >= 1.5
        }
        Preset::BigIntradayRangeNearYearHighHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 1.5
                && hit.year_high_pct < 2.0
        }
        Preset::BigIntradayRangeNearYearLowHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 1.5
                && hit.year_low_pct < 2.0
        }
        Preset::BigIntradayRangeConfirmedAboveYearHighHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 1.5
                && hit.year_high_pct >= -3.0
                && hit.year_high_pct <= -1.0
        }
        Preset::BigIntradayRangeConfirmedBelowYearLowHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 1.5
                && hit.year_low_pct >= -3.0
                && hit.year_low_pct <= -1.0
        }
        Preset::BigIntradayRangeDeepBelowYearHighHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 1.5
                && hit.year_high_pct >= 20.0
        }
        Preset::BigIntradayRangeDeepAboveYearLowHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 1.5
                && hit.year_low_pct >= 20.0
        }
        Preset::BigIntradayRangeMidYearHighHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 1.5
                && hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
        }
        Preset::BigIntradayRangeMidYearLowHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 1.5
                && hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
        }
        Preset::BigIntradayRangeJustOffYearHighHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 1.5
                && hit.year_high_pct >= 2.0
                && hit.year_high_pct < 5.0
        }
        Preset::BigIntradayRangeJustOffYearLowHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 1.5
                && hit.year_low_pct >= 2.0
                && hit.year_low_pct < 5.0
        }
        Preset::TightIntradayRangeNearYearHighHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
                && hit.year_high_pct < 2.0
        }
        Preset::TightIntradayRangeNearYearLowHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
                && hit.year_low_pct < 2.0
        }
        Preset::TightIntradayRangeConfirmedAboveYearHighHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
                && hit.year_high_pct >= -3.0
                && hit.year_high_pct <= -1.0
        }
        Preset::TightIntradayRangeConfirmedBelowYearLowHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
                && hit.year_low_pct >= -3.0
                && hit.year_low_pct <= -1.0
        }
        Preset::TightIntradayRangeDeepBelowYearHighHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
                && hit.year_high_pct >= 20.0
        }
        Preset::TightIntradayRangeDeepAboveYearLowHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
                && hit.year_low_pct >= 20.0
        }
        Preset::TightIntradayRangeMidYearHighHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
                && hit.year_high_pct >= 5.0
                && hit.year_high_pct < 20.0
        }
        Preset::TightIntradayRangeMidYearLowHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
                && hit.year_low_pct >= 5.0
                && hit.year_low_pct < 20.0
        }
        Preset::TightIntradayRangeJustOffYearHighHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
                && hit.year_high_pct >= 2.0
                && hit.year_high_pct < 5.0
        }
        Preset::TightIntradayRangeJustOffYearLowHotVol => {
            hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
                && hit.year_low_pct >= 2.0
                && hit.year_low_pct < 5.0
        }
        Preset::GapUpTightRangeHotVol => {
            hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownTightRangeHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() < 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpWideRangeHotVol => {
            hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapDownWideRangeHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapUpWideRangeNearYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 1.5
                && hit.year_high_pct < 2.0
        }
        Preset::GapDownWideRangeNearYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 1.5
                && hit.year_low_pct < 2.0
        }
        Preset::GapUpWideRangeConfirmedAboveYearHighHotVol => {
            hit.gap_pct > 2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 1.5
                && hit.year_high_pct >= -3.0
                && hit.year_high_pct <= -1.0
        }
        Preset::GapDownWideRangeConfirmedBelowYearLowHotVol => {
            hit.gap_pct < -2.0
                && hit.hod_dist_pct.abs() + hit.lod_dist_pct.abs() > 8.0
                && hit.rel_volume >= 1.5
                && hit.year_low_pct >= -3.0
                && hit.year_low_pct <= -1.0
        }
    }
}

pub fn preset_label(p: Preset) -> &'static str {
    match p {
        Preset::PremarketGappers => "Gappers",
        Preset::MomentumMovers => "Momentum",
        Preset::HighOfDay => "High of Day",
        Preset::LowFloatRunners => "Low-Float Runner",
        Preset::Pct52wHigh => "52w High",
        Preset::Pct52wLow => "52w Low",
        Preset::VolumeSurge => "Volume Surge",
        Preset::Breakdown => "Breakdown",
        Preset::Breakout => "Breakout",
        Preset::OversoldBounce => "Oversold Bounce",
        Preset::GapAndGo => "Gap & Go",
        Preset::GapAndFade => "Gap & Fade",
        Preset::InsideDayLow => "Near Day Low",
        Preset::InsideDayHigh => "Coiling at HOD",
        Preset::RangeContractionDay => "Range Contraction",
        Preset::DistributionDay => "Distribution Day",
        Preset::AccumulationDay => "Accumulation Day",
        Preset::NearYearHighLowVol => "52w High, No Volume",
        Preset::InsideDaySqueeze => "Inside-Day Squeeze",
        Preset::LowVolSqueeze => "Low-Volume Squeeze",
        Preset::CoilingSqueeze => "Coiling Squeeze",
        Preset::MidRangeSqueeze => "Mid-Range Squeeze",
        Preset::BracketSqueeze => "Bracket Squeeze",
        Preset::DojiSqueeze => "Doji Squeeze",
        Preset::GapFillSqueeze => "Gap-Fill Squeeze",
        Preset::EndOfRangeSqueeze => "End-of-Range Squeeze",
        Preset::PreBreakoutSqueeze => "Pre-Breakout Squeeze",
        Preset::PreBreakdownSqueeze => "Pre-Breakdown Squeeze",
        Preset::SymmetricSqueeze => "Symmetric Squeeze",
        Preset::OpenCloseSqueeze => "Open=Close Squeeze",
        Preset::TightHodSqueeze => "Tight-HOD Squeeze",
        Preset::TightLodSqueeze => "Tight-LOD Squeeze",
        Preset::NoGapNoChangeSqueeze => "No-Gap-No-Change Squeeze",
        Preset::QuietTickSqueeze => "Quiet-Tick Squeeze",
        Preset::NarrowGapPostMomentum => "Post-Momentum Squeeze",
        Preset::DistantExtremesSqueeze => "Distant-Extremes Squeeze",
        Preset::BalancedDriftSqueeze => "Balanced-Drift Squeeze",
        Preset::PennyMoveSqueeze => "Penny-Move Squeeze",
        Preset::DryUpSqueeze => "Dry-Up Squeeze",
        Preset::UpperRangeSqueeze => "Upper-Range Squeeze",
        Preset::LowerRangeSqueeze => "Lower-Range Squeeze",
        Preset::GapReversalSqueeze => "Gap-Reversal Squeeze",
        Preset::Pct52wMidSqueeze => "52w-Mid Squeeze",
        Preset::DeepDiscountSqueeze => "Deep-Discount Squeeze",
        Preset::FlatRangeQuietSqueeze => "Flat-Range Quiet Squeeze",
        Preset::NearAthQuietSqueeze => "Near-ATH Quiet Squeeze",
        Preset::NearAtlQuietSqueeze => "Near-ATL Quiet Squeeze",
        Preset::SilentBreakoutSetup => "Silent Breakout Setup",
        Preset::SilentBreakdownSetup => "Silent Breakdown Setup",
        Preset::GapDownNoFollowSqueeze => "Gap-Down No-Follow Squeeze",
        Preset::GapUpNoFollowSqueeze => "Gap-Up No-Follow Squeeze",
        Preset::UnchVolDryUpSqueeze => "Unch-Vol Dry-Up Squeeze",
        Preset::NarrowAfterTrendSqueeze => "Narrow-After-Trend Squeeze",
        Preset::DeadCenterSqueeze => "Dead-Center Squeeze",
        Preset::AnchorDriftSqueeze => "Anchor-Drift Squeeze",
        Preset::PostGapFillSqueeze => "Post-Gap-Fill Squeeze",
        Preset::PostSpikeQuietSqueeze => "Post-Spike Quiet Squeeze",
        Preset::HighSqueezeBracket => "High-Squeeze Bracket",
        Preset::LowSqueezeBracket => "Low-Squeeze Bracket",
        Preset::HighRelVolStallSqueeze => "High-Vol Stall Squeeze",
        Preset::SlightLeanLongSqueeze => "Slight-Lean Long Squeeze",
        Preset::SlightLeanShortSqueeze => "Slight-Lean Short Squeeze",
        Preset::GapWithChangeMatchSqueeze => "Gap+Change Match Squeeze",
        Preset::SlackBetweenExtremesSqueeze => "Wide-Range No-Decision Squeeze",
        Preset::PivotPinSqueeze => "Pivot-Pin Squeeze",
        Preset::EvenSidesSqueeze => "Even-Sides Squeeze",
        Preset::InsideQuarterDaySqueeze => "Quarter-Day Inside Squeeze",
        Preset::EvenVolumeQuietSqueeze => "Even-Volume Quiet Squeeze",
        Preset::TightCoilHighSqueeze => "Tight Coil at 52w-High",
        Preset::TightCoilLowSqueeze => "Tight Coil at 52w-Low",
        Preset::EvenWidthSqueeze => "Even-Width Squeeze",
        Preset::SmallGapNoFollowSqueeze => "Small-Gap No-Follow Squeeze",
        Preset::HoldingHighsSqueeze => "Holding-Highs Squeeze",
        Preset::HoldingLowsSqueeze => "Holding-Lows Squeeze",
        Preset::StableMidSqueeze => "Stable-Mid Squeeze",
        Preset::LeanGapMatchSqueeze => "Lean-Gap Match Squeeze",
        Preset::LongShadowQuietSqueeze => "Long-Shadow Quiet Squeeze",
        Preset::ChangeNoDayPctSqueeze => "Overnight Move Reset Squeeze",
        Preset::DayPctNoChangeSqueeze => "Intraday Wiggle Reset Squeeze",
        Preset::HotDryUpSqueeze => "Hot Dry-Up Squeeze",
        Preset::ColdDryUpSqueeze => "Cold Dry-Up Squeeze",
        Preset::HighVolGapFadeSqueeze => "High-Vol Gap-Fade Squeeze",
        Preset::NearZeroChangeQuietSqueeze => "Chop-And-Rest Quiet Squeeze",
        Preset::SilentInsideSqueeze => "Silent Inside Squeeze",
        Preset::HighVolNoMoveSqueeze => "Heavy-Vol No-Move Squeeze",
        Preset::ChangeButLodNearbySqueeze => "Up-Day-Failed Squeeze",
        Preset::ChangeButHodNearbySqueeze => "Down-Day-Reversed Squeeze",
        Preset::GapAndCloseAtHodSqueeze => "Gap-Up Hold-HOD Squeeze",
        Preset::GapAndCloseAtLodSqueeze => "Gap-Down Hold-LOD Squeeze",
        Preset::LongInsideQuietSqueeze => "Long-Inside Quiet Squeeze",
        Preset::TripleZeroSqueeze => "Triple-Zero Squeeze",
        Preset::Pct52wQuarterFromHighSqueeze => "Quarter-From-High Squeeze",
        Preset::Pct52wQuarterFromLowSqueeze => "Quarter-From-Low Squeeze",
        Preset::NoExtremeAndQuietSqueeze => "Away-From-Extremes Quiet Squeeze",
        Preset::SmallChangeNarrowGapSqueeze => "Small-Change Narrow-Gap Squeeze",
        Preset::BigRangeNoCommitSqueeze => "Big-Range No-Commit Squeeze",
        Preset::EvenSwingSqueeze => "Even-Swing Squeeze",
        Preset::NoMoveAtMidSqueeze => "No-Move-At-Mid Squeeze",
        Preset::BarelyMovingHighSqueeze => "Barely-Moving High Squeeze",
        Preset::BarelyMovingLowSqueeze => "Barely-Moving Low Squeeze",
        Preset::MicroRangeSqueeze => "Micro-Range Squeeze",
        Preset::LowVolGapHoldSqueeze => "Low-Vol Gap-Hold Squeeze",
        Preset::HighVolGapHoldSqueeze => "High-Vol Gap-Hold Squeeze",
        Preset::UpsideAttemptedSqueeze => "Upside-Attempted Reject Squeeze",
        Preset::DownsideAttemptedSqueeze => "Downside-Attempted Reject Squeeze",
        Preset::TightGapSmallChangeSqueeze => "Tight-Gap Small-Change Squeeze",
        Preset::Pct52wMidWideRangeSqueeze => "52w-Mid Wide-Range Squeeze",
        Preset::InsideAndCoiledSqueeze => "Inside-And-Coiled Squeeze",
        Preset::Pct52wHighBreathSqueeze => "52w-High Breath Squeeze",
        Preset::Pct52wLowBreathSqueeze => "52w-Low Breath Squeeze",
        Preset::GapAroundCloseSqueeze => "Gap-Around-Close Squeeze",
        Preset::TightCloseSplitSqueeze => "Tight-Close-Split Squeeze",
        Preset::HiVolNoExtremeSqueeze => "Hi-Vol No-Extreme Squeeze",
        Preset::TinyMoveWithGapSqueeze => "Tiny-Move With-Gap Squeeze",
        Preset::LowVolatilityGreenSqueeze => "Low-Vol Green Squeeze",
        Preset::LowVolatilityRedSqueeze => "Low-Vol Red Squeeze",
        Preset::GapAlignsChangeSqueeze => "Gap-Aligns-Change Squeeze",
        Preset::UnaffectedGapSqueeze => "Unaffected-Gap Squeeze",
        Preset::StackedClosesSqueeze => "Stacked-Closes Squeeze",
        Preset::PullbackToMidSqueeze => "Pullback-To-Mid Squeeze",
        Preset::BounceFromMidSqueeze => "Bounce-From-Mid Squeeze",
        Preset::NarrowGapHotCloseSqueeze => "Narrow-Gap Hot-Close Squeeze",
        Preset::NarrowGapColdCloseSqueeze => "Narrow-Gap Cold-Close Squeeze",
        Preset::AbsorptionUpSqueeze => "Absorption-Up Squeeze",
        Preset::AbsorptionDownSqueeze => "Absorption-Down Squeeze",
        Preset::StallAtMidSqueeze => "Stall-At-Mid Squeeze",
        Preset::NoCloseDecisionSqueeze => "No-Close-Decision Squeeze",
        Preset::GapInsideRangeSqueeze => "Gap-Inside-Range Squeeze",
        Preset::SubpointMoveSqueeze => "Sub-Point Move Squeeze",
        Preset::NoVolNoMoveSqueeze => "No-Vol No-Move Squeeze",
        Preset::VolWithoutChangeSqueeze => "Vol-Without-Change Squeeze",
        Preset::TickInsideOpenSqueeze => "Tick-Inside-Open Squeeze",
        Preset::Pct52wExactHalfSqueeze => "52w Exact-Half Squeeze",
        Preset::UnchangedOnVolumeSqueeze => "Unchanged-On-Volume Squeeze",
        Preset::WideHodNarrowLodSqueeze => "Wide-HOD Narrow-LOD Squeeze",
        Preset::NarrowHodWideLodSqueeze => "Narrow-HOD Wide-LOD Squeeze",
        Preset::PerfectBalanceSqueeze => "Perfect-Balance Squeeze",
        Preset::LowVolHotZoneSqueeze => "Low-Vol Hot-Zone Squeeze",
        Preset::LowVolColdZoneSqueeze => "Low-Vol Cold-Zone Squeeze",
        Preset::DriftHigherSqueeze => "Drift-Higher Squeeze",
        Preset::DriftLowerSqueeze => "Drift-Lower Squeeze",
        Preset::ExtremeQuietSqueeze => "Extreme-Quiet Squeeze",
        Preset::PinnedToOpenSqueeze => "Pinned-To-Open Squeeze",
        Preset::BigGapSmallDaySqueeze => "Big-Gap Small-Day Squeeze",
        Preset::PostCrashSqueeze => "Post-Crash Stabilize Squeeze",
        Preset::PostSpikeStabilizeSqueeze => "Post-Spike Stabilize Squeeze",
        Preset::TightWithSmallGapSqueeze => "Tight With-Small-Gap Squeeze",
        Preset::BigVolWithTinyChangeSqueeze => "Big-Vol Tiny-Change Squeeze",
        Preset::QuietExpansionSqueeze => "Quiet-Expansion Squeeze",
        Preset::InsideBarHighSqueeze => "Inside-Bar At 52w-High",
        Preset::InsideBarLowSqueeze => "Inside-Bar At 52w-Low",
        Preset::FlatGapInsideRangeSqueeze => "Flat-Gap Inside-Range Squeeze",
        Preset::Pct52wEdgeDryUp => "52w-Edge Dry-Up Squeeze",
        Preset::NarrowCenterSqueeze => "Narrow-Center Squeeze",
        Preset::LopsidedQuietSqueeze => "Lopsided Quiet Squeeze",
        Preset::SilentLeaderSqueeze => "Silent-Leader Squeeze",
        Preset::SilentLaggardSqueeze => "Silent-Laggard Squeeze",
        Preset::NearVwapQuietSqueeze => "Near-VWAP Quiet Squeeze",
        Preset::BarelyMovingMidSqueeze => "Barely-Moving Mid-Range Squeeze",
        Preset::Pct52wThirdFromHighSqueeze => "Third-From-High Squeeze",
        Preset::Pct52wThirdFromLowSqueeze => "Third-From-Low Squeeze",
        Preset::HighRangeNoChangeSqueeze => "High-Range No-Change Squeeze",
        Preset::LowRangeNoChangeSqueeze => "Low-Range No-Change Squeeze",
        Preset::LowVolumeUpDaySqueeze => "Low-Volume Up-Day Squeeze",
        Preset::LowVolumeDownDaySqueeze => "Low-Volume Down-Day Squeeze",
        Preset::HighVolumeUpDayNoExtreme => "Hi-Vol Up-Day No-HOD Squeeze",
        Preset::HighVolumeDownDayNoExtreme => "Hi-Vol Down-Day No-LOD Squeeze",
        Preset::GapUpFadeToFlat => "Gap-Up Fade To Flat",
        Preset::GapDownReclaimToFlat => "Gap-Down Reclaim To Flat",
        Preset::GapUpHeldGreen => "Gap-Up Held + Extended",
        Preset::GapDownHeldRed => "Gap-Down Held + Extended",
        Preset::GapUpHalfFade => "Gap-Up Half Fade",
        Preset::GapDownHalfReclaim => "Gap-Down Half Reclaim",
        Preset::GapAndGoXl => "Gap And Go XL",
        Preset::GapAndCrashXl => "Gap And Crash XL",
        Preset::GapUpButDayRed => "Gap-Up But Day Red",
        Preset::GapDownButDayGreen => "Gap-Down But Day Green",
        Preset::GapUpFlushOnVolume => "Gap-Up Flush On Volume",
        Preset::GapDownReversalOnVolume => "Gap-Down Reversal On Volume",
        Preset::Pct52wTopDecileHotVol => "52w Top-Decile on Hot Vol",
        Preset::Pct52wBottomDecileHotVol => "52w Bottom-Decile on Hot Vol",
        Preset::Pct52wTopDecileDryVol => "52w Top-Decile on Dry Vol",
        Preset::Pct52wBottomDecileDryVol => "52w Bottom-Decile on Dry Vol",
        Preset::NewHighGreenDay => "New 52w High + Green Day",
        Preset::NewLowRedDay => "New 52w Low + Red Day",
        Preset::NewHighRedDay => "New 52w High + Red Day",
        Preset::NewLowGreenDay => "New 52w Low + Green Day",
        Preset::NewHighOnHotVol => "New 52w High on Hot Vol",
        Preset::NewLowOnHotVol => "New 52w Low on Hot Vol",
        Preset::QuietNearTheTop => "Quiet Near The Top",
        Preset::QuietNearTheBottom => "Quiet Near The Bottom",
        Preset::NoisyNearTheTop => "Noisy Near The Top",
        Preset::NoisyNearTheBottom => "Noisy Near The Bottom",
        Preset::MidRangeChopHotVol => "Mid-Range Chop on Hot Vol",
        Preset::MidRangeChopDryVol => "Mid-Range Chop on Dry Vol",
        Preset::CloseNearHodNoBreakout => "Close Near HOD, No Breakout",
        Preset::CloseNearLodNoBreakdown => "Close Near LOD, No Breakdown",
        Preset::CloseNearHodStrongDay => "Close @ HOD, Strong Day",
        Preset::CloseNearLodWeakDay => "Close @ LOD, Weak Day",
        Preset::InsideRangeNoVolume => "Inside Range, No Volume",
        Preset::OutsideRangeOnVolume => "Outside Range, On Volume",
        Preset::UpDayLowerHigh => "Up Day But Capped HOD",
        Preset::DownDayHigherLow => "Down Day But Cushioned LOD",
        Preset::StrongDayBalancedRange => "Strong Day, Balanced Range",
        Preset::WeakDayBalancedRange => "Weak Day, Balanced Range",
        Preset::ChannelRideUp => "Channel Ride Up",
        Preset::ChannelRideDown => "Channel Ride Down",
        Preset::PullbackInUptrend => "Pullback In Uptrend",
        Preset::BounceInDowntrend => "Bounce In Downtrend",
        Preset::DeepPullbackInUptrend => "Deep Pullback In Uptrend",
        Preset::DeepBounceInDowntrend => "Deep Bounce In Downtrend",
        Preset::TightAboveMidStrong => "Tight Above Mid, Strong",
        Preset::TightBelowMidWeak => "Tight Below Mid, Weak",
        Preset::HotVolNoMoveAtHigh => "Hot Vol, No Move at 52w High",
        Preset::HotVolNoMoveAtLow => "Hot Vol, No Move at 52w Low",
        Preset::BigUpGapInsideDay => "Big Up-Gap, Inside Day",
        Preset::BigDownGapInsideDay => "Big Down-Gap, Inside Day",
        Preset::SteadyUpDryVol => "Steady Up, Dry Vol",
        Preset::SteadyDownDryVol => "Steady Down, Dry Vol",
        Preset::ImpulsiveUpHotVol => "Impulsive Up, Hot Vol",
        Preset::ImpulsiveDownHotVol => "Impulsive Down, Hot Vol",
        Preset::ParabolicUp => "Parabolic Up",
        Preset::ParabolicDown => "Parabolic Down",
        Preset::BlowOffTop => "Blow-Off Top",
        Preset::SellingClimaxBottom => "Selling Climax Bottom",
        Preset::UpDayGapOnlyMove => "Up-Day Gap-Only Move",
        Preset::DownDayGapOnlyMove => "Down-Day Gap-Only Move",
        Preset::IntradayOnlyGreenDay => "Intraday-Only Green Day",
        Preset::IntradayOnlyRedDay => "Intraday-Only Red Day",
        Preset::ReversalUpFromOpen => "Reversal Up From Open",
        Preset::ReversalDownFromOpen => "Reversal Down From Open",
        Preset::TrendDayUp => "Trend Day Up",
        Preset::TrendDayDown => "Trend Day Down",
        Preset::DoubleBottomCandidate => "Double-Bottom Candidate",
        Preset::DoubleTopCandidate => "Double-Top Candidate",
        Preset::Pct52wMidZone => "52w Mid Zone",
        Preset::Pct52wRangeBreakoutTriggered => "52w Range Breakout Triggered",
        Preset::Pct52wRangeBreakdownTriggered => "52w Range Breakdown Triggered",
        Preset::Pct52wTightCoil => "52w Tight Coil",
        Preset::SymmetricTriangle => "Symmetric Triangle",
        Preset::NarrowingRangeOnFlat => "Narrowing Range on Flat",
        Preset::GapTooFarBigPullback => "Gap-Too-Far Big Pullback",
        Preset::GapTooFarBigBounce => "Gap-Too-Far Big Bounce",
        Preset::ChainBreakoutLevel => "Chain Breakout Level",
        Preset::ChainBreakdownLevel => "Chain Breakdown Level",
        Preset::Pct52wRangePosTop => "52w Range Position: Top Half",
        Preset::Pct52wRangePosBottom => "52w Range Position: Bottom Half",
        Preset::HighRangeHighVolStrong => "Hi-Range Hi-Vol Strong Day",
        Preset::HighRangeHighVolWeak => "Hi-Range Hi-Vol Weak Day",
        Preset::LowRangeLowVolNeutral => "Lo-Range Lo-Vol Neutral",
        Preset::AvgRangeAvgVolNeutral => "Avg-Range Avg-Vol Neutral",
        Preset::FailedBreakoutHighReclaim => "Failed 52w-High Breakout",
        Preset::FailedBreakdownLowReclaim => "Failed 52w-Low Breakdown",
        Preset::HotVolHotGap => "Hot Vol + Hot Gap",
        Preset::DryVolDryGap => "Dry Vol + Dry Gap",
        Preset::OuterEdgePushUp => "Outer-Edge Push Up",
        Preset::OuterEdgePushDown => "Outer-Edge Push Down",
        Preset::MiddleZoneUpDrift => "Middle-Zone Up Drift",
        Preset::MiddleZoneDownDrift => "Middle-Zone Down Drift",
        Preset::MiddleZoneHotVolBreakoutHigh => "Mid-Zone Hot-Vol Breakout High",
        Preset::MiddleZoneHotVolBreakoutLow => "Mid-Zone Hot-Vol Breakout Low",
        Preset::GapUpSmallButHotVol => "Gap-Up Small + Hot Vol",
        Preset::GapDownSmallButHotVol => "Gap-Down Small + Hot Vol",
        Preset::GapUpMediumNeutral => "Gap-Up Medium + Flat Day",
        Preset::GapDownMediumNeutral => "Gap-Down Medium + Flat Day",
        Preset::HodReclaimAfterFlush => "HOD Reclaim After Flush",
        Preset::LodFailAfterPush => "LOD Fail After Push",
        Preset::HodReclaimFromFlatGap => "HOD Climb From Flat Open",
        Preset::LodFailFromFlatGap => "LOD Slide From Flat Open",
        Preset::Pct52wTopBoundaryReject => "52w-High Boundary Reject",
        Preset::Pct52wBottomBoundaryReject => "52w-Low Boundary Reject",
        Preset::Pct52wTopBoundaryAccept => "52w-High Boundary Accept",
        Preset::Pct52wBottomBoundaryAccept => "52w-Low Boundary Accept",
        Preset::UpFromBottomSpring => "Up-From-Bottom Spring",
        Preset::DownFromTopUpthrust => "Down-From-Top Upthrust",
        Preset::UpThrustBarReject => "Upthrust Bar Reject",
        Preset::DownThrustBarReject => "Spring Bar Reject",
        Preset::ExhaustionTopWideRange => "Exhaustion Top Wide-Range",
        Preset::ExhaustionBottomWideRange => "Exhaustion Bottom Wide-Range",
        Preset::UpTrendDayWideRange => "Up-Trend Day Wide-Range",
        Preset::DownTrendDayWideRange => "Down-Trend Day Wide-Range",
        Preset::SilentSpringNear52wLow => "Silent Spring Near 52w Low",
        Preset::SilentUpThrustNear52wHigh => "Silent Upthrust Near 52w High",
        Preset::GapStrongDayOpenPivot => "Gap + Strong Day Pivot",
        Preset::GapWeakDayOpenPivot => "Gap + Weak Day Pivot",
        Preset::ConvictionBreakoutCombo => "Highest-Conviction Breakout",
        Preset::ConvictionBreakdownCombo => "Highest-Conviction Breakdown",
        Preset::PullbackInsideTrendUp => "Orderly Pullback in Uptrend",
        Preset::PullbackInsideTrendDown => "Orderly Bounce in Downtrend",
        Preset::RangeContractionSqueezeHigh => "Range-Contraction Coil at 52w High",
        Preset::RangeContractionSqueezeLow => "Range-Contraction Coil at 52w Low",
        Preset::RangeExpansionAtTopOnVol => "Range-Expansion Churn at 52w High",
        Preset::RangeExpansionAtBottomOnVol => "Range-Expansion Churn at 52w Low",
        Preset::GapInsideRangeBalanced => "Flat-Gap Balance Day",
        Preset::GapInsideRangeImpulse => "Flat-Gap Impulse Breakout",
        Preset::OneWickCloseAtMid => "Upper-Wick Flat-Close Reject",
        Preset::OneWickCloseAtMidDown => "Lower-Wick Flat-Close Reject",
        Preset::UpperWickGreenDayConfirm => "Upper-Wick Green-Day Confirm",
        Preset::LowerWickRedDayConfirm => "Lower-Wick Red-Day Confirm",
        Preset::InsideBarTightAtMid => "Inside Bar Tight At Mid",
        Preset::OutsideBarVolumeBoth => "Outside Bar, Heavy Vol Battle",
        Preset::LeadingUpDayLightVol => "Strong Up, Light Vol (Suspect)",
        Preset::LeadingDownDayLightVol => "Strong Down, Light Vol (Suspect)",
        Preset::SmallChangeOnVolNearHigh => "Small Move + Vol at 52w High",
        Preset::SmallChangeOnVolNearLow => "Small Move + Vol at 52w Low",
        Preset::BigGapBigVolBigDay => "Big Gap + Big Vol + Big Day",
        Preset::BigGapNoFollowThrough => "Big Gap, No Follow-Through",
        Preset::ConfluenceLongSetup => "Confluence Long Setup",
        Preset::ConfluenceShortSetup => "Confluence Short Setup",
        Preset::NoExtremeDay => "No-Extreme Mid-Range Day",
        Preset::AcceleratingUpTrend => "Accelerating Up-Trend",
        Preset::AcceleratingDownTrend => "Accelerating Down-Trend",
        Preset::DivergencePushFromTop => "Divergence Reject From Top",
        Preset::DivergencePushFromBottom => "Divergence Reject From Bottom",
        Preset::PriceFlatVolHotAboveMid => "Silent Hot-Vol Above Mid",
        Preset::PriceFlatVolHotBelowMid => "Silent Hot-Vol Below Mid",
        Preset::SmallChangeOnVolMid => "Small Move + Vol at Mid Range",
        Preset::HotRollingVolGap => "Hot-Vol Rolling Gap",
        Preset::SilentDriftGap => "Silent-Drift Gap",
        Preset::UpDayOnDryVolNear52wHigh => "Up Day on Dry Vol Near 52w High",
        Preset::DownDayOnDryVolNear52wLow => "Down Day on Dry Vol Near 52w Low",
        Preset::UpDayOnHotVolNear52wHigh => "Up Day on Hot Vol Near 52w High",
        Preset::DownDayOnHotVolNear52wLow => "Down Day on Hot Vol Near 52w Low",
        Preset::NarrowDayDryVolMid => "Narrow Dry-Vol Mid-Zone Coil",
        Preset::WideDayHotVolMid => "Wide Hot-Vol Mid-Zone Rotation",
        Preset::HotVolAtMidNoMove => "Hot-Vol at Mid, No Move",
        Preset::DryVolAtMidNoMove => "Dry-Vol at Mid, No Move",
        Preset::BigChangeTinyRangeUp => "Big-Change Tiny-Range Up",
        Preset::BigChangeTinyRangeDown => "Big-Change Tiny-Range Down",
        Preset::TinyChangeWideRangeOnVol => "Tiny-Change Wide-Range Hot Vol",
        Preset::TinyChangeWideRangeOnDryVol => "Tiny-Change Wide-Range Dry Vol",
        Preset::LargeGapModerateMoveHotVol => "Large Gap + Moderate Day, Hot Vol",
        Preset::SmallGapBigMoveHotVol => "Small Gap + Big Day, Hot Vol",
        Preset::NoVolTrendUp => "No-Vol Up Trend (Vacuum)",
        Preset::NoVolTrendDown => "No-Vol Down Trend (Vacuum)",
        Preset::ChurnAtTopDryVol => "Churn at 52w High, Dry Vol",
        Preset::ChurnAtBottomDryVol => "Churn at 52w Low, Dry Vol",
        Preset::HugeGapFlatChange => "Huge Gap + Flat Change (Frozen Open)",
        Preset::NoGapHugeChange => "No Gap + Huge Change (Intraday Catalyst)",
        Preset::ExtremeVolFlatGapFlatDay => "Extreme-Vol Flat-Gap Flat-Day (Churn Trap)",
        Preset::IlliquidBigGapFlatDay => "Illiquid Big-Gap Flat-Day (Gap-Fill Risk)",
        Preset::OrganicUpDayCloseAtHod => "Organic Up Day, Close at HOD",
        Preset::OrganicDownDayCloseAtLod => "Organic Down Day, Close at LOD",
        Preset::StrongDayDryVolUp => "Strong Up Day, Dry Vol (Suspect Rally)",
        Preset::StrongDayDryVolDown => "Strong Down Day, Dry Vol (Suspect Flush)",
        Preset::TightCoilAtMidRange => "Tight Coil at Mid-Range, Dry Vol",
        Preset::WideOutsideRangeDryVol => "Wide Outside Range, Dry Vol (One-Sided Liquidation)",
        Preset::GapHeldAndExtendedUp => "Gap Held & Extended Up (Continuation Buyers)",
        Preset::GapHeldAndExtendedDown => "Gap Held & Extended Down (Continuation Sellers)",
        Preset::Pct52wHighBreakoutCloseAtHod => "52w-High Breakout, Close at HOD, Hot Vol",
        Preset::Pct52wLowBreakdownCloseAtLod => "52w-Low Breakdown, Close at LOD, Hot Vol",
        Preset::Pct52wMidHotVolFlat => "52w-Mid Hot-Vol Flat (Decision Churn)",
        Preset::Pct52wMidDryVolFlat => "52w-Mid Dry-Vol Flat (Forgotten Consolidation)",
        Preset::VolSpikeNoTrend => "Vol Spike (≥5×), No Trend (Climax/Exhaustion)",
        Preset::VolSpikeOnTrend => "Vol Spike (≥5×), On Trend (Institutional)",
        Preset::TightCoilAtHighDryVol => "Tight Coil at 52w High, Dry Vol",
        Preset::TightCoilAtLowDryVol => "Tight Coil at 52w Low, Dry Vol",
        Preset::OrderlyTrendAtHighs => "Orderly Trend at 52w Highs",
        Preset::OrderlyTrendAtLows => "Orderly Trend at 52w Lows",
        Preset::HotVolMidRangeChurn => "Hot-Vol Mid-Range Churn (No Commitment)",
        Preset::DryVolAtExtremeClose => "Dry-Vol Close at Extreme (Unconfirmed Edge)",
        Preset::DayChangeMismatch => "Day/Change Mismatch (Full Intraday Reversal)",
        Preset::DayChangeAlignedBig => "Day/Change Aligned Big, Hot Vol (Full Trend Day)",
        Preset::HugeRangeHotVol => "Huge Range (≥8%) Hot Vol (Volatility Expansion)",
        Preset::HugeRangeDryVol => "Huge Range (≥8%) Dry Vol (Illiquid Swing)",
        Preset::Pct52wLowHotVolUp => "Near 52w Low + Hot Vol Up (Bounce/Accumulation)",
        Preset::Pct52wHighHotVolDown => "Near 52w High + Hot Vol Down (Distribution/Topping)",
        Preset::GapHeldNoExtension => "Gap Held, No Intraday Extension",
        Preset::GapPartialFade => "Gap Partial Fade (>50%, Held Direction)",
        Preset::YearHighIntradayWeak => "At 52w High + Intraday Weak (Failed Continuation Up)",
        Preset::YearLowIntradayStrong => "At 52w Low + Intraday Strong (Failed Continuation Down)",
        Preset::WeakHandsAtHighs => "Weak Hands at 52w Highs (Early Weakness)",
        Preset::StrongHandsAtLows => "Strong Hands at 52w Lows (Early Strength)",
        Preset::NarrowRangeHotVolSqueeze => "Narrow Range, Hot Vol (Absorption Coil)",
        Preset::WideRangeDryVolDrift => "Wide Range, Dry Vol, Flat Change (Low-Participation Drift)",
        Preset::LeadershipTrendDay => "Leadership Trend Day (Near 52w High)",
        Preset::WorstActorFlushDay => "Worst-Actor Flush Day (Near 52w Low)",
        Preset::GapUpAtYearLow => "Gap Up at 52w Low (Oversold Squeeze)",
        Preset::GapDownAtYearHigh => "Gap Down at 52w High (Sudden Distribution)",
        Preset::BigUpMidRangeClose => "Big Up + Mid-Range Close (Topping Action)",
        Preset::BigDownMidRangeClose => "Big Down + Mid-Range Close (Basing Action)",
        Preset::HodCloseHotVolFlat => "HOD Close + Hot Vol + Flat Change (Absorption High)",
        Preset::LodCloseHotVolFlat => "LOD Close + Hot Vol + Flat Change (Absorption Low)",
        Preset::RisingWedgeCoil => "Rising Wedge Coil (Narrow Range, Green, Dry Vol)",
        Preset::FallingWedgeCoil => "Falling Wedge Coil (Narrow Range, Red, Dry Vol)",
        Preset::BigGapAndExtend => "Big Gap + Intraday Extends Beyond (Trend Continuation)",
        Preset::BigGapAndReverse => "Big Gap + Intraday Reverses Past Prior (Full Reverse)",
        Preset::EfficientMoverHotVol => "Efficient Mover (|change| ≥ 1.5×rel_vol, Clean Trend)",
        Preset::InefficientChurnHotVol => "Inefficient Churn (|change| < 0.3×rel_vol, Absorption)",
        Preset::GapUpAtMidRange => "Gap Up at 52w Mid-Range (Consolidation Breakout)",
        Preset::GapDownAtMidRange => "Gap Down at 52w Mid-Range (Consolidation Breakdown)",
        Preset::BattleBarHotVol => "Battle Bar (Wide Range, Flat Change, Hot Vol)",
        Preset::IlliquidSwingDryVol => "Illiquid Swing (Wide Range, Flat Change, Dry Vol)",
        Preset::GapDownIntradayReclaimUp => "Gap Down + HOD Close + Reclaim Above Prior",
        Preset::GapUpIntradayRejectDown => "Gap Up + LOD Close + Reject Below Prior",
        Preset::HotVolModerateChangeFlatDay => "Hot Vol + Moderate Change + Flat Intraday (Gap-Driven)",
        Preset::DryVolModerateChangeFlatDay => "Dry Vol + Moderate Change + Flat Intraday (Sleepy Held)",
        Preset::WideRangeAtYearHigh => "Wide Range at 52w High (Volatility at Top)",
        Preset::WideRangeAtYearLow => "Wide Range at 52w Low (Volatility at Bottom)",
        Preset::HotVolGapHeldAndExtended => "Hot-Vol Gap Held & ≥80% Extended (Institutional Gap-Go)",
        Preset::HotVolGapFadedDeep => "Hot-Vol Gap Faded ≥50% (Institutional Conviction Fade)",
        Preset::TightRangeAtYearHigh => "Tight Range at 52w High (Bullish Base)",
        Preset::TightRangeAtYearLow => "Tight Range at 52w Low (Potential Reversal)",
        Preset::BalancedMidWickHotVol => "Balanced Mid-Wick, Hot Vol (Mid-Range Churn)",
        Preset::BalancedMidWickDryVol => "Balanced Mid-Wick, Dry Vol (Sleepy Mid-Range)",
        Preset::GapUpHodCloseControlled => "Gap Up + HOD Close + Normal Vol (Controlled Trend Up)",
        Preset::GapDownLodCloseControlled => "Gap Down + LOD Close + Normal Vol (Controlled Decline)",
        Preset::AllGreenTightDay => "All Green Directions + Tight Range (Strong Hands)",
        Preset::AllRedTightDay => "All Red Directions + Tight Range (Strong Sellers)",
        Preset::MicroRangeAtYearHigh => "Micro Range at 52w High (Zero-Range Pin Top)",
        Preset::MicroRangeAtYearLow => "Micro Range at 52w Low (Zero-Range Pin Bottom)",
        Preset::ConsolidationBreakUp => "Consolidation Break Up (Mid-Upper 52w, Hot Vol)",
        Preset::ConsolidationBreakDown => "Consolidation Break Down (Mid-Lower 52w, Hot Vol)",
        Preset::HotVolGapHeldFlatChange => "Hot-Vol Gap Held + Flat Change (Heavy Absorption)",
        Preset::DryVolGapHeldFlatChange => "Dry-Vol Gap Held + Flat Change (Thin-Tape Hold)",
        Preset::AllDirectionsAlignedHotVolUp => "All Up + Hot Vol (Full Conviction Bull Bar)",
        Preset::AllDirectionsAlignedHotVolDown => "All Down + Hot Vol (Full Conviction Bear Bar)",
        Preset::IntradayRecoveryFromGapDown => "Intraday Recovery from Gap-Down (Mid-Upper 52w)",
        Preset::IntradayRejectionFromGapUp => "Intraday Rejection from Gap-Up (Mid-Lower 52w)",
        Preset::Pct52wMidUpperHotVolDown => "Mid-Upper 52w + Big Down + Hot Vol (Uptrend Correction)",
        Preset::Pct52wMidLowerHotVolUp => "Mid-Lower 52w + Big Up + Hot Vol (Downtrend Rally)",
        Preset::OrderlyMidRangeRally => "Orderly Mid-Range Rally (Symmetric 52w, Normal Vol)",
        Preset::OrderlyMidRangePullback => "Orderly Mid-Range Pullback (Symmetric 52w, Normal Vol)",
        Preset::StrongBreakoutDay => "Strong Breakout Day (Near 52w High, Big Gap & Change)",
        Preset::StrongBreakdownDay => "Strong Breakdown Day (Near 52w Low, Big Gap & Change)",
        Preset::VolSpikeBigGapBigChange => "Vol Spike + Big Gap + Big Change (Institutional Gap-On-News)",
        Preset::VolSpikeTinyGapBigChange => "Vol Spike + Flat Gap + Big Change (Intraday Catalyst)",
        Preset::StrongCloseAtHodHotVol => "Strong Close at HOD + Green Intraday + Hot Vol",
        Preset::WeakCloseAtLodHotVol => "Weak Close at LOD + Red Intraday + Hot Vol",
        Preset::Pct52wHighDryVolFlat => "Near 52w High, Dry Vol, Flat (Forgotten Leadership)",
        Preset::Pct52wLowDryVolFlat => "Near 52w Low, Dry Vol, Flat (Forgotten Weakness)",
        Preset::OvernightReversalRepositioning => "Overnight Reversal Repositioning (Gap ↔ Change Sign Flip, Flat Day)",
        Preset::OrganicIntradayTrendDay => "Organic Intraday Trend Day (No Gap, Big Change, Normal Vol)",
        Preset::TightRangeFlatDayHotVol => "Tight Range + Flat Intraday + Hot Vol (Absorption Coil)",
        Preset::TightRangeFlatDayDryVol => "Tight Range + Flat Intraday + Dry Vol (Deep Sleep)",
        Preset::HodHotVolMicroRange => "Close Pinned at HOD + Wider Range + Hot Vol (Controlled Mark-up)",
        Preset::LodHotVolMicroRange => "Close Pinned at LOD + Wider Range + Hot Vol (Controlled Mark-down)",
        Preset::GapAndGoStrongClose => "Gap-and-Go + Strong Close at HOD (Follow-through)",
        Preset::GapAndFadeWeakClose => "Gap-and-Fade + Weak Close at LOD (Failed Breakout)",
        Preset::InsideDayDryVolCoiled => "Inside Day + Dry Vol Coil (Pre-expansion Drift)",
        Preset::OutsideDayHotVolExpansion => "Outside Day + Hot Vol Expansion (Volatility Breakout)",
        Preset::MidRangeDryVolNoConviction => "Mid-range Close + Flat Day + Dry Vol (No Conviction)",
        Preset::LowOfYearHotVolPanic => "52w Low + Hot Vol + Red Day (Capitulation Panic)",
        Preset::HighOfYearHotVolEuphoria => "52w High + Hot Vol + Green Day (Euphoric Breakout)",
        Preset::WideRangeFlatCloseHeavyChurn => "Wide Range + Flat Close + Hot Vol (Heavy Churn / Both Sides Absorbed)",
        Preset::RangeExpansionDryVol => "Wide Range + Dry Vol (Thin-liquidity Swing / Algorithmic)",
        Preset::YearHighGapDownHotVol => "Near 52w High + Gap Down + Hot Vol (Distribution from Top)",
        Preset::YearLowGapUpHotVol => "Near 52w Low + Gap Up + Hot Vol (Relief Gap / Reversal off Bottom)",
        Preset::IntradayFakeoutTopReject => "Pullback from HOD + Red Day + Hot Vol (Intraday Top Rejection)",
        Preset::IntradayFakeoutBottomReject => "Bounce from LOD + Green Day + Hot Vol (Intraday Bottom Rejection)",
        Preset::RangeContractionAfterMove => "Narrow Range + Flat Close + Meaningful Change + Normal Vol (Continuation Pause)",
        Preset::RelativeStrengthBuild => "Organic Up + Slight Vol Pickup + No Gap (Relative-Strength Build)",
        Preset::RelativeWeaknessBuild => "Organic Down + Slight Vol Pickup + No Gap (Relative-Weakness Build)",
        Preset::HighVolAbsorbingChange => "Hot Vol + Tight Range + Meaningful Change (Volume Absorbing Directional Pressure)",
        Preset::LowVolWideRangeAccumulator => "Dry Vol + Wider Range + Meaningful Change (Quiet Accumulator Working)",
        Preset::BullishEngulfingHotVol => "Bullish Engulfing + Hot Vol (Gap Down + Strong Reversal)",
        Preset::BearishEngulfingHotVol => "Bearish Engulfing + Hot Vol (Gap Up + Strong Reversal)",
        Preset::DoubleBottomRetest => "Near 52w Low + Holding Day + Decent Vol (Potential Double-Bottom Retest)",
        Preset::DoubleTopRetest => "Near 52w High + Failing Day + Decent Vol (Potential Double-Top Retest)",
        Preset::LiquiditySweepBothSides => "Both Extremes Visited + Flat Close + Hot Vol (Liquidity Sweep)",
        Preset::SteadyGrinderNoVolPickup => "Small Steady Gain + Sub-avg Vol + No Gap (Quiet Uptrend Grinder)",
        Preset::SteadyDeclinerNoVolPickup => "Small Steady Decline + Sub-avg Vol + No Gap (Quiet Downtrend Decliner)",
        Preset::HighVolStallNearHighOfYear => "Near 52w High + Hot Vol + Flat Day (High-Vol Stall at Top)",
        Preset::HighVolStallNearLowOfYear => "Near 52w Low + Hot Vol + Flat Day (High-Vol Stall at Bottom)",
        Preset::OutlierSessionBigMoveBigVol => "Big Move + Big Vol + Wide Range (Outlier / Momentum / News Event)",
        Preset::EodParabolicAccelerationUp => "EOD Parabolic Acceleration Up (Intraday Drive + HOD Close + Hot Vol)",
        Preset::EodParabolicAccelerationDown => "EOD Parabolic Acceleration Down (Intraday Drive + LOD Close + Hot Vol)",
        Preset::FullSpectrumDayUp => "Closed HOD + Visited LOD + Green (Volatile Full-Range Day Up)",
        Preset::FullSpectrumDayDown => "Closed LOD + Visited HOD + Red (Volatile Full-Range Day Down)",
        Preset::GreenStreakAccumulator => "Modest Gain + Green Intraday + Non-neg Gap + Decent Vol (Streak Accumulator)",
        Preset::RedStreakDistributor => "Modest Loss + Red Intraday + Non-pos Gap + Decent Vol (Streak Distributor)",
        Preset::GapDownReclaim => "Gap Down + Reclaimed Positive (Full Intraday Rotation Up)",
        Preset::GapUpFailReclaimed => "Gap Up + Failed Negative (Full Intraday Rotation Down)",
        Preset::MidYearRangeConsolidation => "Mid 52w Range + Flat + Sub-avg Vol (Consolidation; Nowhere)",
        Preset::AtYearExtremeVolatilityExpansion => "At 52w Extreme + Wide Range + Decent Vol (Structural Test)",
        Preset::BreakoutFromMidLevels => "Near 52w High Coming from Mid + Hot Vol (Breakout from Mid-range)",
        Preset::BreakdownFromMidLevels => "Near 52w Low Coming from Mid + Hot Vol (Breakdown from Mid-range)",
        Preset::IntradayStrongerThanGap => "Intraday Move > Gap (All Action Regular Session)",
        Preset::OvernightStrongerThanIntraday => "Overnight Gap > Intraday (Market Accepted Gap Without Expansion)",
        Preset::EfficientMoveLowEffort => "Meaningful Change + Dry Vol + Narrow Range (Efficient Sleeper Move)",
        Preset::SignalVsNoiseChurn => "Flat Close + Hot Vol + Wide Range (Pure Noise / Heavy Churn Day)",
        Preset::GreenCloseRedIntraday => "Green Close + Red Intraday + Hot Vol (Gap Held Positive; Intraday Erosion)",
        Preset::RedCloseGreenIntraday => "Red Close + Green Intraday + Hot Vol (Gap Held Negative; Intraday Recovery)",
        Preset::FullConvictionUpDay => "Gap Up + Green Day + Intraday Up + HOD Close + Hot Vol (Full-Conviction Up Day)",
        Preset::FullConvictionDownDay => "Gap Down + Red Day + Intraday Down + LOD Close + Hot Vol (Full-Conviction Down Day)",
        Preset::YearLowProximityRallyAttempt => "Near 52w Low + Intraday Rally + Green Close (Rally Attempt off Lows)",
        Preset::YearHighProximityFailAttempt => "Near 52w High + Intraday Fail + Red Close (Failed Move at Highs)",
        Preset::OpenGapFilledNetFlat => "Big Gap Opened + Closed Near Flat (Full Gap Absorption / Round-Trip)",
        Preset::CompressedRangeVolatileSession => "Small 52w Range + Wide Intraday + Hot Vol (Coiled Asset Breaking Out)",
        Preset::OrderlyNewHighContinuation => "Fresh 52w High + Moderate Up Move + Decent Vol (Orderly Breakout Continuation)",
        Preset::OrderlyNewLowContinuation => "Fresh 52w Low + Moderate Down Move + Decent Vol (Orderly Breakdown Continuation)",
        Preset::DryVolGapUpFade => "Gap Up + Faded Negative + Dry Vol (Orderly Gap-up Absorption)",
        Preset::DryVolGapDownReclaim => "Gap Down + Reclaimed Positive + Dry Vol (Orderly Gap-down Absorption)",
        Preset::InstitutionalChurnDay => "Very Hot Vol + Flat Change (Institutional Rebalance / Churn Day)",
        Preset::ExtremeTailEvent => "Very Hot Vol + Huge Change (Extreme Tail Event / Major News)",
        Preset::Year52HighRetestStrongClose => "Near 52w High + HOD Close + Hot Vol (52w-High Retest Confirmed)",
        Preset::Year52LowRetestWeakClose => "Near 52w Low + LOD Close + Hot Vol (52w-Low Retest Confirmed)",
        Preset::DivergentGapVsIntraday => "Gap vs Intraday Opposite Directions (Market Disagreed with Gap)",
        Preset::CongruentGapAndIntradaySameDir => "Gap and Intraday Same Direction (Gap Extended by Intraday)",
        Preset::DeepMidRangeQuietSiesta => "Deep Mid-52w-Range + Quiet Vol + Flat Change (Calm Asset Siesta)",
        Preset::DeepMidRangeActiveOutlier => "Deep Mid-52w-Range + Hot Vol + Meaningful Change (Trend Genesis Day)",
        Preset::IntradayDirectionExceedsChange => "Intraday Dominates Same-direction Change by 1.5× (Late-session Continuation)",
        Preset::ChangeExceedsIntradayMagnitude => "Change Dominates Intraday by 2× (Gap-dominant Move; Small Intraday)",
        Preset::JustOffYearLowBouncingUp => "Just Off 52w Low (5–15%) + Hot Vol + Green (Early Bounce off Lows)",
        Preset::JustOffYearHighFadingDown => "Just Off 52w High (5–15%) + Hot Vol + Red (Early Fade from Highs)",
        Preset::OverextendedHighPullbackHealthy => "Near 52w High + HOD Pullback + Still Positive Day (Healthy Retrace in Trend)",
        Preset::OverextendedLowBounceHealthy => "Near 52w Low + LOD Bounce + Still Negative Day (Healthy Retrace in Downtrend)",
        Preset::CleanTrendDayUp => "Gap + Intraday + Change All Positive + HOD Close + Decent Vol (Clean Trend Day Up)",
        Preset::CleanTrendDayDown => "Gap + Intraday + Change All Negative + LOD Close + Decent Vol (Clean Trend Day Down)",
        Preset::ClimaxRedBouncedFromLod => "Big Red Day + LOD Bounce + Hot Vol (Climax Low / Capitulation Reversal)",
        Preset::ClimaxGreenFadedFromHod => "Big Green Day + HOD Fade + Hot Vol (Climax High / Euphoric Reversal)",
        Preset::WideRangeChopMixedVol => "Wide Range + Small Change + Normal Vol (Range Exploration without Conviction)",
        Preset::NarrowRangeBigChangeNoIntraday => "Narrow Range + Big Change + Flat Intraday (Pure Gap Day; All Change Overnight)",
        Preset::EveryAxisExtreme => "Every Signal Axis Extreme Simultaneously (Multi-axis Breakout / Full-feature Outlier)",
        Preset::EveryAxisFlat => "Every Signal Axis Tiny Simultaneously (Silent Day / Full-feature Null)",
        Preset::Year52HighRejectedToLod => "Near 52w High + Closed at LOD + Hot Vol + Red (Sharp Rejection at Highs)",
        Preset::Year52LowReclaimedToHod => "Near 52w Low + Closed at HOD + Hot Vol + Green (Sharp Reclaim from Lows)",
        Preset::HighVolNoGapModerateChange => "Hot Vol + No Gap + Modest Change (Quiet-Price Big-Flow Day)",
        Preset::LowVolWithLargeGap => "Big Gap + Dry Vol + Meaningful Change (Gap Absorbed Quietly)",
        Preset::GapErasedByIntradayFlat => "Gap + Opposite Intraday + Flat Close (Gap Completely Erased)",
        Preset::BothSidesTaggedFlatBalance => "Both Extremes Visited + Flat Change + Decent Vol (Range-bound Balance)",
        Preset::OutsideDayWideBalanceHotVol => "Very Wide Range + Flat Change + Hot Vol (Outside-Day Balance on Heavy Vol)",
        Preset::InsideDayBigChangeBigVol => "Narrow Range + Big Change + Hot Vol (Inside-Day Gap Day on Heavy Participation)",
        Preset::LongCandleUpTrendDay => "Big Green Day + Intraday Up + Wide Range + HOD Close + Hot Vol (Long Candle Up Trend Day)",
        Preset::LongCandleDownTrendDay => "Big Red Day + Intraday Down + Wide Range + LOD Close + Hot Vol (Long Candle Down Trend Day)",
        Preset::Year52HighWithRangeContraction => "Near 52w High + Tight Range + Flat Change (Coil at the Top)",
        Preset::Year52LowWithRangeContraction => "Near 52w Low + Tight Range + Flat Change (Coil at the Bottom)",
        Preset::GapAndIntradayHarmonic => "Gap and Intraday Similar Magnitudes + Decent Vol (Harmonic Day; Balanced Overnight + Intraday)",
        Preset::MicroDayEarlyShakeout => "Small Change + Wide Range + Flat Day + Hot Vol (Early Shakeout; Both Extremes Explored Then Settled)",
        Preset::GreenDaySubOptimalClose => "Green Day + Significant Pullback from HOD (Sub-optimal Close on Up Day)",
        Preset::RedDaySubOptimalClose => "Red Day + Significant Bounce from LOD (Sub-optimal Close on Down Day)",
        Preset::WideRangeNoVolFlat => "Wide Range + Dry Vol + Flat Change (Fake-liquidity Range Exploration)",
        Preset::NarrowRangeMeaningfulChange => "Narrow Range + Meaningful Change (One-print Day; No Intraday Discovery)",
        Preset::Year52HighGapDownReclaimed => "Near 52w High + Gap Down Reclaimed Positive + Decent Vol (Resilience at Highs)",
        Preset::Year52LowGapUpFaded => "Near 52w Low + Gap Up Faded Negative + Decent Vol (Relief Gap Rejected at Lows)",
        Preset::IntradayMatchesChange => "No Gap + Intraday Matches Change (All Move from Regular Session)",
        Preset::IntradayOpposesChange => "Meaningful Change + Intraday Opposes Sign + Meaningful Intraday (Gap Dominated)",
        Preset::SymmetricMidRangeBalance => "Close Exactly Mid-Range + Meaningful Range + Flat Change (Symmetric Balance Day)",
        Preset::AsymmetricExtremeBias => "Asymmetric Close + Wide Range (One-sided Range Exploration)",
        Preset::YearLowExplosiveSqueezeIgnition => "At 52w Low + Huge Green + Extreme Vol (Squeeze Ignition from Multi-month Lows)",
        Preset::YearHighSharpDistribution => "At 52w High + Huge Red + Extreme Vol (Sharp Distribution from Multi-month Highs)",
        Preset::LargeChangeOnNormalVol => "Big Change + Normal Vol + Wide Range (Orderly Directional Day Without Extreme Participation)",
        Preset::MassiveIntradayWithoutGap => "No Gap + Massive Intraday + Hot Vol (Pure Intraday Discovery; Zero Overnight Bias)",
        Preset::MidYearBothSidesTagged => "Mid-52w + Both Extremes Visited + Decent Vol (Mid-range Double-test Day)",
        Preset::ExtremeSilentRange => "At 52w Extreme + Tight Range + Dry Vol (Silence at Extreme; Pre-reversal Exhaustion)",
        Preset::MultiAxisDryDay => "Flat Change + Dry Vol + Narrow Range + Small Gap (Quiet Multi-axis Maintenance Day)",
        Preset::BigGapBigVolBigChange => "Big Gap + Big Vol + Big Change (Catalyst Day; News-driven with Sustained Activity)",
        Preset::GapDownClosedNearHODHotVol => "Gap Down + Closed Green + Near HOD + Hot Vol (V-shape Reclaim Closing on the Highs)",
        Preset::GapUpClosedNearLODHotVol => "Gap Up + Closed Red + Near LOD + Hot Vol (Full Fade Closing on the Lows)",
        Preset::Year52HighGapUpHotVolBigChange => "Within 1% of 52w High + Gap Up + Hot Vol + Finished Up (Near-high Breakout Attempt with Sustained Demand)",
        Preset::Year52LowGapDownHotVolBigDrop => "Within 1% of 52w Low + Gap Down + Hot Vol + Finished Down (Breakdown Capitulation at Multi-year Lows)",
        Preset::WideRangeFlatCloseHotVol => "Wide Range + Flat Close + Hot Vol (Tug-of-war Battle with Heavy Participation; No Winner)",
        Preset::BigGapNarrowIntradayHotVol => "Huge Gap + Tight Intraday Range + Hot Vol (Gap Accepted on Volume; No Follow-through)",
        Preset::Year52HighDryVolNarrowRange => "Near 52w High + Dry Vol + Narrow Range + Flat (Silent Compression at Multi-year Highs; Coiled Spring)",
        Preset::Year52LowDryVolNarrowRange => "Near 52w Low + Dry Vol + Narrow Range + Flat (Silent Compression at Multi-year Lows; Coiled Bounce Setup)",
        Preset::CompressedYearRangeFlatDay => "Compressed 52w Range + Flat + Dry Vol (Structurally Narrow Asset on Another Silent Day; Regime-level Coil)",
        Preset::CompressedYearRangeRegimeBreak => "Compressed 52w Range + Big Change + Hot Vol (Sudden Break of Long-compressed Sideways Range; Regime-level Breakout/Breakdown)",
        Preset::IntradayClimaxTopFade => "Intraday Climax-top Fade + Hot Vol (Pumped Early, Sold All Day; Closed Red at LOD on Volume)",
        Preset::IntradayClimaxBottomReclaim => "Intraday Climax-bottom Reclaim + Hot Vol (Panicked Early, Bid Up All Day; Closed Green at HOD on Volume)",
        Preset::BigChangeDryVolWideRange => "Big Change + Dry Vol + Wide Range (Low-conviction Trend Day with Thin Tape; Volatile but Uncrowded)",
        Preset::BigChangeDryVolFromGap => "Big Change + Dry Vol + Significant Gap (Pre-market Re-rating Absorbed without Daytime Confirmation)",
        Preset::ExtremeVolGapDownReversal => "Extreme Vol (5×+) + Gap Down + Finished Green (Max-conviction Reversal of Overnight Gap-down)",
        Preset::ExtremeVolGapUpReversal => "Extreme Vol (5×+) + Gap Up + Finished Red (Max-conviction Reversal of Overnight Gap-up; Institutional Distribution)",
        Preset::AtYearHighRangeExpansionDryVol => "At 52w High + Wide Range + Dry Vol (No-volume Rally Warning; Distribution at All-time Highs)",
        Preset::AtYearLowRangeExpansionDryVol => "At 52w Low + Wide Range + Dry Vol (Thin-tape Capitulation; Bounces Unlikely to Stick)",
        Preset::IntradayBigDayGapAgainstHotVol => "Big Intraday Day + Gap Against Day + Hot Vol (Institutional Gap-against-trend Reversal)",
        Preset::IntradayBigDayGapWithHotVol => "Big Intraday Day + Gap With Day + Hot Vol (Textbook Gap-and-go Continuation on Heavy Participation)",
        Preset::OvernightDriftDryVol => "Significant Gap + Flat Intraday + Very Dry Vol (News Absorbed Instantly; Nobody Traded the Day)",
        Preset::HotVolHugeGapTinyDay => "Hot Vol (3×+) + Big Gap + Nearly Flat Intraday (Institutional Repositioning at New Gap Level; Massive Churn at One Price)",
        Preset::Year52LowGapUpHeldHotVol => "Near 52w Low + Gap Up + Held & Extended + Hot Vol (Relief Reclaim with Momentum at the Floor)",
        Preset::Year52HighGapDownHeldHotVol => "Near 52w High + Gap Down + Held & Extended + Hot Vol (Distribution at the Highs; Rejection Gap Confirmed)",
        Preset::HotVolSmallChangeSmallGapWideRange => "Hot Vol + Tiny Change + Tiny Gap + Moderate Range (Pure Intraday Redistribution; Heavy Participation, No Resolution)",
        Preset::HotVolFlatCloseBigGap => "Hot Vol + Flat Close + Big Gap (Intraday Round-trip Absorbed Overnight Move with Volume Confirmation)",
        Preset::OrganicMicroGainNormalVol => "Modest Gain (0.3-1%) + Green Intraday + Normal Vol + No Gap (Silent Organic Accumulation Under the Radar)",
        Preset::OrganicMicroDropNormalVol => "Modest Drop (-1 to -0.3%) + Red Intraday + Normal Vol + No Gap (Silent Organic Distribution Under the Radar)",
        Preset::IntradayRangeWiderThanGapHotVol => "Intraday Range > 2× Gap + Hot Vol (Intraday Discovery Dominates the Gap; Market Traded Far Wider Than Overnight Expected)",
        Preset::GapWiderThanIntradayRangeHotVol => "Gap > Intraday Range + Hot Vol (Gap Dominates; Intraday Only Consolidated Near the New Level on Volume)",
        Preset::BigGreenLowVolWeakClose => "Big Green + Dry Vol + Close Off Highs (Fake-Breakout Warning; Up-Move on Weak Participation; Reversion Candidate)",
        Preset::BigRedLowVolWeakClose => "Big Red + Dry Vol + Close Off Lows (Fake-Breakdown Warning; Down-Move on Weak Participation; Reversion Candidate)",
        Preset::GappingNearYearLowExtremeVol => "Near 52w Low + Significant Gap + Extreme Vol (4×+) (Event-driven Catalyst at the Floor; Reversal or Capitulation Candidate)",
        Preset::GappingNearYearHighExtremeVol => "Near 52w High + Significant Gap + Extreme Vol (4×+) (Event-driven Catalyst at the Top; Blow-off or Distribution Candidate)",
        Preset::BothSidesTaggedDryVolFlat => "Both Extremes Tagged + Flat Close + Dry Vol (Thin-tape Stop-hunt; Possible Spoof/Wash on Illiquid Name)",
        Preset::BothSidesTaggedBigChangeHotVol => "Both Extremes Tagged + Big Change + Hot Vol (Full-range Exploration Ending Decisively; Trend Day After Sweeping Both Sides)",
        Preset::ModerateGreenGapDownReversal => "Gap Down + Moderate Green Finish + Slightly Elevated Vol (Moderate-conviction Gap Reversal; Conservative Reclaim)",
        Preset::ModerateRedGapUpFade => "Gap Up + Moderate Red Finish + Slightly Elevated Vol (Moderate-conviction Gap Fade; Conservative Rejection)",
        Preset::GapAndIntradayBothBigSameDirHotVol => "Big Gap + Big Same-direction Intraday + Hot Vol (Both Halves of the Day Pushed Same Way on Volume; Conviction Continuation)",
        Preset::GapAndIntradayBothBigOpposingHotVol => "Big Gap + Big Opposing Intraday + Hot Vol (Gap Rejected and Reversed by Significant Intraday; Full Counter-move on Volume)",
        Preset::CountertrendBounceInDowntrend => "Big Green + Deep Below 52w High + Near 52w Low + Hot Vol (Countertrend Bounce in Long-running Downtrend; Strong-hands Buying Near the Floor)",
        Preset::CountertrendFadeInUptrend => "Big Red + Deep Above 52w Low + Near 52w High + Hot Vol (Countertrend Fade in Long-running Uptrend; Profit-taking Near the Ceiling)",
        Preset::BigGreenNarrowRangeHotVol => "Big Green + Tight Intraday Range + Hot Vol (Gap-and-hold Up; No Giveback; Max-strength Continuation Candidate)",
        Preset::BigRedNarrowRangeHotVol => "Big Red + Tight Intraday Range + Hot Vol (Gap-and-hold Down; No Recovery; Max-weakness Continuation Candidate)",
        Preset::OneSidedRangeCloseAtHODGreen => "Close at HOD + LOD >2% Away + Green + Decent Vol (One-sided Up-day; Sellers Shown Low Side But Couldn't Hold)",
        Preset::OneSidedRangeCloseAtLODRed => "Close at LOD + HOD >2% Away + Red + Decent Vol (One-sided Down-day; Buyers Shown High Side But Couldn't Hold)",
        Preset::BullishOutsideDayHotVol => "Bullish Outside Day + Wide Range + Same-direction Intraday + Hot Vol (Institutional Accumulation Through Range Expansion)",
        Preset::BearishOutsideDayHotVol => "Bearish Outside Day + Wide Range + Same-direction Intraday + Hot Vol (Institutional Distribution Through Range Expansion)",
        Preset::BelowAvgVolBigChangeGreen => "Big Green + Below-avg Vol (50-90%) + No Gap (Efficient Organic Up Move; Low-effort Momentum on Quiet Tape)",
        Preset::BelowAvgVolBigChangeRed => "Big Red + Below-avg Vol (50-90%) + No Gap (Efficient Organic Down Move; Low-effort Weakness on Quiet Tape)",
        Preset::MidRangeFullExpansionHotVol => "Mid 52w + Wide Intraday Range + Hot Vol (Undecided Asset Having a High-vol Expansion Day; Trend Genesis from Balance)",
        Preset::MidRangeCompressionDryVol => "Mid 52w + Tight Intraday + Dry Vol (Structural Undecided + Compression + No Trade; Classic Dormancy / Pre-event Coil)",
        Preset::OpeningRangeHoldCloseAtHODGreen => "Flat Open + Close at HOD + Green Intraday + Normal-elevated Vol (Clean Intraday Discovery to Highs; Pure Organic Trend Day Up)",
        Preset::OpeningRangeHoldCloseAtLODRed => "Flat Open + Close at LOD + Red Intraday + Normal-elevated Vol (Clean Intraday Discovery to Lows; Pure Organic Trend Day Down)",
        Preset::WindowDressingMarkUp => "Tiny Green + Close Near HOD + Hot Vol (Mark-the-close Behavior; Possible Quarter/month-end Window-dressing)",
        Preset::WindowDressingMarkDown => "Tiny Red + Close Near LOD + Hot Vol (Mark-down Behavior; Possible Reverse Window-dressing)",
        Preset::Year52HighSustainedStrengthHotVol => "Near 52w High + Green Intraday + Green Close + Hot Vol (Sustained Strength Confirmation; Intraday-and-Daily Both Up; Breakout Candidate)",
        Preset::Year52LowSustainedWeaknessHotVol => "Near 52w Low + Red Intraday + Red Close + Hot Vol (Sustained Weakness Confirmation; Intraday-and-Daily Both Down; Breakdown Candidate)",
        Preset::BigGreenWithModestGapDecentVol => "Big Green + Modest Gap Up + Decent Vol (Gap-assisted Rally; Overnight Bias Kicked Off the Day, Intraday Extended)",
        Preset::BigRedWithModestGapDownDecentVol => "Big Red + Modest Gap Down + Decent Vol (Gap-assisted Decline; Overnight Bias Kicked Off the Day, Intraday Extended)",
        Preset::CompoundConfirmedBigGreen => "Big Green + Green Intraday + Positive Gap + Close Near HOD + Decent Vol (Every Signal Aligned Bullish; Max-confirmation Long Candidate)",
        Preset::CompoundConfirmedBigRed => "Big Red + Red Intraday + Negative Gap + Close Near LOD + Decent Vol (Every Signal Aligned Bearish; Max-confirmation Short Candidate)",
        Preset::FollowThroughGreen => "Modest Green + Green Intraday + Small Gap + Close Near HOD + Above-avg Vol (Clean Follow-through Up Day; Steady Accumulation with Intraday Confirmation)",
        Preset::FollowThroughRed => "Modest Red + Red Intraday + Small Gap + Close Near LOD + Above-avg Vol (Clean Follow-through Down Day; Steady Distribution with Intraday Confirmation)",
        Preset::Year52HighGapDownStrongCloseHotVol => "Near 52w High + Gap Down + Green Close + Close Near HOD + Decent Vol (Resilience Day at the Highs; Gap Rejected, Closed Strong)",
        Preset::Year52LowGapUpWeakCloseHotVol => "Near 52w Low + Gap Up + Red Close + Close Near LOD + Decent Vol (Rejection Day at the Lows; Relief Gap Faded, Closed Weak)",
        Preset::FlatOpenTrendUpModerate => "Flat Open + Moderate Green + Green Intraday + Same-sign + Normal-elevated Vol (No Overnight Bias + Clean Intraday Trend Up; Conviction Without Spike)",
        Preset::FlatOpenTrendDownModerate => "Flat Open + Moderate Red + Red Intraday + Same-sign + Normal-elevated Vol (No Overnight Bias + Clean Intraday Trend Down; Conviction Without Spike)",
        Preset::MidRangeRecoveryRallyHotVol => "Mid 52w + Big Green + Hot Vol (Sustained Recovery Rally Off Lows; Not At Either Extreme; Mid-range Bullish)",
        Preset::MidRangeSelloffHotVol => "Mid 52w + Big Red + Hot Vol (Sustained Selloff Off Highs; Not At Either Extreme; Mid-range Bearish)",
        Preset::IntermediateGreenStrongClose => "Meaningful Green (3-7%) + Decent Vol (1.5-3×) + Close Near HOD (Intermediate Momentum; Sweet Spot Between Organic and Parabolic)",
        Preset::IntermediateRedWeakClose => "Meaningful Red (-3 to -7%) + Decent Vol (1.5-3×) + Close Near LOD (Intermediate Weakness; Sweet Spot Between Organic and Crash)",
        Preset::MaxVolatilityEventHotVol => "Big Gap + Big Change + Wide Range + Hot Vol (Max-volatility Event Day; Catalyst-driven Volatility Expansion)",
        Preset::MaxRangeFakeOutDryVol => "Big Gap + Big Change + Wide Range + Dry Vol (Max-range Fake-out; Stop-runs Without True Conviction; Algorithmic Noise on Illiquid Name)",
        Preset::BigGreenIntradayOnlyHotVol => "Big Green + Flat Open + Wide Intraday + Hot Vol (Intraday-only Rally; All Gain from Intraday Discovery; No Overnight Bias)",
        Preset::BigRedIntradayOnlyHotVol => "Big Red + Flat Open + Wide Intraday + Hot Vol (Intraday-only Decline; All Loss from Intraday Discovery; No Overnight Bias)",
        Preset::BrokeAbove52wHighHotVol => "Closed Above Prior 52w High + Green + Hot Vol (True New-high Breakout with Volume Confirmation)",
        Preset::BrokeBelow52wLowHotVol => "Closed Below Prior 52w Low + Red + Hot Vol (True New-low Breakdown with Volume Confirmation)",
        Preset::ChangeIntradayDisagreeBothTagged => "Change/Day Signs Disagree + Both Extremes Tagged + Decent Vol (Schizophrenic Day; Gap vs Intraday Opposition with Full-range Exploration)",
        Preset::ChangeIntradayDisagreeFlatRange => "Change/Day Signs Disagree + Tight Intraday + Decent Vol (Overnight News Held While Intraday Tried to Fade in Narrow Range)",
        Preset::BigGapHugeVolHalfFade => "Big Gap + Extreme Vol (3×+) + Change < Half Gap (Institutional Offloading at Gap Level; >50% Gap Fade on Conviction)",
        Preset::BigGapHugeVolFullExtension => "Big Gap + Extreme Vol (3×+) + Change > 1.5× Gap (Institutional Commitment Beyond the Gap; Momentum Continuation on Max Participation)",
        Preset::GapWithChangeWideRangeHotVol => "Gap-with-change + Wide Range + Hot Vol (Max-conviction Trend Day; Gap and Intraday Both Pushed Same Way Through Wide Exploration)",
        Preset::GapAgainstChangeWideRangeHotVol => "Gap-against-change + Wide Range + Hot Vol (Full Institutional Reversal; Intraday More Than Reversed the Gap with Wide-range Exploration)",
        Preset::HotVolNoChangeNoGapTightRange => "Hot Vol + Flat Change + Flat Gap + Tight Intraday (Heavy Participation Without Movement; Classic Absorption Pattern Disguised as Nothing)",
        Preset::ColdVolBigChangeWideRange => "Very Dry Vol + Big Change + Wide Range (Max Thin-tape Exception; Algorithmic Noise / Illiquid Stop-runs Without Real Conviction)",
        Preset::MicroPinTinyRangeHotVol => "Close Pinned at HOD=LOD + Hot Vol (Extreme Pin; Heavy Volume Crossed at One Price; Possible OPEX Pin / Block Cross)",
        Preset::MicroPinTinyRangeDryVol => "Close Pinned at HOD=LOD + Dry Vol (Dead Day; Market Truly Absent; Possibly Halted or Extremely Illiquid)",
        Preset::FullRange52wAtHighSide => "Near 52w High + >50% Above 52w Low + Decent Vol (Stock Has Doubled+ From the Floor; Max-uptrend at Decision Point)",
        Preset::FullRange52wAtLowSide => "Near 52w Low + >50% Below 52w High + Decent Vol (Stock Has Dropped 50%+ From the Ceiling; Max-downtrend at Decision Point)",
        Preset::PullbackAndRallyAtYearHigh => "Moderate Green + Green Intraday + Small Red Gap + Near 52w High + Decent Vol (Textbook Pullback-and-rally Near the Highs; Gap-down Bought Intraday)",
        Preset::DeadCatBounceAtYearLow => "Moderate Red + Red Intraday + Small Green Gap + Near 52w Low + Decent Vol (Textbook Dead-cat-bounce Near the Lows; Gap-up Sold Intraday)",
        Preset::GapDownIntradayReversalCloseAtHOD => "Gap Down + Up Intraday + Close at HOD + Decent Vol (Textbook Reversal-up Day; Gap-down Bought to Close at Highs)",
        Preset::GapUpIntradayReversalCloseAtLOD => "Gap Up + Down Intraday + Close at LOD + Decent Vol (Textbook Reversal-down Day; Gap-up Sold to Close at Lows)",
        Preset::BigGreenMidYearSweetSpot => "Big Green + 5-20% Below 52w High + Above 52w Low + Decent Vol (Sweet-spot Up Move; High-momentum Push From Mid-range; Room to Run)",
        Preset::BigRedMidYearSweetSpot => "Big Red + 5-20% Above 52w Low + Below 52w High + Decent Vol (Sweet-spot Down Move; High-momentum Drop From Mid-range; Room to Fall)",
        Preset::TripleZeroHotVol => "Gap + Change + Day All Near Zero + Hot Vol (Max-absorption Pattern; Massive Participation, Zero Movement; Institutional Cross at One Price)",
        Preset::TripleZeroDryVol => "Gap + Change + Day All Near Zero + Dry Vol (Universal Dormancy; Near-dead Tape on All Axes; Possibly Halted or Unattended)",
        Preset::ExtremeGapModerateMoveHotVol => "Huge Gap (>5%) + Moderate Retained Change (2-5%) + Hot Vol (Extreme Gap Retained Most But Not All; Partial-fill Move with Conviction)",
        Preset::ExtremeGapBigContinuationHotVol => "Extreme Gap (>5%) + Change Exceeds Gap + Hot Vol (Max-momentum Continuation; >100% Gap Extension on Volume)",
        Preset::BigGreenBigGapDryVol => "Big Green + Big Gap Up + Dry Vol (Huge Gap Held Without Participation; Suspect Rally; Fake Breakout or Stealth Squeeze in Thin Tape)",
        Preset::BigRedBigGapDownDryVol => "Big Red + Big Gap Down + Dry Vol (Huge Gap Held Without Participation; Suspect Breakdown; Forced-selling or Low-conviction Capitulation in Thin Tape)",
        Preset::SmoothBigGreenNormalVol => "Big Green + Normal Vol (1-1.5×) + Close at HOD + No Gap (Orderly Trend Day; Not Parabolic But Conviction; Sweet-spot Long)",
        Preset::SmoothBigRedNormalVol => "Big Red + Normal Vol + Close at LOD + No Gap (Orderly Down Day; Not Panic But Conviction; Sweet-spot Short)",
        Preset::BigDayPctFlatChangeHotVol => "Big Intraday Move + Flat Close + Decent Vol (Gap Absorbed All Intraday Move on Volume; Full Round-trip Gap-and-fade)",
        Preset::BigDayPctBigChangeAlignedHotVol => "Big Intraday + Big Change + Same Sign + Decent Vol (Max-aligned Trend Day; Gap and Intraday Push Same Way Through Big Move)",
        Preset::Year52HighBigDayDryVol => "Near 52w High + Big Green Intraday + Dry Vol (No-volume Push to New Highs; Distribution Suspicion / Fake Breakout)",
        Preset::Year52LowBigDayDryVol => "Near 52w Low + Big Red Intraday + Dry Vol (No-volume Push to New Lows; Capitulation Without Conviction / Thin-tape Breakdown)",
        Preset::MidMagnitudeGreenMidWickHotVol => "Moderate Green + Mid-wick Close + Decent Vol (Moderate-conviction Up Day with Mid-range Finish; Consolidation Candidate)",
        Preset::MidMagnitudeRedMidWickHotVol => "Moderate Red + Mid-wick Close + Decent Vol (Moderate-conviction Down Day with Mid-range Finish; Basing Candidate)",
        Preset::HotVolHugeRangeBigChange => "Extreme Vol (3×+) + Extreme Range (>6%) + Big Change (>4%) (Catalyst Day; Massive Participation, Wide Exploration, Big Finish)",
        Preset::HotVolHugeRangeFlatClose => "Extreme Vol + Extreme Range + Flat Close (Max-absorption Pattern at Scale; Institutional Indecision Day)",
        Preset::Year52HighDistributionChurn => "Near 52w High + Flat Close + Hot Vol (Distribution at the Highs; Heavy Churn Without Movement; Institutional Offloading at the Top)",
        Preset::Year52LowAccumulationChurn => "Near 52w Low + Flat Close + Hot Vol (Accumulation at the Lows; Heavy Churn Without Movement; Institutional Bottom-fishing at the Floor)",
        Preset::Year52HighBigGreenBreakoutHotVol => "New 52w High + Big Green + Hot Vol (Decisive Breakout from Year Resistance with Institutional Sponsorship; Trend-following Long Signal)",
        Preset::Year52LowBigRedBreakdownHotVol => "New 52w Low + Big Red + Hot Vol (Decisive Breakdown from Year Support with Institutional Sponsorship; Trend-following Short Signal)",
        Preset::GapUpFailBigRedHotVol => "Gap Up + Closed Red + Hot Vol (Failed Gap Up; Trapped Longs; Reversal Short Signal)",
        Preset::GapDownReclaimBigGreenHotVol => "Gap Down + Closed Green + Hot Vol (Reclaimed Gap Down; Trapped Shorts; Reversal Long Signal)",
        Preset::InsideRangeHotVolCoil => "Tight Intraday Range + Hot Vol (Inside-range Coil with Absorption; Pre-breakout Compression with Elevated Participation)",
        Preset::OutsideRangeFlatCloseHotVol => "Wide Intraday Range + Flat Close + Hot Vol (Outside-range Whip; High Participation but No Commitment; Institutional Indecision with Wide Whipsaw)",
        Preset::CloseAtHodTinyLodHotVol => "Close Pinned to HOD + LOD Far Below + Hot Vol (Full Intraday Range Claim; Momentum Buy Ramp into the Close with Elevated Participation)",
        Preset::CloseAtLodTinyHodHotVol => "Close Pinned to LOD + HOD Far Above + Hot Vol (Full Intraday Range Collapse; Momentum Sell Ramp into the Close with Elevated Participation)",
        Preset::BigGreenCloseAtHodHotVol => "Big Green + Close at HOD + Hot Vol (Strong Trend Day Closing on the Highs with Institutional Sponsorship; Trend-following Long Signal)",
        Preset::BigRedCloseAtLodHotVol => "Big Red + Close at LOD + Hot Vol (Strong Trend Day Closing on the Lows with Institutional Sponsorship; Trend-following Short Signal)",
        Preset::GapAndGoBigGreenCloseAtHod => "Gap Up + Continued Higher + Close at HOD + Hot Vol (Gap-and-go Continuation; Momentum Sustained through the Close)",
        Preset::GapAndDropBigRedCloseAtLod => "Gap Down + Continued Lower + Close at LOD + Hot Vol (Gap-and-drop Continuation; Selling Sustained through the Close)",
        Preset::GapUpFillReverseHotVol => "Gap Up + Partial Fill Reversal + Hot Vol (Gap-fill in Progress from Above; Partial Reversion to Mean with Elevated Participation)",
        Preset::GapDownFillReverseHotVol => "Gap Down + Partial Fill Reversal + Hot Vol (Gap-fill in Progress from Below; Partial Reversion to Mean with Elevated Participation)",
        Preset::Year52HighSqueezeShort => "New 52w High + Big Green + Extreme Vol (Short Squeeze at the Highs; Trapped Shorts Forced to Cover into Resistance Breakout)",
        Preset::Year52LowCapitulation => "New 52w Low + Big Red + Extreme Vol (Capitulation at the Lows; Forced Selling at Floor; Trapped Longs Flushed)",
        Preset::DragonflyDojiHotVol => "Flat Close + LOD Far Below + Close Near HOD + Hot Vol (Dragonfly Doji Recovery; Intraday Plunge Fully Reclaimed by Close with Elevated Participation)",
        Preset::GravestoneDojiHotVol => "Flat Close + HOD Far Above + Close Near LOD + Hot Vol (Gravestone Doji Rejection; Intraday Rip Fully Sold by Close with Elevated Participation)",
        Preset::HammerReversalHotVol => "Green Close + LOD Far Below + Close Near HOD + Hot Vol (Hammer Reversal; Intraday Plunge Reclaimed + Green Finish; Reversal Long Signal with Elevated Participation)",
        Preset::ShootingStarReversalHotVol => "Red Close + HOD Far Above + Close Near LOD + Hot Vol (Shooting Star Reversal; Intraday Rip Sold + Red Finish; Reversal Short Signal with Elevated Participation)",
        Preset::MarubozuGreenHotVol => "Big Green + Close at HOD + No Overnight Gap + Hot Vol (Green Marubozu; Full Intraday Trend Day with No Gap Aid; Max-conviction Long Built Entirely during Regular Hours)",
        Preset::MarubozuRedHotVol => "Big Red + Close at LOD + No Overnight Gap + Hot Vol (Red Marubozu; Full Intraday Trend Day with No Gap Aid; Max-conviction Short Built Entirely during Regular Hours)",
        Preset::Year52HighParabolicExtreme => "New 52w High + Parabolic Green + Extreme Vol (Parabolic Blow-off at New Highs; Exhaustion-vol Squeeze; Either Continuation Rocket or Terminal Top)",
        Preset::Year52LowParabolicExtreme => "New 52w Low + Parabolic Red + Extreme Vol (Panic Capitulation at New Lows; Exhaustion-vol Flush; Either Continuation or Terminal Bottom)",
        Preset::HotVolNoChangeTightRange => "Hot Vol + Tight Intraday Range + Flat Close + No Gap (Extreme Absorption Coil; Institutional Accumulation / Distribution with No Price Expansion; Pre-breakout Compression at Scale)",
        Preset::DryVolBigMoveNoFollow => "Dry Vol + Big Move + Tight Close Range (Low-participation Thrust; Illiquidity-driven Move without Follow-through; Fade Candidate)",
        Preset::BigGapBigContinuationBigRange => "Big Gap + 2x-Gap Continuation + Wide Range + Hot Vol (Gap-and-rip Extension; Momentum Doubled the Overnight Thrust during Regular Hours; Conviction Trend Day with Full Range Expansion)",
        Preset::BigGapFullReversalBigRange => "Big Gap + Sign-flipped Intraday Move + Wide Range + Hot Vol (Full Gap Reversal; Opposite-side Dominance after the Gap; Trapped Gap Traders Flushed Both Ways during the Session)",
        Preset::TinyGapBigMoveTightWicks => "No Gap + Big Intraday Move + Tight Wicks + Hot Vol (Clean Trend Day off the Open with No Gap Aid and Minimal Noise; Pure Directional Conviction Built Entirely Intraday)",
        Preset::BigGapTinyMoveTightWicks => "Big Gap + Flat Intraday + Tight Wicks + Hot Vol (Overnight Gap Held with Intraday Consolidation; Market Accepted the Gap with No Participation Rotation; Pre-extension Coil)",
        Preset::HotVolBigGreenWideRangeYearLow => "Big Green + Wide Range + Near 52w Low + Hot Vol (Bottom-fishing Thrust; Capitulation Reversal off the Floor with Elevated Participation; Potential Bear-trap Reversal)",
        Preset::HotVolBigRedWideRangeYearHigh => "Big Red + Wide Range + Near 52w High + Hot Vol (Distribution Flush; Rejection Reversal off the Ceiling with Elevated Participation; Potential Bull-trap Reversal)",
        Preset::HotVolBigGreenWideRangeYearHigh => "Big Green + Wide Range + Near 52w High + Hot Vol (Breakout Extension off the Ceiling with Elevated Participation; Trend Acceleration into New Highs with Full Range Expansion)",
        Preset::HotVolBigRedWideRangeYearLow => "Big Red + Wide Range + Near 52w Low + Hot Vol (Breakdown Extension off the Floor with Elevated Participation; Trend Acceleration into New Lows with Full Range Expansion)",
        Preset::RangeContractionHotVolBigGap => "Tight Intraday Range + Flat Close + Hot Vol + Big Overnight Gap (Gap Absorbed into Intraday Coil; Market Accepted the Gap with Elevated Participation but No Further Expansion; Trapped Gap Traders Compressing into a Spring)",
        Preset::RangeExpansionHotVolBigIntraday => "Wide Intraday Range + Big Intraday Move from Open + Hot Vol + No Overnight Gap (Full Intraday Range Expansion off the Open with No Gap Aid; Pure Intraday Breakout Day Driven by Regular-hours Conviction)",
        Preset::GapPlusDriveBullHotVol => "Gap Up + Big Intraday Drive Up + Hot Vol (Two-leg Bullish Conviction: Overnight Gap Held + Extended Further during Regular Hours; Gap-and-extend Trend Day)",
        Preset::GapPlusDriveBearHotVol => "Gap Down + Big Intraday Drive Down + Hot Vol (Two-leg Bearish Conviction: Overnight Gap Held + Extended Further during Regular Hours; Gap-and-extend Trend Day)",
        Preset::GapFadeBullDayPctOpposite => "Gap Down + Closed Up from Open + Net Green + Hot Vol (Full Gap-down Fade Reversal: Opened Lower, Recovered Intraday and Closed above Prior Close; Trapped Overnight Shorts Squeezed during Regular Hours)",
        Preset::GapFadeBearDayPctOpposite => "Gap Up + Closed Down from Open + Net Red + Hot Vol (Full Gap-up Fade Rejection: Opened Higher, Sold off Intraday and Closed below Prior Close; Trapped Overnight Longs Flushed during Regular Hours)",
        Preset::DayPctBigGreenChangeFlat => "Big Intraday Drive Up + Flat Net Close + Big Overnight Gap Down + Hot Vol (Full Gap-down Recovery: Opened Way below Prior Close, Rallied Hard Intraday and Finished Flat for the Session; Intraday Short-cover Squeeze Fully Unwound the Overnight Drop)",
        Preset::DayPctBigRedChangeFlat => "Big Intraday Drive Down + Flat Net Close + Big Overnight Gap Up + Hot Vol (Full Gap-up Rejection: Opened Way above Prior Close, Sold off Intraday and Finished Flat for the Session; Intraday Long-liquidation Fully Unwound the Overnight Pop)",
        Preset::Year52HighBreakoutOpenDriveHotVol => "New 52w High + Big Intraday Drive from Open + No Overnight Gap + Hot Vol (Intraday Breakout to New 52w High Built Entirely in Regular Hours with No Overnight Aid; Pure Conviction Breakout Day)",
        Preset::Year52LowBreakdownOpenDriveHotVol => "New 52w Low + Big Intraday Drive from Open + No Overnight Gap + Hot Vol (Intraday Breakdown to New 52w Low Built Entirely in Regular Hours with No Overnight Aid; Pure Conviction Breakdown Day)",
        Preset::Year52HighGapAndGoExtremeVol => "New 52w High + Big Gap Up + Intraday Continuation + Extreme Vol (Gap-and-go Breakout at New Highs with Overnight Gap Held and Extended during Regular Hours; Max-conviction Trend Acceleration)",
        Preset::Year52LowGapAndDropExtremeVol => "New 52w Low + Big Gap Down + Intraday Continuation + Extreme Vol (Gap-and-drop Breakdown at New Lows with Overnight Gap Held and Extended during Regular Hours; Max-conviction Trend Acceleration)",
        Preset::Year52HighFailedBreakoutFade => "Close Just below 52w High + Gap Up + Intraday Sold from Open + Red Close + Hot Vol (Failed Breakout at the Highs: Ran into Resistance, Gap Rejected and Faded All Session; Trapped Breakout Buyers Flushed during the Session)",
        Preset::Year52LowFailedBreakdownReclaim => "Close Just above 52w Low + Gap Down + Intraday Recovered from Open + Green Close + Hot Vol (Failed Breakdown at the Lows: Bounced off Support, Gap Reclaimed and Rallied All Session; Trapped Breakdown Shorts Squeezed during the Session)",
        Preset::Year52HighRangeCompressionLowVol => "Close Just below 52w High + Tight Intraday Range + Flat Close + Dry Vol (Low-vol Compression Just below Resistance; No Participation Rotation; Pre-breakout Coil at the Ceiling)",
        Preset::Year52LowRangeCompressionLowVol => "Close Just above 52w Low + Tight Intraday Range + Flat Close + Dry Vol (Low-vol Compression Just above Support; No Participation Rotation; Pre-breakdown Coil at the Floor)",
        Preset::DistantFromYearHighDryVolCoil => "Far below 52w High + Tight Intraday Range + Flat Close + Extremely Dry Vol (Deep-discount Basing; No Participation; Potential Turnaround Setup after Extended Pullback)",
        Preset::DistantFromYearLowDryVolCoil => "Far above 52w Low + Tight Intraday Range + Flat Close + Extremely Dry Vol (Deep-premium Basing; No Participation; Potential Exhaustion Setup after Extended Uptrend)",
        Preset::DistantFromYearHighBigGreenHotVol => "Far below 52w High + Big Green + Hot Vol (Snap-back Rally from Deep Discount; Mean-reversion Thrust with Elevated Participation toward the Prior Peak)",
        Preset::DistantFromYearLowBigRedHotVol => "Far above 52w Low + Big Red + Hot Vol (Snap-back Decline from Deep Premium; Mean-reversion Drop with Elevated Participation toward the Prior Trough)",
        Preset::MidRangeChurnHotVolBigDayPct => "Close Near Mid of Intraday Range + Big Intraday Move + Hot Vol (Mid-range Churn with Intraday Displacement; Net Move but No Follow-through to Either Extreme; Failed-trend Day with Continued Participation)",
        Preset::MidRangeChurnHotVolFlatDayPct => "Close Near Mid of Intraday Range + Flat Intraday Move + Hot Vol (Max-indecision Day at Scale; Full Range with No Net Direction and Elevated Participation; Institutional Indecision with Rotation)",
        Preset::Year52HighRetestPullbackDryVol => "Pulled Back 3-10 % from 52w High + Small Red + Dry Vol (Low-conviction Pullback toward Retest of Recent Highs; Potential Continuation Setup with Shallow Consolidation)",
        Preset::Year52LowRetestBounceDryVol => "Bounced 3-10 % off 52w Low + Small Green + Dry Vol (Low-conviction Bounce toward Retest of Recent Lows; Potential Continuation Setup with Shallow Rebound)",
        Preset::Year52HighRetestPullbackHotVol => "Pulled Back 3-10 % from 52w High + Meaningful Red + Hot Vol (High-conviction Pullback toward Retest of Recent Highs; Institutional Profit-taking with Elevated Participation; Potential Continuation Setup)",
        Preset::Year52LowRetestBounceHotVol => "Bounced 3-10 % off 52w Low + Meaningful Green + Hot Vol (High-conviction Bounce toward Retest of Recent Lows; Institutional Bottom-fishing with Elevated Participation; Potential Continuation Setup)",
        Preset::HotVolBigChangeDayPctOpposite => "Big Net Move + Intraday Move in Opposite Direction + Hot Vol (Intraday Reversal Fading the Prior-close Direction: Gap Dominated the Net Change, but Regular Hours Pushed Back the Other Way with Elevated Participation)",
        Preset::HotVolBigChangeDayPctAligned => "Big Net Move + Intraday Move Aligned with Same Direction + Hot Vol (Full-conviction Directional Day: Both Overnight + Regular Hours Pushed the Same Direction with Elevated Participation; Two-leg Trend Confirmation)",
        Preset::Year52HighBreakoutHotVolNoExtreme => "New 52w High + Modest Green + Moderate Hot Vol (Controlled-conviction Breakout to New Highs; Institutional Accumulation without Exhaustion; Sustainable Trend Continuation Candidate)",
        Preset::Year52LowBreakdownHotVolNoExtreme => "New 52w Low + Modest Red + Moderate Hot Vol (Controlled-conviction Breakdown to New Lows; Institutional Distribution without Panic; Sustainable Trend Continuation Candidate)",
        Preset::BigGreenTopWickRejectHotVol => "Green Close + HOD Far Above (Long Upper Wick) + Hot Vol (Upper-wick Rejection on a Green Day: Rally Faded into the Close but Still Finished Green; Supply Tested with Elevated Participation; Potential Follow-through Hesitation)",
        Preset::BigRedBottomWickRejectHotVol => "Red Close + LOD Far Below (Long Lower Wick) + Hot Vol (Lower-wick Rejection on a Red Day: Sell-off Bounced into the Close but Still Finished Red; Demand Tested with Elevated Participation; Potential Follow-through Hesitation)",
        Preset::DryVolGreenCloseAtHodTinyRange => "Small Green + Close at HOD + Tight Intraday Range + Dry Vol (Low-conviction Grind-up Day; Small Directional Drift with No Participation; Weak-hands Trend Continuation Candidate)",
        Preset::DryVolRedCloseAtLodTinyRange => "Small Red + Close at LOD + Tight Intraday Range + Dry Vol (Low-conviction Grind-down Day; Small Directional Drift with No Participation; Weak-hands Trend Continuation Candidate)",
        Preset::Year52HighGapDownDryVolReclaim => "Near 52w High + Gap Down Opening + Recovered to Positive Close + Dry Vol (Gap Reclaimed Back into the Breakout Zone on Light Vol; Weak-hands Shaken Out without Participation Flush; Bullish Continuation Setup at the Highs)",
        Preset::Year52LowGapUpDryVolReject => "Near 52w Low + Gap Up Opening + Sold Back into Red Close + Dry Vol (Gap Rejected Back into the Breakdown Zone on Light Vol; Weak-hands Trapped without Participation Flush; Bearish Continuation Setup at the Lows)",
        Preset::Year52HighInsideDayHotVol => "Near 52w High + Tight Intraday Range + Flat Close + Hot Vol (Inside-day Coil at the Breakout Zone with Absorption; Institutional Accumulation Just below Resistance; High-probability Breakout Setup)",
        Preset::Year52LowInsideDayHotVol => "Near 52w Low + Tight Intraday Range + Flat Close + Hot Vol (Inside-day Coil at the Breakdown Zone with Absorption; Institutional Distribution Just above Support; High-probability Breakdown Setup)",
        Preset::Year52HighOutsideDayHotVol => "Near 52w High + Wide Intraday Range + Hot Vol (Outside-day Rotation at the Breakout Zone; Both Supply and Demand Active Just below Resistance; Volatility Expansion Preceding Directional Resolution)",
        Preset::Year52LowOutsideDayHotVol => "Near 52w Low + Wide Intraday Range + Hot Vol (Outside-day Rotation at the Breakdown Zone; Both Supply and Demand Active Just above Support; Volatility Expansion Preceding Directional Resolution)",
        Preset::YearHighGapDownHotVolRecovery => "New 52w High Prior + Gap Down Opening + Huge Intraday Recovery + Green Close + Hot Vol (Failed Gap-down at the Highs: Shorts Overpressed Overnight, Intraday Short-squeeze Fully Reclaimed and Pushed Back into Trend with Elevated Participation)",
        Preset::YearLowGapUpHotVolRejection => "New 52w Low Prior + Gap Up Opening + Huge Intraday Rejection + Red Close + Hot Vol (Failed Gap-up at the Lows: Longs Overpressed Overnight, Intraday Long-liquidation Fully Unwound and Pushed Back into Trend with Elevated Participation)",
        Preset::Year52HighReclaimAfterFlush => "New 52w High + LOD Far Below + Close Near HOD + Green Close + Hot Vol (Intraday Flush below the Breakout Level Reclaimed back to Highs by Close; Trapped Breakdown Shorts Squeezed; Conviction Continuation Candidate)",
        Preset::Year52LowReclaimAfterPop => "New 52w Low + HOD Far Above + Close Near LOD + Red Close + Hot Vol (Intraday Pop above the Breakdown Level Rejected back to Lows by Close; Trapped Breakout Longs Flushed; Conviction Continuation Candidate)",
        Preset::BigDayPctSmallChangeHotVol => "Big Intraday Move + Flat Net Close + Hot Vol (Full Intraday Reversal of Overnight Position: Regular Hours Fully Unwound Any Prior-close Drift with Elevated Participation; Rotation Day with No Net Commitment)",
        Preset::SmallDayPctBigChangeHotVol => "Flat Intraday Move + Big Net Close + Hot Vol (Overnight Gap Held Intact through Regular Hours: the Entire Daily Move Was the Gap, Intraday Flat Acceptance with Elevated Participation; Gap-acceptance Day)",
        Preset::Year52HighRangeExpansionHotVol => "New 52w High + Wide Intraday Range + Green Close + Hot Vol (Volatility-expansion Breakout at New Highs: Wide-range Trend Day after the Breakout Level; Institutional Follow-through with Elevated Participation)",
        Preset::Year52LowRangeExpansionHotVol => "New 52w Low + Wide Intraday Range + Red Close + Hot Vol (Volatility-expansion Breakdown at New Lows: Wide-range Trend Day after the Breakdown Level; Institutional Follow-through with Elevated Participation)",
        Preset::Year52HighRangeContractionHotVol => "New 52w High + Tight Intraday Range + Small Green Close + Hot Vol (Post-breakout Absorption Coil at New Highs: Tight Digestion with Elevated Participation; Pre-extension Consolidation)",
        Preset::Year52LowRangeContractionHotVol => "New 52w Low + Tight Intraday Range + Small Red Close + Hot Vol (Post-breakdown Absorption Coil at New Lows: Tight Digestion with Elevated Participation; Pre-continuation Consolidation)",
        Preset::Year52HighBreakoutDryVolPullback => "Just below 52w High + Small Red Pullback + Dry Vol (Low-vol Pullback to Retest the Prior Breakout Level; Weak-hands Shaken Out without Participation; Reclaim Setup)",
        Preset::Year52LowBreakdownDryVolBounce => "Just above 52w Low + Small Green Bounce + Dry Vol (Low-vol Bounce to Retest the Prior Breakdown Level; Weak-hands Shaken Out without Participation; Rejection Setup)",
        Preset::BigGapBigCounterMoveBigRangeHotVol => "Big Gap + Big Opposite Intraday Move + Wide Range + Hot Vol (Gap-and-fight Reversal: Significant Overnight Gap Fully Battled by the Intraday Session; Two-sided Rotation Day with Extended Range and Elevated Participation; Trapped Gap Traders Flushed)",
        Preset::BigGapBigContinuationBigDayPctHotVol => "Big Gap + Big Same-direction Intraday + Wide Range + Hot Vol (Gap-and-go Acceleration: Overnight Gap Held + Extended Further Intraday with Wide Range; Two-leg Directional Conviction with Full Range Expansion)",
        Preset::NoGapBigChangeBigDayPctHotVol => "No Overnight Gap + Big Net Move + Big Intraday Move + Hot Vol (Intraday-only Conviction Trend Day: Entire Move Built during Regular Hours with No Overnight Aid + Matching Intraday Displacement; Pure Regular-hours Directional Thrust)",
        Preset::MidYearHighBigGreenHotVol => "Middle-of-year-range + Big Green + Hot Vol (Mid-range Bullish Thrust Well above the Floor but Well below the Ceiling; Institutional Momentum without Breakout-fatigue or Basing Context)",
        Preset::MidYearHighBigRedHotVol => "Middle-of-year-range + Big Red + Hot Vol (Mid-range Bearish Thrust Well below the Ceiling but Well above the Floor; Institutional Distribution without Breakdown-fatigue or Topping Context)",
        Preset::MidYearLowBigRedHotVol => "Middle-of-year-range from Low + Big Red + Hot Vol (Rejection Thrust Well off the Floor but Still Well below the Ceiling; Institutional Distribution in the Rebuild Zone)",
        Preset::MidYearLowBigGreenHotVol => "Middle-of-year-range from Low + Big Green + Hot Vol (Continuation Thrust off the Floor with Institutional Accumulation in the Rebuild Zone; Recovery Momentum without Near-extreme Volatility)",
        Preset::Year52HighFullRangeDryVol => "At 52w High + Wide Intraday Range + Dry Vol (Low-participation Outside-day Rotation at the Highs; Supply Tested without Conviction; Failed Exhaustion-vol Setup)",
        Preset::Year52LowFullRangeDryVol => "At 52w Low + Wide Intraday Range + Dry Vol (Low-participation Outside-day Rotation at the Lows; Demand Tested without Conviction; Failed Capitulation-vol Setup)",
        Preset::BigChangeBigRangeDryVol => "Big Net Move + Wide Intraday Range + Dry Vol (No-participation Thrust + Wide Range; Illiquidity-driven Volatility Expansion without Institutional Commitment; Fade Candidate at Scale)",
        Preset::ExtremeVolFlatDay => "Extreme Vol + Flat Net + Tight Range (Stealth Absorption at Scale: Extreme Participation with No Price Expansion; Institutional Accumulation or Distribution Masked as a Quiet Day)",
        Preset::ExtremeVolBigChangeClimax => "Extreme Vol + Big Net Move (Climax-style Print: Extreme Participation + Extreme Directional Commitment; Potential Trend Continuation or Terminal Exhaustion Depending on Follow-through)",
        Preset::ExtremeGapBigContinuationExtremeVol => "Extreme Gap + Same-direction Extreme Continuation + Extreme Vol (Gap-and-go Acceleration at Extreme Scale: Overnight Thrust Extended Further during Regular Hours with Climax-level Participation; Max-conviction Trend Day)",
        Preset::ExtremeGapFullReversalExtremeVol => "Extreme Gap + Sign-flipped Net Close + Extreme Vol (Extreme-gap Fade: Overnight Thrust Fully Reversed by the Intraday Session with Climax-level Participation; Trapped Gap Traders Flushed at Scale)",
        Preset::ApathyAtYearHigh => "At 52w High + Extreme Dry Vol + Flat Close + Tight Range (Total Apathy at the Breakout Zone; Neither Buyers nor Sellers Committed; Coiled-spring Pre-breakout Setup with No Participation Flush Yet)",
        Preset::ApathyAtYearLow => "At 52w Low + Extreme Dry Vol + Flat Close + Tight Range (Total Apathy at the Breakdown Zone; Neither Buyers nor Sellers Committed; Coiled-spring Pre-breakdown Setup with No Participation Flush Yet)",
        Preset::StealthAtYear52High => "At 52w High + Flat Close + Extreme Vol (Stealth Distribution at the Breakout Zone: Extreme Participation with No Net Price Expansion; Institutional Offloading Masked as Quiet Acceptance)",
        Preset::StealthAtYear52Low => "At 52w Low + Flat Close + Extreme Vol (Stealth Accumulation at the Breakdown Zone: Extreme Participation with No Net Price Expansion; Institutional Bottom-fishing Masked as Quiet Acceptance)",
        Preset::ExtremeVolCloseAtHod => "Extreme Vol + Close Pinned to HOD (Max-conviction Bullish Close at Any Price Level: Extreme Participation Finishing on the Highs; Institutional Ramp into the Close)",
        Preset::ExtremeVolCloseAtLod => "Extreme Vol + Close Pinned to LOD (Max-conviction Bearish Close at Any Price Level: Extreme Participation Finishing on the Lows; Institutional Dump into the Close)",
        Preset::ExtremeRangeExtremeVol => "Extreme Intraday Range + Extreme Vol (Extreme Two-sided Rotation: Institutional Fight Day with Wide Whipsaw Range and Climax-level Participation; Max-volatility Regime Print)",
        Preset::ExtremeRangeDryVol => "Extreme Intraday Range + Dry Vol (Thin-liquidity Whipsaw: Extreme Range Expansion with No Institutional Sponsorship; Gappy Market-maker Void or Low-volume Rip; Fade with Caution)",
        Preset::BigGreenUpperRangeHotVol => "Big Green + Close Clearly in Upper Portion of Intraday Range + Hot Vol (Bullish Strength Close in the Top Half of the Intraday Range without Requiring Close Pinned to HOD; Demand-side Dominance with Elevated Participation)",
        Preset::BigRedLowerRangeHotVol => "Big Red + Close Clearly in Lower Portion of Intraday Range + Hot Vol (Bearish Weakness Close in the Bottom Half of the Intraday Range without Requiring Close Pinned to LOD; Supply-side Dominance with Elevated Participation)",
        Preset::BigBreakoutAboveYearHigh => "Close More than 3% above the Prior 52w High + Green + Hot Vol (Deep Breakout Extension: Not Just a Fresh Peak but Materially above It; Price-discovery Expansion with Elevated Participation)",
        Preset::BigBreakdownBelowYearLow => "Close More than 3% below the Prior 52w Low + Red + Hot Vol (Deep Breakdown Extension: Not Just a Fresh Trough but Materially below It; Price-discovery Contraction with Elevated Participation)",
        Preset::DeepPullbackBigGreenHotVol => "10-30 % below 52w High + Big Green + Hot Vol (Recovery Thrust from Deep-pullback Zone with Institutional Accumulation; Pre-retest Momentum Candidate)",
        Preset::DeepPullbackBigRedHotVol => "10-30 % below 52w High + Big Red + Hot Vol (Continuation Thrust Deeper into Pullback Zone with Institutional Distribution; Trend-break Confirmation Candidate)",
        Preset::DeepBounceBigGreenHotVol => "10-30 % above 52w Low + Big Green + Hot Vol (Continuation Thrust Away from the Floor with Institutional Accumulation; Recovery Momentum Well off the Trough)",
        Preset::DeepBounceBigRedHotVol => "10-30 % above 52w Low + Big Red + Hot Vol (Retracement Thrust Back toward the Floor with Institutional Distribution; Bounce-failure Confirmation Candidate)",
        Preset::BigGapDownReclaimedToHodHotVol => "Gap Down + Close Pinned to HOD + Hot Vol (Full Intraday Recovery from the Gap-down Open to the Session High; Trapped Overnight Shorts Squeezed All the Way to the Highs with Elevated Participation)",
        Preset::BigGapUpRejectedToLodHotVol => "Gap Up + Close Pinned to LOD + Hot Vol (Full Intraday Rejection from the Gap-up Open to the Session Low; Trapped Overnight Longs Flushed All the Way to the Lows with Elevated Participation)",
        Preset::TenXVolMicroChange => "10x Average Vol + Microchange Close (Rare Absorption-at-scale Print: Extreme Participation with Virtually No Net Price Movement; Large Institutional Position-build or Unwind Masked as a Quiet Day)",
        Preset::TenXVolNoGapBigIntradayMove => "10x Average Vol + No Overnight Gap + Big Intraday Move (Pure Regular-hours Extreme Thrust: No Overnight Aid, Climax-level Participation, All Directional Commitment Built during the Session)",
        Preset::MicroVolBigChange => "10 % of Average Vol + Big Net Move (Dead-stock Surprise: Illiquidity-driven Extreme Move with Virtually No Participation; Thin Market-maker Quote Rip or Holiday/holiday-eve Thin-tape Print)",
        Preset::MicroVolFlatDay => "10 % of Average Vol + Flat Close + Tight Range (Total Dead Stock: No Participation and No Price Movement; Delisting Candidate or Fully Forgotten Name)",
        Preset::ConfirmedBreakoutAboveYearHigh => "1-3 % above Prior 52w High + Green + Hot Vol (Solid Confirmed Breakout: Clearly Past Resistance but Not Yet Parabolic; Trend-establishment Zone for New Highs)",
        Preset::ConfirmedBreakdownBelowYearLow => "1-3 % below Prior 52w Low + Red + Hot Vol (Solid Confirmed Breakdown: Clearly Past Support but Not Yet Panicked; Trend-establishment Zone for New Lows)",
        Preset::UpperWickFlatCloseHotVol => "Long Upper Wick + Flat Net Close + No Overnight Gap + Hot Vol (Pure Supply Test: Intraday Rally Rejected Back to Roughly the Open with Elevated Participation; Neither Bull nor Bear Net but Ceiling Tested)",
        Preset::LowerWickFlatCloseHotVol => "Long Lower Wick + Flat Net Close + No Overnight Gap + Hot Vol (Pure Demand Test: Intraday Sell-off Bounced Back to Roughly the Open with Elevated Participation; Neither Bull nor Bear Net but Floor Tested)",
        Preset::PartialGapUpHoldHotVol => "Gap Up + Intraday Sold from Open + Still Closed Green + Hot Vol (Partial Gap-up Fade: Overnight Thrust Partially Eroded Intraday but the Gap Held the Prior Close; Tested but Not Flushed)",
        Preset::PartialGapDownHoldHotVol => "Gap Down + Intraday Recovered from Open + Still Closed Red + Hot Vol (Partial Gap-down Fade: Overnight Thrust Partially Recovered Intraday but the Gap Held below the Prior Close; Tested but Not Reclaimed)",
        Preset::BreakoutZoneRangeExpansionHotVol => "Fresh 52w Breakout Zone + Wide Intraday Range + Hot Vol (Volatility Expansion Right at the Breakout Level: Institutional Fight Day Occurring as New High Is Being Established with Elevated Participation)",
        Preset::BreakdownZoneRangeExpansionHotVol => "Fresh 52w Breakdown Zone + Wide Intraday Range + Hot Vol (Volatility Expansion Right at the Breakdown Level: Institutional Fight Day Occurring as New Low Is Being Established with Elevated Participation)",
        Preset::Year52HighFreshConsolidationDryVol => "Close above Prior 52w High + Tight Intraday Range + Flat Close + Dry Vol (Quiet Acceptance of the New High: Post-breakout Consolidation without Participation Flush; Move Stalled but Did Not Reverse)",
        Preset::Year52LowFreshConsolidationDryVol => "Close below Prior 52w Low + Tight Intraday Range + Flat Close + Dry Vol (Quiet Acceptance of the New Low: Post-breakdown Consolidation without Participation Flush; Move Stalled but Did Not Reverse)",
        Preset::ModerateGapBullContinuationHotVol => "Moderate Gap Up + Bull Continuation + Hot Vol (Modest Gap Held + Extended Further during Regular Hours; the In-between Gap Regime Not Large Enough for Blow-off but Big Enough for Directional Commitment with Elevated Participation)",
        Preset::ModerateGapBearContinuationHotVol => "Moderate Gap Down + Bear Continuation + Hot Vol (Modest Gap Held + Extended Further during Regular Hours; the In-between Gap Regime Not Large Enough for Panic but Big Enough for Directional Commitment with Elevated Participation)",
        Preset::ModerateGapBullContinuationDryVol => "Moderate Gap Up + Bull Continuation + Dry Vol (Modest Gap Extended on No Participation; Suspect Rally without Institutional Sponsorship; Fade-prone Setup despite Positive Net Move)",
        Preset::ModerateGapBearContinuationDryVol => "Moderate Gap Down + Bear Continuation + Dry Vol (Modest Gap Extended on No Participation; Suspect Decline without Institutional Sponsorship; Fade-prone Setup despite Negative Net Move)",
        Preset::ModerateGapBullFadeHotVol => "Moderate Gap Up + Closed Red + Hot Vol (Moderate-gap Fade Reversal: Gap Up Was Sold throughout the Session below Prior Close; Trapped Overnight Longs Flushed with Elevated Participation; Reversal-short Signal)",
        Preset::ModerateGapBearReclaimHotVol => "Moderate Gap Down + Closed Green + Hot Vol (Moderate-gap Reclaim Reversal: Gap Down Was Bought throughout the Session above Prior Close; Trapped Overnight Shorts Squeezed with Elevated Participation; Reversal-long Signal)",
        Preset::ConfirmedBreakoutFadeHotVol => "1-3 % above Prior 52w High + Red + Hot Vol (Confirmed-breakout Pullback: Was Clearly Past Resistance, Now Dropping Back into the Breakout Zone with Elevated Participation; Failed-breakout Risk Signal)",
        Preset::ConfirmedBreakdownReclaimHotVol => "1-3 % below Prior 52w Low + Green + Hot Vol (Confirmed-breakdown Bounce: Was Clearly Past Support, Now Rallying Back into the Breakdown Zone with Elevated Participation; Failed-breakdown Risk Signal)",
        Preset::IntradayBullDriveAtYear52High => "At 52w High + Big Intraday Drive Up + Hot Vol (Intraday-led Bullish Thrust at the Breakout Zone Regardless of Overnight Context; Regular-hours Momentum Confirmation Right at Resistance)",
        Preset::IntradayBearDriveAtYear52Low => "At 52w Low + Big Intraday Drive Down + Hot Vol (Intraday-led Bearish Thrust at the Breakdown Zone Regardless of Overnight Context; Regular-hours Momentum Confirmation Right at Support)",
        Preset::IntradayBearDriveAtYear52High => "At 52w High + Big Intraday Drive Down + Hot Vol (Intraday-led Bearish Rejection at the Breakout Zone: Selling Pressure through Regular Hours Pushed the Price Back from the Highs with Elevated Participation; Failed-breakout Candidate)",
        Preset::IntradayBullDriveAtYear52Low => "At 52w Low + Big Intraday Drive Up + Hot Vol (Intraday-led Bullish Recovery at the Breakdown Zone: Buying Pressure through Regular Hours Lifted the Price off the Lows with Elevated Participation; Failed-breakdown Candidate)",
        Preset::IntradayBullDriveBelowYearHigh => "Within 5% below 52w High + Big Intraday Drive Up + Hot Vol (Pre-breakout Intraday Surge: Approaching Resistance with Regular-hours Momentum and Elevated Participation; Breakout-setup Candidate)",
        Preset::IntradayBearDriveAboveYearLow => "Within 5% above 52w Low + Big Intraday Drive Down + Hot Vol (Pre-breakdown Intraday Plunge: Approaching Support with Regular-hours Momentum and Elevated Participation; Breakdown-setup Candidate)",
        Preset::HammerAtYear52Low => "At 52w Low + Long Lower Wick + Close Near HOD + Green Close + Hot Vol (Classic Hammer Reversal at the Breakdown Floor: Intraday Plunge Reclaimed with Green Finish and Elevated Participation; High-probability Bottom-fishing Signal)",
        Preset::ShootingStarAtYear52High => "At 52w High + Long Upper Wick + Close Near LOD + Red Close + Hot Vol (Classic Shooting Star Reversal at the Breakout Ceiling: Intraday Rip Sold with Red Finish and Elevated Participation; High-probability Topping Signal)",
        Preset::MarubozuGreenAtYear52High => "At 52w High + Green Marubozu + No Overnight Gap + Hot Vol (Full Intraday Breakout Trend Day at the Breakout Zone: Regular-hours Conviction Climbed from the Open to the High with No Gap Aid; Max-conviction Breakout Day)",
        Preset::MarubozuRedAtYear52Low => "At 52w Low + Red Marubozu + No Overnight Gap + Hot Vol (Full Intraday Breakdown Trend Day at the Breakdown Zone: Regular-hours Conviction Fell from the Open to the Low with No Gap Aid; Max-conviction Breakdown Day)",
        Preset::DragonflyDojiAtYear52Low => "At 52w Low + Flat Close + LOD Far Below + Close Near HOD + Hot Vol (Classic Dragonfly Doji at the Breakdown Floor: Intraday Plunge Fully Reclaimed by the Close with Elevated Participation; High-probability Bottom-fishing Signal at the Year Low)",
        Preset::GravestoneDojiAtYear52High => "At 52w High + Flat Close + HOD Far Above + Close Near LOD + Hot Vol (Classic Gravestone Doji at the Breakout Ceiling: Intraday Rip Fully Sold by the Close with Elevated Participation; High-probability Topping Signal at the Year High)",
        Preset::HammerAtMidYearLowRange => "Mid-range from Low (5-20 % above 52w Low) + Long Lower Wick + Close Near HOD + Green Close + Hot Vol (Hammer Reversal in the Recovery Zone: Intraday Plunge Reclaimed with Green Finish above the Floor; Mid-cycle Bottom-fishing Signal)",
        Preset::ShootingStarAtMidYearHighRange => "Mid-range from High (5-20 % below 52w High) + Long Upper Wick + Close Near LOD + Red Close + Hot Vol (Shooting Star Reversal in the Topping Zone: Intraday Rip Sold with Red Finish below the Ceiling; Mid-cycle Topping Signal)",
        Preset::HammerAtDeepPullback => "Deep Pullback Zone (10-30 % below 52w High) + Long Lower Wick + Close Near HOD + Green Close + Hot Vol (Hammer Reversal Deep into the Pullback: Intraday Plunge Reclaimed in the Rebuild Zone; Counter-trend Bounce Candidate)",
        Preset::ShootingStarAtDeepBounce => "Deep Bounce Zone (10-30 % above 52w Low) + Long Upper Wick + Close Near LOD + Red Close + Hot Vol (Shooting Star Reversal Deep into the Bounce: Intraday Rip Sold in the Rebuild Zone; Counter-trend Rejection Candidate)",
        Preset::DragonflyDojiAtMidYearLow => "Mid-range from Low (5-20 % above 52w Low) + Flat Close + LOD Far Below + Close Near HOD + Hot Vol (Dragonfly Doji Reversal in the Recovery Zone: Intraday Plunge Fully Reclaimed by Close in the Mid-range; Demand-test Signal away from the Floor)",
        Preset::GravestoneDojiAtMidYearHigh => "Mid-range from High (5-20 % below 52w High) + Flat Close + HOD Far Above + Close Near LOD + Hot Vol (Gravestone Doji Reversal in the Topping Zone: Intraday Rip Fully Sold by Close in the Mid-range; Supply-test Signal away from the Ceiling)",
        Preset::DragonflyDojiAtDeepPullback => "Deep Pullback Zone (10-30 % below 52w High) + Flat Close + LOD Far Below + Close Near HOD + Hot Vol (Dragonfly Doji Reversal Deep into the Pullback: Intraday Plunge Reclaimed in the Rebuild Zone with Flat Close; Counter-trend Demand-test Signal)",
        Preset::GravestoneDojiAtDeepBounce => "Deep Bounce Zone (10-30 % above 52w Low) + Flat Close + HOD Far Above + Close Near LOD + Hot Vol (Gravestone Doji Reversal Deep into the Bounce: Intraday Rip Sold in the Rebuild Zone with Flat Close; Counter-trend Supply-test Signal)",
        Preset::MarubozuGreenAtMidYearHigh => "Mid-range from High (5-20 % below 52w High) + Green Marubozu + No Overnight Gap + Hot Vol (Full Intraday Recovery Trend Day in the Pullback Zone: Regular-hours Conviction Lifted from the Open to the High with No Gap Aid; Max-conviction Mid-cycle Bounce Day)",
        Preset::MarubozuRedAtMidYearLow => "Mid-range from Low (5-20 % above 52w Low) + Red Marubozu + No Overnight Gap + Hot Vol (Full Intraday Retracement Trend Day in the Bounce Zone: Regular-hours Conviction Fell from the Open to the Low with No Gap Aid; Max-conviction Mid-cycle Rejection Day)",
        Preset::MarubozuGreenAtDeepPullback => "Deep Pullback Zone (10-30 % below 52w High) + Green Marubozu + No Overnight Gap + Hot Vol (Full Intraday Recovery Trend Day Deep in the Pullback: Regular-hours Conviction Lifted from the Open to the High with No Gap Aid; Max-conviction Counter-trend Bounce Thrust)",
        Preset::MarubozuRedAtDeepBounce => "Deep Bounce Zone (10-30 % above 52w Low) + Red Marubozu + No Overnight Gap + Hot Vol (Full Intraday Retracement Trend Day Deep in the Bounce: Regular-hours Conviction Fell from the Open to the Low with No Gap Aid; Max-conviction Counter-trend Rejection Thrust)",
        Preset::HammerAtDeepDiscount => "Deep Discount Zone (>=30 % below 52w High) + Long Lower Wick + Close Near HOD + Green Close + Hot Vol (Hammer Reversal at the Deep-discount Floor: Intraday Plunge Reclaimed in a Beaten-down Name with Elevated Participation; Turnaround Candidate after Extended Decline)",
        Preset::ShootingStarAtDeepPremium => "Deep Premium Zone (>=30 % above 52w Low) + Long Upper Wick + Close Near LOD + Red Close + Hot Vol (Shooting Star Reversal at the Deep-premium Ceiling: Intraday Rip Sold in a Runaway Name with Elevated Participation; Exhaustion Candidate after Extended Advance)",
        Preset::DragonflyDojiAtDeepDiscount => "Deep Discount Zone (>=30 % below 52w High) + Flat Close + LOD Far Below + Close Near HOD + Hot Vol (Dragonfly Doji Reversal in the Deep-discount Floor: Intraday Plunge Fully Reclaimed by Close in a Beaten-down Name with Elevated Participation; Turnaround Demand-test Signal)",
        Preset::GravestoneDojiAtDeepPremium => "Deep Premium Zone (>=30 % above 52w Low) + Flat Close + HOD Far Above + Close Near LOD + Hot Vol (Gravestone Doji Reversal in the Deep-premium Ceiling: Intraday Rip Fully Sold by Close in a Runaway Name with Elevated Participation; Exhaustion Supply-test Signal)",
        Preset::MarubozuGreenAtDeepDiscount => "Deep Discount Zone (>=30 % below 52w High) + Green Marubozu + No Overnight Gap + Hot Vol (Full Intraday Recovery Trend Day in a Beaten-down Name: Regular-hours Conviction Lifted from Open to High with No Gap Aid; Max-conviction Turnaround Thrust after Extended Decline)",
        Preset::MarubozuRedAtDeepPremium => "Deep Premium Zone (>=30 % above 52w Low) + Red Marubozu + No Overnight Gap + Hot Vol (Full Intraday Rejection Trend Day in a Runaway Name: Regular-hours Conviction Fell from Open to Low with No Gap Aid; Max-conviction Exhaustion Thrust after Extended Advance)",
        Preset::HammerAtYear52High => "At 52w High + Long Lower Wick + Close Near HOD + Green Close + Hot Vol (Buying Pressure Tested but Reclaimed at the Breakout Ceiling: Intraday Plunge Bought Back to the Highs; Continuation-of-trend Resilience Signal at the Top)",
        Preset::ShootingStarAtYear52Low => "At 52w Low + Long Upper Wick + Close Near LOD + Red Close + Hot Vol (Selling Pressure Persists at the Breakdown Floor: Intraday Rip Sold Back to the Lows; Continuation-of-trend Weakness Signal at the Bottom)",
        Preset::HammerAtMidYearHighRange => "Mid-range from High (5-20 % below 52w High) + Long Lower Wick + Close Near HOD + Green Close + Hot Vol (Buying Pressure Tested but Reclaimed in the Pullback Zone: Intraday Plunge Bought Back toward the Prior Peak; Bull-continuation Hammer Signaling Pullback Exhaustion)",
        Preset::ShootingStarAtMidYearLowRange => "Mid-range from Low (5-20 % above 52w Low) + Long Upper Wick + Close Near LOD + Red Close + Hot Vol (Selling Pressure Tested in the Bounce Zone: Intraday Rip Sold Back toward the Prior Trough; Bear-continuation Shooting Star Signaling Bounce Exhaustion)",
        Preset::HammerAtDeepBounceContinuation => "Deep Bounce Zone (10-30 % above 52w Low) + Long Lower Wick + Close Near HOD + Green Close + Hot Vol (Bull-continuation Hammer in the Rebuild Zone: Intraday Plunge Reclaimed without Retesting the Floor; Recovery Momentum Confirmation Deep into the Bounce)",
        Preset::ShootingStarAtDeepPullbackContinuation => "Deep Pullback Zone (10-30 % below 52w High) + Long Upper Wick + Close Near LOD + Red Close + Hot Vol (Bear-continuation Shooting Star in the Pullback Zone: Intraday Rip Sold without Retesting the Ceiling; Decline Momentum Confirmation Deep into the Pullback)",
        Preset::HammerAtDeepPremiumContinuation => "Deep Premium Zone (>=30 % above 52w Low) + Long Lower Wick + Close Near HOD + Green Close + Hot Vol (Bull-continuation Hammer in a Runaway Name: Intraday Plunge Reclaimed Deep in the Extended Advance; Trend-resilience Signal Far from the Floor)",
        Preset::ShootingStarAtDeepDiscountContinuation => "Deep Discount Zone (>=30 % below 52w High) + Long Upper Wick + Close Near LOD + Red Close + Hot Vol (Bear-continuation Shooting Star in a Beaten-down Name: Intraday Rip Sold Deep in the Extended Decline; Trend-weakness Signal Far from the Ceiling)",
        Preset::BothLongWicksHotVol => "Long Upper Wick + Long Lower Wick + Hot Vol (Two-sided Exploration Day with Elevated Participation: Both Supply and Demand Tested at Extremes; High-rotation Indecision Day with Full Intraday Whipsaw Range)",
        Preset::BothShortWicksTinyChangeHotVol => "Short Upper Wick + Short Lower Wick + Flat Close + Hot Vol (Compressed-cylinder Day: Close Pinned with No Wick Exploration on Either Side; Pre-breakout Coil with Elevated Absorption at a Specific Price)",
        Preset::HammerAtConfirmedBreakdown => "1-3 % below Prior 52w Low + Long Lower Wick + Close Near HOD + Green Close + Hot Vol (Failed-breakdown Hammer: Confirmed Breakdown Level Retested Intraday then Reclaimed Back above Prior Support; Potential Failed-breakdown Reversal Signal)",
        Preset::ShootingStarAtConfirmedBreakout => "1-3 % above Prior 52w High + Long Upper Wick + Close Near LOD + Red Close + Hot Vol (Failed-breakout Shooting Star: Confirmed Breakout Level Retested Intraday then Rejected Back below Prior Resistance; Potential Failed-breakout Reversal Signal)",
        Preset::DragonflyDojiAtConfirmedBreakdown => "1-3 % below Prior 52w Low + Flat Close + LOD Far Below + Close Near HOD + Hot Vol (Dragonfly Doji at Confirmed Breakdown: Intraday Plunge Fully Reclaimed by Close with Flat Finish above Prior Support; Failed-breakdown Demand-test Signal)",
        Preset::GravestoneDojiAtConfirmedBreakout => "1-3 % above Prior 52w High + Flat Close + HOD Far Above + Close Near LOD + Hot Vol (Gravestone Doji at Confirmed Breakout: Intraday Rip Fully Sold by Close with Flat Finish below Prior Resistance; Failed-breakout Supply-test Signal)",
        Preset::MarubozuGreenAtConfirmedBreakout => "1-3 % above Prior 52w High + Green Marubozu + No Overnight Gap + Hot Vol (Full Intraday Extension Trend Day after Confirmed Breakout: Regular-hours Conviction Lifted from Open to High with No Gap Aid; Max-conviction Follow-through above Resistance)",
        Preset::MarubozuRedAtConfirmedBreakdown => "1-3 % below Prior 52w Low + Red Marubozu + No Overnight Gap + Hot Vol (Full Intraday Extension Trend Day after Confirmed Breakdown: Regular-hours Conviction Fell from Open to Low with No Gap Aid; Max-conviction Follow-through below Support)",
        Preset::MarubozuRedAtConfirmedBreakout => "1-3 % above Prior 52w High + Red Marubozu + No Overnight Gap + Hot Vol (Full Intraday Rejection Trend Day after Confirmed Breakout: Regular-hours Conviction Fell from Open to Low with No Gap Aid; Max-conviction Failed-breakout Fade Returning below Resistance)",
        Preset::MarubozuGreenAtConfirmedBreakdown => "1-3 % below Prior 52w Low + Green Marubozu + No Overnight Gap + Hot Vol (Full Intraday Recovery Trend Day after Confirmed Breakdown: Regular-hours Conviction Lifted from Open to High with No Gap Aid; Max-conviction Failed-breakdown Reclaim above Support)",
        Preset::TripleAlignedBullBigConvictionDay => "Gap Up + Big Net Move + Big Intraday Up + Hot Vol (Triple-aligned Bullish Conviction: Overnight, Regular Hours, and Net All Moved Meaningfully in the Same Direction with Elevated Participation; Full-stack Directional Commitment Day)",
        Preset::TripleAlignedBearBigConvictionDay => "Gap Down + Big Net Move + Big Intraday Down + Hot Vol (Triple-aligned Bearish Conviction: Overnight, Regular Hours, and Net All Moved Meaningfully in the Same Direction with Elevated Participation; Full-stack Directional Commitment Day)",
        Preset::DistantFromYearHighRangeContractionHotVol => "Far below 52w High (>=20 %) + Tight Intraday Range + Flat Close + Hot Vol (Stealth Absorption in the Pullback Territory: Tight Digestion with Elevated Participation Deep below the Prior Peak; Pre-reversal Coil with Institutional Positioning)",
        Preset::DistantFromYearLowRangeContractionHotVol => "Far above 52w Low (>=20 %) + Tight Intraday Range + Flat Close + Hot Vol (Stealth Absorption in the Advance Territory: Tight Digestion with Elevated Participation Deep above the Prior Trough; Pre-reversal Coil with Institutional Positioning)",
        Preset::DistantFromYearHighRangeExpansionHotVol => "Far below 52w High (>=20 %) + Wide Intraday Range + Hot Vol (Volatility Expansion Deep in the Pullback Territory: Institutional Fight Day Occurring Well below the Prior Peak with Elevated Participation; Regime-shift Candidate after Extended Decline)",
        Preset::DistantFromYearLowRangeExpansionHotVol => "Far above 52w Low (>=20 %) + Wide Intraday Range + Hot Vol (Volatility Expansion Deep in the Advance Territory: Institutional Fight Day Occurring Well above the Prior Trough with Elevated Participation; Regime-shift Candidate after Extended Advance)",
        Preset::CloseAtHodMidYearLowHotVol => "Mid-range from Low (5-20 %) + Close Pinned to HOD + Green Close + Hot Vol (Closing-strength Signal in the Recovery Zone: Bull Conviction Ramped into the Close without Requiring a Long Lower-wick Reclaim; Demand-led Mid-cycle Continuation)",
        Preset::CloseAtLodMidYearHighHotVol => "Mid-range from High (5-20 %) + Close Pinned to LOD + Red Close + Hot Vol (Closing-weakness Signal in the Pullback Zone: Bear Conviction Dumped into the Close without Requiring a Long Upper-wick Rejection; Supply-led Mid-cycle Continuation)",
        Preset::CloseAtHodDeepBelowYearHighHotVol => "Far below 52w High (>=20 %) + Close Pinned to HOD + Green Close + Hot Vol (Closing-strength Signal Deep in the Pullback Zone: Bull Conviction Ramped into the Close Well below the Prior Peak; Early-reversal Candidate after Extended Decline without Requiring Wick-rejection Context)",
        Preset::CloseAtLodDeepAboveYearLowHotVol => "Far above 52w Low (>=20 %) + Close Pinned to LOD + Red Close + Hot Vol (Closing-weakness Signal Deep in the Advance Zone: Bear Conviction Dumped into the Close Well above the Prior Trough; Early-reversal Candidate after Extended Advance without Requiring Wick-rejection Context)",
        Preset::CloseAtHodNearYearHighHotVol => "At/near 52w High (<2 %) + Close Pinned to HOD + Green Close + Hot Vol (Freshest Possible Breakout Signal: Closing at the High of Day at the High of the Year on Elevated Participation; Momentum-continuation with No Overhead Supply)",
        Preset::CloseAtLodNearYearLowHotVol => "At/near 52w Low (<2 %) + Close Pinned to LOD + Red Close + Hot Vol (Freshest Possible Breakdown Signal: Closing at the Low of Day at the Low of the Year on Elevated Participation; Momentum-continuation with No Underlying Support)",
        Preset::CloseAtHodJustOffYearHighHotVol => "Just off 52w High (2-5 %) + Close Pinned to HOD + Green Close + Hot Vol (Re-assertion of Breakout Momentum after a Shallow Pullback: Closing Strength in the Immediate Post-extreme Zone Signals Continuation Candidate with Minimal Overhead Resistance)",
        Preset::CloseAtLodJustOffYearLowHotVol => "Just off 52w Low (2-5 %) + Close Pinned to LOD + Red Close + Hot Vol (Re-assertion of Breakdown Momentum after a Shallow Bounce: Closing Weakness in the Immediate Post-extreme Zone Signals Continuation Candidate with Minimal Underlying Support)",
        Preset::CloseAtHodConfirmedAboveYearHighHotVol => "Confirmed-breakout Zone (1-3 % past 52w High) + Close Pinned to HOD + Green Close + Hot Vol (Confirmed-breakout Closing Strength: Price Has Cleared the Prior Peak and Continues to Close at the Day's High; Momentum-continuation with Breakout Already Validated)",
        Preset::CloseAtLodConfirmedBelowYearLowHotVol => "Confirmed-breakdown Zone (1-3 % past 52w Low) + Close Pinned to LOD + Red Close + Hot Vol (Confirmed-breakdown Closing Weakness: Price Has Cleared the Prior Trough and Continues to Close at the Day's Low; Momentum-continuation with Breakdown Already Validated)",
        Preset::MidpointCloseNearYearHighHotVol => "At/near 52w High (<2 %) + Midpoint Close between HOD and LOD + Hot Vol (Stall at the 52w Extreme: Neither Bulls nor Bears Closed in Control at the High of the Year on Elevated Participation; Potential Indecision-reversal Candidate after Extended Advance)",
        Preset::MidpointCloseNearYearLowHotVol => "At/near 52w Low (<2 %) + Midpoint Close between HOD and LOD + Hot Vol (Stall at the 52w Extreme: Neither Bulls nor Bears Closed in Control at the Low of the Year on Elevated Participation; Potential Indecision-reversal Candidate after Extended Decline)",
        Preset::MidpointCloseConfirmedAboveYearHighHotVol => "Confirmed-breakout Zone (1-3 % past 52w High) + Midpoint Close between HOD and LOD + Hot Vol (Stall in the Confirmed-breakout Zone: Price Has Cleared the Prior Peak but Failed to Push Higher into the Close on Elevated Participation; Potential Failed-breakout Warning)",
        Preset::MidpointCloseConfirmedBelowYearLowHotVol => "Confirmed-breakdown Zone (1-3 % past 52w Low) + Midpoint Close between HOD and LOD + Hot Vol (Stall in the Confirmed-breakdown Zone: Price Has Cleared the Prior Trough but Failed to Push Lower into the Close on Elevated Participation; Potential Failed-breakdown Warning)",
        Preset::MidpointCloseDeepBelowYearHighHotVol => "Far below 52w High (>=20 %) + Midpoint Close between HOD and LOD + Hot Vol (Stall Deep in the Pullback Zone: Neither Bulls nor Bears Closed in Control Well below the Prior Peak after Extended Decline on Elevated Participation; Potential Trend-exhaustion Candidate)",
        Preset::MidpointCloseDeepAboveYearLowHotVol => "Far above 52w Low (>=20 %) + Midpoint Close between HOD and LOD + Hot Vol (Stall Deep in the Advance Zone: Neither Bulls nor Bears Closed in Control Well above the Prior Trough after Extended Advance on Elevated Participation; Potential Trend-exhaustion Candidate)",
        Preset::MidpointCloseMidYearHighHotVol => "Mid-range from High (5-20 %) + Midpoint Close between HOD and LOD + Hot Vol (Context-free Intraday Indecision in the Mid-cycle Pullback Zone: Neither Bulls nor Bears Closed in Control in the Proper Consolidation Range on Elevated Participation; Standoff in the Middle of the Year-range)",
        Preset::MidpointCloseMidYearLowHotVol => "Mid-range from Low (5-20 %) + Midpoint Close between HOD and LOD + Hot Vol (Context-free Intraday Indecision in the Mid-cycle Recovery Zone: Neither Bulls nor Bears Closed in Control in the Proper Consolidation Range on Elevated Participation; Standoff in the Middle of the Year-range)",
        Preset::MidpointCloseJustOffYearHighHotVol => "Just off 52w High (2-5 %) + Midpoint Close between HOD and LOD + Hot Vol (Post-tag Intraday Indecision: Neither Bulls nor Bears Closed in Control Immediately after Fresh Pullback from the 52w High on Elevated Participation; Standoff in the Post-extreme Zone)",
        Preset::MidpointCloseJustOffYearLowHotVol => "Just off 52w Low (2-5 %) + Midpoint Close between HOD and LOD + Hot Vol (Post-tag Intraday Indecision: Neither Bulls nor Bears Closed in Control Immediately after Fresh Bounce from the 52w Low on Elevated Participation; Standoff in the Post-extreme Zone)",
        Preset::GapUpCloseAtHodHotVol => "Gap Up (>2 %) + Close Pinned to HOD + Green Close + Hot Vol (Strongest Possible Bullish Gap: Gap up Held without Fade and Price Closed at the Day's High on Elevated Participation; Sustained Buying through the Bell with No Profit-taking)",
        Preset::GapDownCloseAtLodHotVol => "Gap Down (<-2 %) + Close Pinned to LOD + Red Close + Hot Vol (Strongest Possible Bearish Gap: Gap down Held without Bounce and Price Closed at the Day's Low on Elevated Participation; Sustained Selling through the Bell with No Dip-buying)",
        Preset::GapUpCloseAtLodHotVol => "Gap Up (>2 %) Faded Completely to LOD + Red Close + Hot Vol (Bull-trap Reversal: Sellers Absorbed the Entire Gap and Pushed below the Open on Elevated Participation; Classic Gap-and-reverse Failed-breakout Pattern)",
        Preset::GapDownCloseAtHodHotVol => "Gap Down (<-2 %) Absorbed Completely to HOD + Green Close + Hot Vol (Bear-trap Reversal: Buyers Absorbed the Entire Gap and Pushed above the Open on Elevated Participation; Classic Gap-and-reverse Failed-breakdown Pattern)",
        Preset::GapUpMidpointCloseHotVol => "Gap Up (>2 %) + Midpoint Close between HOD and LOD + Hot Vol (Inconclusive Gap-up Follow-through: Gap Held but Neither Extended to a HOD Close nor Failed to a LOD Close on Elevated Participation; Standoff Inside the Gap Day with No Directional Resolution)",
        Preset::GapDownMidpointCloseHotVol => "Gap Down (<-2 %) + Midpoint Close between HOD and LOD + Hot Vol (Inconclusive Gap-down Follow-through: Gap Held but Neither Extended to a LOD Close nor Absorbed to a HOD Close on Elevated Participation; Standoff Inside the Gap Day with No Directional Resolution)",
        Preset::BigGapUpCloseAtHodHotVol => "Large Gap Up (>5 %) + Close Pinned to HOD + Big Green Close + Hot Vol (Institutional-conviction Gap-up with No Profit-taking: Large Gap Held All Session and Closed at the Day's High on Doubled Participation; Earnings-reaction / News-driven Sustained Buying through the Bell)",
        Preset::BigGapDownCloseAtLodHotVol => "Large Gap Down (<-5 %) + Close Pinned to LOD + Big Red Close + Hot Vol (Institutional-conviction Gap-down with No Dip-buying: Large Gap Held All Session and Closed at the Day's Low on Doubled Participation; Earnings-disappointment / News-driven Sustained Selling through the Bell)",
        Preset::BigGapUpCloseAtLodHotVol => "Large Gap Up (>5 %) Completely Faded to LOD + Red Close + Hot Vol (Institutional Bull-trap Reversal: Dramatic Earnings-reaction Gap Fully Absorbed and Pushed below the Open on Doubled Participation; High-conviction Failed-breakout Signal Rare Enough to Mark a Regime-shift Candidate)",
        Preset::BigGapDownCloseAtHodHotVol => "Large Gap Down (<-5 %) Completely Absorbed to HOD + Green Close + Hot Vol (Institutional Bear-trap Reversal: Dramatic Capitulation Gap Fully Absorbed and Pushed above the Open on Doubled Participation; High-conviction Failed-breakdown Signal Rare Enough to Mark a Regime-shift Candidate)",
        Preset::BigGapUpMidpointCloseHotVol => "Large Gap Up (>5 %) + Midpoint Close between HOD and LOD + Hot Vol (Inconclusive Institutional Reaction: Large Gap Held but Neither Extended to a HOD Close nor Failed to a LOD Close on Doubled Participation; High-stakes Standoff after an Earnings/news Gap with No Directional Resolution)",
        Preset::BigGapDownMidpointCloseHotVol => "Large Gap Down (<-5 %) + Midpoint Close between HOD and LOD + Hot Vol (Inconclusive Institutional Capitulation: Large Gap Held but Neither Extended to a LOD Close nor Absorbed to a HOD Close on Doubled Participation; High-stakes Standoff after an Earnings/news Gap with No Directional Resolution)",
        Preset::GapUpCloseAtHodNearYearHighHotVol => "Gap Up (>2 %) + Close Pinned to HOD + At/near 52w High (<2 %) + Green Close + Hot Vol (Maximum-conviction Breakout Day: Gap up at the 52w High Held All Session and Closed at the Day's High on Elevated Participation; Rarest Multi-axis Aligned Bullish Signal with No Overhead Supply)",
        Preset::GapDownCloseAtLodNearYearLowHotVol => "Gap Down (<-2 %) + Close Pinned to LOD + At/near 52w Low (<2 %) + Red Close + Hot Vol (Maximum-conviction Breakdown Day: Gap down at the 52w Low Held All Session and Closed at the Day's Low on Elevated Participation; Rarest Multi-axis Aligned Bearish Signal with No Underlying Support)",
        Preset::GapUpCloseAtHodDeepBelowYearHighHotVol => "Gap Up (>2 %) + Close Pinned to HOD + Far below 52w High (>=20 %) + Green Close + Hot Vol (High-conviction Recovery from Extended Pullback: Gap up Well below the Prior Peak Held All Session and Closed at the Day's High on Elevated Participation; Trend-change Reversal Candidate after Extended Decline)",
        Preset::GapDownCloseAtLodDeepAboveYearLowHotVol => "Gap Down (<-2 %) + Close Pinned to LOD + Far above 52w Low (>=20 %) + Red Close + Hot Vol (High-conviction Rejection of Extended Advance: Gap down Well above the Prior Trough Held All Session and Closed at the Day's Low on Elevated Participation; Trend-change Reversal Candidate after Extended Advance)",
        Preset::GapUpCloseAtHodConfirmedAboveYearHighHotVol => "Gap Up (>2 %) + Close Pinned to HOD + Confirmed-breakout Zone (1-3 % past 52w High) + Green Close + Hot Vol (Confirmed-breakout Day-trade Signal: Gap up Continues Breakout that Was Already Validated and Closed at the Day's High on Elevated Participation; Aligned-axis Momentum Extension above the Prior Peak)",
        Preset::GapDownCloseAtLodConfirmedBelowYearLowHotVol => "Gap Down (<-2 %) + Close Pinned to LOD + Confirmed-breakdown Zone (1-3 % past 52w Low) + Red Close + Hot Vol (Confirmed-breakdown Day-trade Signal: Gap down Continues Breakdown that Was Already Validated and Closed at the Day's Low on Elevated Participation; Aligned-axis Momentum Extension below the Prior Trough)",
        Preset::GapUpCloseAtHodJustOffYearHighHotVol => "Gap Up (>2 %) + Close Pinned to HOD + Just off 52w High (2-5 %) + Green Close + Hot Vol (Post-pullback Re-assertion of Breakout Momentum: Gap up after Shallow Pullback Held All Session and Closed at the Day's High on Elevated Participation; Aligned-axis Continuation Candidate Immediately Back toward the Prior Peak)",
        Preset::GapDownCloseAtLodJustOffYearLowHotVol => "Gap Down (<-2 %) + Close Pinned to LOD + Just off 52w Low (2-5 %) + Red Close + Hot Vol (Post-bounce Re-assertion of Breakdown Momentum: Gap down after Shallow Bounce Held All Session and Closed at the Day's Low on Elevated Participation; Aligned-axis Continuation Candidate Immediately Back toward the Prior Trough)",
        Preset::GapUpCloseAtHodMidYearHighHotVol => "Gap Up (>2 %) + Close Pinned to HOD + Mid-range from High (5-20 %) + Green Close + Hot Vol (Mid-cycle Recovery Momentum: Gap up Well into the Consolidation Zone Held All Session and Closed at the Day's High on Elevated Participation; Aligned-axis Push back toward the Prior Peak from a Proper Pullback)",
        Preset::GapDownCloseAtLodMidYearLowHotVol => "Gap Down (<-2 %) + Close Pinned to LOD + Mid-range from Low (5-20 %) + Red Close + Hot Vol (Mid-cycle Reversal Momentum: Gap down Well into the Consolidation Zone Held All Session and Closed at the Day's Low on Elevated Participation; Aligned-axis Push back toward the Prior Trough from a Proper Recovery)",
        Preset::GapUpCloseAtLodNearYearHighHotVol => "Gap Up (>2 %) Faded Completely to LOD + At/near 52w High (<2 %) + Red Close + Hot Vol (Distribution-top Signal at the 52w High: Gap up Attempt at the Year Peak Completely Absorbed and Pushed below the Open on Elevated Participation; High-conviction Failed-breakout at the Worst Possible Location for Bulls)",
        Preset::GapDownCloseAtHodNearYearLowHotVol => "Gap Down (<-2 %) Absorbed Completely to HOD + At/near 52w Low (<2 %) + Green Close + Hot Vol (Accumulation-bottom Signal at the 52w Low: Gap down Attempt at the Year Trough Completely Absorbed and Pushed above the Open on Elevated Participation; High-conviction Failed-breakdown at the Worst Possible Location for Bears)",
        Preset::GapUpCloseAtLodConfirmedAboveYearHighHotVol => "Gap Up (>2 %) Faded Completely to LOD + Confirmed-breakout Zone (1-3 % past 52w High) + Red Close + Hot Vol (Post-breakout Distribution Signal: Gap up in the Already-confirmed Breakout Zone Completely Reversed and Closed at the Day's Low on Elevated Participation; Failed-extension Warning that Breakout Buyers Are Getting Trapped above the Prior Peak)",
        Preset::GapDownCloseAtHodConfirmedBelowYearLowHotVol => "Gap Down (<-2 %) Absorbed Completely to HOD + Confirmed-breakdown Zone (1-3 % past 52w Low) + Green Close + Hot Vol (Post-breakdown Accumulation Signal: Gap down in the Already-confirmed Breakdown Zone Completely Reversed and Closed at the Day's High on Elevated Participation; Failed-extension Warning that Breakdown Sellers Are Getting Trapped below the Prior Trough)",
        Preset::GapUpCloseAtLodDeepBelowYearHighHotVol => "Gap Up (>2 %) Faded Completely to LOD + Far below 52w High (>=20 %) + Red Close + Hot Vol (Failed Dead-cat Bounce Signal: Gap up Deep in the Pullback Territory Completely Reversed and Closed at the Day's Low on Elevated Participation; Bear-market Continuation Candidate with Sellers in Control Well below the Prior Peak)",
        Preset::GapDownCloseAtHodDeepAboveYearLowHotVol => "Gap Down (<-2 %) Absorbed Completely to HOD + Far above 52w Low (>=20 %) + Green Close + Hot Vol (Failed Shake-out Signal: Gap down Deep in the Advance Territory Completely Reversed and Closed at the Day's High on Elevated Participation; Bull-market Continuation Candidate with Buyers in Control Well above the Prior Trough)",
        Preset::GapUpCloseAtLodJustOffYearHighHotVol => "Gap Up (>2 %) Faded Completely to LOD + Just off 52w High (2-5 %) + Red Close + Hot Vol (Failed Recovery Attempt: Gap up Immediately after Shallow Pullback from the Year Peak Completely Reversed and Closed at the Day's Low on Elevated Participation; Second Leg Lower Starting Candidate)",
        Preset::GapDownCloseAtHodJustOffYearLowHotVol => "Gap Down (<-2 %) Absorbed Completely to HOD + Just off 52w Low (2-5 %) + Green Close + Hot Vol (Failed Rejection Attempt: Gap down Immediately after Shallow Bounce from the Year Trough Completely Reversed and Closed at the Day's High on Elevated Participation; Second Leg Higher Starting Candidate)",
        Preset::GapUpCloseAtLodMidYearHighHotVol => "Gap Up (>2 %) Faded Completely to LOD + Mid-range from High (5-20 %) + Red Close + Hot Vol (Failed Mid-cycle Bounce: Gap up in the Proper Consolidation Zone Completely Reversed and Closed at the Day's Low on Elevated Participation; Continuation of Mid-cycle Pullback with Sellers in Control)",
        Preset::GapDownCloseAtHodMidYearLowHotVol => "Gap Down (<-2 %) Absorbed Completely to HOD + Mid-range from Low (5-20 %) + Green Close + Hot Vol (Failed Mid-cycle Pullback: Gap down in the Proper Consolidation Zone Completely Reversed and Closed at the Day's High on Elevated Participation; Continuation of Mid-cycle Recovery with Buyers in Control)",
        Preset::GapUpMidpointCloseNearYearHighHotVol => "Gap Up (>2 %) + Midpoint Close between HOD and LOD + At/near 52w High (<2 %) + Hot Vol (Inconclusive Breakout-day Reaction: Gap up at the Year Peak Held but Neither Extended nor Failed into the Close on Elevated Participation; Standoff at the 52w High with the Next Breakout Still Undecided)",
        Preset::GapDownMidpointCloseNearYearLowHotVol => "Gap Down (<-2 %) + Midpoint Close between HOD and LOD + At/near 52w Low (<2 %) + Hot Vol (Inconclusive Breakdown-day Reaction: Gap down at the Year Trough Held but Neither Extended nor Absorbed into the Close on Elevated Participation; Standoff at the 52w Low with the Next Breakdown Still Undecided)",
        Preset::GapUpMidpointCloseConfirmedAboveYearHighHotVol => "Gap Up (>2 %) + Midpoint Close between HOD and LOD + Confirmed-breakout Zone (1-3 % past 52w High) + Hot Vol (Uncertain Follow-through after Validated Breakout: Gap up in the Already-cleared Zone Held but Neither Extended nor Failed into the Close on Elevated Participation; Post-breakout Stall Warning that the Extension Is Losing Conviction)",
        Preset::GapDownMidpointCloseConfirmedBelowYearLowHotVol => "Gap Down (<-2 %) + Midpoint Close between HOD and LOD + Confirmed-breakdown Zone (1-3 % past 52w Low) + Hot Vol (Uncertain Follow-through after Validated Breakdown: Gap down in the Already-cleared Zone Held but Neither Extended nor Absorbed into the Close on Elevated Participation; Post-breakdown Stall Warning that the Extension Is Losing Conviction)",
        Preset::GapUpMidpointCloseDeepBelowYearHighHotVol => "Gap Up (>2 %) + Midpoint Close between HOD and LOD + Far below 52w High (>=20 %) + Hot Vol (Inconclusive Bounce Attempt Deep in Pullback Territory: Gap up Well below the Prior Peak Held but Neither Extended to a HOD Close nor Failed to a LOD Close on Elevated Participation; Standoff after Extended Decline with No Directional Commitment from the Bounce)",
        Preset::GapDownMidpointCloseDeepAboveYearLowHotVol => "Gap Down (<-2 %) + Midpoint Close between HOD and LOD + Far above 52w Low (>=20 %) + Hot Vol (Inconclusive Pullback Attempt Deep in Advance Territory: Gap down Well above the Prior Trough Held but Neither Extended to a LOD Close nor Absorbed to a HOD Close on Elevated Participation; Standoff after Extended Advance with No Directional Commitment from the Pullback)",
        Preset::GapUpMidpointCloseMidYearHighHotVol => "Gap Up (>2 %) + Midpoint Close between HOD and LOD + Mid-range from High (5-20 %) + Hot Vol (Inconclusive Bounce in Mid-cycle Pullback: Gap up in the Proper Consolidation Zone Held but Neither Extended to a HOD Close nor Failed to a LOD Close on Elevated Participation; Standoff at Mid-range with No Directional Commitment toward the Prior Peak)",
        Preset::GapDownMidpointCloseMidYearLowHotVol => "Gap Down (<-2 %) + Midpoint Close between HOD and LOD + Mid-range from Low (5-20 %) + Hot Vol (Inconclusive Pullback in Mid-cycle Recovery: Gap down in the Proper Consolidation Zone Held but Neither Extended to a LOD Close nor Absorbed to a HOD Close on Elevated Participation; Standoff at Mid-range with No Directional Commitment back toward the Prior Trough)",
        Preset::GapUpMidpointCloseJustOffYearHighHotVol => "Gap Up (>2 %) + Midpoint Close between HOD and LOD + Just off 52w High (2-5 %) + Hot Vol (Inconclusive Bounce Just off the Year Peak: Gap up in the Post-extreme Zone Held but Neither Extended to a HOD Close nor Failed to a LOD Close on Elevated Participation; Standoff in the Immediate Post-tag Zone with the Recovery Still Undecided)",
        Preset::GapDownMidpointCloseJustOffYearLowHotVol => "Gap Down (<-2 %) + Midpoint Close between HOD and LOD + Just off 52w Low (2-5 %) + Hot Vol (Inconclusive Pullback Just off the Year Trough: Gap down in the Post-extreme Zone Held but Neither Extended to a LOD Close nor Absorbed to a HOD Close on Elevated Participation; Standoff in the Immediate Post-tag Zone with the Rejection Still Undecided)",
        Preset::HotVolFlatCloseNearYearHighHotVol => "Flat Close (|change|<0.5 %) + Hot Vol (>=2) + At/near 52w High (<2 %) (Institutional Churn at the 52w High: Doubled Participation with No Net Price Impact at the Year Peak; Potential Distribution-into-strength Signal Where Smart Money Exchanges Hands without Moving the Tape)",
        Preset::HotVolFlatCloseNearYearLowHotVol => "Flat Close (|change|<0.5 %) + Hot Vol (>=2) + At/near 52w Low (<2 %) (Institutional Churn at the 52w Low: Doubled Participation with No Net Price Impact at the Year Trough; Potential Accumulation-into-weakness Signal Where Smart Money Exchanges Hands without Moving the Tape)",
        Preset::HotVolFlatCloseConfirmedAboveYearHighHotVol => "Flat Close (|change|<0.5 %) + Hot Vol (>=2) + Confirmed-breakout Zone (1-3 % past 52w High) (Stealth Distribution in the Confirmed-breakout Zone: Doubled Participation with No Net Price Impact above the Prior Peak; Institutions Handling Supply at the Validated Breakout Level without Giving Back the Move)",
        Preset::HotVolFlatCloseConfirmedBelowYearLowHotVol => "Flat Close (|change|<0.5 %) + Hot Vol (>=2) + Confirmed-breakdown Zone (1-3 % past 52w Low) (Stealth Accumulation in the Confirmed-breakdown Zone: Doubled Participation with No Net Price Impact below the Prior Trough; Institutions Handling Demand at the Validated Breakdown Level without Giving Back the Move)",
        Preset::HotVolFlatCloseDeepBelowYearHighHotVol => "Flat Close (|change|<0.5 %) + Hot Vol (>=2) + Far below 52w High (>=20 %) (Stealth Accumulation Deep in Pullback Territory: Doubled Participation with No Net Price Impact Well below the Prior Peak; Potential Base-building Signal Where Smart Money Builds Position during Depressed-tape Conditions)",
        Preset::HotVolFlatCloseDeepAboveYearLowHotVol => "Flat Close (|change|<0.5 %) + Hot Vol (>=2) + Far above 52w Low (>=20 %) (Stealth Distribution Deep in Advance Territory: Doubled Participation with No Net Price Impact Well above the Prior Trough; Potential Topping Signal Where Smart Money Exits Position during Euphoric-tape Conditions)",
        Preset::HotVolFlatCloseMidYearHighHotVol => "Flat Close (|change|<0.5 %) + Hot Vol (>=2) + Mid-range from High (5-20 %) (Wyckoff-style Stealth Accumulation Zone in Mid-cycle Pullback: Doubled Participation with No Net Price Impact in the Proper Consolidation Range; Textbook Accumulation Phase Where Institutions Absorb Supply at Fair Value below the Prior Peak)",
        Preset::HotVolFlatCloseMidYearLowHotVol => "Flat Close (|change|<0.5 %) + Hot Vol (>=2) + Mid-range from Low (5-20 %) (Wyckoff-style Stealth Distribution Zone in Mid-cycle Recovery: Doubled Participation with No Net Price Impact in the Proper Consolidation Range; Textbook Distribution Phase Where Institutions Release Supply at Fair Value above the Prior Trough)",
        Preset::HotVolFlatCloseJustOffYearHighHotVol => "Flat Close (|change|<0.5 %) + Hot Vol (>=2) + Just off 52w High (2-5 %) (Post-tag Stealth Absorption: Doubled Participation with No Net Price Impact Immediately after Fresh Pullback from the 52w High; Institutions Exchange Hands in the Post-extreme Zone While Price Digests the Recent Peak)",
        Preset::HotVolFlatCloseJustOffYearLowHotVol => "Flat Close (|change|<0.5 %) + Hot Vol (>=2) + Just off 52w Low (2-5 %) (Post-tag Stealth Release: Doubled Participation with No Net Price Impact Immediately after Fresh Bounce from the 52w Low; Institutions Exchange Hands in the Post-extreme Zone While Price Digests the Recent Trough)",
        Preset::DryVolBigUpNearYearHighHotVol => "Big Up Move (>3 %) + Dry Vol (<0.5) + At/near 52w High (<2 %) (Thin-tape Breakout: Large Gains Push Price to New Highs but Participation Is below Average; Air-pocket Move with Little Resistance, Fragile if Vol Returns)",
        Preset::DryVolBigDownNearYearLowHotVol => "Big Down Move (<-3 %) + Dry Vol (<0.5) + At/near 52w Low (<2 %) (Thin-tape Breakdown: Large Losses Push Price to New Lows but Participation Is below Average; Air-pocket Move with Little Support, Fragile if Vol Returns)",
        Preset::DryVolBigUpConfirmedAboveYearHighHotVol => "Big Up Move (>3 %) + Dry Vol (<0.5) + Confirmed-breakout Zone (1-3 % past 52w High) (Thin-tape Extension after Validated Breakout: Momentum Continues on Below-average Participation past the Prior Peak; Volume-unconfirmed Extension Prone to Mean-reversion)",
        Preset::DryVolBigDownConfirmedBelowYearLowHotVol => "Big Down Move (<-3 %) + Dry Vol (<0.5) + Confirmed-breakdown Zone (1-3 % past 52w Low) (Thin-tape Extension after Validated Breakdown: Momentum Continues on Below-average Participation past the Prior Trough; Volume-unconfirmed Extension Prone to Mean-reversion)",
        Preset::DryVolBigUpDeepBelowYearHighHotVol => "Big Up Move (>3 %) + Dry Vol (<0.5) + Far below 52w High (>=20 %) (Unconvincing Recovery Rally: Large Gains Deep in the Pullback Territory on Below-average Participation; Sympathy/short-cover Bounce Lacking Institutional Buy-in, Fragile if Vol Returns)",
        Preset::DryVolBigDownDeepAboveYearLowHotVol => "Big Down Move (<-3 %) + Dry Vol (<0.5) + Far above 52w Low (>=20 %) (Unconvincing Pullback: Large Losses Deep in the Advance Territory on Below-average Participation; Sympathy/long-unwind Dip Lacking Institutional Sell-in, Fragile if Vol Returns)",
        Preset::DryVolBigUpMidYearHighHotVol => "Big Up Move (>3 %) + Dry Vol (<0.5) + Mid-range from High (5-20 %) (Low-quality Bounce in Mid-cycle Pullback: Large Gains in the Proper Consolidation Zone on Below-average Participation; Sympathy Move Lacking Institutional Follow-through, Prone to Fade)",
        Preset::DryVolBigDownMidYearLowHotVol => "Big Down Move (<-3 %) + Dry Vol (<0.5) + Mid-range from Low (5-20 %) (Low-quality Pullback in Mid-cycle Recovery: Large Losses in the Proper Consolidation Zone on Below-average Participation; Sympathy Move Lacking Institutional Follow-through, Prone to Bounce)",
        Preset::DryVolBigUpJustOffYearHighHotVol => "Big Up Move (>3 %) + Dry Vol (<0.5) + Just off 52w High (2-5 %) (Thin-tape Recovery from Shallow Pullback: Large Gains in the Immediate Post-extreme Zone on Below-average Participation; Quick Post-tag Bounce without Institutional Buy-in, Fragile Re-test Candidate)",
        Preset::DryVolBigDownJustOffYearLowHotVol => "Big Down Move (<-3 %) + Dry Vol (<0.5) + Just off 52w Low (2-5 %) (Thin-tape Rejection from Shallow Bounce: Large Losses in the Immediate Post-extreme Zone on Below-average Participation; Quick Post-tag Dip without Institutional Sell-in, Fragile Re-test Candidate)",
        Preset::UltraDeepBelowYearHighHotVol => "Ultra-deep Distance from 52w High (>=50 %) + Hot Vol (Deep-value / Distressed-equity Territory: Price Has Lost Half or More of Its 52w Peak with Elevated Participation; Either Turnaround Candidate or Bankruptcy-watch Depending on Fundamentals)",
        Preset::UltraDeepAboveYearLowHotVol => "Ultra-deep Distance from 52w Low (>=50 %) + Hot Vol (Multibagger Territory: Price Has Doubled or More off Its 52w Trough with Elevated Participation; Momentum-leader Candidate Riding a Multi-month Trend with Sustained Institutional Interest)",
        Preset::UltraDeepBelowYearHighCloseAtHodHotVol => "Distressed Stock (>=50 % below 52w High) + Close Pinned to HOD + Green Close + Hot Vol (Turnaround Momentum Signal: Distressed Equity Closes at the Day's High on Elevated Participation; Rare Bullish-conviction Tape in Beaten-down Territory Worth a Reversal-trade Screen)",
        Preset::UltraDeepAboveYearLowCloseAtLodHotVol => "Multibagger (>=50 % above 52w Low) + Close Pinned to LOD + Red Close + Hot Vol (Topping Signal: Extended-trend Leader Closes at the Day's Low on Elevated Participation; Rare Bearish-conviction Tape in Stretched Territory Worth a Top-fade Trade Screen)",
        Preset::UltraDeepBelowYearHighGapUpHotVol => "Distressed Stock (>=50 % below 52w High) + Gap Up (>2 %) + Hot Vol (Catalyst-driven Turn Candidate: Beaten-down Equity Gaps up on News/earnings with Elevated Participation; Potential Institutional Re-rating Event Worth a Turnaround-trade Screen)",
        Preset::UltraDeepAboveYearLowGapDownHotVol => "Multibagger (>=50 % above 52w Low) + Gap Down (<-2 %) + Hot Vol (Catalyst-driven Top Candidate: Extended-trend Leader Gaps down on News/earnings with Elevated Participation; Potential Institutional De-rating Event Worth a Top-fade Trade Screen)",
        Preset::UltraDeepBelowYearHighGapUpFadedHotVol => "Distressed Stock (>=50 % below 52w High) + Gap Up (>2 %) Faded Completely to LOD + Red Close + Hot Vol (Failed Turnaround Catalyst: Beaten-down Equity Attempts a Catalyst-driven Gap but Sellers Absorb the Entire Move on Elevated Participation; Turnaround-thesis Rejection that Confirms Downtrend Control)",
        Preset::UltraDeepAboveYearLowGapDownAbsorbedHotVol => "Multibagger (>=50 % above 52w Low) + Gap Down (<-2 %) Absorbed Completely to HOD + Green Close + Hot Vol (Failed Topping Catalyst: Extended-trend Leader Attempts a Catalyst-driven Gap down but Buyers Absorb the Entire Move on Elevated Participation; Top-fade Thesis Rejection that Confirms Uptrend Control)",
        Preset::UltraDeepBelowYearHighGapUpHeldHotVol => "Distressed Stock (>=50 % below 52w High) + Gap Up (>2 %) Held All Session + Close Pinned to HOD + Green Close + Hot Vol (Validated Turnaround Day: Beaten-down Equity Gaps up on Catalyst, Holds the Entire Gap, and Closes at the Day's High on Elevated Participation; Institutional Re-rating Event with No Profit-taking = Highest-conviction Turnaround Signal in Distressed Territory)",
        Preset::UltraDeepAboveYearLowGapDownHeldHotVol => "Multibagger (>=50 % above 52w Low) + Gap Down (<-2 %) Held All Session + Close Pinned to LOD + Red Close + Hot Vol (Validated Topping Day: Extended-trend Leader Gaps down on Catalyst, Holds the Entire Gap, and Closes at the Day's Low on Elevated Participation; Institutional De-rating Event with No Dip-buying = Highest-conviction Topping Signal in Stretched Territory)",
        Preset::UltraDeepBelowYearHighGapUpMidpointHotVol => "Distressed Stock (>=50 % below 52w High) + Gap Up (>2 %) + Midpoint Close between HOD and LOD + Hot Vol (Uncertain Turnaround Follow-through: Beaten-down Equity Gaps up on Catalyst but Neither Holds the Gap to a HOD Close nor Fully Fades to a LOD Close on Elevated Participation; Ambiguous Re-rating Event Requiring Confirmation)",
        Preset::UltraDeepAboveYearLowGapDownMidpointHotVol => "Multibagger (>=50 % above 52w Low) + Gap Down (<-2 %) + Midpoint Close between HOD and LOD + Hot Vol (Uncertain Topping Follow-through: Extended-trend Leader Gaps down on Catalyst but Neither Holds the Gap to a LOD Close nor Fully Absorbs to a HOD Close on Elevated Participation; Ambiguous De-rating Event Requiring Confirmation)",
        Preset::UltraDeepBelowYearHighHammerHotVol => "Distressed Stock (>=50 % below 52w High) + Long Lower Wick (>3 %) + Close Pinned to HOD + Green Close + Hot Vol (Capitulation-reversal Hammer in Distressed Territory: Beaten-down Equity Probed Lower then Reclaimed the Entire Move to Close at the Day's High on Elevated Participation; Classic Capitulation-day Pattern at Deep Distress Worth a Bounce-trade Screen)",
        Preset::UltraDeepAboveYearLowShootingStarHotVol => "Multibagger (>=50 % above 52w Low) + Long Upper Wick (>3 %) + Close Pinned to LOD + Red Close + Hot Vol (Exhaustion-reversal Shooting Star in Multibagger Territory: Extended-trend Leader Probed Higher then Gave Back the Entire Move to Close at the Day's Low on Elevated Participation; Classic Exhaustion-day Pattern at Extended Advance Worth a Top-fade Trade Screen)",
        Preset::UltraDeepBelowYearHighShootingStarHotVol => "Distressed Stock (>=50 % below 52w High) + Long Upper Wick (>3 %) + Close Pinned to LOD + Red Close + Hot Vol (Failed-bounce Shooting Star in Distressed Territory: Beaten-down Equity Probed Higher Intraday then Sellers Reclaimed Entire Move to Close at the Day's Low on Elevated Participation; Bear-trend Continuation Candidate Confirming Downtrend Control)",
        Preset::UltraDeepAboveYearLowHammerHotVol => "Multibagger (>=50 % above 52w Low) + Long Lower Wick (>3 %) + Close Pinned to HOD + Green Close + Hot Vol (Trend-continuation Hammer in Multibagger Territory: Extended-trend Leader Probed Lower Intraday then Buyers Reclaimed Entire Move to Close at the Day's High on Elevated Participation; Bull-trend Continuation Candidate Confirming Uptrend Control)",
        Preset::WideYearRangeHotVol => "Wide Annual Range (both year_*_pct >=20 %) + Hot Vol (High-beta Volatile-equity Territory: Stock Has Traveled Significant Distance from Both Prior Peak and Prior Trough Indicating Large 52w Range; Volatility-screen Candidate for Momentum and Mean-reversion Strategies)",
        Preset::NarrowYearRangeHotVol => "Narrow Annual Range (both year_*_pct <5 %) + Hot Vol (Compressed Range-bound Territory: Stock Has Stayed within ~10 % Band over 52w with Elevated Current Participation; Tightest Possible Coil at Annual Scale, Breakout-candidate Worth Watching for Direction Commitment)",
        Preset::NarrowYearRangeCloseAtHodHotVol => "Narrow Annual Range + Close Pinned to HOD + Green Close + Hot Vol (Annual-coil Breakout Candidate: Compressed Range-bound Stock Attempts Directional Commitment Higher, Closes at the Day's High with Hot Vol; Highest-conviction Breakout-from-coil Signal at the Annual Scale)",
        Preset::NarrowYearRangeCloseAtLodHotVol => "Narrow Annual Range + Close Pinned to LOD + Red Close + Hot Vol (Annual-coil Breakdown Candidate: Compressed Range-bound Stock Attempts Directional Commitment Lower, Closes at the Day's Low with Hot Vol; Highest-conviction Breakdown-from-coil Signal at the Annual Scale)",
        Preset::NarrowYearRangeGapUpHotVol => "Narrow Annual Range + Gap Up (>2 %) + Hot Vol (Catalyst-driven Attempt to Break the Annual Coil Higher: Compressed Range-bound Stock Gaps up on News/earnings with Elevated Participation; First Sign of Directional Commitment after 52w of Tight Range)",
        Preset::NarrowYearRangeGapDownHotVol => "Narrow Annual Range + Gap Down (<-2 %) + Hot Vol (Catalyst-driven Attempt to Break the Annual Coil Lower: Compressed Range-bound Stock Gaps down on News/earnings with Elevated Participation; First Sign of Directional Commitment after 52w of Tight Range)",
        Preset::NarrowYearRangeBigUpHotVol => "Narrow Annual Range + Big Up Move (>3 %) + Hot Vol (>=2) (Intraday-led Coil Release Higher: Compressed Range-bound Stock Prints First Volatility Expansion in 52w with Doubled Participation; Regime-shift Candidate from Tight Range to Trending Higher)",
        Preset::NarrowYearRangeBigDownHotVol => "Narrow Annual Range + Big Down Move (<-3 %) + Hot Vol (>=2) (Intraday-led Coil Release Lower: Compressed Range-bound Stock Prints First Volatility Expansion in 52w with Doubled Participation; Regime-shift Candidate from Tight Range to Trending Lower)",
        Preset::WideYearRangeCloseAtHodHotVol => "Wide Annual Range + Close Pinned to HOD + Green Close + Hot Vol (Strong-conviction Tape in High-beta Name: Volatile Stock with Large 52w Range Closes at the Day's High on Elevated Participation; Momentum-continuation Signal Where Intraday Strength Aligns with the Wider Price-action Regime)",
        Preset::WideYearRangeCloseAtLodHotVol => "Wide Annual Range + Close Pinned to LOD + Red Close + Hot Vol (Strong-conviction Tape in High-beta Name: Volatile Stock with Large 52w Range Closes at the Day's Low on Elevated Participation; Momentum-continuation Signal Where Intraday Weakness Aligns with the Wider Price-action Regime)",
        Preset::WideYearRangeGapUpHotVol => "Wide Annual Range + Gap Up (>2 %) + Hot Vol (Catalyst-event in High-beta Name: Volatile Stock with Large 52w Range Gaps up on News/earnings with Elevated Participation; Mean-reversion-from-gap Candidate or Trend-continuation Depending on Intraday Follow-through)",
        Preset::WideYearRangeGapDownHotVol => "Wide Annual Range + Gap Down (<-2 %) + Hot Vol (Catalyst-event in High-beta Name: Volatile Stock with Large 52w Range Gaps down on News/earnings with Elevated Participation; Mean-reversion-from-gap Candidate or Trend-continuation Depending on Intraday Follow-through)",
        Preset::AsymmetricRangeNearLowFarHighHotVol => "Near 52w Low (<5 %) + Far below 52w High (>=20 %) + Hot Vol (Persistent-downtrend Stock Testing the Low Again with Elevated Participation; Either Capitulation-bottom Candidate or Breakdown to Fresh Lows Depending on Price-action Resolution at the Support Level)",
        Preset::AsymmetricRangeNearHighFarLowHotVol => "Near 52w High (<5 %) + Far above 52w Low (>=20 %) + Hot Vol (Persistent-uptrend Stock Testing the High Again with Elevated Participation; Either Breakout-to-fresh-highs Candidate or Distribution-top Depending on Price-action Resolution at the Resistance Level)",
        Preset::AsymmetricRangeNearLowFarHighCloseAtLodHotVol => "Persistent-downtrend Stock (Near Low + Far below High) + Close Pinned to LOD + Red Close + Hot Vol (Breakdown-confirmation Day: Downtrend Stock Fails the Support Test at the 52w Low and Closes at the Day's Low on Elevated Participation; Fresh-low Extension Confirmed by Intraday Weakness)",
        Preset::AsymmetricRangeNearHighFarLowCloseAtHodHotVol => "Persistent-uptrend Stock (Near High + Far above Low) + Close Pinned to HOD + Green Close + Hot Vol (Breakout-confirmation Day: Uptrend Stock Clears the Resistance Test at the 52w High and Closes at the Day's High on Elevated Participation; Fresh-high Extension Confirmed by Intraday Strength)",
        Preset::AsymmetricRangeNearLowFarHighCloseAtHodHotVol => "Persistent-downtrend Stock (Near Low + Far below High) + Close Pinned to HOD + Green Close + Hot Vol (Capitulation-reversal Candidate: Downtrend Stock Defends the Support Test at the 52w Low and Closes at the Day's High on Elevated Participation; Potential Bottom-formation Signal Worth a Bounce-trade Screen)",
        Preset::AsymmetricRangeNearHighFarLowCloseAtLodHotVol => "Persistent-uptrend Stock (Near High + Far above Low) + Close Pinned to LOD + Red Close + Hot Vol (Distribution-top Candidate: Uptrend Stock Rejects the Resistance Test at the 52w High and Closes at the Day's Low on Elevated Participation; Potential Top-formation Signal Worth a Top-fade Trade Screen)",
        Preset::AsymmetricRangeNearLowFarHighGapUpHotVol => "Persistent-downtrend Stock (Near Low + Far below High) + Gap Up (>2 %) + Hot Vol (Relief-rally Catalyst at the 52w Low: Downtrend Stock Attempts Catalyst-driven Turn off the Year Trough with Elevated Participation; Potential Turnaround-thesis Signal Worth a Bounce-trade Screen)",
        Preset::AsymmetricRangeNearHighFarLowGapDownHotVol => "Persistent-uptrend Stock (Near High + Far above Low) + Gap Down (<-2 %) + Hot Vol (Pullback-catalyst at the 52w High: Uptrend Stock Attempts Catalyst-driven Reversal off the Year Peak with Elevated Participation; Potential Top-warning Signal Worth a Top-fade Trade Screen)",
        Preset::AsymmetricRangeNearLowFarHighGapDownHotVol => "Persistent-downtrend Stock (Near Low + Far below High) + Gap Down (<-2 %) + Hot Vol (Catalyst-driven Breakdown to Fresh Lows: Downtrend Stock Attempts Catalyst-confirmed Continuation through the 52w Support Level with Elevated Participation; Trend-extension Signal Worth a Breakdown-trade Screen)",
        Preset::AsymmetricRangeNearHighFarLowGapUpHotVol => "Persistent-uptrend Stock (Near High + Far above Low) + Gap Up (>2 %) + Hot Vol (Catalyst-driven Breakout to Fresh Highs: Uptrend Stock Attempts Catalyst-confirmed Continuation through the 52w Resistance Level with Elevated Participation; Trend-extension Signal Worth a Breakout-trade Screen)",
        Preset::GapUpFlatDayHotVol => "Gap Up (>2 %) + Flat-day Bar (|day_pct|<0.5 %) + Hot Vol (Gap-and-hold Institutional Signal: Overnight Gap up Held through the Regular Session without Intraday Giveback or Further Follow-through; Supports the Gap without Re-test, Suggesting Buyer Accumulation at the Gap Level)",
        Preset::GapDownFlatDayHotVol => "Gap Down (<-2 %) + Flat-day Bar (|day_pct|<0.5 %) + Hot Vol (Gap-and-hold Institutional Signal: Overnight Gap down Held through the Regular Session without Intraday Recovery or Further Deterioration; Supports the Gap without Re-test, Suggesting Seller Distribution at the Gap Level)",
        Preset::GapUpBigDayHotVol => "Gap Up (>2 %) + Big Intraday Up (>2 %) + Hot Vol (Double-momentum Day: Overnight Gap up Followed by Intraday Continuation Higher with Elevated Participation; Gap+intraday Both Aligned Green = Aggregate Change_pct >4 % from Prior Close with Sustained Buy Pressure across Both Regular and After-hours Sessions)",
        Preset::GapDownBigDayHotVol => "Gap Down (<-2 %) + Big Intraday Down (<-2 %) + Hot Vol (Double-momentum Day: Overnight Gap down Followed by Intraday Continuation Lower with Elevated Participation; Gap+intraday Both Aligned Red = Aggregate Change_pct <-4 % from Prior Close with Sustained Sell Pressure across Both Regular and After-hours Sessions)",
        Preset::GapUpBigDayDownHotVol => "Gap Up (>2 %) + Big Intraday Down (<-2 %) + Hot Vol (Gap-fade Pressure: Overnight Gap up Met with Intraday Selling that Erodes the Gap by 2 %+; Counter-trend Intraday Pressure on a Positive Gap that May or May Not Flip the Close to Red Depending on Gap Size; Suggests Active Sellers Stepping in at the Gap Level)",
        Preset::GapDownBigDayUpHotVol => "Gap Down (<-2 %) + Big Intraday Up (>2 %) + Hot Vol (Gap-absorb Pressure: Overnight Gap down Met with Intraday Buying that Reclaims the Gap by 2 %+; Counter-trend Intraday Pressure on a Negative Gap that May or May Not Flip the Close to Green Depending on Gap Size; Suggests Active Buyers Stepping in at the Gap Level)",
        Preset::SmallGapBigDayUpHotVol => "Small Gap (|gap|<0.5 %) + Big Intraday Up (>3 %) + Hot Vol (Clean Intraday-led Rally: Open Is Essentially Flat to Prior Close, Then Regular Session Prints a Sustained Buy-driven Move Higher; Intraday Momentum Signal Isolated from Overnight Repricing or After-hours Catalyst Noise)",
        Preset::SmallGapBigDayDownHotVol => "Small Gap (|gap|<0.5 %) + Big Intraday Down (<-3 %) + Hot Vol (Clean Intraday-led Decline: Open Is Essentially Flat to Prior Close, Then Regular Session Prints a Sustained Sell-driven Move Lower; Intraday Momentum Signal Isolated from Overnight Repricing or After-hours Catalyst Noise)",
        Preset::SmallGapBigDayUpNearYearHighHotVol => "Small Gap (|gap|<0.5 %) + Big Intraday Up (>3 %) + At/near 52w High (<2 %) + Hot Vol (Pure Intraday-driven Breakout to 52w High: Open Is Essentially Flat to Prior Close then Regular Session Prints a Sustained Buy-driven Move to the Year Peak; Cleanest Possible Breakout Signal with No Overnight Gap Contribution Muddying the Cause)",
        Preset::SmallGapBigDayDownNearYearLowHotVol => "Small Gap (|gap|<0.5 %) + Big Intraday Down (<-3 %) + At/near 52w Low (<2 %) + Hot Vol (Pure Intraday-driven Breakdown to 52w Low: Open Is Essentially Flat to Prior Close then Regular Session Prints a Sustained Sell-driven Move to the Year Trough; Cleanest Possible Breakdown Signal with No Overnight Gap Contribution Muddying the Cause)",
        Preset::SmallGapBigDayUpConfirmedAboveYearHighHotVol => "Small Gap (|gap|<0.5 %) + Big Intraday Up (>3 %) + Confirmed-breakout Zone (1-3 % past 52w High) + Hot Vol (Pure Intraday Extension past Validated Breakout: Regular Session Prints a Sustained Buy-driven Move Further past the Already-cleared Peak with No Overnight Repricing Component; High-conviction Intraday Extension Confirming Breakout Follow-through)",
        Preset::SmallGapBigDayDownConfirmedBelowYearLowHotVol => "Small Gap (|gap|<0.5 %) + Big Intraday Down (<-3 %) + Confirmed-breakdown Zone (1-3 % past 52w Low) + Hot Vol (Pure Intraday Extension past Validated Breakdown: Regular Session Prints a Sustained Sell-driven Move Further past the Already-cleared Trough with No Overnight Repricing Component; High-conviction Intraday Extension Confirming Breakdown Follow-through)",
        Preset::SmallGapBigDayUpDeepBelowYearHighHotVol => "Small Gap (|gap|<0.5 %) + Big Intraday Up (>3 %) + Far below 52w High (>=20 %) + Hot Vol (Pure Intraday Recovery Rally from Depressed Level: Open Is Essentially Flat to Prior Close then Regular Session Prints a Sustained Buy-driven Move Deep in Pullback Territory; Institutional-bid Activation Signal that Contrasts with Overnight Catalyst Noise)",
        Preset::SmallGapBigDayDownDeepAboveYearLowHotVol => "Small Gap (|gap|<0.5 %) + Big Intraday Down (<-3 %) + Far above 52w Low (>=20 %) + Hot Vol (Pure Intraday Rejection from Elevated Level: Open Is Essentially Flat to Prior Close then Regular Session Prints a Sustained Sell-driven Move Deep in Advance Territory; Institutional-offer Activation Signal that Contrasts with Overnight Catalyst Noise)",
        Preset::SmallGapBigDayUpMidYearHighHotVol => "Small Gap (|gap|<0.5 %) + Big Intraday Up (>3 %) + Mid-range from High (5-20 %) + Hot Vol (Pure Intraday Rally in Mid-cycle Pullback Zone: Open Is Essentially Flat to Prior Close then Regular Session Prints a Sustained Buy-driven Move in the Proper Consolidation Range; Pure-intraday Push back toward the Prior Peak without Overnight Catalyst Contribution)",
        Preset::SmallGapBigDayDownMidYearLowHotVol => "Small Gap (|gap|<0.5 %) + Big Intraday Down (<-3 %) + Mid-range from Low (5-20 %) + Hot Vol (Pure Intraday Rejection in Mid-cycle Recovery Zone: Open Is Essentially Flat to Prior Close then Regular Session Prints a Sustained Sell-driven Move in the Proper Consolidation Range; Pure-intraday Push back toward the Prior Trough without Overnight Catalyst Contribution)",
        Preset::SmallGapBigDayUpJustOffYearHighHotVol => "Small Gap (|gap|<0.5 %) + Big Intraday Up (>3 %) + Just off 52w High (2-5 %) + Hot Vol (Pure Intraday Recovery from Shallow Pullback: Open Is Essentially Flat to Prior Close then Regular Session Prints a Sustained Buy-driven Move back toward the Recent Peak; Pure-intraday Post-tag Re-test Attempt without Overnight Catalyst Contribution)",
        Preset::SmallGapBigDayDownJustOffYearLowHotVol => "Small Gap (|gap|<0.5 %) + Big Intraday Down (<-3 %) + Just off 52w Low (2-5 %) + Hot Vol (Pure Intraday Rejection from Shallow Bounce: Open Is Essentially Flat to Prior Close then Regular Session Prints a Sustained Sell-driven Move back toward the Recent Trough; Pure-intraday Post-tag Re-test Attempt without Overnight Catalyst Contribution)",
        Preset::BigUpDayCloseAtHodHotVol => "Big Intraday Up (>3 %) + Close Pinned to HOD + Hot Vol (Strongest Possible Intraday Rally Pattern: Regular Session Prints a Sustained Buy-driven Move from Open to Close with No End-of-day Fade; Isolates Intraday-only Conviction without Conflating Gap Contribution; Ideal for Measuring Real-session Bull Pressure)",
        Preset::BigDownDayCloseAtLodHotVol => "Big Intraday Down (<-3 %) + Close Pinned to LOD + Hot Vol (Strongest Possible Intraday Selloff Pattern: Regular Session Prints a Sustained Sell-driven Move from Open to Close with No End-of-day Bounce; Isolates Intraday-only Conviction without Conflating Gap Contribution; Ideal for Measuring Real-session Bear Pressure)",
        Preset::BigUpDayDoubledVolHotVol => "Big Intraday Up (>3 %) + Doubled Vol (>=2) (Institutional Intraday Accumulation Signal: Regular Session Prints a Sustained Buy-driven Move on Doubled Participation Regardless of Gap and Close-position Context; Pure-intraday Volume-conviction Filter that Ignores Overnight Repricing Noise)",
        Preset::BigDownDayDoubledVolHotVol => "Big Intraday Down (<-3 %) + Doubled Vol (>=2) (Institutional Intraday Distribution Signal: Regular Session Prints a Sustained Sell-driven Move on Doubled Participation Regardless of Gap and Close-position Context; Pure-intraday Volume-conviction Filter that Ignores Overnight Repricing Noise)",
        Preset::BigUpDayDoubledVolNearYearHighHotVol => "Big Intraday Up (>3 %) + Doubled Vol (>=2) + At/near 52w High (<2 %) (Institutional Intraday Accumulation at the Year Peak: Regular Session Prints a Sustained Buy-driven Move on Doubled Participation While Price Prints Fresh Highs; Gap-agnostic Conviction Filter for Breakout Participation)",
        Preset::BigDownDayDoubledVolNearYearLowHotVol => "Big Intraday Down (<-3 %) + Doubled Vol (>=2) + At/near 52w Low (<2 %) (Institutional Intraday Distribution at the Year Trough: Regular Session Prints a Sustained Sell-driven Move on Doubled Participation While Price Prints Fresh Lows; Gap-agnostic Conviction Filter for Breakdown Participation)",
        Preset::BigUpDayDoubledVolConfirmedAboveYearHighHotVol => "Big Intraday Up (>3 %) + Doubled Vol (>=2) + Confirmed-breakout Zone (1-3 % past 52w High) (Institutional Intraday Accumulation in the Validated-breakout Zone: Regular Session Prints a Sustained Buy-driven Move on Doubled Participation While Price Extends past the Prior Peak; Gap-agnostic Conviction Filter for Breakout Follow-through)",
        Preset::BigDownDayDoubledVolConfirmedBelowYearLowHotVol => "Big Intraday Down (<-3 %) + Doubled Vol (>=2) + Confirmed-breakdown Zone (1-3 % past 52w Low) (Institutional Intraday Distribution in the Validated-breakdown Zone: Regular Session Prints a Sustained Sell-driven Move on Doubled Participation While Price Extends past the Prior Trough; Gap-agnostic Conviction Filter for Breakdown Follow-through)",
        Preset::BigUpDayDoubledVolDeepBelowYearHighHotVol => "Big Intraday Up (>3 %) + Doubled Vol (>=2) + Far below 52w High (>=20 %) (Institutional Intraday Accumulation Deep in Pullback Territory: Regular Session Prints a Sustained Buy-driven Move on Doubled Participation While Price Remains Well below the Prior Peak; Conviction-recovery-rally Signal Worth a Turnaround-screen)",
        Preset::BigDownDayDoubledVolDeepAboveYearLowHotVol => "Big Intraday Down (<-3 %) + Doubled Vol (>=2) + Far above 52w Low (>=20 %) (Institutional Intraday Distribution Deep in Advance Territory: Regular Session Prints a Sustained Sell-driven Move on Doubled Participation While Price Remains Well above the Prior Trough; Conviction-rejection Signal Worth a Top-fade-screen)",
        Preset::BigUpDayDoubledVolMidYearHighHotVol => "Big Intraday Up (>3 %) + Doubled Vol (>=2) + Mid-range from High (5-20 %) (Institutional Intraday Accumulation in Mid-cycle Pullback Zone: Regular Session Prints a Sustained Buy-driven Move on Doubled Participation in the Proper Consolidation Range below the Prior Peak; Conviction-mid-cycle-rally Signal Worth a Swing-screen)",
        Preset::BigDownDayDoubledVolMidYearLowHotVol => "Big Intraday Down (<-3 %) + Doubled Vol (>=2) + Mid-range from Low (5-20 %) (Institutional Intraday Distribution in Mid-cycle Recovery Zone: Regular Session Prints a Sustained Sell-driven Move on Doubled Participation in the Proper Consolidation Range above the Prior Trough; Conviction-mid-cycle-rejection Signal Worth a Swing-screen)",
        Preset::BigUpDayDoubledVolJustOffYearHighHotVol => "Big Intraday Up (>3 %) + Doubled Vol (>=2) + Just off 52w High (2-5 %) (Institutional Intraday Accumulation Just off the Year Peak: Regular Session Prints a Sustained Buy-driven Move on Doubled Participation Immediately after a Shallow Pullback from the 52w High; Conviction-post-tag-recovery Signal Worth a Re-test-screen)",
        Preset::BigDownDayDoubledVolJustOffYearLowHotVol => "Big Intraday Down (<-3 %) + Doubled Vol (>=2) + Just off 52w Low (2-5 %) (Institutional Intraday Distribution Just off the Year Trough: Regular Session Prints a Sustained Sell-driven Move on Doubled Participation Immediately after a Shallow Bounce from the 52w Low; Conviction-post-tag-rejection Signal Worth a Re-test-screen)",
        Preset::QuintupledVolUpHotVol => "Quintupled Vol (>=5) + Big Up Move (>3 %) (Extreme Participation Event with Bull-direction: Vol Is 5x Its Average and Price Prints a Significant Up Move; Rare News/earnings/catalyst Day at the Highest Possible Conviction Tier, Typically a Once-per-quarter Occurrence per Name)",
        Preset::QuintupledVolDownHotVol => "Quintupled Vol (>=5) + Big Down Move (<-3 %) (Extreme Participation Event with Bear-direction: Vol Is 5x Its Average and Price Prints a Significant Down Move; Rare News/earnings/catalyst Day at the Highest Possible Conviction Tier, Typically a Once-per-quarter Occurrence per Name)",
        Preset::QuintupledVolUpNearYearHighHotVol => "Quintupled Vol (>=5) + Big Up Move (>3 %) + At/near 52w High (<2 %) (Once-per-quarter Breakout Event at the Year Peak: Vol Is 5x Its Average, Price Prints a Significant Up Move and Reaches the 52w High Simultaneously; Highest-conviction Breakout-at-extreme Signal Worth a Tier-1 Alert)",
        Preset::QuintupledVolDownNearYearLowHotVol => "Quintupled Vol (>=5) + Big Down Move (<-3 %) + At/near 52w Low (<2 %) (Once-per-quarter Breakdown Event at the Year Trough: Vol Is 5x Its Average, Price Prints a Significant Down Move and Reaches the 52w Low Simultaneously; Highest-conviction Breakdown-at-extreme Signal Worth a Tier-1 Alert)",
        Preset::QuintupledVolUpDeepBelowYearHighHotVol => "Quintupled Vol (>=5) + Big Up Move (>3 %) + Far below 52w High (>=20 %) (Extreme Catalyst Recovery from Deep Pullback: Vol Is 5x Average and Price Prints a Significant Up Move While Still Well below the Prior Peak; Turnaround-catalyst at the Highest Possible Tier Worth a Regime-change Alert)",
        Preset::QuintupledVolDownDeepAboveYearLowHotVol => "Quintupled Vol (>=5) + Big Down Move (<-3 %) + Far above 52w Low (>=20 %) (Extreme Catalyst Rejection from Deep Advance: Vol Is 5x Average and Price Prints a Significant Down Move While Still Well above the Prior Trough; Top-fade Catalyst at the Highest Possible Tier Worth a Regime-change Alert)",
        Preset::QuintupledVolUpConfirmedAboveYearHighHotVol => "Quintupled Vol (>=5) + Big Up Move (>3 %) + Confirmed-breakout Zone (1-3 % past 52w High) (Extreme Catalyst Extending Validated Breakout: Vol Is 5x Average and Price Prints a Significant Up Move While Extending Further past the Prior Peak; Trend-extension at the Highest Possible Tier Worth a Follow-through Alert)",
        Preset::QuintupledVolDownConfirmedBelowYearLowHotVol => "Quintupled Vol (>=5) + Big Down Move (<-3 %) + Confirmed-breakdown Zone (1-3 % past 52w Low) (Extreme Catalyst Extending Validated Breakdown: Vol Is 5x Average and Price Prints a Significant Down Move While Extending Further past the Prior Trough; Trend-extension at the Highest Possible Tier Worth a Follow-through Alert)",
        Preset::QuintupledVolUpMidYearHighHotVol => "Quintupled Vol (>=5) + Big Up Move (>3 %) + Mid-range from High (5-20 %) (Extreme Catalyst Rally in Mid-cycle Pullback Zone: Vol Is 5x Average and Price Prints a Significant Up Move in the Proper Consolidation Range below the Prior Peak; Tier-1 Conviction-recovery Signal Worth a Swing-screen)",
        Preset::QuintupledVolDownMidYearLowHotVol => "Quintupled Vol (>=5) + Big Down Move (<-3 %) + Mid-range from Low (5-20 %) (Extreme Catalyst Rejection in Mid-cycle Recovery Zone: Vol Is 5x Average and Price Prints a Significant Down Move in the Proper Consolidation Range above the Prior Trough; Tier-1 Conviction-rejection Signal Worth a Swing-screen)",
        Preset::QuintupledVolUpJustOffYearHighHotVol => "Quintupled Vol (>=5) + Big Up Move (>3 %) + Just off 52w High (2-5 %) (Extreme Catalyst Rally Just off the Year Peak: Vol Is 5x Average and Price Prints a Significant Up Move Immediately after a Shallow Pullback from the 52w High; Tier-1 Post-tag-recovery Catalyst Worth a Re-test-screen)",
        Preset::QuintupledVolDownJustOffYearLowHotVol => "Quintupled Vol (>=5) + Big Down Move (<-3 %) + Just off 52w Low (2-5 %) (Extreme Catalyst Rejection Just off the Year Trough: Vol Is 5x Average and Price Prints a Significant Down Move Immediately after a Shallow Bounce from the 52w Low; Tier-1 Post-tag-rejection Catalyst Worth a Re-test-screen)",
        Preset::QuintupledVolCloseAtHodHotVol => "Quintupled Vol (>=5) + Close Pinned to HOD + Green Close + Hot Vol (Tier-1 Institutional Rally with No End-of-day Fade: Vol Is 5x Average and Close Pins to the Day's High with Positive Change; Rarest Possible Bull-conviction Close at the Highest Participation Tier)",
        Preset::QuintupledVolCloseAtLodHotVol => "Quintupled Vol (>=5) + Close Pinned to LOD + Red Close + Hot Vol (Tier-1 Institutional Selloff with No End-of-day Bounce: Vol Is 5x Average and Close Pins to the Day's Low with Negative Change; Rarest Possible Bear-conviction Close at the Highest Participation Tier)",
        Preset::QuintupledVolGapUpHotVol => "Quintupled Vol (>=5) + Gap Up (>2 %) (Tier-1 Catalyst Gap-up: Vol Is 5x Average and Overnight Repricing Pushes the Open More than 2 % above Prior Close; Rare News/earnings/sector-rotation Event with Full Session Participation Confirming the Bull-direction Catalyst)",
        Preset::QuintupledVolGapDownHotVol => "Quintupled Vol (>=5) + Gap Down (<-2 %) (Tier-1 Catalyst Gap-down: Vol Is 5x Average and Overnight Repricing Pushes the Open More than 2 % below Prior Close; Rare News/earnings/sector-rotation Event with Full Session Participation Confirming the Bear-direction Catalyst)",
        Preset::QuintupledVolGapUpCloseAtHodHotVol => "Quintupled Vol (>=5) + Gap Up (>2 %) + Close Pinned to HOD + Green Close (Highest-conviction Catalyst Gap-up that Holds: Vol Is 5x Average, Overnight Gap up Holds without Fade and Price Closes at the Day's High; Rarest Possible Validated Bull-catalyst Event across All Sessions)",
        Preset::QuintupledVolGapDownCloseAtLodHotVol => "Quintupled Vol (>=5) + Gap Down (<-2 %) + Close Pinned to LOD + Red Close (Highest-conviction Catalyst Gap-down that Holds: Vol Is 5x Average, Overnight Gap down Holds without Bounce and Price Closes at the Day's Low; Rarest Possible Validated Bear-catalyst Event across All Sessions)",
        Preset::QuintupledVolGapUpCloseAtLodHotVol => "Quintupled Vol (>=5) + Gap Up (>2 %) Faded Completely to LOD + Red Close (Highest-conviction Failed Catalyst at Tier-1: Vol Is 5x Average, Overnight Gap up Fails Completely and Price Closes at the Day's Low; Rarest Possible Failed-bull-catalyst Event, Capital-S-shift Regime-rejection Signal)",
        Preset::QuintupledVolGapDownCloseAtHodHotVol => "Quintupled Vol (>=5) + Gap Down (<-2 %) Absorbed Completely to HOD + Green Close (Highest-conviction Failed Catalyst at Tier-1: Vol Is 5x Average, Overnight Gap down Absorbed Completely and Price Closes at the Day's High; Rarest Possible Failed-bear-catalyst Event, Capital-S-shift Regime-acceptance Signal)",
        Preset::QuintupledVolGapUpMidpointHotVol => "Quintupled Vol (>=5) + Gap Up (>2 %) + Midpoint Close (Tier-1 Catalyst Gap-up with Inconclusive Intraday Follow-through: Vol Is 5x Average, Overnight Gap up Holds but Regular Session Neither Extends nor Fails Decisively; High-stakes Standoff after Catalyst Event with Unresolved Direction)",
        Preset::QuintupledVolGapDownMidpointHotVol => "Quintupled Vol (>=5) + Gap Down (<-2 %) + Midpoint Close (Tier-1 Catalyst Gap-down with Inconclusive Intraday Follow-through: Vol Is 5x Average, Overnight Gap down Holds but Regular Session Neither Extends nor Absorbs Decisively; High-stakes Standoff after Catalyst Event with Unresolved Direction)",
        Preset::BigIntradayRangeHotVol => "Wide Intraday Range (>8 % High-low Spread) + Hot Vol (Volatility Expansion Day: Regular Session Prints a Much Wider than Normal Trading Range with Elevated Participation; High-volatility Regime Worth a Directional-bias-screen at the Close and an Overnight-gap-screen the Next Morning)",
        Preset::TightIntradayRangeHotVol => "Tight Intraday Range (<1 % High-low Spread) + Hot Vol (Intraday Compression with Elevated Participation: Regular Session Prints a Much Narrower than Normal Trading Range Despite Hot Vol; Institutional Positioning Event Where Heavy Hands Trade without Moving the Tape; Breakout-candidate Worth a Watch-list-add)",
        Preset::BigIntradayRangeNearYearHighHotVol => "Wide Intraday Range (>8 %) + Hot Vol + At/near 52w High (<2 %) (Volatility-expansion Battle at the Year Peak: Regular Session Prints a Wide Trading Range Right at the 52w High with Elevated Participation; Bulls and Bears Fighting Hard at the Key Resistance with No Decisive Winner from Range Alone)",
        Preset::BigIntradayRangeNearYearLowHotVol => "Wide Intraday Range (>8 %) + Hot Vol + At/near 52w Low (<2 %) (Volatility-expansion Battle at the Year Trough: Regular Session Prints a Wide Trading Range Right at the 52w Low with Elevated Participation; Bulls and Bears Fighting Hard at the Key Support with No Decisive Winner from Range Alone)",
        Preset::BigIntradayRangeConfirmedAboveYearHighHotVol => "Wide Intraday Range (>8 %) + Hot Vol + Confirmed-breakout Zone (1-3 % past 52w High) (Volatility-expansion Battle in the Validated-breakout Zone: Regular Session Prints a Wide Trading Range Right after Price Cleared the Prior Peak with Elevated Participation; Post-breakout Consolidation Fight Where Bulls Defend the Breakout and Bears Test It)",
        Preset::BigIntradayRangeConfirmedBelowYearLowHotVol => "Wide Intraday Range (>8 %) + Hot Vol + Confirmed-breakdown Zone (1-3 % past 52w Low) (Volatility-expansion Battle in the Validated-breakdown Zone: Regular Session Prints a Wide Trading Range Right after Price Cleared the Prior Trough with Elevated Participation; Post-breakdown Consolidation Fight Where Bears Defend the Breakdown and Bulls Test It)",
        Preset::BigIntradayRangeDeepBelowYearHighHotVol => "Wide Intraday Range (>8 %) + Hot Vol + Far below 52w High (>=20 %) (Volatility-expansion Battle Deep in Pullback Territory: Regular Session Prints a Wide Trading Range Well below the Prior Peak with Elevated Participation; Capitulation-or-recovery Decision Fight Where Extended Decline Meets Institutional Pushback)",
        Preset::BigIntradayRangeDeepAboveYearLowHotVol => "Wide Intraday Range (>8 %) + Hot Vol + Far above 52w Low (>=20 %) (Volatility-expansion Battle Deep in Advance Territory: Regular Session Prints a Wide Trading Range Well above the Prior Trough with Elevated Participation; Top-or-continuation Decision Fight Where Extended Advance Meets Institutional Pushback)",
        Preset::BigIntradayRangeMidYearHighHotVol => "Wide Intraday Range (>8 %) + Hot Vol + Mid-range from High (5-20 %) (Volatility-expansion Battle in Mid-cycle Pullback Zone: Regular Session Prints a Wide Trading Range in the Proper Consolidation Range below the Prior Peak with Elevated Participation; Mid-cycle Indecision-resolution Fight Requiring Close-position Confirmation)",
        Preset::BigIntradayRangeMidYearLowHotVol => "Wide Intraday Range (>8 %) + Hot Vol + Mid-range from Low (5-20 %) (Volatility-expansion Battle in Mid-cycle Recovery Zone: Regular Session Prints a Wide Trading Range in the Proper Consolidation Range above the Prior Trough with Elevated Participation; Mid-cycle Indecision-resolution Fight Requiring Close-position Confirmation)",
        Preset::BigIntradayRangeJustOffYearHighHotVol => "Wide Intraday Range (>8 %) + Hot Vol + Just off 52w High (2-5 %) (Volatility-expansion Battle Just off the Year Peak: Regular Session Prints a Wide Trading Range Immediately after a Shallow Pullback from the 52w High with Elevated Participation; Post-tag Re-test Fight Where Bulls Attempt the High Again and Bears Defend It)",
        Preset::BigIntradayRangeJustOffYearLowHotVol => "Wide Intraday Range (>8 %) + Hot Vol + Just off 52w Low (2-5 %) (Volatility-expansion Battle Just off the Year Trough: Regular Session Prints a Wide Trading Range Immediately after a Shallow Bounce from the 52w Low with Elevated Participation; Post-tag Re-test Fight Where Bears Attempt the Low Again and Bulls Defend It)",
        Preset::TightIntradayRangeNearYearHighHotVol => "Tight Intraday Range (<1 %) + Hot Vol + At/near 52w High (<2 %) (Institutional Compression at the Year Peak: Regular Session Prints a Tight Trading Range Right at the 52w High with Elevated Participation; Coiled-spring Breakout Setup Where Heavy Hands Position without Moving the Tape and the Next Directional Break Carries Weight)",
        Preset::TightIntradayRangeNearYearLowHotVol => "Tight Intraday Range (<1 %) + Hot Vol + At/near 52w Low (<2 %) (Institutional Compression at the Year Trough: Regular Session Prints a Tight Trading Range Right at the 52w Low with Elevated Participation; Coiled-spring Breakdown-or-bottom Setup Where Heavy Hands Position without Moving the Tape and the Next Directional Break Carries Weight)",
        Preset::TightIntradayRangeConfirmedAboveYearHighHotVol => "Tight Intraday Range (<1 %) + Hot Vol + Confirmed-breakout Zone (1-3 % past 52w High) (Institutional Digestion of Validated Breakout: Regular Session Prints a Tight Trading Range Immediately after Price Cleared the Prior Peak with Elevated Participation; Post-breakout Consolidation Where Bulls Absorb and Digest the Breakout before the Next Leg)",
        Preset::TightIntradayRangeConfirmedBelowYearLowHotVol => "Tight Intraday Range (<1 %) + Hot Vol + Confirmed-breakdown Zone (1-3 % past 52w Low) (Institutional Digestion of Validated Breakdown: Regular Session Prints a Tight Trading Range Immediately after Price Cleared the Prior Trough with Elevated Participation; Post-breakdown Consolidation Where Bears Absorb and Digest the Breakdown before the Next Leg)",
        Preset::TightIntradayRangeDeepBelowYearHighHotVol => "Tight Intraday Range (<1 %) + Hot Vol + Far below 52w High (>=20 %) (Institutional Accumulation Deep in Pullback Territory: Regular Session Prints a Tight Trading Range Well below the Prior Peak with Elevated Participation; Basing-pattern Signal Where Smart Money Builds Position Quietly in Depressed-tape Conditions)",
        Preset::TightIntradayRangeDeepAboveYearLowHotVol => "Tight Intraday Range (<1 %) + Hot Vol + Far above 52w Low (>=20 %) (Institutional Distribution Deep in Advance Territory: Regular Session Prints a Tight Trading Range Well above the Prior Trough with Elevated Participation; Topping-pattern Signal Where Smart Money Exits Position Quietly in Euphoric-tape Conditions)",
        Preset::TightIntradayRangeMidYearHighHotVol => "Tight Intraday Range (<1 %) + Hot Vol + Mid-range from High (5-20 %) (Institutional Pause in Mid-cycle Pullback Zone: Regular Session Prints a Tight Trading Range in the Proper Consolidation Range below the Prior Peak with Elevated Participation; Mid-cycle Accumulation/pause Where Smart Money Positions before the Next Directional Move)",
        Preset::TightIntradayRangeMidYearLowHotVol => "Tight Intraday Range (<1 %) + Hot Vol + Mid-range from Low (5-20 %) (Institutional Pause in Mid-cycle Recovery Zone: Regular Session Prints a Tight Trading Range in the Proper Consolidation Range above the Prior Trough with Elevated Participation; Mid-cycle Distribution/pause Where Smart Money Positions before the Next Directional Move)",
        Preset::TightIntradayRangeJustOffYearHighHotVol => "Tight Intraday Range (<1 %) + Hot Vol + Just off 52w High (2-5 %) (Institutional Positioning Just off the Year Peak: Regular Session Prints a Tight Trading Range Immediately after a Shallow Pullback from the 52w High with Elevated Participation; Post-tag Pause Where Heavy Hands Re-position Quietly before Attempting the High Again)",
        Preset::TightIntradayRangeJustOffYearLowHotVol => "Tight Intraday Range (<1 %) + Hot Vol + Just off 52w Low (2-5 %) (Institutional Positioning Just off the Year Trough: Regular Session Prints a Tight Trading Range Immediately after a Shallow Bounce from the 52w Low with Elevated Participation; Post-tag Pause Where Heavy Hands Re-position Quietly before Testing the Low Again)",
        Preset::GapUpTightRangeHotVol => "Gap Up (>2 %) + Tight Intraday Range (<1 %) + Hot Vol (Gap-and-park Institutional Signal: Overnight Gap up but Regular Session Prints a Very Tight Intraday Range with No Follow-through or Fade; Price Pins at the Gap Level with Heavy Participation, Suggesting Strong Absorption at the New Price)",
        Preset::GapDownTightRangeHotVol => "Gap Down (<-2 %) + Tight Intraday Range (<1 %) + Hot Vol (Gap-and-park Institutional Signal: Overnight Gap down but Regular Session Prints a Very Tight Intraday Range with No Follow-through or Recovery; Price Pins at the Gap Level with Heavy Participation, Suggesting Strong Absorption at the New Price)",
        Preset::GapUpWideRangeHotVol => "Gap Up (>2 %) + Wide Intraday Range (>8 %) + Hot Vol (Gap-then-volatility-expansion Catalyst Signal: Overnight Gap up Followed by a Wide Trading Range with Elevated Participation; Two-way Fight Intraday after the Catalyst with Bulls and Bears Trading Aggressively in the Gap Zone; Close-position Resolves Direction)",
        Preset::GapDownWideRangeHotVol => "Gap Down (<-2 %) + Wide Intraday Range (>8 %) + Hot Vol (Gap-then-volatility-expansion Catalyst Signal: Overnight Gap down Followed by a Wide Trading Range with Elevated Participation; Two-way Fight Intraday after the Catalyst with Bears and Bulls Trading Aggressively in the Gap Zone; Close-position Resolves Direction)",
        Preset::GapUpWideRangeNearYearHighHotVol => "Gap Up (>2 %) + Wide Intraday Range (>8 %) + Hot Vol + At/near 52w High (<2 %) (Gap-and-fight at the Year Peak: Overnight Gap up Followed by a Wide Trading Range at the 52w High with Elevated Participation; High-stakes Breakout-day Battle Where Catalyst Meets Prior Peak Resistance; Close-position Resolves Whether the Breakout Sticks)",
        Preset::GapDownWideRangeNearYearLowHotVol => "Gap Down (<-2 %) + Wide Intraday Range (>8 %) + Hot Vol + At/near 52w Low (<2 %) (Gap-and-fight at the Year Trough: Overnight Gap down Followed by a Wide Trading Range at the 52w Low with Elevated Participation; High-stakes Breakdown-day Battle Where Catalyst Meets Prior Trough Support; Close-position Resolves Whether the Breakdown Sticks)",
        Preset::GapUpWideRangeConfirmedAboveYearHighHotVol => "Gap Up (>2 %) + Wide Intraday Range (>8 %) + Hot Vol + Confirmed-breakout Zone (1-3 % past 52w High) (Gap-and-fight in the Validated-breakout Zone: Overnight Gap up Followed by a Wide Trading Range Right after Price Cleared the Prior Peak with Elevated Participation; Post-breakout Extension Battle Where Bulls Defend the Breakout and Bears Test It)",
        Preset::GapDownWideRangeConfirmedBelowYearLowHotVol => "Gap Down (<-2 %) + Wide Intraday Range (>8 %) + Hot Vol + Confirmed-breakdown Zone (1-3 % past 52w Low) (Gap-and-fight in the Validated-breakdown Zone: Overnight Gap down Followed by a Wide Trading Range Right after Price Cleared the Prior Trough with Elevated Participation; Post-breakdown Extension Battle Where Bears Defend the Breakdown and Bulls Test It)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use rust_decimal::Decimal;

    fn bar(open: u64, high: u64, low: u64, close: u64, vol: u64, ts: i64) -> PriceBar {
        PriceBar {
            symbol: "X".into(),
            interval: crate::models::BarInterval::D1,
            bar_time: Utc.timestamp_opt(ts, 0).unwrap(),
            open: Decimal::from(open),
            high: Decimal::from(high),
            low: Decimal::from(low),
            close: Decimal::from(close),
            volume: Decimal::from(vol),
            source: "test".into(),
        }
    }

    #[test]
    fn gappers_fires_on_5pct_gap() {
        let bars = vec![
            bar(100, 100, 95, 100, 1_000_000, 1),
            bar(106, 110, 105, 108, 1_000_000, 2),
        ];
        let hit = stats_for("X", &bars).unwrap();
        assert!(matches(&hit, Preset::PremarketGappers));
    }

    #[test]
    fn gap_and_go_fires_on_upgap_with_strong_close_at_hod() {
        // Prior close 100. Open gaps to 105 (5% gap up).
        // Closes near day's high (108 vs HOD 108).
        // Need rel_volume >= 1.5 — build a 5-bar baseline at 1M then today at 2M.
        let bars = vec![
            bar(100, 101, 99,  100, 1_000_000, 1),
            bar(100, 101, 99,  100, 1_000_000, 2),
            bar(100, 101, 99,  100, 1_000_000, 3),
            bar(100, 101, 99,  100, 1_000_000, 4),
            bar(105, 108, 104, 108, 2_000_000, 5),
        ];
        let hit = stats_for("X", &bars).unwrap();
        assert!(matches(&hit, Preset::GapAndGo),
            "gap={} day_pct={} hod_dist={} rel_vol={}",
            hit.gap_pct, hit.day_pct, hit.hod_dist_pct, hit.rel_volume);
    }

    #[test]
    fn distribution_day_fires_on_2pct_down_with_high_volume() {
        let bars = vec![
            bar(100, 101, 99, 100, 1_000_000, 1),
            bar(100, 101, 99, 100, 1_000_000, 2),
            bar(100, 101, 99, 100, 1_000_000, 3),
            bar(100, 101, 99, 100, 1_000_000, 4),
            bar(100, 100, 95,  97, 2_000_000, 5),    // close -3%, vol 2x avg
        ];
        let hit = stats_for("X", &bars).unwrap();
        assert!(matches(&hit, Preset::DistributionDay));
    }

    #[test]
    fn accumulation_day_fires_on_2pct_up_with_high_volume() {
        let bars = vec![
            bar(100, 101, 99, 100, 1_000_000, 1),
            bar(100, 101, 99, 100, 1_000_000, 2),
            bar(100, 101, 99, 100, 1_000_000, 3),
            bar(100, 101, 99, 100, 1_000_000, 4),
            bar(100, 104, 100, 103, 2_000_000, 5),
        ];
        let hit = stats_for("X", &bars).unwrap();
        assert!(matches(&hit, Preset::AccumulationDay));
    }

    #[test]
    fn range_contraction_fires_on_tiny_day_with_low_volume() {
        let bars = vec![
            bar(100, 105, 95, 100, 2_000_000, 1),
            bar(100, 105, 95, 100, 2_000_000, 2),
            bar(100, 105, 95, 100, 2_000_000, 3),
            bar(100, 105, 95, 100, 2_000_000, 4),
            bar(100, 100, 100, 100, 1_000_000, 5),    // doji-like, half avg vol
        ];
        let hit = stats_for("X", &bars).unwrap();
        assert!(matches(&hit, Preset::RangeContractionDay),
            "day_pct={} gap_pct={} rel_vol={}",
            hit.day_pct, hit.gap_pct, hit.rel_volume);
    }

    #[test]
    fn momentum_needs_both_pct_and_volume() {
        // SMA-window = min(20, n). With 2 bars, avg = mean of both.
        // To clear rel_volume >= 2.0, need today >= 2× avg.
        // Here today=4M, prior=1M → avg=2.5M → rel_vol = 4/2.5 = 1.6 — too low.
        // Use 5 bars to get a meaningful avg, then a big surge on the last.
        let bars = vec![
            bar(100, 100, 95, 100, 1_000_000, 1),
            bar(100, 101, 99, 100, 1_000_000, 2),
            bar(100, 101, 99, 100, 1_000_000, 3),
            bar(100, 101, 99, 100, 1_000_000, 4),
            bar(100, 110, 100, 108, 6_000_000, 5),
        ];
        let hit = stats_for("X", &bars).unwrap();
        assert!(hit.change_pct >= 5.0, "change_pct = {}", hit.change_pct);
        assert!(hit.rel_volume >= 2.0, "rel_volume = {}", hit.rel_volume);
        assert!(matches(&hit, Preset::MomentumMovers));
    }
}

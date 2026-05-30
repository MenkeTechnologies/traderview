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

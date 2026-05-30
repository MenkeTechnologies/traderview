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

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

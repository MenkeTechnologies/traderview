// Stock scanners — Warrior/Zendoo presets across the user's watchlist universe.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

const PRESETS = [
    { id: 'premarket_gappers', label: t('chart.series.premarket_gappers'),  desc: '≥ 5% gap (open vs prior close)' },
    { id: 'momentum_movers',   label: t('chart.series.momentum_movers'),    desc: '≥ 5% move + 2× rel-volume' },
    { id: 'low_float_runners', label: t('chart.series.lowfloat_runners'),  desc: '≥ 10% move + 5× rel-volume' },
    { id: 'high_of_day',       label: t('chart.series.high_of_day'),        desc: 'within 0.5% of session high' },
    { id: 'volume_surge',      label: t('chart.series.volume_surge'),       desc: '≥ 3× 20-day avg volume' },
    { id: 'breakout',          label: t('view.scanners.preset.breakout.label'),     desc: t('view.scanners.preset.breakout.desc') },
    { id: 'breakdown',         label: t('view.scanners.preset.breakdown.label'),    desc: t('view.scanners.preset.breakdown.desc') },
    { id: 'pct52w_high',       label: t('view.scanners.preset.pct52w_high.label'),  desc: t('view.scanners.preset.pct52w_high.desc') },
    { id: 'pct52w_low',        label: t('view.scanners.preset.pct52w_low.label'),   desc: t('view.scanners.preset.pct52w_low.desc') },
    { id: 'oversold_bounce',   label: t('chart.series.oversold_bounce'),    desc: 'green day off oversold base' },
    { id: 'inside_day_squeeze', label: t('view.scanners.preset.inside_day_squeeze.label'), desc: t('view.scanners.preset.inside_day_squeeze.desc') },
    { id: 'low_vol_squeeze',    label: t('view.scanners.preset.low_vol_squeeze.label'),    desc: t('view.scanners.preset.low_vol_squeeze.desc') },
    { id: 'coiling_squeeze',    label: t('view.scanners.preset.coiling_squeeze.label'),    desc: t('view.scanners.preset.coiling_squeeze.desc') },
    { id: 'mid_range_squeeze',  label: t('view.scanners.preset.mid_range_squeeze.label'),  desc: t('view.scanners.preset.mid_range_squeeze.desc') },
    { id: 'bracket_squeeze',    label: t('view.scanners.preset.bracket_squeeze.label'),    desc: t('view.scanners.preset.bracket_squeeze.desc') },
    { id: 'doji_squeeze',       label: t('view.scanners.preset.doji_squeeze.label'),       desc: t('view.scanners.preset.doji_squeeze.desc') },
    { id: 'gap_fill_squeeze',   label: t('view.scanners.preset.gap_fill_squeeze.label'),   desc: t('view.scanners.preset.gap_fill_squeeze.desc') },
    { id: 'end_of_range_squeeze', label: t('view.scanners.preset.end_of_range_squeeze.label'), desc: t('view.scanners.preset.end_of_range_squeeze.desc') },
    { id: 'pre_breakout_squeeze', label: t('view.scanners.preset.pre_breakout_squeeze.label'), desc: t('view.scanners.preset.pre_breakout_squeeze.desc') },
    { id: 'pre_breakdown_squeeze', label: t('view.scanners.preset.pre_breakdown_squeeze.label'), desc: t('view.scanners.preset.pre_breakdown_squeeze.desc') },
    { id: 'symmetric_squeeze',    label: t('view.scanners.preset.symmetric_squeeze.label'),    desc: t('view.scanners.preset.symmetric_squeeze.desc') },
    { id: 'open_close_squeeze',   label: t('view.scanners.preset.open_close_squeeze.label'),   desc: t('view.scanners.preset.open_close_squeeze.desc') },
    { id: 'tight_hod_squeeze',    label: t('view.scanners.preset.tight_hod_squeeze.label'),    desc: t('view.scanners.preset.tight_hod_squeeze.desc') },
    { id: 'tight_lod_squeeze',    label: t('view.scanners.preset.tight_lod_squeeze.label'),    desc: t('view.scanners.preset.tight_lod_squeeze.desc') },
    { id: 'no_gap_no_change_squeeze', label: t('view.scanners.preset.no_gap_no_change_squeeze.label'), desc: t('view.scanners.preset.no_gap_no_change_squeeze.desc') },
    { id: 'quiet_tick_squeeze',   label: t('view.scanners.preset.quiet_tick_squeeze.label'),   desc: t('view.scanners.preset.quiet_tick_squeeze.desc') },
    { id: 'post_momentum_squeeze', label: t('view.scanners.preset.post_momentum_squeeze.label'), desc: t('view.scanners.preset.post_momentum_squeeze.desc') },
    { id: 'distant_extremes_squeeze', label: t('view.scanners.preset.distant_extremes_squeeze.label'), desc: t('view.scanners.preset.distant_extremes_squeeze.desc') },
    { id: 'balanced_drift_squeeze',   label: t('view.scanners.preset.balanced_drift_squeeze.label'),   desc: t('view.scanners.preset.balanced_drift_squeeze.desc') },
    { id: 'penny_move_squeeze',       label: t('view.scanners.preset.penny_move_squeeze.label'),       desc: t('view.scanners.preset.penny_move_squeeze.desc') },
    { id: 'dry_up_squeeze',            label: t('view.scanners.preset.dry_up_squeeze.label'),            desc: t('view.scanners.preset.dry_up_squeeze.desc') },
    { id: 'upper_range_squeeze',       label: t('view.scanners.preset.upper_range_squeeze.label'),       desc: t('view.scanners.preset.upper_range_squeeze.desc') },
    { id: 'lower_range_squeeze',       label: t('view.scanners.preset.lower_range_squeeze.label'),       desc: t('view.scanners.preset.lower_range_squeeze.desc') },
    { id: 'gap_reversal_squeeze',      label: t('view.scanners.preset.gap_reversal_squeeze.label'),      desc: t('view.scanners.preset.gap_reversal_squeeze.desc') },
    { id: 'pct52w_mid_squeeze',        label: t('view.scanners.preset.pct52w_mid_squeeze.label'),        desc: t('view.scanners.preset.pct52w_mid_squeeze.desc') },
    { id: 'deep_discount_squeeze',     label: t('view.scanners.preset.deep_discount_squeeze.label'),     desc: t('view.scanners.preset.deep_discount_squeeze.desc') },
    { id: 'flat_range_quiet_squeeze',  label: t('view.scanners.preset.flat_range_quiet_squeeze.label'),  desc: t('view.scanners.preset.flat_range_quiet_squeeze.desc') },
    { id: 'near_ath_quiet_squeeze',    label: t('view.scanners.preset.near_ath_quiet_squeeze.label'),    desc: t('view.scanners.preset.near_ath_quiet_squeeze.desc') },
    { id: 'near_atl_quiet_squeeze',    label: t('view.scanners.preset.near_atl_quiet_squeeze.label'),    desc: t('view.scanners.preset.near_atl_quiet_squeeze.desc') },
    { id: 'silent_breakout_setup',     label: t('view.scanners.preset.silent_breakout_setup.label'),     desc: t('view.scanners.preset.silent_breakout_setup.desc') },
    { id: 'silent_breakdown_setup',    label: t('view.scanners.preset.silent_breakdown_setup.label'),    desc: t('view.scanners.preset.silent_breakdown_setup.desc') },
    { id: 'gap_down_no_follow_squeeze', label: t('view.scanners.preset.gap_down_no_follow_squeeze.label'), desc: t('view.scanners.preset.gap_down_no_follow_squeeze.desc') },
    { id: 'gap_up_no_follow_squeeze',   label: t('view.scanners.preset.gap_up_no_follow_squeeze.label'),   desc: t('view.scanners.preset.gap_up_no_follow_squeeze.desc') },
    { id: 'unch_vol_dry_up_squeeze',    label: t('view.scanners.preset.unch_vol_dry_up_squeeze.label'),    desc: t('view.scanners.preset.unch_vol_dry_up_squeeze.desc') },
    { id: 'narrow_after_trend_squeeze', label: t('view.scanners.preset.narrow_after_trend_squeeze.label'), desc: t('view.scanners.preset.narrow_after_trend_squeeze.desc') },
    { id: 'dead_center_squeeze',       label: t('view.scanners.preset.dead_center_squeeze.label'),       desc: t('view.scanners.preset.dead_center_squeeze.desc') },
    { id: 'anchor_drift_squeeze',      label: t('view.scanners.preset.anchor_drift_squeeze.label'),      desc: t('view.scanners.preset.anchor_drift_squeeze.desc') },
    { id: 'post_gap_fill_squeeze',     label: t('view.scanners.preset.post_gap_fill_squeeze.label'),     desc: t('view.scanners.preset.post_gap_fill_squeeze.desc') },
    { id: 'post_spike_quiet_squeeze',  label: t('view.scanners.preset.post_spike_quiet_squeeze.label'),  desc: t('view.scanners.preset.post_spike_quiet_squeeze.desc') },
    { id: 'high_squeeze_bracket',      label: t('view.scanners.preset.high_squeeze_bracket.label'),      desc: t('view.scanners.preset.high_squeeze_bracket.desc') },
    { id: 'low_squeeze_bracket',       label: t('view.scanners.preset.low_squeeze_bracket.label'),       desc: t('view.scanners.preset.low_squeeze_bracket.desc') },
    { id: 'high_vol_stall_squeeze',    label: t('view.scanners.preset.high_vol_stall_squeeze.label'),    desc: t('view.scanners.preset.high_vol_stall_squeeze.desc') },
    { id: 'slight_lean_long_squeeze',  label: t('view.scanners.preset.slight_lean_long_squeeze.label'),  desc: t('view.scanners.preset.slight_lean_long_squeeze.desc') },
    { id: 'slight_lean_short_squeeze', label: t('view.scanners.preset.slight_lean_short_squeeze.label'), desc: t('view.scanners.preset.slight_lean_short_squeeze.desc') },
    { id: 'gap_change_match_squeeze',  label: t('view.scanners.preset.gap_change_match_squeeze.label'),  desc: t('view.scanners.preset.gap_change_match_squeeze.desc') },
    { id: 'wide_range_no_decision_squeeze', label: t('view.scanners.preset.wide_range_no_decision_squeeze.label'), desc: t('view.scanners.preset.wide_range_no_decision_squeeze.desc') },
    { id: 'pivot_pin_squeeze',         label: t('view.scanners.preset.pivot_pin_squeeze.label'),         desc: t('view.scanners.preset.pivot_pin_squeeze.desc') },
    { id: 'even_sides_squeeze',        label: t('view.scanners.preset.even_sides_squeeze.label'),        desc: t('view.scanners.preset.even_sides_squeeze.desc') },
    { id: 'quarter_day_inside_squeeze', label: t('view.scanners.preset.quarter_day_inside_squeeze.label'), desc: t('view.scanners.preset.quarter_day_inside_squeeze.desc') },
    { id: 'even_volume_quiet_squeeze',  label: t('view.scanners.preset.even_volume_quiet_squeeze.label'),  desc: t('view.scanners.preset.even_volume_quiet_squeeze.desc') },
    { id: 'tight_coil_high_squeeze',    label: t('view.scanners.preset.tight_coil_high_squeeze.label'),    desc: t('view.scanners.preset.tight_coil_high_squeeze.desc') },
    { id: 'tight_coil_low_squeeze',     label: t('view.scanners.preset.tight_coil_low_squeeze.label'),     desc: t('view.scanners.preset.tight_coil_low_squeeze.desc') },
    { id: 'even_width_squeeze',         label: t('view.scanners.preset.even_width_squeeze.label'),         desc: t('view.scanners.preset.even_width_squeeze.desc') },
    { id: 'small_gap_no_follow_squeeze', label: t('view.scanners.preset.small_gap_no_follow_squeeze.label'), desc: t('view.scanners.preset.small_gap_no_follow_squeeze.desc') },
    { id: 'holding_highs_squeeze',      label: t('view.scanners.preset.holding_highs_squeeze.label'),      desc: t('view.scanners.preset.holding_highs_squeeze.desc') },
    { id: 'holding_lows_squeeze',       label: t('view.scanners.preset.holding_lows_squeeze.label'),       desc: t('view.scanners.preset.holding_lows_squeeze.desc') },
    { id: 'stable_mid_squeeze',         label: t('view.scanners.preset.stable_mid_squeeze.label'),         desc: t('view.scanners.preset.stable_mid_squeeze.desc') },
    { id: 'lean_gap_match_squeeze',     label: t('view.scanners.preset.lean_gap_match_squeeze.label'),     desc: t('view.scanners.preset.lean_gap_match_squeeze.desc') },
    { id: 'long_shadow_quiet_squeeze',  label: t('view.scanners.preset.long_shadow_quiet_squeeze.label'),  desc: t('view.scanners.preset.long_shadow_quiet_squeeze.desc') },
    { id: 'overnight_move_reset_squeeze', label: t('view.scanners.preset.overnight_move_reset_squeeze.label'), desc: t('view.scanners.preset.overnight_move_reset_squeeze.desc') },
    { id: 'intraday_wiggle_reset_squeeze', label: t('view.scanners.preset.intraday_wiggle_reset_squeeze.label'), desc: t('view.scanners.preset.intraday_wiggle_reset_squeeze.desc') },
    { id: 'hot_dry_up_squeeze',         label: t('view.scanners.preset.hot_dry_up_squeeze.label'),         desc: t('view.scanners.preset.hot_dry_up_squeeze.desc') },
    { id: 'cold_dry_up_squeeze',        label: t('view.scanners.preset.cold_dry_up_squeeze.label'),        desc: t('view.scanners.preset.cold_dry_up_squeeze.desc') },
    { id: 'high_vol_gap_fade_squeeze',  label: t('view.scanners.preset.high_vol_gap_fade_squeeze.label'),  desc: t('view.scanners.preset.high_vol_gap_fade_squeeze.desc') },
    { id: 'chop_and_rest_quiet_squeeze', label: t('view.scanners.preset.chop_and_rest_quiet_squeeze.label'), desc: t('view.scanners.preset.chop_and_rest_quiet_squeeze.desc') },
    { id: 'silent_inside_squeeze',      label: t('view.scanners.preset.silent_inside_squeeze.label'),      desc: t('view.scanners.preset.silent_inside_squeeze.desc') },
    { id: 'heavy_vol_no_move_squeeze',  label: t('view.scanners.preset.heavy_vol_no_move_squeeze.label'),  desc: t('view.scanners.preset.heavy_vol_no_move_squeeze.desc') },
    { id: 'up_day_failed_squeeze',      label: t('view.scanners.preset.up_day_failed_squeeze.label'),      desc: t('view.scanners.preset.up_day_failed_squeeze.desc') },
    { id: 'down_day_reversed_squeeze',  label: t('view.scanners.preset.down_day_reversed_squeeze.label'),  desc: t('view.scanners.preset.down_day_reversed_squeeze.desc') },
    { id: 'gap_up_hold_hod_squeeze',    label: t('view.scanners.preset.gap_up_hold_hod_squeeze.label'),    desc: t('view.scanners.preset.gap_up_hold_hod_squeeze.desc') },
    { id: 'gap_down_hold_lod_squeeze',  label: t('view.scanners.preset.gap_down_hold_lod_squeeze.label'),  desc: t('view.scanners.preset.gap_down_hold_lod_squeeze.desc') },
    { id: 'long_inside_quiet_squeeze',  label: t('view.scanners.preset.long_inside_quiet_squeeze.label'),  desc: t('view.scanners.preset.long_inside_quiet_squeeze.desc') },
    { id: 'triple_zero_squeeze',        label: t('view.scanners.preset.triple_zero_squeeze.label'),        desc: t('view.scanners.preset.triple_zero_squeeze.desc') },
    { id: 'quarter_from_high_squeeze',  label: t('view.scanners.preset.quarter_from_high_squeeze.label'),  desc: t('view.scanners.preset.quarter_from_high_squeeze.desc') },
    { id: 'quarter_from_low_squeeze',   label: t('view.scanners.preset.quarter_from_low_squeeze.label'),   desc: t('view.scanners.preset.quarter_from_low_squeeze.desc') },
    { id: 'away_from_extremes_quiet_squeeze', label: t('view.scanners.preset.away_from_extremes_quiet_squeeze.label'), desc: t('view.scanners.preset.away_from_extremes_quiet_squeeze.desc') },
    { id: 'small_change_narrow_gap_squeeze', label: t('view.scanners.preset.small_change_narrow_gap_squeeze.label'), desc: t('view.scanners.preset.small_change_narrow_gap_squeeze.desc') },
    { id: 'big_range_no_commit_squeeze', label: t('view.scanners.preset.big_range_no_commit_squeeze.label'), desc: t('view.scanners.preset.big_range_no_commit_squeeze.desc') },
    { id: 'even_swing_squeeze',         label: t('view.scanners.preset.even_swing_squeeze.label'),         desc: t('view.scanners.preset.even_swing_squeeze.desc') },
    { id: 'no_move_at_mid_squeeze',     label: t('view.scanners.preset.no_move_at_mid_squeeze.label'),     desc: t('view.scanners.preset.no_move_at_mid_squeeze.desc') },
    { id: 'barely_moving_high_squeeze', label: t('view.scanners.preset.barely_moving_high_squeeze.label'), desc: t('view.scanners.preset.barely_moving_high_squeeze.desc') },
    { id: 'barely_moving_low_squeeze',  label: t('view.scanners.preset.barely_moving_low_squeeze.label'),  desc: t('view.scanners.preset.barely_moving_low_squeeze.desc') },
    { id: 'micro_range_squeeze',        label: t('view.scanners.preset.micro_range_squeeze.label'),        desc: t('view.scanners.preset.micro_range_squeeze.desc') },
    { id: 'low_vol_gap_hold_squeeze',   label: t('view.scanners.preset.low_vol_gap_hold_squeeze.label'),   desc: t('view.scanners.preset.low_vol_gap_hold_squeeze.desc') },
    { id: 'high_vol_gap_hold_squeeze',  label: t('view.scanners.preset.high_vol_gap_hold_squeeze.label'),  desc: t('view.scanners.preset.high_vol_gap_hold_squeeze.desc') },
    { id: 'upside_attempted_reject_squeeze', label: t('view.scanners.preset.upside_attempted_reject_squeeze.label'), desc: t('view.scanners.preset.upside_attempted_reject_squeeze.desc') },
    { id: 'downside_attempted_reject_squeeze', label: t('view.scanners.preset.downside_attempted_reject_squeeze.label'), desc: t('view.scanners.preset.downside_attempted_reject_squeeze.desc') },
    { id: 'tight_gap_small_change_squeeze', label: t('view.scanners.preset.tight_gap_small_change_squeeze.label'), desc: t('view.scanners.preset.tight_gap_small_change_squeeze.desc') },
    { id: 'pct52w_mid_wide_range_squeeze',  label: t('view.scanners.preset.pct52w_mid_wide_range_squeeze.label'),  desc: t('view.scanners.preset.pct52w_mid_wide_range_squeeze.desc') },
    { id: 'inside_and_coiled_squeeze',  label: t('view.scanners.preset.inside_and_coiled_squeeze.label'),  desc: t('view.scanners.preset.inside_and_coiled_squeeze.desc') },
    { id: 'pct52w_high_breath_squeeze', label: t('view.scanners.preset.pct52w_high_breath_squeeze.label'), desc: t('view.scanners.preset.pct52w_high_breath_squeeze.desc') },
    { id: 'pct52w_low_breath_squeeze',  label: t('view.scanners.preset.pct52w_low_breath_squeeze.label'),  desc: t('view.scanners.preset.pct52w_low_breath_squeeze.desc') },
    { id: 'gap_around_close_squeeze',   label: t('view.scanners.preset.gap_around_close_squeeze.label'),   desc: t('view.scanners.preset.gap_around_close_squeeze.desc') },
    { id: 'tight_close_split_squeeze',  label: t('view.scanners.preset.tight_close_split_squeeze.label'),  desc: t('view.scanners.preset.tight_close_split_squeeze.desc') },
    { id: 'hi_vol_no_extreme_squeeze',  label: t('view.scanners.preset.hi_vol_no_extreme_squeeze.label'),  desc: t('view.scanners.preset.hi_vol_no_extreme_squeeze.desc') },
    { id: 'tiny_move_with_gap_squeeze', label: t('view.scanners.preset.tiny_move_with_gap_squeeze.label'), desc: t('view.scanners.preset.tiny_move_with_gap_squeeze.desc') },
    { id: 'low_vol_green_squeeze',      label: t('view.scanners.preset.low_vol_green_squeeze.label'),      desc: t('view.scanners.preset.low_vol_green_squeeze.desc') },
    { id: 'low_vol_red_squeeze',        label: t('view.scanners.preset.low_vol_red_squeeze.label'),        desc: t('view.scanners.preset.low_vol_red_squeeze.desc') },
    { id: 'gap_aligns_change_squeeze',  label: t('view.scanners.preset.gap_aligns_change_squeeze.label'),  desc: t('view.scanners.preset.gap_aligns_change_squeeze.desc') },
    { id: 'unaffected_gap_squeeze',     label: t('view.scanners.preset.unaffected_gap_squeeze.label'),     desc: t('view.scanners.preset.unaffected_gap_squeeze.desc') },
    { id: 'stacked_closes_squeeze',     label: t('view.scanners.preset.stacked_closes_squeeze.label'),     desc: t('view.scanners.preset.stacked_closes_squeeze.desc') },
    { id: 'pullback_to_mid_squeeze',    label: t('view.scanners.preset.pullback_to_mid_squeeze.label'),    desc: t('view.scanners.preset.pullback_to_mid_squeeze.desc') },
    { id: 'bounce_from_mid_squeeze',    label: t('view.scanners.preset.bounce_from_mid_squeeze.label'),    desc: t('view.scanners.preset.bounce_from_mid_squeeze.desc') },
    { id: 'narrow_gap_hot_close_squeeze', label: t('view.scanners.preset.narrow_gap_hot_close_squeeze.label'), desc: t('view.scanners.preset.narrow_gap_hot_close_squeeze.desc') },
    { id: 'narrow_gap_cold_close_squeeze', label: t('view.scanners.preset.narrow_gap_cold_close_squeeze.label'), desc: t('view.scanners.preset.narrow_gap_cold_close_squeeze.desc') },
    { id: 'absorption_up_squeeze',      label: t('view.scanners.preset.absorption_up_squeeze.label'),      desc: t('view.scanners.preset.absorption_up_squeeze.desc') },
    { id: 'absorption_down_squeeze',    label: t('view.scanners.preset.absorption_down_squeeze.label'),    desc: t('view.scanners.preset.absorption_down_squeeze.desc') },
    { id: 'stall_at_mid_squeeze',       label: t('view.scanners.preset.stall_at_mid_squeeze.label'),       desc: t('view.scanners.preset.stall_at_mid_squeeze.desc') },
    { id: 'no_close_decision_squeeze',  label: t('view.scanners.preset.no_close_decision_squeeze.label'),  desc: t('view.scanners.preset.no_close_decision_squeeze.desc') },
    { id: 'gap_inside_range_squeeze',   label: t('view.scanners.preset.gap_inside_range_squeeze.label'),   desc: t('view.scanners.preset.gap_inside_range_squeeze.desc') },
    { id: 'subpoint_move_squeeze',      label: t('view.scanners.preset.subpoint_move_squeeze.label'),      desc: t('view.scanners.preset.subpoint_move_squeeze.desc') },
    { id: 'no_vol_no_move_squeeze',     label: t('view.scanners.preset.no_vol_no_move_squeeze.label'),     desc: t('view.scanners.preset.no_vol_no_move_squeeze.desc') },
    { id: 'vol_without_change_squeeze', label: t('view.scanners.preset.vol_without_change_squeeze.label'), desc: t('view.scanners.preset.vol_without_change_squeeze.desc') },
    { id: 'tick_inside_open_squeeze',   label: t('view.scanners.preset.tick_inside_open_squeeze.label'),   desc: t('view.scanners.preset.tick_inside_open_squeeze.desc') },
    { id: 'pct52w_exact_half_squeeze',  label: t('view.scanners.preset.pct52w_exact_half_squeeze.label'),  desc: t('view.scanners.preset.pct52w_exact_half_squeeze.desc') },
    { id: 'unchanged_on_volume_squeeze', label: t('view.scanners.preset.unchanged_on_volume_squeeze.label'), desc: t('view.scanners.preset.unchanged_on_volume_squeeze.desc') },
    { id: 'wide_hod_narrow_lod_squeeze', label: t('view.scanners.preset.wide_hod_narrow_lod_squeeze.label'), desc: t('view.scanners.preset.wide_hod_narrow_lod_squeeze.desc') },
    { id: 'narrow_hod_wide_lod_squeeze', label: t('view.scanners.preset.narrow_hod_wide_lod_squeeze.label'), desc: t('view.scanners.preset.narrow_hod_wide_lod_squeeze.desc') },
    { id: 'perfect_balance_squeeze',    label: t('view.scanners.preset.perfect_balance_squeeze.label'),    desc: t('view.scanners.preset.perfect_balance_squeeze.desc') },
    { id: 'low_vol_hot_zone_squeeze',   label: t('view.scanners.preset.low_vol_hot_zone_squeeze.label'),   desc: t('view.scanners.preset.low_vol_hot_zone_squeeze.desc') },
    { id: 'low_vol_cold_zone_squeeze',  label: t('view.scanners.preset.low_vol_cold_zone_squeeze.label'),  desc: t('view.scanners.preset.low_vol_cold_zone_squeeze.desc') },
    { id: 'drift_higher_squeeze',       label: t('view.scanners.preset.drift_higher_squeeze.label'),       desc: t('view.scanners.preset.drift_higher_squeeze.desc') },
    { id: 'drift_lower_squeeze',        label: t('view.scanners.preset.drift_lower_squeeze.label'),        desc: t('view.scanners.preset.drift_lower_squeeze.desc') },
    { id: 'extreme_quiet_squeeze',      label: t('view.scanners.preset.extreme_quiet_squeeze.label'),      desc: t('view.scanners.preset.extreme_quiet_squeeze.desc') },
    { id: 'pinned_to_open_squeeze',     label: t('view.scanners.preset.pinned_to_open_squeeze.label'),     desc: t('view.scanners.preset.pinned_to_open_squeeze.desc') },
    { id: 'big_gap_small_day_squeeze',  label: t('view.scanners.preset.big_gap_small_day_squeeze.label'),  desc: t('view.scanners.preset.big_gap_small_day_squeeze.desc') },
    { id: 'post_crash_squeeze',         label: t('view.scanners.preset.post_crash_squeeze.label'),         desc: t('view.scanners.preset.post_crash_squeeze.desc') },
    { id: 'post_spike_stabilize_squeeze', label: t('view.scanners.preset.post_spike_stabilize_squeeze.label'), desc: t('view.scanners.preset.post_spike_stabilize_squeeze.desc') },
    { id: 'tight_with_small_gap_squeeze', label: t('view.scanners.preset.tight_with_small_gap_squeeze.label'), desc: t('view.scanners.preset.tight_with_small_gap_squeeze.desc') },
    { id: 'big_vol_tiny_change_squeeze', label: t('view.scanners.preset.big_vol_tiny_change_squeeze.label'), desc: t('view.scanners.preset.big_vol_tiny_change_squeeze.desc') },
    { id: 'quiet_expansion_squeeze',    label: t('view.scanners.preset.quiet_expansion_squeeze.label'),    desc: t('view.scanners.preset.quiet_expansion_squeeze.desc') },
    { id: 'inside_bar_high_squeeze',    label: t('view.scanners.preset.inside_bar_high_squeeze.label'),    desc: t('view.scanners.preset.inside_bar_high_squeeze.desc') },
    { id: 'inside_bar_low_squeeze',     label: t('view.scanners.preset.inside_bar_low_squeeze.label'),     desc: t('view.scanners.preset.inside_bar_low_squeeze.desc') },
    { id: 'flat_gap_inside_range_squeeze', label: t('view.scanners.preset.flat_gap_inside_range_squeeze.label'), desc: t('view.scanners.preset.flat_gap_inside_range_squeeze.desc') },
    { id: 'pct52w_edge_dry_up_squeeze',  label: t('view.scanners.preset.pct52w_edge_dry_up_squeeze.label'),  desc: t('view.scanners.preset.pct52w_edge_dry_up_squeeze.desc') },
    { id: 'narrow_center_squeeze',      label: t('view.scanners.preset.narrow_center_squeeze.label'),      desc: t('view.scanners.preset.narrow_center_squeeze.desc') },
    { id: 'lopsided_quiet_squeeze',     label: t('view.scanners.preset.lopsided_quiet_squeeze.label'),     desc: t('view.scanners.preset.lopsided_quiet_squeeze.desc') },
    { id: 'silent_leader_squeeze',      label: t('view.scanners.preset.silent_leader_squeeze.label'),      desc: t('view.scanners.preset.silent_leader_squeeze.desc') },
    { id: 'silent_laggard_squeeze',     label: t('view.scanners.preset.silent_laggard_squeeze.label'),     desc: t('view.scanners.preset.silent_laggard_squeeze.desc') },
    { id: 'near_vwap_quiet_squeeze',    label: t('view.scanners.preset.near_vwap_quiet_squeeze.label'),    desc: t('view.scanners.preset.near_vwap_quiet_squeeze.desc') },
    { id: 'barely_moving_mid_range_squeeze', label: t('view.scanners.preset.barely_moving_mid_range_squeeze.label'), desc: t('view.scanners.preset.barely_moving_mid_range_squeeze.desc') },
    { id: 'third_from_high_squeeze',    label: t('view.scanners.preset.third_from_high_squeeze.label'),    desc: t('view.scanners.preset.third_from_high_squeeze.desc') },
    { id: 'third_from_low_squeeze',     label: t('view.scanners.preset.third_from_low_squeeze.label'),     desc: t('view.scanners.preset.third_from_low_squeeze.desc') },
    { id: 'high_range_no_change_squeeze', label: t('view.scanners.preset.high_range_no_change_squeeze.label'), desc: t('view.scanners.preset.high_range_no_change_squeeze.desc') },
    { id: 'low_range_no_change_squeeze',  label: t('view.scanners.preset.low_range_no_change_squeeze.label'),  desc: t('view.scanners.preset.low_range_no_change_squeeze.desc') },
    { id: 'low_volume_up_day_squeeze',  label: t('view.scanners.preset.low_volume_up_day_squeeze.label'),  desc: t('view.scanners.preset.low_volume_up_day_squeeze.desc') },
    { id: 'low_volume_down_day_squeeze', label: t('view.scanners.preset.low_volume_down_day_squeeze.label'), desc: t('view.scanners.preset.low_volume_down_day_squeeze.desc') },
    { id: 'hi_vol_up_day_no_hod_squeeze', label: t('view.scanners.preset.hi_vol_up_day_no_hod_squeeze.label'), desc: t('view.scanners.preset.hi_vol_up_day_no_hod_squeeze.desc') },
    { id: 'hi_vol_down_day_no_lod_squeeze', label: t('view.scanners.preset.hi_vol_down_day_no_lod_squeeze.label'), desc: t('view.scanners.preset.hi_vol_down_day_no_lod_squeeze.desc') },
    { id: 'gap_up_fade_to_flat', label: t('view.scanners.preset.gap_up_fade_to_flat.label'), desc: t('view.scanners.preset.gap_up_fade_to_flat.desc') },
    { id: 'gap_down_reclaim_to_flat', label: t('view.scanners.preset.gap_down_reclaim_to_flat.label'), desc: t('view.scanners.preset.gap_down_reclaim_to_flat.desc') },
    { id: 'gap_up_held_green', label: t('view.scanners.preset.gap_up_held_green.label'), desc: t('view.scanners.preset.gap_up_held_green.desc') },
    { id: 'gap_down_held_red', label: t('view.scanners.preset.gap_down_held_red.label'), desc: t('view.scanners.preset.gap_down_held_red.desc') },
    { id: 'gap_up_half_fade', label: t('view.scanners.preset.gap_up_half_fade.label'), desc: t('view.scanners.preset.gap_up_half_fade.desc') },
    { id: 'gap_down_half_reclaim', label: t('view.scanners.preset.gap_down_half_reclaim.label'), desc: t('view.scanners.preset.gap_down_half_reclaim.desc') },
    { id: 'gap_and_go_xl', label: t('view.scanners.preset.gap_and_go_xl.label'), desc: t('view.scanners.preset.gap_and_go_xl.desc') },
    { id: 'gap_and_crash_xl', label: t('view.scanners.preset.gap_and_crash_xl.label'), desc: t('view.scanners.preset.gap_and_crash_xl.desc') },
    { id: 'gap_up_but_day_red', label: t('view.scanners.preset.gap_up_but_day_red.label'), desc: t('view.scanners.preset.gap_up_but_day_red.desc') },
    { id: 'gap_down_but_day_green', label: t('view.scanners.preset.gap_down_but_day_green.label'), desc: t('view.scanners.preset.gap_down_but_day_green.desc') },
    { id: 'gap_up_flush_on_volume', label: t('view.scanners.preset.gap_up_flush_on_volume.label'), desc: t('view.scanners.preset.gap_up_flush_on_volume.desc') },
    { id: 'gap_down_reversal_on_volume', label: t('view.scanners.preset.gap_down_reversal_on_volume.label'), desc: t('view.scanners.preset.gap_down_reversal_on_volume.desc') },
    { id: 'pct52w_top_decile_hot_vol', label: t('view.scanners.preset.pct52w_top_decile_hot_vol.label'), desc: t('view.scanners.preset.pct52w_top_decile_hot_vol.desc') },
    { id: 'pct52w_bottom_decile_hot_vol', label: t('view.scanners.preset.pct52w_bottom_decile_hot_vol.label'), desc: t('view.scanners.preset.pct52w_bottom_decile_hot_vol.desc') },
    { id: 'pct52w_top_decile_dry_vol', label: t('view.scanners.preset.pct52w_top_decile_dry_vol.label'), desc: t('view.scanners.preset.pct52w_top_decile_dry_vol.desc') },
    { id: 'pct52w_bottom_decile_dry_vol', label: t('view.scanners.preset.pct52w_bottom_decile_dry_vol.label'), desc: t('view.scanners.preset.pct52w_bottom_decile_dry_vol.desc') },
    { id: 'new_high_green_day', label: t('view.scanners.preset.new_high_green_day.label'), desc: t('view.scanners.preset.new_high_green_day.desc') },
    { id: 'new_low_red_day', label: t('view.scanners.preset.new_low_red_day.label'), desc: t('view.scanners.preset.new_low_red_day.desc') },
    { id: 'new_high_red_day', label: t('view.scanners.preset.new_high_red_day.label'), desc: t('view.scanners.preset.new_high_red_day.desc') },
    { id: 'new_low_green_day', label: t('view.scanners.preset.new_low_green_day.label'), desc: t('view.scanners.preset.new_low_green_day.desc') },
    { id: 'new_high_on_hot_vol', label: t('view.scanners.preset.new_high_on_hot_vol.label'), desc: t('view.scanners.preset.new_high_on_hot_vol.desc') },
    { id: 'new_low_on_hot_vol', label: t('view.scanners.preset.new_low_on_hot_vol.label'), desc: t('view.scanners.preset.new_low_on_hot_vol.desc') },
    { id: 'quiet_near_the_top', label: t('view.scanners.preset.quiet_near_the_top.label'), desc: t('view.scanners.preset.quiet_near_the_top.desc') },
    { id: 'quiet_near_the_bottom', label: t('view.scanners.preset.quiet_near_the_bottom.label'), desc: t('view.scanners.preset.quiet_near_the_bottom.desc') },
    { id: 'noisy_near_the_top', label: t('view.scanners.preset.noisy_near_the_top.label'), desc: t('view.scanners.preset.noisy_near_the_top.desc') },
    { id: 'noisy_near_the_bottom', label: t('view.scanners.preset.noisy_near_the_bottom.label'), desc: t('view.scanners.preset.noisy_near_the_bottom.desc') },
    { id: 'mid_range_chop_hot_vol', label: t('view.scanners.preset.mid_range_chop_hot_vol.label'), desc: t('view.scanners.preset.mid_range_chop_hot_vol.desc') },
    { id: 'mid_range_chop_dry_vol', label: t('view.scanners.preset.mid_range_chop_dry_vol.label'), desc: t('view.scanners.preset.mid_range_chop_dry_vol.desc') },
    { id: 'close_near_hod_no_breakout', label: t('view.scanners.preset.close_near_hod_no_breakout.label'), desc: t('view.scanners.preset.close_near_hod_no_breakout.desc') },
    { id: 'close_near_lod_no_breakdown', label: t('view.scanners.preset.close_near_lod_no_breakdown.label'), desc: t('view.scanners.preset.close_near_lod_no_breakdown.desc') },
    { id: 'close_near_hod_strong_day', label: t('view.scanners.preset.close_near_hod_strong_day.label'), desc: t('view.scanners.preset.close_near_hod_strong_day.desc') },
    { id: 'close_near_lod_weak_day', label: t('view.scanners.preset.close_near_lod_weak_day.label'), desc: t('view.scanners.preset.close_near_lod_weak_day.desc') },
    { id: 'inside_range_no_volume', label: t('view.scanners.preset.inside_range_no_volume.label'), desc: t('view.scanners.preset.inside_range_no_volume.desc') },
    { id: 'outside_range_on_volume', label: t('view.scanners.preset.outside_range_on_volume.label'), desc: t('view.scanners.preset.outside_range_on_volume.desc') },
    { id: 'up_day_lower_high', label: t('view.scanners.preset.up_day_lower_high.label'), desc: t('view.scanners.preset.up_day_lower_high.desc') },
    { id: 'down_day_higher_low', label: t('view.scanners.preset.down_day_higher_low.label'), desc: t('view.scanners.preset.down_day_higher_low.desc') },
    { id: 'strong_day_balanced_range', label: t('view.scanners.preset.strong_day_balanced_range.label'), desc: t('view.scanners.preset.strong_day_balanced_range.desc') },
    { id: 'weak_day_balanced_range', label: t('view.scanners.preset.weak_day_balanced_range.label'), desc: t('view.scanners.preset.weak_day_balanced_range.desc') },
    { id: 'channel_ride_up', label: t('view.scanners.preset.channel_ride_up.label'), desc: t('view.scanners.preset.channel_ride_up.desc') },
    { id: 'channel_ride_down', label: t('view.scanners.preset.channel_ride_down.label'), desc: t('view.scanners.preset.channel_ride_down.desc') },
    { id: 'pullback_in_uptrend', label: t('view.scanners.preset.pullback_in_uptrend.label'), desc: t('view.scanners.preset.pullback_in_uptrend.desc') },
    { id: 'bounce_in_downtrend', label: t('view.scanners.preset.bounce_in_downtrend.label'), desc: t('view.scanners.preset.bounce_in_downtrend.desc') },
    { id: 'deep_pullback_in_uptrend', label: t('view.scanners.preset.deep_pullback_in_uptrend.label'), desc: t('view.scanners.preset.deep_pullback_in_uptrend.desc') },
    { id: 'deep_bounce_in_downtrend', label: t('view.scanners.preset.deep_bounce_in_downtrend.label'), desc: t('view.scanners.preset.deep_bounce_in_downtrend.desc') },
    { id: 'tight_above_mid_strong', label: t('view.scanners.preset.tight_above_mid_strong.label'), desc: t('view.scanners.preset.tight_above_mid_strong.desc') },
    { id: 'tight_below_mid_weak', label: t('view.scanners.preset.tight_below_mid_weak.label'), desc: t('view.scanners.preset.tight_below_mid_weak.desc') },
    { id: 'hot_vol_no_move_at_high', label: t('view.scanners.preset.hot_vol_no_move_at_high.label'), desc: t('view.scanners.preset.hot_vol_no_move_at_high.desc') },
    { id: 'hot_vol_no_move_at_low', label: t('view.scanners.preset.hot_vol_no_move_at_low.label'), desc: t('view.scanners.preset.hot_vol_no_move_at_low.desc') },
    { id: 'big_up_gap_inside_day', label: t('view.scanners.preset.big_up_gap_inside_day.label'), desc: t('view.scanners.preset.big_up_gap_inside_day.desc') },
    { id: 'big_down_gap_inside_day', label: t('view.scanners.preset.big_down_gap_inside_day.label'), desc: t('view.scanners.preset.big_down_gap_inside_day.desc') },
    { id: 'steady_up_dry_vol', label: t('view.scanners.preset.steady_up_dry_vol.label'), desc: t('view.scanners.preset.steady_up_dry_vol.desc') },
    { id: 'steady_down_dry_vol', label: t('view.scanners.preset.steady_down_dry_vol.label'), desc: t('view.scanners.preset.steady_down_dry_vol.desc') },
    { id: 'impulsive_up_hot_vol', label: t('view.scanners.preset.impulsive_up_hot_vol.label'), desc: t('view.scanners.preset.impulsive_up_hot_vol.desc') },
    { id: 'impulsive_down_hot_vol', label: t('view.scanners.preset.impulsive_down_hot_vol.label'), desc: t('view.scanners.preset.impulsive_down_hot_vol.desc') },
    { id: 'parabolic_up', label: t('view.scanners.preset.parabolic_up.label'), desc: t('view.scanners.preset.parabolic_up.desc') },
    { id: 'parabolic_down', label: t('view.scanners.preset.parabolic_down.label'), desc: t('view.scanners.preset.parabolic_down.desc') },
    { id: 'blow_off_top', label: t('view.scanners.preset.blow_off_top.label'), desc: t('view.scanners.preset.blow_off_top.desc') },
    { id: 'selling_climax_bottom', label: t('view.scanners.preset.selling_climax_bottom.label'), desc: t('view.scanners.preset.selling_climax_bottom.desc') },
    { id: 'up_day_gap_only_move', label: t('view.scanners.preset.up_day_gap_only_move.label'), desc: t('view.scanners.preset.up_day_gap_only_move.desc') },
    { id: 'down_day_gap_only_move', label: t('view.scanners.preset.down_day_gap_only_move.label'), desc: t('view.scanners.preset.down_day_gap_only_move.desc') },
    { id: 'intraday_only_green_day', label: t('view.scanners.preset.intraday_only_green_day.label'), desc: t('view.scanners.preset.intraday_only_green_day.desc') },
    { id: 'intraday_only_red_day', label: t('view.scanners.preset.intraday_only_red_day.label'), desc: t('view.scanners.preset.intraday_only_red_day.desc') },
    { id: 'reversal_up_from_open', label: t('view.scanners.preset.reversal_up_from_open.label'), desc: t('view.scanners.preset.reversal_up_from_open.desc') },
    { id: 'reversal_down_from_open', label: t('view.scanners.preset.reversal_down_from_open.label'), desc: t('view.scanners.preset.reversal_down_from_open.desc') },
    { id: 'trend_day_up', label: t('view.scanners.preset.trend_day_up.label'), desc: t('view.scanners.preset.trend_day_up.desc') },
    { id: 'trend_day_down', label: t('view.scanners.preset.trend_day_down.label'), desc: t('view.scanners.preset.trend_day_down.desc') },
    { id: 'double_bottom_candidate', label: t('view.scanners.preset.double_bottom_candidate.label'), desc: t('view.scanners.preset.double_bottom_candidate.desc') },
    { id: 'double_top_candidate', label: t('view.scanners.preset.double_top_candidate.label'), desc: t('view.scanners.preset.double_top_candidate.desc') },
    { id: 'pct52w_mid_zone', label: t('view.scanners.preset.pct52w_mid_zone.label'), desc: t('view.scanners.preset.pct52w_mid_zone.desc') },
    { id: 'pct52w_range_breakout_triggered', label: t('view.scanners.preset.pct52w_range_breakout_triggered.label'), desc: t('view.scanners.preset.pct52w_range_breakout_triggered.desc') },
    { id: 'pct52w_range_breakdown_triggered', label: t('view.scanners.preset.pct52w_range_breakdown_triggered.label'), desc: t('view.scanners.preset.pct52w_range_breakdown_triggered.desc') },
    { id: 'pct52w_tight_coil', label: t('view.scanners.preset.pct52w_tight_coil.label'), desc: t('view.scanners.preset.pct52w_tight_coil.desc') },
    { id: 'symmetric_triangle', label: t('view.scanners.preset.symmetric_triangle.label'), desc: t('view.scanners.preset.symmetric_triangle.desc') },
    { id: 'narrowing_range_on_flat', label: t('view.scanners.preset.narrowing_range_on_flat.label'), desc: t('view.scanners.preset.narrowing_range_on_flat.desc') },
    { id: 'gap_too_far_big_pullback', label: t('view.scanners.preset.gap_too_far_big_pullback.label'), desc: t('view.scanners.preset.gap_too_far_big_pullback.desc') },
    { id: 'gap_too_far_big_bounce', label: t('view.scanners.preset.gap_too_far_big_bounce.label'), desc: t('view.scanners.preset.gap_too_far_big_bounce.desc') },
    { id: 'chain_breakout_level', label: t('view.scanners.preset.chain_breakout_level.label'), desc: t('view.scanners.preset.chain_breakout_level.desc') },
    { id: 'chain_breakdown_level', label: t('view.scanners.preset.chain_breakdown_level.label'), desc: t('view.scanners.preset.chain_breakdown_level.desc') },
    { id: 'pct52w_range_pos_top', label: t('view.scanners.preset.pct52w_range_pos_top.label'), desc: t('view.scanners.preset.pct52w_range_pos_top.desc') },
    { id: 'pct52w_range_pos_bottom', label: t('view.scanners.preset.pct52w_range_pos_bottom.label'), desc: t('view.scanners.preset.pct52w_range_pos_bottom.desc') },
    { id: 'high_range_high_vol_strong', label: t('view.scanners.preset.high_range_high_vol_strong.label'), desc: t('view.scanners.preset.high_range_high_vol_strong.desc') },
    { id: 'high_range_high_vol_weak', label: t('view.scanners.preset.high_range_high_vol_weak.label'), desc: t('view.scanners.preset.high_range_high_vol_weak.desc') },
    { id: 'low_range_low_vol_neutral', label: t('view.scanners.preset.low_range_low_vol_neutral.label'), desc: t('view.scanners.preset.low_range_low_vol_neutral.desc') },
    { id: 'avg_range_avg_vol_neutral', label: t('view.scanners.preset.avg_range_avg_vol_neutral.label'), desc: t('view.scanners.preset.avg_range_avg_vol_neutral.desc') },
    { id: 'failed_breakout_high_reclaim', label: t('view.scanners.preset.failed_breakout_high_reclaim.label'), desc: t('view.scanners.preset.failed_breakout_high_reclaim.desc') },
    { id: 'failed_breakdown_low_reclaim', label: t('view.scanners.preset.failed_breakdown_low_reclaim.label'), desc: t('view.scanners.preset.failed_breakdown_low_reclaim.desc') },
    { id: 'hot_vol_hot_gap', label: t('view.scanners.preset.hot_vol_hot_gap.label'), desc: t('view.scanners.preset.hot_vol_hot_gap.desc') },
    { id: 'dry_vol_dry_gap', label: t('view.scanners.preset.dry_vol_dry_gap.label'), desc: t('view.scanners.preset.dry_vol_dry_gap.desc') },
    { id: 'outer_edge_push_up', label: t('view.scanners.preset.outer_edge_push_up.label'), desc: t('view.scanners.preset.outer_edge_push_up.desc') },
    { id: 'outer_edge_push_down', label: t('view.scanners.preset.outer_edge_push_down.label'), desc: t('view.scanners.preset.outer_edge_push_down.desc') },
    { id: 'middle_zone_up_drift', label: t('view.scanners.preset.middle_zone_up_drift.label'), desc: t('view.scanners.preset.middle_zone_up_drift.desc') },
    { id: 'middle_zone_down_drift', label: t('view.scanners.preset.middle_zone_down_drift.label'), desc: t('view.scanners.preset.middle_zone_down_drift.desc') },
    { id: 'middle_zone_hot_vol_breakout_high', label: t('view.scanners.preset.middle_zone_hot_vol_breakout_high.label'), desc: t('view.scanners.preset.middle_zone_hot_vol_breakout_high.desc') },
    { id: 'middle_zone_hot_vol_breakout_low', label: t('view.scanners.preset.middle_zone_hot_vol_breakout_low.label'), desc: t('view.scanners.preset.middle_zone_hot_vol_breakout_low.desc') },
    { id: 'gap_up_small_but_hot_vol', label: t('view.scanners.preset.gap_up_small_but_hot_vol.label'), desc: t('view.scanners.preset.gap_up_small_but_hot_vol.desc') },
    { id: 'gap_down_small_but_hot_vol', label: t('view.scanners.preset.gap_down_small_but_hot_vol.label'), desc: t('view.scanners.preset.gap_down_small_but_hot_vol.desc') },
    { id: 'gap_up_medium_neutral', label: t('view.scanners.preset.gap_up_medium_neutral.label'), desc: t('view.scanners.preset.gap_up_medium_neutral.desc') },
    { id: 'gap_down_medium_neutral', label: t('view.scanners.preset.gap_down_medium_neutral.label'), desc: t('view.scanners.preset.gap_down_medium_neutral.desc') },
    { id: 'hod_reclaim_after_flush', label: t('view.scanners.preset.hod_reclaim_after_flush.label'), desc: t('view.scanners.preset.hod_reclaim_after_flush.desc') },
    { id: 'lod_fail_after_push', label: t('view.scanners.preset.lod_fail_after_push.label'), desc: t('view.scanners.preset.lod_fail_after_push.desc') },
    { id: 'hod_reclaim_from_flat_gap', label: t('view.scanners.preset.hod_reclaim_from_flat_gap.label'), desc: t('view.scanners.preset.hod_reclaim_from_flat_gap.desc') },
    { id: 'lod_fail_from_flat_gap', label: t('view.scanners.preset.lod_fail_from_flat_gap.label'), desc: t('view.scanners.preset.lod_fail_from_flat_gap.desc') },
    { id: 'pct52w_top_boundary_reject', label: t('view.scanners.preset.pct52w_top_boundary_reject.label'), desc: t('view.scanners.preset.pct52w_top_boundary_reject.desc') },
    { id: 'pct52w_bottom_boundary_reject', label: t('view.scanners.preset.pct52w_bottom_boundary_reject.label'), desc: t('view.scanners.preset.pct52w_bottom_boundary_reject.desc') },
    { id: 'pct52w_top_boundary_accept', label: t('view.scanners.preset.pct52w_top_boundary_accept.label'), desc: t('view.scanners.preset.pct52w_top_boundary_accept.desc') },
    { id: 'pct52w_bottom_boundary_accept', label: t('view.scanners.preset.pct52w_bottom_boundary_accept.label'), desc: t('view.scanners.preset.pct52w_bottom_boundary_accept.desc') },
    { id: 'up_from_bottom_spring', label: t('view.scanners.preset.up_from_bottom_spring.label'), desc: t('view.scanners.preset.up_from_bottom_spring.desc') },
    { id: 'down_from_top_upthrust', label: t('view.scanners.preset.down_from_top_upthrust.label'), desc: t('view.scanners.preset.down_from_top_upthrust.desc') },
    { id: 'up_thrust_bar_reject', label: t('view.scanners.preset.up_thrust_bar_reject.label'), desc: t('view.scanners.preset.up_thrust_bar_reject.desc') },
    { id: 'down_thrust_bar_reject', label: t('view.scanners.preset.down_thrust_bar_reject.label'), desc: t('view.scanners.preset.down_thrust_bar_reject.desc') },
    { id: 'exhaustion_top_wide_range', label: t('view.scanners.preset.exhaustion_top_wide_range.label'), desc: t('view.scanners.preset.exhaustion_top_wide_range.desc') },
    { id: 'exhaustion_bottom_wide_range', label: t('view.scanners.preset.exhaustion_bottom_wide_range.label'), desc: t('view.scanners.preset.exhaustion_bottom_wide_range.desc') },
    { id: 'up_trend_day_wide_range', label: t('view.scanners.preset.up_trend_day_wide_range.label'), desc: t('view.scanners.preset.up_trend_day_wide_range.desc') },
    { id: 'down_trend_day_wide_range', label: t('view.scanners.preset.down_trend_day_wide_range.label'), desc: t('view.scanners.preset.down_trend_day_wide_range.desc') },
    { id: 'silent_spring_near_52w_low', label: t('view.scanners.preset.silent_spring_near_52w_low.label'), desc: t('view.scanners.preset.silent_spring_near_52w_low.desc') },
    { id: 'silent_up_thrust_near_52w_high', label: t('view.scanners.preset.silent_up_thrust_near_52w_high.label'), desc: t('view.scanners.preset.silent_up_thrust_near_52w_high.desc') },
    { id: 'gap_strong_day_open_pivot', label: t('view.scanners.preset.gap_strong_day_open_pivot.label'), desc: t('view.scanners.preset.gap_strong_day_open_pivot.desc') },
    { id: 'gap_weak_day_open_pivot', label: t('view.scanners.preset.gap_weak_day_open_pivot.label'), desc: t('view.scanners.preset.gap_weak_day_open_pivot.desc') },
    { id: 'conviction_breakout_combo', label: t('view.scanners.preset.conviction_breakout_combo.label'), desc: t('view.scanners.preset.conviction_breakout_combo.desc') },
    { id: 'conviction_breakdown_combo', label: t('view.scanners.preset.conviction_breakdown_combo.label'), desc: t('view.scanners.preset.conviction_breakdown_combo.desc') },
    { id: 'pullback_inside_trend_up', label: t('view.scanners.preset.pullback_inside_trend_up.label'), desc: t('view.scanners.preset.pullback_inside_trend_up.desc') },
    { id: 'pullback_inside_trend_down', label: t('view.scanners.preset.pullback_inside_trend_down.label'), desc: t('view.scanners.preset.pullback_inside_trend_down.desc') },
    { id: 'range_contraction_squeeze_high', label: t('view.scanners.preset.range_contraction_squeeze_high.label'), desc: t('view.scanners.preset.range_contraction_squeeze_high.desc') },
    { id: 'range_contraction_squeeze_low', label: t('view.scanners.preset.range_contraction_squeeze_low.label'), desc: t('view.scanners.preset.range_contraction_squeeze_low.desc') },
    { id: 'range_expansion_at_top_on_vol', label: t('view.scanners.preset.range_expansion_at_top_on_vol.label'), desc: t('view.scanners.preset.range_expansion_at_top_on_vol.desc') },
    { id: 'range_expansion_at_bottom_on_vol', label: t('view.scanners.preset.range_expansion_at_bottom_on_vol.label'), desc: t('view.scanners.preset.range_expansion_at_bottom_on_vol.desc') },
    { id: 'gap_inside_range_balanced', label: t('view.scanners.preset.gap_inside_range_balanced.label'), desc: t('view.scanners.preset.gap_inside_range_balanced.desc') },
    { id: 'gap_inside_range_impulse', label: t('view.scanners.preset.gap_inside_range_impulse.label'), desc: t('view.scanners.preset.gap_inside_range_impulse.desc') },
    { id: 'one_wick_close_at_mid', label: t('view.scanners.preset.one_wick_close_at_mid.label'), desc: t('view.scanners.preset.one_wick_close_at_mid.desc') },
    { id: 'one_wick_close_at_mid_down', label: t('view.scanners.preset.one_wick_close_at_mid_down.label'), desc: t('view.scanners.preset.one_wick_close_at_mid_down.desc') },
    { id: 'upper_wick_green_day_confirm', label: t('view.scanners.preset.upper_wick_green_day_confirm.label'), desc: t('view.scanners.preset.upper_wick_green_day_confirm.desc') },
    { id: 'lower_wick_red_day_confirm', label: t('view.scanners.preset.lower_wick_red_day_confirm.label'), desc: t('view.scanners.preset.lower_wick_red_day_confirm.desc') },
    { id: 'inside_bar_tight_at_mid', label: t('view.scanners.preset.inside_bar_tight_at_mid.label'), desc: t('view.scanners.preset.inside_bar_tight_at_mid.desc') },
    { id: 'outside_bar_volume_both', label: t('view.scanners.preset.outside_bar_volume_both.label'), desc: t('view.scanners.preset.outside_bar_volume_both.desc') },
    { id: 'leading_up_day_light_vol', label: t('view.scanners.preset.leading_up_day_light_vol.label'), desc: t('view.scanners.preset.leading_up_day_light_vol.desc') },
    { id: 'leading_down_day_light_vol', label: t('view.scanners.preset.leading_down_day_light_vol.label'), desc: t('view.scanners.preset.leading_down_day_light_vol.desc') },
    { id: 'small_change_on_vol_near_high', label: t('view.scanners.preset.small_change_on_vol_near_high.label'), desc: t('view.scanners.preset.small_change_on_vol_near_high.desc') },
    { id: 'small_change_on_vol_near_low', label: t('view.scanners.preset.small_change_on_vol_near_low.label'), desc: t('view.scanners.preset.small_change_on_vol_near_low.desc') },
    { id: 'big_gap_big_vol_big_day', label: t('view.scanners.preset.big_gap_big_vol_big_day.label'), desc: t('view.scanners.preset.big_gap_big_vol_big_day.desc') },
    { id: 'big_gap_no_follow_through', label: t('view.scanners.preset.big_gap_no_follow_through.label'), desc: t('view.scanners.preset.big_gap_no_follow_through.desc') },
    { id: 'confluence_long_setup', label: t('view.scanners.preset.confluence_long_setup.label'), desc: t('view.scanners.preset.confluence_long_setup.desc') },
    { id: 'confluence_short_setup', label: t('view.scanners.preset.confluence_short_setup.label'), desc: t('view.scanners.preset.confluence_short_setup.desc') },
    { id: 'no_extreme_day', label: t('view.scanners.preset.no_extreme_day.label'), desc: t('view.scanners.preset.no_extreme_day.desc') },
    { id: 'accelerating_up_trend', label: t('view.scanners.preset.accelerating_up_trend.label'), desc: t('view.scanners.preset.accelerating_up_trend.desc') },
    { id: 'accelerating_down_trend', label: t('view.scanners.preset.accelerating_down_trend.label'), desc: t('view.scanners.preset.accelerating_down_trend.desc') },
    { id: 'divergence_push_from_top', label: t('view.scanners.preset.divergence_push_from_top.label'), desc: t('view.scanners.preset.divergence_push_from_top.desc') },
    { id: 'divergence_push_from_bottom', label: t('view.scanners.preset.divergence_push_from_bottom.label'), desc: t('view.scanners.preset.divergence_push_from_bottom.desc') },
    { id: 'price_flat_vol_hot_above_mid', label: t('view.scanners.preset.price_flat_vol_hot_above_mid.label'), desc: t('view.scanners.preset.price_flat_vol_hot_above_mid.desc') },
    { id: 'price_flat_vol_hot_below_mid', label: t('view.scanners.preset.price_flat_vol_hot_below_mid.label'), desc: t('view.scanners.preset.price_flat_vol_hot_below_mid.desc') },
    { id: 'small_change_on_vol_mid', label: t('view.scanners.preset.small_change_on_vol_mid.label'), desc: t('view.scanners.preset.small_change_on_vol_mid.desc') },
    { id: 'hot_rolling_vol_gap', label: t('view.scanners.preset.hot_rolling_vol_gap.label'), desc: t('view.scanners.preset.hot_rolling_vol_gap.desc') },
    { id: 'silent_drift_gap', label: t('view.scanners.preset.silent_drift_gap.label'), desc: t('view.scanners.preset.silent_drift_gap.desc') },
    { id: 'up_day_on_dry_vol_near_52w_high', label: t('view.scanners.preset.up_day_on_dry_vol_near_52w_high.label'), desc: t('view.scanners.preset.up_day_on_dry_vol_near_52w_high.desc') },
    { id: 'down_day_on_dry_vol_near_52w_low', label: t('view.scanners.preset.down_day_on_dry_vol_near_52w_low.label'), desc: t('view.scanners.preset.down_day_on_dry_vol_near_52w_low.desc') },
    { id: 'up_day_on_hot_vol_near_52w_high', label: t('view.scanners.preset.up_day_on_hot_vol_near_52w_high.label'), desc: t('view.scanners.preset.up_day_on_hot_vol_near_52w_high.desc') },
    { id: 'down_day_on_hot_vol_near_52w_low', label: t('view.scanners.preset.down_day_on_hot_vol_near_52w_low.label'), desc: t('view.scanners.preset.down_day_on_hot_vol_near_52w_low.desc') },
    { id: 'narrow_day_dry_vol_mid', label: t('view.scanners.preset.narrow_day_dry_vol_mid.label'), desc: t('view.scanners.preset.narrow_day_dry_vol_mid.desc') },
    { id: 'wide_day_hot_vol_mid', label: t('view.scanners.preset.wide_day_hot_vol_mid.label'), desc: t('view.scanners.preset.wide_day_hot_vol_mid.desc') },
    { id: 'hot_vol_at_mid_no_move', label: t('view.scanners.preset.hot_vol_at_mid_no_move.label'), desc: t('view.scanners.preset.hot_vol_at_mid_no_move.desc') },
    { id: 'dry_vol_at_mid_no_move', label: t('view.scanners.preset.dry_vol_at_mid_no_move.label'), desc: t('view.scanners.preset.dry_vol_at_mid_no_move.desc') },
    { id: 'big_change_tiny_range_up', label: t('view.scanners.preset.big_change_tiny_range_up.label'), desc: t('view.scanners.preset.big_change_tiny_range_up.desc') },
    { id: 'big_change_tiny_range_down', label: t('view.scanners.preset.big_change_tiny_range_down.label'), desc: t('view.scanners.preset.big_change_tiny_range_down.desc') },
    { id: 'tiny_change_wide_range_on_vol', label: t('view.scanners.preset.tiny_change_wide_range_on_vol.label'), desc: t('view.scanners.preset.tiny_change_wide_range_on_vol.desc') },
    { id: 'tiny_change_wide_range_on_dry_vol', label: t('view.scanners.preset.tiny_change_wide_range_on_dry_vol.label'), desc: t('view.scanners.preset.tiny_change_wide_range_on_dry_vol.desc') },
    { id: 'large_gap_moderate_move_hot_vol', label: t('view.scanners.preset.large_gap_moderate_move_hot_vol.label'), desc: t('view.scanners.preset.large_gap_moderate_move_hot_vol.desc') },
    { id: 'small_gap_big_move_hot_vol', label: t('view.scanners.preset.small_gap_big_move_hot_vol.label'), desc: t('view.scanners.preset.small_gap_big_move_hot_vol.desc') },
    { id: 'no_vol_trend_up', label: t('view.scanners.preset.no_vol_trend_up.label'), desc: t('view.scanners.preset.no_vol_trend_up.desc') },
    { id: 'no_vol_trend_down', label: t('view.scanners.preset.no_vol_trend_down.label'), desc: t('view.scanners.preset.no_vol_trend_down.desc') },
    { id: 'churn_at_top_dry_vol', label: t('view.scanners.preset.churn_at_top_dry_vol.label'), desc: t('view.scanners.preset.churn_at_top_dry_vol.desc') },
    { id: 'churn_at_bottom_dry_vol', label: t('view.scanners.preset.churn_at_bottom_dry_vol.label'), desc: t('view.scanners.preset.churn_at_bottom_dry_vol.desc') },
    { id: 'huge_gap_flat_change', label: t('view.scanners.preset.huge_gap_flat_change.label'), desc: t('view.scanners.preset.huge_gap_flat_change.desc') },
    { id: 'no_gap_huge_change', label: t('view.scanners.preset.no_gap_huge_change.label'), desc: t('view.scanners.preset.no_gap_huge_change.desc') },
    { id: 'extreme_vol_flat_gap_flat_day', label: t('view.scanners.preset.extreme_vol_flat_gap_flat_day.label'), desc: t('view.scanners.preset.extreme_vol_flat_gap_flat_day.desc') },
    { id: 'illiquid_big_gap_flat_day', label: t('view.scanners.preset.illiquid_big_gap_flat_day.label'), desc: t('view.scanners.preset.illiquid_big_gap_flat_day.desc') },
    { id: 'organic_up_day_close_at_hod', label: t('view.scanners.preset.organic_up_day_close_at_hod.label'), desc: t('view.scanners.preset.organic_up_day_close_at_hod.desc') },
    { id: 'organic_down_day_close_at_lod', label: t('view.scanners.preset.organic_down_day_close_at_lod.label'), desc: t('view.scanners.preset.organic_down_day_close_at_lod.desc') },
    { id: 'strong_day_dry_vol_up', label: t('view.scanners.preset.strong_day_dry_vol_up.label'), desc: t('view.scanners.preset.strong_day_dry_vol_up.desc') },
    { id: 'strong_day_dry_vol_down', label: t('view.scanners.preset.strong_day_dry_vol_down.label'), desc: t('view.scanners.preset.strong_day_dry_vol_down.desc') },
    { id: 'tight_coil_at_mid_range', label: t('view.scanners.preset.tight_coil_at_mid_range.label'), desc: t('view.scanners.preset.tight_coil_at_mid_range.desc') },
    { id: 'wide_outside_range_dry_vol', label: t('view.scanners.preset.wide_outside_range_dry_vol.label'), desc: t('view.scanners.preset.wide_outside_range_dry_vol.desc') },
    { id: 'gap_held_and_extended_up', label: t('view.scanners.preset.gap_held_and_extended_up.label'), desc: t('view.scanners.preset.gap_held_and_extended_up.desc') },
    { id: 'gap_held_and_extended_down', label: t('view.scanners.preset.gap_held_and_extended_down.label'), desc: t('view.scanners.preset.gap_held_and_extended_down.desc') },
    { id: 'pct52w_high_breakout_close_at_hod', label: t('view.scanners.preset.pct52w_high_breakout_close_at_hod.label'), desc: t('view.scanners.preset.pct52w_high_breakout_close_at_hod.desc') },
    { id: 'pct52w_low_breakdown_close_at_lod', label: t('view.scanners.preset.pct52w_low_breakdown_close_at_lod.label'), desc: t('view.scanners.preset.pct52w_low_breakdown_close_at_lod.desc') },
    { id: 'pct52w_mid_hot_vol_flat', label: t('view.scanners.preset.pct52w_mid_hot_vol_flat.label'), desc: t('view.scanners.preset.pct52w_mid_hot_vol_flat.desc') },
    { id: 'pct52w_mid_dry_vol_flat', label: t('view.scanners.preset.pct52w_mid_dry_vol_flat.label'), desc: t('view.scanners.preset.pct52w_mid_dry_vol_flat.desc') },
    { id: 'vol_spike_no_trend', label: t('view.scanners.preset.vol_spike_no_trend.label'), desc: t('view.scanners.preset.vol_spike_no_trend.desc') },
    { id: 'vol_spike_on_trend', label: t('view.scanners.preset.vol_spike_on_trend.label'), desc: t('view.scanners.preset.vol_spike_on_trend.desc') },
    { id: 'tight_coil_at_high_dry_vol', label: t('view.scanners.preset.tight_coil_at_high_dry_vol.label'), desc: t('view.scanners.preset.tight_coil_at_high_dry_vol.desc') },
    { id: 'tight_coil_at_low_dry_vol', label: t('view.scanners.preset.tight_coil_at_low_dry_vol.label'), desc: t('view.scanners.preset.tight_coil_at_low_dry_vol.desc') },
    { id: 'orderly_trend_at_highs', label: t('view.scanners.preset.orderly_trend_at_highs.label'), desc: t('view.scanners.preset.orderly_trend_at_highs.desc') },
    { id: 'orderly_trend_at_lows', label: t('view.scanners.preset.orderly_trend_at_lows.label'), desc: t('view.scanners.preset.orderly_trend_at_lows.desc') },
    { id: 'hot_vol_mid_range_churn', label: t('view.scanners.preset.hot_vol_mid_range_churn.label'), desc: t('view.scanners.preset.hot_vol_mid_range_churn.desc') },
    { id: 'dry_vol_at_extreme_close', label: t('view.scanners.preset.dry_vol_at_extreme_close.label'), desc: t('view.scanners.preset.dry_vol_at_extreme_close.desc') },
    { id: 'day_change_mismatch', label: t('view.scanners.preset.day_change_mismatch.label'), desc: t('view.scanners.preset.day_change_mismatch.desc') },
    { id: 'day_change_aligned_big', label: t('view.scanners.preset.day_change_aligned_big.label'), desc: t('view.scanners.preset.day_change_aligned_big.desc') },
    { id: 'huge_range_hot_vol', label: t('view.scanners.preset.huge_range_hot_vol.label'), desc: t('view.scanners.preset.huge_range_hot_vol.desc') },
    { id: 'huge_range_dry_vol', label: t('view.scanners.preset.huge_range_dry_vol.label'), desc: t('view.scanners.preset.huge_range_dry_vol.desc') },
    { id: 'pct52w_low_hot_vol_up', label: t('view.scanners.preset.pct52w_low_hot_vol_up.label'), desc: t('view.scanners.preset.pct52w_low_hot_vol_up.desc') },
    { id: 'pct52w_high_hot_vol_down', label: t('view.scanners.preset.pct52w_high_hot_vol_down.label'), desc: t('view.scanners.preset.pct52w_high_hot_vol_down.desc') },
    { id: 'gap_held_no_extension', label: t('view.scanners.preset.gap_held_no_extension.label'), desc: t('view.scanners.preset.gap_held_no_extension.desc') },
    { id: 'gap_partial_fade', label: t('view.scanners.preset.gap_partial_fade.label'), desc: t('view.scanners.preset.gap_partial_fade.desc') },
    { id: 'year_high_intraday_weak', label: t('view.scanners.preset.year_high_intraday_weak.label'), desc: t('view.scanners.preset.year_high_intraday_weak.desc') },
    { id: 'year_low_intraday_strong', label: t('view.scanners.preset.year_low_intraday_strong.label'), desc: t('view.scanners.preset.year_low_intraday_strong.desc') },
    { id: 'weak_hands_at_highs', label: t('view.scanners.preset.weak_hands_at_highs.label'), desc: t('view.scanners.preset.weak_hands_at_highs.desc') },
    { id: 'strong_hands_at_lows', label: t('view.scanners.preset.strong_hands_at_lows.label'), desc: t('view.scanners.preset.strong_hands_at_lows.desc') },
    { id: 'narrow_range_hot_vol_squeeze', label: t('view.scanners.preset.narrow_range_hot_vol_squeeze.label'), desc: t('view.scanners.preset.narrow_range_hot_vol_squeeze.desc') },
    { id: 'wide_range_dry_vol_drift', label: t('view.scanners.preset.wide_range_dry_vol_drift.label'), desc: t('view.scanners.preset.wide_range_dry_vol_drift.desc') },
    { id: 'leadership_trend_day', label: t('view.scanners.preset.leadership_trend_day.label'), desc: t('view.scanners.preset.leadership_trend_day.desc') },
    { id: 'worst_actor_flush_day', label: t('view.scanners.preset.worst_actor_flush_day.label'), desc: t('view.scanners.preset.worst_actor_flush_day.desc') },
    { id: 'gap_up_at_year_low', label: t('view.scanners.preset.gap_up_at_year_low.label'), desc: t('view.scanners.preset.gap_up_at_year_low.desc') },
    { id: 'gap_down_at_year_high', label: t('view.scanners.preset.gap_down_at_year_high.label'), desc: t('view.scanners.preset.gap_down_at_year_high.desc') },
    { id: 'big_up_mid_range_close', label: t('view.scanners.preset.big_up_mid_range_close.label'), desc: t('view.scanners.preset.big_up_mid_range_close.desc') },
    { id: 'big_down_mid_range_close', label: t('view.scanners.preset.big_down_mid_range_close.label'), desc: t('view.scanners.preset.big_down_mid_range_close.desc') },
    { id: 'hod_close_hot_vol_flat', label: t('view.scanners.preset.hod_close_hot_vol_flat.label'), desc: t('view.scanners.preset.hod_close_hot_vol_flat.desc') },
    { id: 'lod_close_hot_vol_flat', label: t('view.scanners.preset.lod_close_hot_vol_flat.label'), desc: t('view.scanners.preset.lod_close_hot_vol_flat.desc') },
    { id: 'rising_wedge_coil', label: t('view.scanners.preset.rising_wedge_coil.label'), desc: t('view.scanners.preset.rising_wedge_coil.desc') },
    { id: 'falling_wedge_coil', label: t('view.scanners.preset.falling_wedge_coil.label'), desc: t('view.scanners.preset.falling_wedge_coil.desc') },
    { id: 'big_gap_and_extend', label: t('view.scanners.preset.big_gap_and_extend.label'), desc: t('view.scanners.preset.big_gap_and_extend.desc') },
    { id: 'big_gap_and_reverse', label: t('view.scanners.preset.big_gap_and_reverse.label'), desc: t('view.scanners.preset.big_gap_and_reverse.desc') },
    { id: 'efficient_mover_hot_vol', label: t('view.scanners.preset.efficient_mover_hot_vol.label'), desc: t('view.scanners.preset.efficient_mover_hot_vol.desc') },
    { id: 'inefficient_churn_hot_vol', label: t('view.scanners.preset.inefficient_churn_hot_vol.label'), desc: t('view.scanners.preset.inefficient_churn_hot_vol.desc') },
    { id: 'gap_up_at_mid_range', label: t('view.scanners.preset.gap_up_at_mid_range.label'), desc: t('view.scanners.preset.gap_up_at_mid_range.desc') },
    { id: 'gap_down_at_mid_range', label: t('view.scanners.preset.gap_down_at_mid_range.label'), desc: t('view.scanners.preset.gap_down_at_mid_range.desc') },
    { id: 'battle_bar_hot_vol', label: t('view.scanners.preset.battle_bar_hot_vol.label'), desc: t('view.scanners.preset.battle_bar_hot_vol.desc') },
    { id: 'illiquid_swing_dry_vol', label: t('view.scanners.preset.illiquid_swing_dry_vol.label'), desc: t('view.scanners.preset.illiquid_swing_dry_vol.desc') },
    { id: 'gap_down_intraday_reclaim_up', label: t('view.scanners.preset.gap_down_intraday_reclaim_up.label'), desc: t('view.scanners.preset.gap_down_intraday_reclaim_up.desc') },
    { id: 'gap_up_intraday_reject_down', label: t('view.scanners.preset.gap_up_intraday_reject_down.label'), desc: t('view.scanners.preset.gap_up_intraday_reject_down.desc') },
    { id: 'hot_vol_moderate_change_flat_day', label: t('view.scanners.preset.hot_vol_moderate_change_flat_day.label'), desc: t('view.scanners.preset.hot_vol_moderate_change_flat_day.desc') },
    { id: 'dry_vol_moderate_change_flat_day', label: t('view.scanners.preset.dry_vol_moderate_change_flat_day.label'), desc: t('view.scanners.preset.dry_vol_moderate_change_flat_day.desc') },
    { id: 'wide_range_at_year_high', label: t('view.scanners.preset.wide_range_at_year_high.label'), desc: t('view.scanners.preset.wide_range_at_year_high.desc') },
    { id: 'wide_range_at_year_low', label: t('view.scanners.preset.wide_range_at_year_low.label'), desc: t('view.scanners.preset.wide_range_at_year_low.desc') },
    { id: 'hot_vol_gap_held_and_extended', label: t('view.scanners.preset.hot_vol_gap_held_and_extended.label'), desc: t('view.scanners.preset.hot_vol_gap_held_and_extended.desc') },
    { id: 'hot_vol_gap_faded_deep', label: t('view.scanners.preset.hot_vol_gap_faded_deep.label'), desc: t('view.scanners.preset.hot_vol_gap_faded_deep.desc') },
    { id: 'tight_range_at_year_high', label: t('view.scanners.preset.tight_range_at_year_high.label'), desc: t('view.scanners.preset.tight_range_at_year_high.desc') },
    { id: 'tight_range_at_year_low', label: t('view.scanners.preset.tight_range_at_year_low.label'), desc: t('view.scanners.preset.tight_range_at_year_low.desc') },
    { id: 'balanced_mid_wick_hot_vol', label: t('view.scanners.preset.balanced_mid_wick_hot_vol.label'), desc: t('view.scanners.preset.balanced_mid_wick_hot_vol.desc') },
    { id: 'balanced_mid_wick_dry_vol', label: t('view.scanners.preset.balanced_mid_wick_dry_vol.label'), desc: t('view.scanners.preset.balanced_mid_wick_dry_vol.desc') },
    { id: 'gap_up_hod_close_controlled', label: t('view.scanners.preset.gap_up_hod_close_controlled.label'), desc: t('view.scanners.preset.gap_up_hod_close_controlled.desc') },
    { id: 'gap_down_lod_close_controlled', label: t('view.scanners.preset.gap_down_lod_close_controlled.label'), desc: t('view.scanners.preset.gap_down_lod_close_controlled.desc') },
    { id: 'all_green_tight_day', label: t('view.scanners.preset.all_green_tight_day.label'), desc: t('view.scanners.preset.all_green_tight_day.desc') },
    { id: 'all_red_tight_day', label: t('view.scanners.preset.all_red_tight_day.label'), desc: t('view.scanners.preset.all_red_tight_day.desc') },
    { id: 'micro_range_at_year_high', label: t('view.scanners.preset.micro_range_at_year_high.label'), desc: t('view.scanners.preset.micro_range_at_year_high.desc') },
    { id: 'micro_range_at_year_low', label: t('view.scanners.preset.micro_range_at_year_low.label'), desc: t('view.scanners.preset.micro_range_at_year_low.desc') },
    { id: 'consolidation_break_up', label: t('view.scanners.preset.consolidation_break_up.label'), desc: t('view.scanners.preset.consolidation_break_up.desc') },
    { id: 'consolidation_break_down', label: t('view.scanners.preset.consolidation_break_down.label'), desc: t('view.scanners.preset.consolidation_break_down.desc') },
    { id: 'hot_vol_gap_held_flat_change', label: t('view.scanners.preset.hot_vol_gap_held_flat_change.label'), desc: t('view.scanners.preset.hot_vol_gap_held_flat_change.desc') },
    { id: 'dry_vol_gap_held_flat_change', label: t('view.scanners.preset.dry_vol_gap_held_flat_change.label'), desc: t('view.scanners.preset.dry_vol_gap_held_flat_change.desc') },
    { id: 'all_directions_aligned_hot_vol_up', label: t('view.scanners.preset.all_directions_aligned_hot_vol_up.label'), desc: t('view.scanners.preset.all_directions_aligned_hot_vol_up.desc') },
    { id: 'all_directions_aligned_hot_vol_down', label: t('view.scanners.preset.all_directions_aligned_hot_vol_down.label'), desc: t('view.scanners.preset.all_directions_aligned_hot_vol_down.desc') },
    { id: 'intraday_recovery_from_gap_down', label: t('view.scanners.preset.intraday_recovery_from_gap_down.label'), desc: t('view.scanners.preset.intraday_recovery_from_gap_down.desc') },
    { id: 'intraday_rejection_from_gap_up', label: t('view.scanners.preset.intraday_rejection_from_gap_up.label'), desc: t('view.scanners.preset.intraday_rejection_from_gap_up.desc') },
    { id: 'pct52w_mid_upper_hot_vol_down', label: t('view.scanners.preset.pct52w_mid_upper_hot_vol_down.label'), desc: t('view.scanners.preset.pct52w_mid_upper_hot_vol_down.desc') },
    { id: 'pct52w_mid_lower_hot_vol_up', label: t('view.scanners.preset.pct52w_mid_lower_hot_vol_up.label'), desc: t('view.scanners.preset.pct52w_mid_lower_hot_vol_up.desc') },
];

export async function renderScanners(mount) {
    const tok = currentViewToken();
    const lists = await api.watchlists();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.scanners.h1.scanners" class="view-title">// SCANNERS</h1>
        <p data-i18n="view.scanners.hint.warrior_zendoo_style_preset_scans_across_your_watc" class="muted small">Warrior/Zendoo-style preset scans across your watchlist universe.
        Click a preset to run.</p>

        <div class="chart-panel">
            <label><span data-i18n="view.scanners.label.universe">Universe</span>
                <select id="wl">
                    <option data-i18n="view.scanners.opt.all_my_watchlists" value="">all my watchlists</option>
                    ${lists.map(w => `<option value="${w.id}">${esc(w.name)}</option>`).join('')}
                </select>
            </label>
        </div>

        <div class="scanner-grid">
            ${PRESETS.map(p => {
                const labelKey = `view.scanners.preset.${p.id}.label`;
                const descKey  = `view.scanners.preset.${p.id}.desc`;
                const labelTr = (() => { const v = t(labelKey); return (v && v !== labelKey) ? v : p.label; })();
                const descTr  = (() => { const v = t(descKey);  return (v && v !== descKey)  ? v : p.desc;  })();
                return `<button class="scanner-tile" data-preset="${p.id}">
                    <div class="scanner-title">${esc(labelTr)}</div>
                    <div class="scanner-desc">${esc(descTr)}</div>
                </button>`;
            }).join('')}
        </div>

        <div id="scan-result"></div>
    `;
    mount.querySelectorAll('[data-preset]').forEach(b =>
        b.addEventListener('click', async () => {
            const wlEl = mount.querySelector('#wl');
            const wid = (wlEl && wlEl.value) || null;
            const el = mount.querySelector('#scan-result');
            if (!el) return;
            el.innerHTML = '<div class="boot" data-i18n="common.status.scanning">scanning…</div>';
            mount.querySelectorAll('.scanner-tile').forEach(t => t.classList.toggle('active', t === b));
            try {
                const r = await api.scanRun(b.dataset.preset, wid, 100);
                if (!viewIsCurrent(tok)) return;
                const elNow = mount.querySelector('#scan-result');
                if (elNow) elNow.innerHTML = renderHits(r);
                renderMoveVsVolChart(r.hits || []);
                renderGapChart(r.hits || []);
            } catch (e) {
                if (!viewIsCurrent(tok)) return;
                const elNow = mount.querySelector('#scan-result');
                if (elNow) elNow.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
            }
        }));
}

function renderHits(r) {
    return `<div class="chart-panel">
        <h2>${esc(t('view.scanners.h2.hits_summary', { label: r.label, hits: r.hits.length, universe: r.universe_size }))}</h2>
        ${r.hits.length ? `<table class="trades">
            <thead><tr>
                <th data-i18n="view.scanners.th.symbol">Symbol</th><th data-i18n="view.scanners.th.price">Price</th><th data-i18n="view.scanners.th.gap">Gap%</th><th data-i18n="view.scanners.th.day">Day%</th><th data-i18n="view.scanners.th.vs_prior">Δ vs prior</th>
                <th data-i18n="view.scanners.th.vol">Vol</th><th data-i18n="view.scanners.th.rvol">RVol</th><th data-i18n="view.scanners.th.hod_dist">HOD dist</th><th data-i18n="view.scanners.th.52w">52w</th>
            </tr></thead><tbody>${r.hits.map(h => {
                const cls = h.change_pct >= 0 ? 'pos' : 'neg';
                return `<tr data-context-scope="symbol-row" data-symbol="${esc(h.symbol)}">
                    <td><a href="#research/${encodeURIComponent(h.symbol)}">${esc(h.symbol)}</a></td>
                    <td>${fmt(h.price)}</td>
                    <td class="${h.gap_pct >= 0 ? 'pos' : 'neg'}">${fmt(h.gap_pct, 2)}%</td>
                    <td class="${h.day_pct >= 0 ? 'pos' : 'neg'}">${fmt(h.day_pct, 2)}%</td>
                    <td class="${cls}">${fmt(h.change_pct, 2)}%</td>
                    <td>${h.volume.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>${fmt(h.rel_volume, 2)}×</td>
                    <td>${fmt(h.hod_dist_pct, 2)}%</td>
                    <td>${fmt(h.year_high_pct, 1)}% / ${fmt(h.year_low_pct, 1)}%</td>
                </tr>`;
            }).join('')}</tbody></table>` : '<p data-i18n="view.scanners.hint.no_matches" class="muted">No matches.</p>'}
    </div>
    <div class="chart-panel">
        <h2 data-i18n="view.scanners.h2.move_vs_vol_chart">Change % vs relative volume</h2>
        <div id="sc-chart" style="width:100%;height:240px"></div>
    </div>
    <div class="chart-panel">
        <h2 data-i18n="view.scanners.h2.gap_chart">Gap % vs day change %</h2>
        <div id="sc-gap-chart" style="width:100%;height:220px"></div>
    </div>`;
}

function renderGapChart(hits) {
    const el = document.getElementById('sc-gap-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (hits || []).filter(h =>
        Number.isFinite(Number(h.gap_pct)) && Number.isFinite(Number(h.change_pct)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.scanners.empty_gap_chart">${esc(t('view.scanners.empty_gap_chart'))}</div>`;
        return;
    }
    const xs = rows.map(h => Number(h.gap_pct));
    const greenY = rows.map(h => Number(h.change_pct) >= 0 ? Number(h.change_pct) : null);
    const redY   = rows.map(h => Number(h.change_pct) <  0 ? Number(h.change_pct) : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { auto: true }, y: { auto: true } },
        series: [
            { label: t('view.scanners.chart.gap_pct') },
            { label: t('view.scanners.chart.day_up'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 10, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.scanners.chart.day_down'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 10, fill: '#ff3860', stroke: '#ff3860' } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 40 } ],
        legend: { show: true },
    }, [xs, greenY, redY], el);
}

function renderMoveVsVolChart(hits) {
    const el = document.getElementById('sc-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (hits || []).filter(h =>
        Number.isFinite(Number(h.change_pct)) && Number.isFinite(Number(h.rel_volume)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.scanners.empty_chart">${esc(t('view.scanners.empty_chart'))}</div>`;
        return;
    }
    const xs = rows.map(h => Number(h.change_pct));
    const upY   = rows.map(h => Number(h.change_pct) >= 0 ? Number(h.rel_volume) : null);
    const downY = rows.map(h => Number(h.change_pct) <  0 ? Number(h.rel_volume) : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { auto: true }, y: { auto: true } },
        series: [
            { label: t('view.scanners.chart.change_pct') },
            { label: t('view.scanners.chart.rvol_up'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 10, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.scanners.chart.rvol_down'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 10, fill: '#ff3860', stroke: '#ff3860' } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 40 } ],
        legend: { show: true },
    }, [xs, upY, downY], el);
}

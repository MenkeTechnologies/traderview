// Open-Interest Change-Rate Alerter helpers shared by view + vitest.
//
// Backend body shape: { snapshots: [{strike, call_oi, put_oi,
// call_oi_baseline, put_oi_baseline}, ...], pct_threshold: f64,
// min_oi: u64 }. Backend returns separate call/put alert arrays sorted
// by largest absolute OI change first.

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

// Five-token-per-line "strike call_oi put_oi call_baseline put_baseline".
export function parseSnapshotBlob(text) {
    const snapshots = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { snapshots, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 5) {
            errors.push({ line_no: i + 1, raw, message: `expected 5 tokens (strike call_oi put_oi call_baseline put_baseline), got ${parts.length}` });
            continue;
        }
        const strike = Number(parts[0]);
        const callOi = Number(parts[1]);
        const putOi  = Number(parts[2]);
        const callBl = Number(parts[3]);
        const putBl  = Number(parts[4]);
        if (!Number.isFinite(strike) || strike <= 0) {
            errors.push({ line_no: i + 1, raw, message: `strike must be > 0` });
            continue;
        }
        if (!Number.isInteger(callOi) || callOi < 0) {
            errors.push({ line_no: i + 1, raw, message: `call_oi must be non-negative integer` });
            continue;
        }
        if (!Number.isInteger(putOi) || putOi < 0) {
            errors.push({ line_no: i + 1, raw, message: `put_oi must be non-negative integer` });
            continue;
        }
        if (!Number.isFinite(callBl) || callBl < 0) {
            errors.push({ line_no: i + 1, raw, message: `call_oi_baseline must be ≥ 0` });
            continue;
        }
        if (!Number.isFinite(putBl) || putBl < 0) {
            errors.push({ line_no: i + 1, raw, message: `put_oi_baseline must be ≥ 0` });
            continue;
        }
        snapshots.push({
            strike,
            call_oi: callOi,
            put_oi: putOi,
            call_oi_baseline: callBl,
            put_oi_baseline: putBl,
        });
    }
    return { snapshots, errors };
}

export function validateInputs(snapshots, pctThreshold, minOi) {
    if (!Array.isArray(snapshots) || snapshots.length === 0) return 'need at least 1 strike snapshot';
    if (!Number.isFinite(pctThreshold) || pctThreshold <= 0) return 'pct_threshold must be > 0';
    if (!Number.isInteger(minOi) || minOi < 0) return 'min_oi must be non-negative integer';
    return null;
}

export function buildBody(snapshots, pctThreshold, minOi) {
    return { snapshots, pct_threshold: pctThreshold, min_oi: minOi };
}

// Categorize an alert into a 4-tier severity badge for the row color.
// Bigger pct_change OR larger absolute change → louder tier.
export function alertTier(alert) {
    if (!alert) return { label: '—', cls: '' };
    const pct = Math.abs(alert.pct_change || 0);
    const abs = Math.abs(alert.abs_change || 0);
    if (pct >= 1.0 || abs >= 50_000) return { label: t('view.oi_change.tier.surge'),   cls: 'neg' };
    if (pct >= 0.5 || abs >= 20_000) return { label: t('view.oi_change.tier.strong'),  cls: 'neg' };
    if (pct >= 0.25 || abs >= 5000)  return { label: t('view.oi_change.tier.notable'), cls: '' };
    return                              { label: t('view.oi_change.tier.mild'),     cls: 'pos' };
}

// "buy_to_open" semantic: positive abs_change = positioning building;
// negative = liquidation. Backend doesn't infer direction; the UI does.
export function flowDirection(absChange) {
    if (!Number.isFinite(absChange) || absChange === 0) return { label: t('view.oi_change.flow.flat'),     cls: '' };
    if (absChange > 0)                                   return { label: t('view.oi_change.flow.building'), cls: 'neg' };
    return                                                       { label: t('view.oi_change.flow.unwind'),   cls: 'pos' };
}

// Aggregate stats across both sides for the summary cards.
export function summarize(report) {
    const empty = { totalCallAlerts: 0, totalPutAlerts: 0,
                    netCallChange: 0, netPutChange: 0,
                    maxCallStrike: null, maxPutStrike: null };
    if (!report) return empty;
    const c = report.call_alerts || [];
    const p = report.put_alerts || [];
    return {
        totalCallAlerts: c.length,
        totalPutAlerts:  p.length,
        netCallChange:   c.reduce((a, x) => a + (x.abs_change || 0), 0),
        netPutChange:    p.reduce((a, x) => a + (x.abs_change || 0), 0),
        maxCallStrike:   c[0]?.strike ?? null,    // backend already sorts biggest first
        maxPutStrike:    p[0]?.strike ?? null,
    };
}

// Deterministic 8-strike demo chain: clear surge on the 510 call (new
// upside positioning) and the 470 put (downside hedge build). Default
// thresholds (25% / 1000) light up several rows.
export function makeDemoSnapshots() {
    return [
        // strike,  call_oi, put_oi, call_baseline, put_baseline
        { strike: 470, call_oi: 1200,  put_oi: 28000, call_oi_baseline: 1100,  put_oi_baseline: 15000 }, // PUT SURGE
        { strike: 480, call_oi: 1800,  put_oi: 14000, call_oi_baseline: 2000,  put_oi_baseline: 13000 },
        { strike: 490, call_oi: 4500,  put_oi: 8000,  call_oi_baseline: 4200,  put_oi_baseline: 7800 },
        { strike: 500, call_oi: 25000, put_oi: 6000,  call_oi_baseline: 24000, put_oi_baseline: 6200 }, // ATM (max OI)
        { strike: 505, call_oi: 18000, put_oi: 4500,  call_oi_baseline: 16500, put_oi_baseline: 4400 },
        { strike: 510, call_oi: 32000, put_oi: 3000,  call_oi_baseline: 12000, put_oi_baseline: 3100 }, // CALL SURGE
        { strike: 520, call_oi: 9500,  put_oi: 1800,  call_oi_baseline: 6500,  put_oi_baseline: 1900 },
        { strike: 530, call_oi: 7800,  put_oi: 800,   call_oi_baseline: 4000,  put_oi_baseline: 850  },
    ];
}

export function fmtN(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return Math.round(v).toLocaleString('en-US');
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v >= 0 ? '+' : '';
    return sign + (v * 100).toFixed(1) + '%';
}

export function fmtSignedInt(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v >= 0 ? '+' : '';
    return sign + Math.round(v).toLocaleString('en-US');
}
